use crate::platform;

/// Reports the changes to the files of a source, as [`Filesystem::watch`] started it.
///
/// [`Filesystem::watch`]: crate::Filesystem::watch
#[derive(Debug)]
pub struct Watcher(pub(crate) platform::Watcher);

impl Watcher {
    /// Says whether a file came, went or was rewritten since the last `true`, once the source has
    /// been still for a moment. Reading a file is no change, and neither is touching a hidden one,
    /// such as an editor's swap file.
    pub fn changed(&mut self) -> bool {
        self.0.changed()
    }
}
