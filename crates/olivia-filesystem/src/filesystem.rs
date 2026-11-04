use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::platform::{self, Save, Source};
use crate::{Error, Watcher};

/// A read-only source and a writable save area, seen as one tree of files.
///
/// On the web, the source is fetched into memory by the time the game loads, and does not change
/// after that. The save area is the browser's `localStorage`: about 5 MiB per origin, shared by
/// every game published there, of which base64 takes a third. The browser may clear it, as Safari
/// does after seven days without a visit.
#[derive(Debug)]
pub struct Filesystem {
    source: Source,
    save: Save,
}

impl Filesystem {
    /// Creates the filesystem of the game named `identity`. Nothing is written yet.
    ///
    /// The source is `assets`; by default, `assets/` at the project root under `cargo run` and
    /// next to the executable otherwise. On the web, `assets` is a URL path relative to the page's
    /// base URL, `assets/` by default. The save area is the platform's user data directory for
    /// `identity`, or on the web the `localStorage` keys under `olivia/<identity>/`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidIdentity`] if `identity` cannot name a directory, or [`Error::Io`]
    /// if the directories cannot be found, or on the web if `localStorage` cannot be reached.
    pub fn new(identity: &str, assets: Option<PathBuf>) -> Result<Self, Error> {
        if !is_valid_segment(identity) || identity.trim() != identity {
            return Err(Error::InvalidIdentity(identity.to_owned()));
        }
        let (source, save) = platform::areas(identity, assets)?;
        Ok(Self { source, save })
    }

    /// Creates a filesystem over these two areas: directories on desktop, and on the web the URL
    /// path of the source and the name under which the saves are kept.
    #[doc(hidden)]
    #[must_use]
    pub fn with_areas(source: PathBuf, save: PathBuf) -> Self {
        Self {
            source: Source::new(source),
            save: Save::new(save),
        }
    }

    /// Fetches the source into memory on the web, and does nothing on desktop.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the manifest or a file it lists cannot be fetched.
    #[doc(hidden)]
    pub async fn preload(&mut self) -> Result<(), Error> {
        self.source.preload().await
    }

    /// Reads a file, from the save area if it is there and from the source otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] if the file is in neither area, or [`Error::Io`] if it cannot
    /// be read, or [`Error::InvalidPath`].
    pub fn read(&self, path: &str) -> Result<Vec<u8>, Error> {
        validate(path)?;
        if let Some(bytes) = self.save.read(path)? {
            return Ok(bytes);
        }
        self.source
            .read(path)?
            .ok_or_else(|| Error::NotFound(path.to_owned()))
    }

    /// Writes a new file to the save area, creating its parent directories. It may shadow a file
    /// in the source.
    ///
    /// # Errors
    ///
    /// Returns [`Error::AlreadyExists`] if the file is in the save area, which is left as it was,
    /// [`Error::Io`] if it cannot be written, or [`Error::InvalidPath`]. A full save area is
    /// [`Error::Io`] with [`ErrorKind::StorageFull`](std::io::ErrorKind::StorageFull), and nothing
    /// is written.
    pub fn write(&mut self, path: &str, bytes: &[u8]) -> Result<(), Error> {
        validate(path)?;
        self.save.write(path, bytes)
    }

    /// Says whether [`read`](Self::read) would find the file, in either area.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidPath`], or [`Error::Io`] if an area cannot be examined.
    pub fn exists(&self, path: &str) -> Result<bool, Error> {
        validate(path)?;
        Ok(self.save.is_file(path)? || self.source.is_file(path)?)
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
        for names in [self.save.list(path)?, self.source.list(path)?] {
            let Some(names) = names else { continue };
            found = true;
            files.extend(names.into_iter().map(|name| format!("{path}/{name}")));
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
        validate(path)?;
        self.save.remove(path)
    }

    /// Starts watching the source for changes.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the source cannot be watched, as when it does not exist, and on
    /// the web, where the source never changes, with
    /// [`ErrorKind::Unsupported`](std::io::ErrorKind::Unsupported).
    pub fn watch(&self) -> Result<Watcher, Error> {
        self.source.watch().map(Watcher)
    }
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
pub(crate) fn is_valid_segment(segment: &str) -> bool {
    !matches!(segment, "" | "." | "..") && !segment.contains(['/', '\\', ':'])
}
