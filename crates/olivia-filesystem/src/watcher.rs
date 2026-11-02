use std::io;
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

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
        })
    }

    /// Says whether a file came, went or was rewritten since the last `true`, once the source has
    /// been still for a moment. Reading a file is no change, and neither is touching a hidden one,
    /// such as an editor's swap file.
    pub fn changed(&mut self) -> bool {
        if self.events.try_iter().any(|event| is_change(&event)) {
            self.last_change = Some(Instant::now());
        }
        let is_due = self.last_change.is_some_and(|last| last.elapsed() >= QUIET);
        if is_due {
            self.last_change = None;
        }
        is_due
    }
}

fn is_change(event: &notify::Result<Event>) -> bool {
    let Ok(event) = event else {
        return false;
    };
    let is_edit = match event.kind {
        EventKind::Create(_) | EventKind::Remove(_) => true,
        EventKind::Modify(kind) => !matches!(kind, ModifyKind::Metadata(_)),
        _ => false,
    };
    is_edit && !event.paths.iter().all(|path| is_hidden(path))
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name.as_encoded_bytes().starts_with(b"."))
}
