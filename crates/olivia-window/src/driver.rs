use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{self, Context, Poll, Waker};

use olivia_math::Vec2;
use tracing::info;
use web_time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{StartCause, WindowEvent as WinitEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
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
    /// Once what the driver awaits has landed, which [`Driver::poll`] reports.
    Await,
    /// Never: the driver has stopped.
    Stop,
}

/// What a driver awaits before it loads, polled on the main thread by the window's own executor.
pub type Starting<T, E> = Pin<Box<dyn Future<Output = Result<T, E>>>>;

/// What [`run`] drives once the window exists.
pub trait Driver: Sized {
    /// What [`start`](Self::start) receives from [`run`], and what it yields to
    /// [`load`](Self::load).
    type Seed;

    /// What stops the driver, which a failure to create the window must fit into.
    type Error: From<Error>;

    /// Starts from `seed` on the new window, returning what to await before
    /// [`load`](Self::load). Window events until it lands are dropped.
    ///
    /// # Errors
    ///
    /// An `Err` from the future closes the window and returns from [`run`].
    fn start(seed: Self::Seed, window: &Window) -> Starting<Self::Seed, Self::Error>;

    /// Loads from what [`start`](Self::start) yielded, on the window, whose logical size is `size`.
    ///
    /// # Errors
    ///
    /// An `Err` closes the window and returns from [`run`].
    fn load(seed: Self::Seed, window: &mut Window, size: Vec2) -> Result<Self, Self::Error>;

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

    /// Polls what the driver awaits after a frame returned [`Next::Await`]. No frame runs until it
    /// is ready.
    ///
    /// # Errors
    ///
    /// An `Err` closes the window and returns from [`run`].
    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    /// Reports the failure that stops the driver, or its start, before the window closes.
    fn fail(error: &Self::Error);
}

/// Opens a window and drives `D`, started from `seed`, on it until it stops. Must be called on the
/// main thread, at most once per process.
///
/// # Errors
///
/// Returns [`Error::Create`] if the window cannot be created, or the first `Err` from `D`.
pub fn run<D: Driver>(attributes: Attributes, seed: D::Seed) -> Result<(), D::Error> {
    let event_loop = EventLoop::with_user_event()
        .build()
        .map_err(Error::create)?;
    let mut host = Host::<D> {
        attributes,
        seed: Some(seed),
        state: State::Closed,
        waker: Arc::new(Proxy(event_loop.create_proxy())).into(),
        result: Ok(()),
    };
    let outcome = event_loop.run_app(&mut host);
    // The game's own failure comes first: it is what stopped the loop.
    host.result?;
    Ok(outcome.map_err(Error::create)?)
}

/// What the waker sends the event loop, so that the host polls again.
#[derive(Debug)]
struct Wake;

/// Wakes the host through its event loop.
struct Proxy(EventLoopProxy<Wake>);

impl task::Wake for Proxy {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        // A closed event loop has nothing left to poll.
        let _ = self.0.send_event(Wake);
    }
}

/// Where the driver is. Fields are in teardown order: the window goes last.
enum State<D: Driver> {
    /// No window, before the first resume and after stopping.
    Closed,
    /// Awaiting the driver's start; window events are dropped.
    Starting {
        start: Starting<D::Seed, D::Error>,
        window: Window,
    },
    /// Running frames, or awaiting the driver between them.
    Running {
        driver: D,
        window: Window,
        awaiting: bool,
    },
}

struct Host<D: Driver> {
    attributes: Attributes,
    /// Taken by the first resume; later ones find the window already there.
    seed: Option<D::Seed>,
    state: State<D>,
    waker: Waker,
    result: Result<(), D::Error>,
}

impl<D: Driver> Host<D> {
    fn open(&self, event_loop: &ActiveEventLoop) -> Result<Window, Error> {
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
        Ok(Window::new(raw))
    }

