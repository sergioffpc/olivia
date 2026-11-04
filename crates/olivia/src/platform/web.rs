use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_wasm::WASMLayer;

/// Logs to the browser console at `info`, with spans as `performance` marks, unless the game has
/// installed a subscriber of its own, and sends panics to the console.
pub(crate) fn init_logging() {
    console_error_panic_hook::set_once();
    // A subscriber the game installed earlier wins.
    let _ = tracing_subscriber::registry()
        .with(LevelFilter::INFO)
        .with(WASMLayer::default())
        .try_init();
}
