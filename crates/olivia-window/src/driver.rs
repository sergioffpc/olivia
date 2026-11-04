use olivia_math::Vec2;
use tracing::info;
use web_time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{StartCause, WindowEvent as WinitEvent};
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

/// When a driver wants its next frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Next {
    /// As soon as the window is redrawn: presenting the frame just run has paced it.
    Redraw,
    /// No sooner than this: nothing paced the frame just run.
    At(Instant),
    /// Never: the driver has stopped.
    Stop,
}

/// What [`run`] drives once the window exists.
pub trait Driver: Sized {
    /// What [`start`](Self::start) receives from [`run`].
    type Seed;

    /// What stops the driver, which a failure to create the window must fit into.
    type Error: From<Error>;

    /// Starts from `seed` on the new window, whose logical size is `size`.
    ///
    /// # Errors
    ///
    /// An `Err` closes the window and returns from [`run`].
    fn start(seed: Self::Seed, window: &mut Window, size: Vec2) -> Result<Self, Self::Error>;

    /// Receives an event from the window.
    fn event(&mut self, event: WindowEvent);

    /// Receives every event as the platform reports it, on a window of this `scale_factor`.
    fn raw_event(&mut self, event: &WinitEvent, scale_factor: f64);

    /// Runs one frame, when the window is redrawn, and returns when to run the next.
    ///
    /// A window the player cannot see, such as a minimised one, may not be redrawn, so frames stop
    /// there until it comes back.
    ///
    /// # Errors
    ///
    /// An `Err` closes the window and returns from [`run`].
    fn frame(&mut self, window: &mut Window) -> Result<Next, Self::Error>;
}

/// Opens a window and drives `D`, started from `seed`, on it until it stops. Must be called on the
/// main thread, at most once per process.
///
/// # Errors
///
/// Returns [`Error::Create`] if the window cannot be created, or the first `Err` from `D`.
pub fn run<D: Driver>(attributes: Attributes, seed: D::Seed) -> Result<(), D::Error> {
    let event_loop = EventLoop::new().map_err(Error::create)?;
    let mut host = Host::<D> {
        attributes,
        seed: Some(seed),
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
}

struct Host<D: Driver> {
    attributes: Attributes,
    /// Taken by the first resume; later ones find the window already there.
    seed: Option<D::Seed>,
    running: Option<Running<D>>,
    result: Result<(), D::Error>,
}

impl<D: Driver> Host<D> {
    fn start(&self, seed: D::Seed, event_loop: &ActiveEventLoop) -> Result<Running<D>, D::Error> {
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
        let driver = D::start(seed, &mut window, size)?;
        Ok(Running { driver, window })
    }

    /// Runs a frame and asks for the next one.
    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let Some(running) = &mut self.running else {
            return;
        };
        if let Some(size) = running.window.applied_size.take() {
            running.driver.event(WindowEvent::Resized(size));
        }
        match running.driver.frame(&mut running.window) {
            Ok(Next::Redraw) => {
                event_loop.set_control_flow(ControlFlow::Wait);
                running.window.raw().request_redraw();
            }
            // The redraw is asked for once this time comes, in `new_events`.
            Ok(Next::At(time)) => event_loop.set_control_flow(ControlFlow::WaitUntil(time)),
            Ok(Next::Stop) => self.stop(event_loop, Ok(())),
            Err(error) => self.stop(event_loop, Err(error)),
        }
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
        let Some(seed) = self.seed.take() else {
            return;
        };
        match self.start(seed, event_loop) {
            Ok(running) => {
                running.window.raw().request_redraw();
                self.running = Some(running);
            }
            Err(error) => self.stop(event_loop, Err(error)),
        }
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if let (StartCause::ResumeTimeReached { .. }, Some(running)) = (cause, &self.running) {
            // Waiting on, so that a window that is not redrawn does not wake the loop again.
            event_loop.set_control_flow(ControlFlow::Wait);
            running.window.raw().request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WinitEvent) {
        let Some(running) = &mut self.running else {
            return;
        };
        let scale_factor = running.window.raw().scale_factor();
        running.driver.raw_event(&event, scale_factor);
        let event = match event {
            // Windows reports a minimised window as empty, which is not a size for the game.
            WinitEvent::Resized(size) if size.width == 0 || size.height == 0 => return,
            WinitEvent::Resized(size) => WindowEvent::Resized(logical_size(size, scale_factor)),
            WinitEvent::Focused(focused) => WindowEvent::Focus(focused),
            WinitEvent::CloseRequested => WindowEvent::CloseRequested,
            WinitEvent::RedrawRequested => return self.frame(event_loop),
            _ => return,
        };
        running.driver.event(event);
    }
}
