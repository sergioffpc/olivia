use winit::dpi::LogicalSize;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Fullscreen, Window};

use crate::driver::{Host, Wake};
use crate::{Attributes, Driver, Error};

/// Runs `host` on `event_loop` until it stops, and returns what stopped it.
pub(crate) fn run<D: Driver>(
    event_loop: EventLoop<Wake>,
    mut host: Host<D>,
) -> Result<(), D::Error> {
    let outcome = event_loop.run_app(&mut host);
    // The game's own failure comes first: it is what stopped the loop.
    host.result?;
    Ok(outcome.map_err(Error::create)?)
}

/// Opens an OS window.
pub(crate) fn open(event_loop: &ActiveEventLoop, attributes: &Attributes) -> Result<Window, Error> {
    let Attributes {
        title,
        size,
        resizable,
        fullscreen,
    } = attributes;
    let attributes = Window::default_attributes()
        .with_title(title)
        .with_inner_size(LogicalSize::new(size.x, size.y))
        .with_resizable(*resizable)
        .with_fullscreen(fullscreen.then_some(Fullscreen::Borderless(None)));
    event_loop.create_window(attributes).map_err(Error::create)
}

/// Sets the window's title.
pub(crate) fn set_title(raw: &Window, title: &str) {
    raw.set_title(title);
}

/// Enters or leaves fullscreen.
pub(crate) fn set_fullscreen(raw: &Window, fullscreen: bool) {
    raw.set_fullscreen(fullscreen.then_some(Fullscreen::Borderless(None)));
}
