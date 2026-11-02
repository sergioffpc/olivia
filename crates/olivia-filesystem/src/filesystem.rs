use std::collections::BTreeSet;
use std::env;
use std::fs::{self, File};
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

use crate::Error;

/// A read-only source and a writable save area, seen as one tree of files.
#[derive(Debug)]
pub struct Filesystem {
    source: PathBuf,
    save: PathBuf,
}

impl Filesystem {
    /// Creates the filesystem of the game named `identity`. Nothing is created on disk yet.
    ///
    /// The source is `assets`; by default, `assets/` at the project root under `cargo run` and
    /// next to the executable otherwise. The save area is the platform's user data directory for
    /// `identity`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidIdentity`] if `identity` cannot name a directory, or [`Error::Io`]
    /// if the directories cannot be found.
    pub fn new(identity: &str, assets: Option<PathBuf>) -> Result<Self, Error> {
        if !is_valid_segment(identity) || identity.trim() != identity {
            return Err(Error::InvalidIdentity(identity.to_owned()));
        }
        let save = ProjectDirs::from("", "", identity)
            .ok_or_else(|| Error::Io {
                path: PathBuf::from(identity),
                source: io::Error::new(ErrorKind::NotFound, "no home directory"),
            })?
            .data_dir()
            .to_owned();
        let source = assets.map_or_else(default_source, Ok)?;
        Ok(Self { source, save })
    }

    /// Creates a filesystem over these two directories.
    #[doc(hidden)]
    #[must_use]
    pub fn with_areas(source: PathBuf, save: PathBuf) -> Self {
        Self { source, save }
    }

    /// Reads a file, from the save area if it is there and from the source otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] if the file is in neither area, or [`Error::Io`] if it cannot
    /// be read, or [`Error::InvalidPath`].
    pub fn read(&self, path: &str) -> Result<Vec<u8>, Error> {
        let file = self
            .locate(path)?
            .ok_or_else(|| Error::NotFound(path.to_owned()))?;
        fs::read(&file).map_err(|source| Error::Io { path: file, source })
    }

    /// Writes a new file to the save area, creating its parent directories. It may shadow a file
    /// in the source.
    ///
    /// # Errors
    ///
    /// Returns [`Error::AlreadyExists`] if the file is in the save area, which is left as it was,
    /// [`Error::Io`] if it cannot be written, or [`Error::InvalidPath`].
    pub fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), Error> {
        let file = save_path(&self.save, path)?;
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent).map_err(|source| Error::Io {
                path: parent.to_owned(),
                source,
            })?;
        }
        write_new(&file, bytes).map_err(|source| match source.kind() {
            ErrorKind::AlreadyExists => Error::AlreadyExists(path.to_owned()),
            _ => Error::Io { path: file, source },
        })
    }

    /// Says whether [`read`](Self::read) would find the file, in either area.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidPath`], or [`Error::Io`] if an area cannot be examined.
    pub fn exists(&self, path: &str) -> Result<bool, Error> {
        Ok(self.locate(path)?.is_some())
    }

    /// Lists the files directly inside a directory, in either area, as sorted paths that
    /// [`read`](Self::read) accepts. A file in both areas appears once, and a name no path can
    /// reach is left out.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] if the directory is in neither area, [`Error::Io`] if it cannot
    /// be listed, or [`Error::InvalidPath`].
    pub fn list(&self, path: &str) -> Result<Vec<String>, Error> {
        validate(path)?;
        let mut found = false;
        let mut files = BTreeSet::new();
        for area in [&self.save, &self.source] {
            let dir = area.join(path);
            let io_error = |source| Error::Io {
                path: dir.clone(),
                source,
            };
            let entries = match fs::read_dir(&dir) {
                Ok(entries) => entries,
                Err(source) if is_missing(&source) => continue,
                Err(source) => return Err(io_error(source)),
            };
            found = true;
            for entry in entries {
                let entry = entry.map_err(io_error)?;
                let name = entry.file_name();
                let Some(name) = name.to_str().filter(|name| is_valid_segment(name)) else {
                    continue;
                };
                if is_file(&entry.path())? {
                    files.insert(format!("{path}/{name}"));
                }
            }
        }
        if !found {
            return Err(Error::NotFound(path.to_owned()));
        }
        Ok(files.into_iter().collect())
    }

    /// Deletes a file from the save area.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] if the file is not in the save area, [`Error::Io`] if it cannot
    /// be deleted, or [`Error::InvalidPath`].
    pub fn remove(&mut self, path: &str) -> Result<(), Error> {
        let file = save_path(&self.save, path)?;
        fs::remove_file(&file).map_err(|source| {
            if is_missing(&source) {
                Error::NotFound(path.to_owned())
            } else {
                Error::Io { path: file, source }
            }
        })
    }

    /// Finds the file `read` would read.
    fn locate(&self, path: &str) -> Result<Option<PathBuf>, Error> {
        validate(path)?;
        for area in [&self.save, &self.source] {
            let file = area.join(path);
            if is_file(&file)? {
                return Ok(Some(file));
            }
        }
        Ok(None)
    }
}

/// `assets/` at the project root under `cargo run`, and next to the executable otherwise.
fn default_source() -> Result<PathBuf, Error> {
    if let Some(root) = env::var_os("CARGO_MANIFEST_DIR") {
        return Ok(PathBuf::from(root).join("assets"));
    }
    let executable = env::current_exe().map_err(|source| Error::Io {
        path: PathBuf::from("assets"),
        source,
    })?;
    Ok(executable.with_file_name("assets"))
}

fn is_file(path: &Path) -> Result<bool, Error> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_file()),
        Err(source) if is_missing(&source) => Ok(false),
        Err(source) => Err(Error::Io {
            path: path.to_owned(),
            source,
        }),
    }
}

/// Whether nothing is at a path, including when one of its parents is a file.
fn is_missing(error: &io::Error) -> bool {
    matches!(error.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory)
}

/// Creates `path` with `bytes`, or leaves no file behind: a partial one could never be replaced.
fn write_new(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let result = File::create_new(path)?.write_all(bytes);
    if result.is_err() {
        let _ = fs::remove_file(path);
    }
    result
}

fn save_path(save: &Path, path: &str) -> Result<PathBuf, Error> {
    validate(path)?;
    Ok(save.join(path))
}

fn validate(path: &str) -> Result<(), Error> {
    if path.split('/').all(is_valid_segment) {
        Ok(())
    } else {
        Err(Error::InvalidPath(path.to_owned()))
    }
}

/// Whether `segment` names one entry of a directory on every platform. A backslash or a colon
/// would escape the area on Windows.
fn is_valid_segment(segment: &str) -> bool {
    !matches!(segment, "" | "." | "..") && !segment.contains(['/', '\\', ':'])
}