    /// Polls what the host awaits, and moves on if it has landed.
    fn poll(&mut self, event_loop: &ActiveEventLoop) {
        let mut cx = Context::from_waker(&self.waker);
        match mem::replace(&mut self.state, State::Closed) {
            State::Starting { mut start, window } => match start.as_mut().poll(&mut cx) {
                Poll::Pending => self.state = State::Starting { start, window },
                Poll::Ready(seed) => {
                    drop(start);
                    self.load(event_loop, seed, window);
                }
            },
            State::Running {
                mut driver,
                window,
                awaiting: true,
            } => match driver.poll(&mut cx) {
                Poll::Pending => {
                    self.state = State::Running {
                        driver,
                        window,
                        awaiting: true,
                    };
                }
                Poll::Ready(Ok(())) => {
                    window.raw().request_redraw();
                    self.state = State::Running {
                        driver,
                        window,
                        awaiting: false,
                    };
                }
                Poll::Ready(Err(error)) => {
                    drop(driver);
                    self.stop(event_loop, Err(error));
                }
            },
            state => self.state = state,
        }
    }

    /// Loads the driver from what its start yielded, and asks for the first frame.
    fn load(
        &mut self,
        event_loop: &ActiveEventLoop,
        seed: Result<D::Seed, D::Error>,
        mut window: Window,
    ) {
        let loaded = seed.and_then(|seed| {
            let raw = window.raw();
            let size = logical_size(raw.inner_size(), raw.scale_factor());
            D::load(seed, &mut window, size)
        });
        match loaded {
            Ok(driver) => {
                window.raw().request_redraw();
                self.state = State::Running {
                    driver,
                    window,
                    awaiting: false,
                };
            }
            Err(error) => self.stop(event_loop, Err(error)),
        }
    }

    /// Runs a frame and asks for the next one.
    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let State::Running {
            driver,
            window,
            awaiting: awaiting @ false,
        } = &mut self.state
        else {
            return;
        };
        if let Some(size) = window.applied_size.take() {
            driver.event(WindowEvent::Resized(size));
        }
        match driver.frame(window) {
            Ok(Next::Redraw) => {
                event_loop.set_control_flow(ControlFlow::Wait);
                window.raw().request_redraw();
            }
            // The redraw is asked for once this time comes, in `new_events`.
            Ok(Next::At(time)) => event_loop.set_control_flow(ControlFlow::WaitUntil(time)),
            Ok(Next::Await) => {
                *awaiting = true;
                event_loop.set_control_flow(ControlFlow::Wait);
                self.poll(event_loop);
            }
            Ok(Next::Stop) => self.stop(event_loop, Ok(())),
            Err(error) => self.stop(event_loop, Err(error)),
        }
    }

    /// Tears down the driver and the window, and leaves `result` for [`run`] to return. Every
    /// failure ends here.
    fn stop(&mut self, event_loop: &ActiveEventLoop, result: Result<(), D::Error>) {
        if let Err(error) = &result {
            D::fail(error);
        }
        self.state = State::Closed;
        self.result = result;
        event_loop.exit();
    }
}

impl<D: Driver> ApplicationHandler<Wake> for Host<D> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let Some(seed) = self.seed.take() else {
            return;
        };
        match self.open(event_loop) {
            Ok(window) => {
                let start = D::start(seed, &window);
                self.state = State::Starting { start, window };
                self.poll(event_loop);
            }
            Err(error) => self.stop(event_loop, Err(error.into())),
        }
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if let (
            StartCause::ResumeTimeReached { .. },
            State::Running {
                window,
                awaiting: false,
                ..
            },
        ) = (cause, &self.state)
        {
            // Waiting on, so that a window that is not redrawn does not wake the loop again.
            event_loop.set_control_flow(ControlFlow::Wait);
            window.raw().request_redraw();
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, Wake: Wake) {
        self.poll(event_loop);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WinitEvent) {
        let State::Running { driver, window, .. } = &mut self.state else {
            return;
        };
        let scale_factor = window.raw().scale_factor();
        driver.raw_event(&event, scale_factor);
        let event = match event {
            // Windows reports a minimised window as empty, which is not a size for the game.
            WinitEvent::Resized(size) if size.width == 0 || size.height == 0 => return,
            WinitEvent::Resized(size) => WindowEvent::Resized(logical_size(size, scale_factor)),
            WinitEvent::Focused(focused) => WindowEvent::Focus(focused),
            WinitEvent::CloseRequested => WindowEvent::CloseRequested,
            WinitEvent::RedrawRequested => return self.frame(event_loop),
            _ => return,
        };
        driver.event(event);
    }
}
