use tracing_subscriber::EnvFilter;

/// Logs to the terminal at `info`, or as `RUST_LOG` says when it is set, unless the game has
/// installed a subscriber of its own.
pub(crate) fn init_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    // A subscriber the game installed earlier wins.
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
