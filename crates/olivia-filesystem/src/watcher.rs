use std::fs;
use std::io;
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant, SystemTime};

use notify::event::ModifyKind;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as _};

use crate::Error;

/// How long the source has to stay still before a change counts, so that a file is not read half
/// written and a burst of saves counts once.
const QUIET: Duration = Duration::from_millis(100);

/// Reports the changes to the files of a source, as [`Filesystem::watch`] started it.
///
/// [`Filesystem::watch`]: crate::Filesystem::watch
#[derive(Debug)]
pub struct Watcher {
    /// Stops watching when dropped.
    _inner: RecommendedWatcher,
    events: Receiver<notify::Result<Event>>,
    last_change: Option<Instant>,
    /// When the last change reported was over, or the watch began.
    settled: SystemTime,
}

impl Watcher {
    pub(crate) fn new(source: &Path) -> Result<Self, Error> {
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

    /// Says whether a file came, went or was rewritten since the last `true`, once the source has
    /// been still for a moment. Reading a file is no change, and neither is touching a hidden one,
    /// such as an editor's swap file.
    pub fn changed(&mut self) -> bool {
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
