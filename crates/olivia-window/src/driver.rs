use olivia_math::Vec2;
use tracing::info;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent as WinitEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Fullscreen, WindowId};

use crate::window::logical_size;
use crate::{Error, Window, WindowEvent};

/// How a window starts.
#[derive(Clone, Debug, PartialEq)]
pub struct Attributes {
    /// The title.
    pub title: String,
    /// The logical size.
    pub size: Vec2,
    /// Whether the player can resize the window.
    pub resizable: bool,
    /// Whether the window starts fullscreen.
    pub fullscreen: bool,
}

/// What [`run`] drives once the window exists.
pub trait Driver: Sized {
    /// What stops the driver, which a failure to create the window must fit into.
    type Error: From<Error>;

    /// Starts on the new window, whose logical size is `size`.
    ///
    /// # Errors
    ///
    /// An `Err` closes the window and returns from [`run`].
    fn start(window: &mut Window, size: Vec2) -> Result<Self, Self::Error>;

    /// Receives an event from the window.
    fn event(&mut self, event: WindowEvent);

    /// Runs one frame and returns whether to keep going.
    ///
    /// # Errors
    ///
    /// An `Err` closes the window and returns from [`run`].
    fn frame(&mut self, window: &mut Window) -> Result<bool, Self::Error>;
}

/// Opens a window and drives `D` on it until it stops. Must be called on the main thread, at most
/// once per process.
///
/// # Errors
///
/// Returns [`Error::Create`] if the window cannot be created, or the first `Err` from `D`.
pub fn run<D: Driver>(attributes: Attributes) -> Result<(), D::Error> {
    let event_loop = EventLoop::new().map_err(Error::create)?;
    let mut host = Host::<D> {
        attributes,
        running: None,
        result: Ok(()),
    };
    let outcome = event_loop.run_app(&mut host);
    // The game's own failure comes first: it is what stopped the loop.
    host.result?;
    Ok(outcome.map_err(Error::create)?)
}

/// A driver on its window, in teardown order: the driver goes first.
struct Running<D> {
    driver: D,
    window: Window,
    is_minimized: bool,
}

struct Host<D: Driver> {
    attributes: Attributes,
    running: Option<Running<D>>,
    result: Result<(), D::Error>,
}

impl<D: Driver> Host<D> {
    fn start(&self, event_loop: &ActiveEventLoop) -> Result<Running<D>, D::Error> {
        let Attributes {
            title,
            size,
            resizable,
            fullscreen,
        } = &self.attributes;
        let attributes = winit::window::Window::default_attributes()
            .with_title(title)
            .with_inner_size(LogicalSize::new(size.x, size.y))
            .with_resizable(*resizable)
            .with_fullscreen(fullscreen.then_some(Fullscreen::Borderless(None)));
        let raw = event_loop
            .create_window(attributes)
            .map_err(Error::create)?;
        info!("window created");

        let size = logical_size(raw.inner_size(), raw.scale_factor());
        let mut window = Window::new(raw);
        let driver = D::start(&mut window, size)?;
        Ok(Running {
            driver,
            window,
            is_minimized: false,
        })
    }

    /// Tears down the driver and the window, and leaves `result` for [`run`] to return.
    fn stop(&mut self, event_loop: &ActiveEventLoop, result: Result<(), D::Error>) {
        self.running = None;
        self.result = result;
        event_loop.exit();
    }
}

impl<D: Driver> ApplicationHandler for Host<D> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.running.is_some() {
            return;
        }
        match self.start(event_loop) {
            Ok(running) => {
                self.running = Some(running);
                event_loop.set_control_flow(ControlFlow::Poll);
            }
            Err(error) => self.stop(event_loop, Err(error)),
        }
    }

    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, event: WinitEvent) {
        let Some(running) = &mut self.running else {
            return;
        };
        let event = match event {
            // Windows reports a minimised window as empty, which is not a size for the game.
            WinitEvent::Resized(size) if size.width == 0 || size.height == 0 => return,
            WinitEvent::Resized(size) => {
                let scale_factor = running.window.raw().scale_factor();
                WindowEvent::Resized(logical_size(size, scale_factor))
            }
            WinitEvent::Focused(focused) => WindowEvent::Focus(focused),
            WinitEvent::CloseRequested => WindowEvent::CloseRequested,
            _ => return,
        };
        running.driver.event(event);
    }

    // Frames run from here and not on redraw requests, which stop while the window is minimised.
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(running) = &mut self.running else {
            return;
        };

        // No platform sends an event for this, so it has to be asked for.
        let is_minimized = running.window.raw().is_minimized().unwrap_or(false);
        if is_minimized != running.is_minimized {
            running.is_minimized = is_minimized;
            running.driver.event(WindowEvent::Minimized(is_minimized));
        }
        if let Some(size) = running.window.applied_size.take() {
            running.driver.event(WindowEvent::Resized(size));
        }

        match running.driver.frame(&mut running.window) {
            Ok(true) => {}
            Ok(false) => self.stop(event_loop, Ok(())),
            Err(error) => self.stop(event_loop, Err(error)),
        }
    }
}
