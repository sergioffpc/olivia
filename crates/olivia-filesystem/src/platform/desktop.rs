use std::env;
use std::fs::{self, File};
use std::future;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant, SystemTime};

use directories::ProjectDirs;
use notify::event::ModifyKind;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as _};

use crate::Error;
use crate::filesystem::is_valid_segment;

/// The source, a directory.
pub(crate) type Source = Dir;
/// The save area, a directory.
pub(crate) type Save = Dir;

/// The source in `assets`, or by default `assets/` at the project root under `cargo run` and
/// next to the executable otherwise, and the user data directory of `identity`.
pub(crate) fn areas(identity: &str, assets: Option<PathBuf>) -> Result<(Source, Save), Error> {
    let save = ProjectDirs::from("", "", identity)
        .ok_or_else(|| Error::Io {
            path: PathBuf::from(identity),
            source: io::Error::new(ErrorKind::NotFound, "no home directory"),
        })?
        .data_dir()
        .to_owned();
    let source = assets.map_or_else(default_source, Ok)?;
    Ok((Dir(source), Dir(save)))
}

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

/// An area on disk, which need not exist yet.
#[derive(Debug)]
pub(crate) struct Dir(PathBuf);

impl Dir {
    pub(crate) fn new(dir: PathBuf) -> Self {
        Self(dir)
    }

    /// Nothing to do: the source on disk is read when asked for.
    #[expect(clippy::unused_self, reason = "the web fetches the source here")]
    pub(crate) fn preload(&mut self) -> impl Future<Output = Result<(), Error>> {
        future::ready(Ok(()))
    }

    pub(crate) fn is_file(&self, path: &str) -> Result<bool, Error> {
        is_file(&self.0.join(path))
    }

    /// The file's bytes, or `None` if it is not here.
    pub(crate) fn read(&self, path: &str) -> Result<Option<Vec<u8>>, Error> {
        let file = self.0.join(path);
        if !is_file(&file)? {
            return Ok(None);
        }
        fs::read(&file)
            .map(Some)
            .map_err(|source| Error::Io { path: file, source })
    }

    /// Creates the file and its parent directories, or leaves no file behind.
    pub(crate) fn write(&self, path: &str, bytes: &[u8]) -> Result<(), Error> {
        let file = self.0.join(path);
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

    pub(crate) fn remove(&self, path: &str) -> Result<(), Error> {
        let file = self.0.join(path);
        fs::remove_file(&file).map_err(|source| {
            if is_missing(&source) {
                Error::NotFound(path.to_owned())
            } else {
                Error::Io { path: file, source }
            }
        })
    }

    /// The names of the files directly in `path`, or `None` if it is not a directory here.
    pub(crate) fn list(&self, path: &str) -> Result<Option<Vec<String>>, Error> {
        let dir = self.0.join(path);
        let io_error = |source| Error::Io {
            path: dir.clone(),
            source,
        };
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(source) if is_missing(&source) => return Ok(None),
            Err(source) => return Err(io_error(source)),
        };
        let mut names = Vec::new();
        for entry in entries {
            let entry = entry.map_err(io_error)?;
            let name = entry.file_name();
            let Some(name) = name.to_str().filter(|name| is_valid_segment(name)) else {
                continue;
            };
            if is_file(&entry.path())? {
                names.push(name.to_owned());
            }
        }
        Ok(Some(names))
    }

    pub(crate) fn watch(&self) -> Result<Watcher, Error> {
        Watcher::new(&self.0)
    }
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

/// How long the source has to stay still before a change counts, so that a file is not read half
/// written and a burst of saves counts once.
const QUIET: Duration = Duration::from_millis(100);

/// The changes to the files of a directory.
#[derive(Debug)]
pub(crate) struct Watcher {
    /// Stops watching when dropped.
    _inner: RecommendedWatcher,
    events: Receiver<notify::Result<Event>>,
    last_change: Option<Instant>,
    /// When the last change reported was over, or the watch began.
    settled: SystemTime,
}

impl Watcher {
    fn new(source: &Path) -> Result<Self, Error> {
        let io_error = |error| Error::Io {
            path: source.to_owned(),
            source: io::Error::other(error),
        };
        let (sender, events) = mpsc::channel();
        let mut watcher = notify::recommended_watcher(sender).map_err(io_error)?;
        watcher
            .watch(source, RecursiveMode::Recursive)
            .map_err(io_error)?;
        Ok(Self {
            _inner: watcher,
            events,
            last_change: None,
            settled: SystemTime::now(),
        })
    }

    pub(crate) fn changed(&mut self) -> bool {
        let settled = self.settled;
        if self
            .events
            .try_iter()
            .any(|event| is_change(&event, settled))
        {
            self.last_change = Some(Instant::now());
        }
        let is_due = self.last_change.is_some_and(|last| last.elapsed() >= QUIET);
        if is_due {
            self.last_change = None;
            // The file clock can lag, and the last write was at least `QUIET` ago.
            self.settled = SystemTime::now() - QUIET / 2;
        }
        is_due
    }
}

fn is_change(event: &notify::Result<Event>, settled: SystemTime) -> bool {
    let Ok(event) = event else {
        return false;
    };
    let is_edit = match event.kind {
        EventKind::Modify(ModifyKind::Metadata(_)) => false,
        // All Windows says, also for a directory whose contents changed, and for a file whose
        // metadata reaches the disk only when something next opens it.
        EventKind::Modify(ModifyKind::Any) => {
            event.paths.iter().any(|path| is_written(path, settled))
        }
        EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_) => true,
        _ => false,
    };
    is_edit && !event.paths.iter().all(|path| is_hidden(path))
}

/// Whether `path` is a file written since `time`.
fn is_written(path: &Path, time: SystemTime) -> bool {
    fs::metadata(path).is_ok_and(|metadata| {
        metadata.is_file() && metadata.modified().is_ok_and(|modified| modified >= time)
    })
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name.as_encoded_bytes().starts_with(b"."))
}
