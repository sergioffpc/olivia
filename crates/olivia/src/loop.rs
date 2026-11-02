use std::cell::Cell;
use std::time::Duration;

use olivia_math::Vec2;
use olivia_window::{Driver, Window, WindowEvent};
use tracing::{info_span, trace_span};

use crate::clock::SystemClock;
use crate::{Config, Frame, Game, Modules, Result, Setup, Timer, Update};

/// Opens a window and runs `G` on it until it quits or a phase fails.
///
/// Must be called on the main thread, at most once per process.
///
/// # Errors
///
/// Returns an error if the identity is invalid, before any window opens;
/// [`window::Error`](crate::window::Error) if the window cannot be created; a `graphics::Error` if
/// no GPU can draw to it, at the start or after the GPU is lost; or the first `Err` from
/// [`Game::load`] or [`Game::update`]. The window is closed by then.
pub fn run<G: Game>(config: Config) -> Result<()> {
    let modules = Modules::new(&config)?;
    olivia_window::run::<GameLoop<G>>(config.window_attributes(), modules)
}

/// A game in the middle of being run, one frame at a time.
#[derive(Debug)]
pub struct GameLoop<G> {
    game: G,
    modules: Modules,
    clock: SystemClock,
    timer: Timer,
    quit: Cell<bool>,
    /// The events waiting for the next step.
    events: Vec<WindowEvent>,
    is_minimized: bool,
    /// Whether the last frame reached the window, so the clock leaves its pace to vsync.
    presented: bool,
}

impl<G: Game> GameLoop<G> {
    /// Loads the game on a window of logical size `size`.
    ///
    /// # Errors
    ///
    /// Returns the `Err` from [`Game::load`].
    pub fn new(window: &mut Window, size: Vec2, mut modules: Modules) -> Result<Self> {
        let game = info_span!("load").in_scope(|| G::load(Setup::new(window, &mut modules)))?;
        Ok(Self {
            game,
            modules,
            clock: SystemClock::default(),
            timer: Timer::new(),
            quit: Cell::new(false),
            events: vec![WindowEvent::Resized(size)],
            is_minimized: false,
            presented: false,
        })
    }

    /// Receives an event from the window.
    pub fn push(&mut self, event: WindowEvent) {
        if let WindowEvent::Minimized(is_minimized) = event {
            self.is_minimized = is_minimized;
        }
        self.events.push(event);
    }

    /// Runs a frame that took `frame_time` and returns whether to keep going.
    ///
    /// # Errors
    ///
    /// Returns the first `Err` from [`Game::update`], or a `graphics::Error` if the GPU is lost and
    /// cannot be brought back.
    pub fn run_frame(&mut self, frame_time: Duration, window: &mut Window) -> Result<bool> {
        for _ in 0..self.timer.steps_for_frame(frame_time) {
            let update = Update::new(
                window,
                &mut self.modules,
                &self.events,
                &self.timer,
                &self.quit,
            );
            trace_span!("update").in_scope(|| self.game.update(update))?;
            self.events.clear();
            self.timer.finish_step();
            if self.quit.get() {
                return Ok(false);
            }
        }

        self.presented = !self.is_minimized && {
            #[cfg(feature = "graphics")]
            self.modules.graphics.recover()?;
            let mut frame = Frame::new(self.timer.interpolation_factor(), &mut self.modules);
            trace_span!("draw").in_scope(|| self.game.draw(&mut frame));
            frame.present()
        };
        Ok(true)
    }
}

impl<G: Game> Driver for GameLoop<G> {
    type Seed = Modules;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    #[cfg_attr(not(feature = "graphics"), expect(unused_mut))]
    fn start(mut modules: Modules, window: &mut Window, size: Vec2) -> Result<Self> {
        #[cfg(feature = "graphics")]
        modules.graphics.attach(window)?;
        Self::new(window, size, modules)
    }

    fn event(&mut self, event: WindowEvent) {
        self.push(event);
    }

    fn frame(&mut self, window: &mut Window) -> Result<bool> {
        let frame_time = self.clock.next_frame(self.presented);
        self.run_frame(frame_time, window)
    }
}
