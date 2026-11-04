use std::cell::RefCell;
use std::panic::{self, PanicHookInfo};
use std::sync::Once;

use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_wasm::WASMLayer;
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlElement};

/// The progress bar's track, centred above the canvas.
const BAR: &[(&str, &str)] = &[
    ("position", "fixed"),
    ("top", "50%"),
    ("left", "50%"),
    ("transform", "translate(-50%, -50%)"),
    ("width", "min(20rem, 60vw)"),
    ("height", "0.375rem"),
    ("border-radius", "0.1875rem"),
    ("overflow", "hidden"),
    ("background", "rgb(128 128 128 / 30%)"),
    ("z-index", "1"),
];

/// The part of the bar that fills as the source arrives.
const FILL: &[(&str, &str)] = &[
    ("width", "0%"),
    ("height", "100%"),
    ("background", "rgb(128 128 128)"),
];

/// The failure screen, which dims the canvas's last frame.
const FAILURE: &[(&str, &str)] = &[
    ("position", "fixed"),
    ("inset", "0"),
    ("display", "flex"),
    ("flex-direction", "column"),
    ("align-items", "center"),
    ("justify-content", "center"),
    ("gap", "0.5rem"),
    ("padding", "1rem"),
    ("background", "rgb(0 0 0 / 60%)"),
    ("color", "white"),
    ("font-family", "system-ui, sans-serif"),
    ("text-align", "center"),
    ("z-index", "1"),
];

/// The failure screen's wordless sign.
const SIGN: &[(&str, &str)] = &[("font-size", "3rem"), ("line-height", "1")];

/// The failure screen's error text, in small type.
const TEXT: &[(&str, &str)] = &[
    ("font-size", "0.75rem"),
    ("max-width", "40rem"),
    ("white-space", "pre-wrap"),
    ("overflow-wrap", "anywhere"),
];

/// The progress bar on the page.
struct Progress {
    bar: HtmlElement,
    fill: HtmlElement,
}

thread_local! {
    /// The progress bar, from the start until the game loads or fails.
    static PROGRESS: RefCell<Option<Progress>> = const { RefCell::new(None) };
}

/// Logs to the browser console at `info`, with spans as `performance` marks, unless the game has
/// installed a subscriber of its own.
pub(crate) fn init_logging() {
    // A subscriber the game installed earlier wins.
    let _ = tracing_subscriber::registry()
        .with(LevelFilter::INFO)
        .with(WASMLayer::default())
        .try_init();
}

/// Sends panics to the browser console, and reports them as failures.
pub(crate) fn init_panic_hook() {
    static PANIC_HOOK: Once = Once::new();
    PANIC_HOOK.call_once(|| panic::set_hook(Box::new(on_panic)));
}

/// Reports a panic, after which the module aborts and nothing else runs.
fn on_panic(info: &PanicHookInfo<'_>) {
    console_error_panic_hook::hook(info);
    match info.payload_as_str() {
        Some(message) => fail(message),
        None => fail(&info.to_string()),
    }
}

/// Shows the progress bar, filled by `fetched` out of `total` files, and full when there are none.
pub(crate) fn show_progress(fetched: usize, total: usize) {
    let percent = (100 * fetched).checked_div(total).unwrap_or(100);
    PROGRESS.with_borrow_mut(|progress| {
        if progress.is_none() {
            *progress = new_progress();
        }
        if let Some(progress) = progress {
            let _ = progress
                .fill
                .style()
                .set_property("width", &format!("{percent}%"));
        }
    });
}

/// Puts an empty progress bar at the end of the page's `<body>`.
fn new_progress() -> Option<Progress> {
    let document = document()?;
    let bar = element(&document, BAR)?;
    let fill = element(&document, FILL)?;
    bar.append_child(&fill).ok()?;
    document.body()?.append_child(&bar).ok()?;
    Some(Progress { bar, fill })
}

/// Removes the progress bar.
pub(crate) fn hide_progress() {
    // A panic while the bar is borrowed leaves it there, under the failure screen.
    let progress = PROGRESS.with(|progress| progress.try_borrow_mut().ok()?.take());
    if let Some(Progress { bar, .. }) = progress {
        bar.remove();
    }
}

/// Stops the audio and shows the failure screen with `text`, in place of the progress bar and
/// out of fullscreen.
pub(crate) fn fail(text: &str) {
    // The `AudioContext` would play on after a panic, which aborts before anything is dropped.
    #[cfg(feature = "audio")]
    olivia_audio::suspend_all();
    hide_progress();
    // Fullscreen shows only the canvas, which would hide the failure screen.
    if let Some(document) = document()
        && document.fullscreen_element().is_some()
    {
        document.exit_fullscreen();
    }
    let _ = new_failure(text);
}

/// Puts the failure screen at the end of the page's `<body>`.
fn new_failure(text: &str) -> Option<()> {
    let document = document()?;
    let screen = element(&document, FAILURE)?;
    let sign = element(&document, SIGN)?;
    sign.set_text_content(Some("⚠"));
    let message = element(&document, TEXT)?;
    message.set_text_content(Some(text));
    screen.append_child(&sign).ok()?;
    screen.append_child(&message).ok()?;
    document.body()?.append_child(&screen).ok()?;
    Some(())
}

/// The page's document.
fn document() -> Option<Document> {
    web_sys::window()?.document()
}

/// A new `<div>`, laid out by its own inline `styles`.
fn element(document: &Document, styles: &[(&str, &str)]) -> Option<HtmlElement> {
    let element: HtmlElement = document.create_element("div").ok()?.dyn_into().ok()?;
    let style = element.style();
    for (property, value) in styles {
        style.set_property(property, value).ok()?;
    }
    Some(element)
}
