use std::cell::Cell;

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Document, HtmlCanvasElement};
use winit::dpi::LogicalSize;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::platform::web::{EventLoopExtWebSys, WindowAttributesExtWebSys, WindowExtWebSys};
use winit::window::{Fullscreen, Window};

use crate::driver::{Host, Wake};
use crate::{Attributes, Driver, Error};

/// The canvas's layout at a fixed size, which `winit` sets: centred in the browser window.
const FIXED: &[(&str, &str)] = &[("position", "fixed"), ("inset", "0"), ("margin", "auto")];

/// The canvas's layout when the player can resize it: filling the browser window.
const RESIZABLE: &[(&str, &str)] = &[
    ("position", "fixed"),
    ("inset", "0"),
    ("width", "100%"),
    ("height", "100%"),
];

thread_local! {
    /// Whether the browser refused the last request for fullscreen, which is then repeated on the
    /// player's next key or pointer press.
    static FULLSCREEN_REFUSED: Cell<bool> = const { Cell::new(false) };
}

/// Hands `host` to the browser, which runs it on `event_loop` from then on, and returns at once.
/// What stops the driver is reported by the host itself.
#[expect(
    clippy::unnecessary_wraps,
    reason = "the desktop's run returns its failures"
)]
pub(crate) fn run<D: Driver>(event_loop: EventLoop<Wake>, host: Host<D>) -> Result<(), D::Error> {
    // `run_app` throws on the web, to hand the stack back to the browser.
    event_loop.spawn_app(host);
    Ok(())
}

/// Opens a canvas at the end of the page's `<body>`, laid out by its own inline styles, and
/// focuses it, so the keyboard works without a click.
pub(crate) fn open(event_loop: &ActiveEventLoop, attributes: &Attributes) -> Result<Window, Error> {
    let Attributes {
        title,
        size,
        resizable,
        fullscreen,
    } = attributes;
    let mut window_attributes = Window::default_attributes()
        .with_append(true)
        .with_fullscreen(fullscreen.then_some(Fullscreen::Borderless(None)));
    if !resizable {
        window_attributes = window_attributes.with_inner_size(LogicalSize::new(size.x, size.y));
    }
    let raw = event_loop
        .create_window(window_attributes)
        .map_err(Error::create)?;
    let canvas = raw.canvas().ok_or_else(|| Error::create("no canvas"))?;
    let document = canvas
        .owner_document()
        .ok_or_else(|| Error::create("no document"))?;

    let style = canvas.style();
    let layout = if *resizable { RESIZABLE } else { FIXED };
    for (property, value) in [("display", "block"), ("outline", "none")]
        .iter()
        .chain(layout)
    {
        style
            .set_property(property, value)
            .map_err(|_| Error::create("the canvas cannot be styled"))?;
    }
    // Focused again now that it is laid out, in case the first focus found it empty.
    let _ = canvas.focus();
    document.set_title(title);
    repeat_refused_fullscreen(&document, &canvas);
    Ok(raw)
}

/// Sets the page's title.
pub(crate) fn set_title(raw: &Window, title: &str) {
    if let Some(document) = raw.canvas().and_then(|canvas| canvas.owner_document()) {
        document.set_title(title);
    }
}

/// Enters or leaves fullscreen. The browser enters it only during a key or pointer press, so a
/// refused request is repeated on the next one.
pub(crate) fn set_fullscreen(raw: &Window, fullscreen: bool) {
    FULLSCREEN_REFUSED.set(false);
    raw.set_fullscreen(fullscreen.then_some(Fullscreen::Borderless(None)));
}

/// Listens for refused fullscreen requests, and repeats one on the player's next press. The
/// listeners live as long as the page.
fn repeat_refused_fullscreen(document: &Document, canvas: &HtmlCanvasElement) {
    let refused = Closure::<dyn FnMut()>::new(|| FULLSCREEN_REFUSED.set(true));
    let _ = document
        .add_event_listener_with_callback("fullscreenerror", refused.as_ref().unchecked_ref());
    refused.forget();

    let canvas = canvas.clone();
    let press = Closure::<dyn FnMut()>::new(move || {
        if FULLSCREEN_REFUSED.replace(false) {
            let _ = canvas.request_fullscreen();
        }
    });
    for event in ["keydown", "pointerdown"] {
        let _ = document.add_event_listener_with_callback(event, press.as_ref().unchecked_ref());
    }
    press.forget();
}
