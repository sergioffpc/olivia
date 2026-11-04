use tracing_subscriber::EnvFilter;

/// Logs to the terminal at `info`, or as `RUST_LOG` says when it is set, unless the game has
/// installed a subscriber of its own.
pub(crate) fn init_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    // A subscriber the game installed earlier wins.
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

/// The desktop shows no progress while the game starts.
pub(crate) fn show_progress(_fetched: usize, _total: usize) {}

/// The desktop has no progress bar to remove.
pub(crate) fn hide_progress() {}

/// The desktop's default panic hook stays.
pub(crate) fn init_panic_hook() {}

/// The desktop shows nothing on a failure: the error is in the log, and the audio stops as the
/// window drops the game and closes.
pub(crate) fn fail(_text: &str) {}
