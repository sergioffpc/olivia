use std::cell::Cell;
use std::fmt;
#[cfg(feature = "graphics")]
use std::future::Future;
#[cfg(feature = "graphics")]
use std::mem;
#[cfg(feature = "graphics")]
use std::pin::Pin;
#[cfg(feature = "graphics")]
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

#[cfg(feature = "animation")]
use olivia_animation::Character;
#[cfg(feature = "filesystem")]
use olivia_filesystem::Watcher;
#[cfg(feature = "graphics")]
use olivia_graphics::Graphics;
#[cfg(feature = "input")]
use olivia_input::Input;
use olivia_math::Vec2;
use olivia_window::{Driver, Next, Starting, Window, WindowEvent, raw};
use tracing::{error, info_span, trace_span};
#[cfg(feature = "filesystem")]
use tracing::{info, warn};
use web_time::Instant;

use crate::clock::SystemClock;
use crate::{Config, Game, Modules, Result, Setup, Timer, Update, platform};

/// Opens a window and runs `G` on it until it quits or a phase fails.
///
/// Must be called on the main thread, at most once per process.
///
/// It first sets up logging at `info`, to the terminal on desktop, where `RUST_LOG` replaces the
/// filter when set, and to the browser console on the web. A `tracing` subscriber the game
/// installed before is kept instead.
///
/// # Errors
///
/// Returns an error if the identity is invalid, before any window opens;
/// [`window::Error`](crate::window::Error) if the window cannot be created; a `graphics::Error` if
/// no GPU can draw to it, at the start or after the GPU is lost; a `filesystem::Error` if the
/// source cannot be fetched at the start on the web; or the first `Err` from
/// [`Game::load`] or [`Game::update`]. The window is closed by then.
pub fn run<G: Game>(config: Config) -> Result<()> {
    platform::init_logging();
    let modules = Modules::new(&config)?;
    olivia_window::run::<GameLoop<G>>(config.window_attributes(), modules)
}

/// The graphics being brought back after the GPU device was lost.
#[cfg(feature = "graphics")]
type Recovery = Pin<Box<dyn Future<Output = Result<Graphics, olivia_graphics::Error>>>>;

/// A game in the middle of being run, one frame at a time.
pub struct GameLoop<G> {
    game: G,
    modules: Modules,
    clock: SystemClock,
    timer: Timer,
    quit: Cell<bool>,
    /// The events waiting for the next step.
    events: Vec<WindowEvent>,
    /// Whether the last frame reached the window, so the clock leaves its pace to vsync.
    presented: bool,
    /// The window's logical size, which a reloaded game has to be told.
    #[cfg(feature = "filesystem")]
    size: Vec2,
    /// Present when hot reload is on.
    #[cfg(feature = "filesystem")]
    watcher: Option<Watcher>,
    /// Present while the GPU device is being brought back, which pauses the game.
    #[cfg(feature = "graphics")]
    recovery: Option<Recovery>,
}

impl<G> fmt::Debug for GameLoop<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GameLoop").finish_non_exhaustive()
    }
}

/// Runs [`Game::load`] on `modules`, and gives its entities their bodies.
fn load<G: Game>(window: &mut Window, modules: &mut Modules) -> Result<G> {
    let game = info_span!("load").in_scope(|| G::load(Setup::new(window, modules)))?;
    #[cfg(feature = "physics")]
    modules.simulation.sync(&mut modules.world);
    Ok(game)
}

/// Whether `event` presses a key or a mouse button, which lets audio play on the web.
#[cfg(feature = "audio")]
fn is_press(event: &raw::Event) -> bool {
    match event {
        raw::Event::KeyboardInput { event, .. } => event.state.is_pressed(),
        raw::Event::MouseInput { state, .. } => state.is_pressed(),
        _ => false,
    }
}

impl<G: Game> GameLoop<G> {
    /// Loads the game on a window of logical size `size`.
    ///
    /// # Errors
    ///
    /// Returns the `Err` from [`Game::load`].
    pub fn new(window: &mut Window, size: Vec2, mut modules: Modules) -> Result<Self> {
        // Started before loading, so a change during the load is not missed.
        #[cfg(feature = "filesystem")]
        let watcher = modules
            .config
            .hot_reload_value()
            .then(|| modules.filesystem.watch());
        #[cfg(feature = "filesystem")]
        let watcher = watcher.and_then(|watcher| {
            watcher
                .inspect_err(
                    |error| warn!(%error, "hot reload is off, the source cannot be watched"),
                )
                .ok()
        });
        let game = load(window, &mut modules)?;
        Ok(Self {
            game,
            modules,
            clock: SystemClock::default(),
            timer: Timer::new(),
            quit: Cell::new(false),
            events: vec![WindowEvent::Resized(size)],
            presented: false,
            #[cfg(feature = "filesystem")]
            size,
            #[cfg(feature = "filesystem")]
            watcher,
            #[cfg(feature = "graphics")]
            recovery: None,
        })
    }

    /// Receives an event from the window.
    pub fn push(&mut self, event: WindowEvent) {
        #[cfg(feature = "filesystem")]
        if let WindowEvent::Resized(size) = event {
            self.size = size;
        }
        self.events.push(event);
    }

    /// The input waiting for the next step.
    #[cfg(feature = "input")]
    pub fn input(&mut self) -> &mut Input {
        &mut self.modules.input
    }

    /// Runs a frame that starts at `now`, timed by the wall clock, and returns when to run the next.
    ///
    /// # Errors
    ///
    /// As [`run_frame`](Self::run_frame).
    pub fn run_frame_at(&mut self, now: Instant, window: &mut Window) -> Result<Next> {
        let frame_time = self.clock.start_frame(now);
        if !self.run_frame(frame_time, window)? {
            return Ok(Next::Stop);
        }
        #[cfg(feature = "graphics")]
        if self.recovery.is_some() {
            return Ok(Next::Await);
        }
        Ok(self.clock.end_frame(self.presented))
    }

    /// Runs a frame that took `frame_time` and returns whether to keep going.
    ///
    /// # Errors
    ///
    /// Returns the first `Err` from [`Game::update`], or a `graphics::Error` if the surface is lost
    /// and cannot be brought back.
    pub fn run_frame(&mut self, frame_time: Duration, window: &mut Window) -> Result<bool> {
        #[cfg(feature = "filesystem")]
        self.reload_if_changed(window);
        for _ in 0..self.timer.steps_for_frame(frame_time) {
            let update = Update::new(
                window,
                &mut self.modules,
                &self.events,
                &self.timer,
                &self.quit,
            );
            trace_span!("update").in_scope(|| self.game.update(update))?;
            #[cfg(feature = "physics")]
            trace_span!("physics")
                .in_scope(|| self.modules.simulation.step(&mut self.modules.world));
            #[cfg(feature = "animation")]
            for character in self.modules.world.query_mut::<&mut Character>() {
                character.advance(self.timer.dt());
            }
            self.events.clear();
            #[cfg(feature = "input")]
            self.modules.input.on_step();
            self.timer.finish_step();
            if self.quit.get() {
                return Ok(false);
            }
        }

        self.presented = self.present()?;
        Ok(true)
    }

    /// Loads the game again if the source has changed.
    #[cfg(feature = "filesystem")]
    fn reload_if_changed(&mut self, window: &mut Window) {
        if !self.watcher.as_mut().is_some_and(Watcher::changed) {
            return;
        }
        let mut stage = self.modules.empty_stage();
        self.modules.swap_stage(&mut stage);
        match load(window, &mut self.modules) {
            Ok(game) => {
                self.game = game;
                self.events.push(WindowEvent::Resized(self.size));
                info!("reloaded the game");
            }
            Err(error) => {
                self.modules.swap_stage(&mut stage);
                error!(%error, "the game could not reload, it carries on as it was");
            }
        }
    }

    /// Draws the world and returns whether the frame reached the window. A lost GPU device starts
    /// its recovery instead.
    #[cfg(feature = "graphics")]
    fn present(&mut self) -> Result<bool> {
        if self.modules.graphics.is_device_lost() {
            let placeholder = self.modules.config.graphics();
            let mut graphics = mem::replace(&mut self.modules.graphics, placeholder);
            self.recovery = Some(Box::pin(async move {
                graphics.recover().await.map(|()| graphics)
            }));
            return Ok(false);
        }
        self.modules.graphics.recreate_lost_surface()?;
        let factor = self.timer.interpolation_factor();
        let target = trace_span!("draw").in_scope(|| self.modules.render(factor));
        Ok(target.present())
    }

    /// With no graphics a frame never reaches the window.
    #[cfg(not(feature = "graphics"))]
    #[expect(clippy::unused_self, reason = "matches the graphics version")]
    fn present(&mut self) -> Result<bool> {
        trace_span!("draw").in_scope(|| Ok(false))
    }

    /// How far the frame just run reached into the next fixed step, from 0 to 1.
    pub fn interpolation_factor(&self) -> f32 {
        self.timer.interpolation_factor()
    }

    /// What the world looks like now, as the next frame would draw it.
    #[cfg(feature = "graphics")]
    pub fn mesh(&mut self) -> olivia_graphics::Mesh {
        self.modules
            .render(self.timer.interpolation_factor())
            .mesh()
    }

    /// The game.
    pub fn game(&self) -> &G {
        &self.game
    }

    /// The entities.
    pub fn world(&mut self) -> &mut crate::ecs::World {
        &mut self.modules.world
    }
}

impl<G: Game> Driver for GameLoop<G> {
    type Seed = Modules;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    /// Preloads the source and sets up the GPU. Every future the game waits for before it loads
    /// goes here.
    #[cfg_attr(
        not(any(feature = "filesystem", feature = "graphics")),
        expect(unused_mut)
    )]
    #[cfg_attr(not(feature = "graphics"), expect(unused_variables))]
    fn start(mut modules: Modules, window: &Window) -> Starting<Modules, Self::Error> {
        #[cfg(feature = "graphics")]
        let raw = Arc::clone(window.raw());
        Box::pin(async move {
            #[cfg(feature = "filesystem")]
            modules.filesystem.preload().await?;
            #[cfg(feature = "graphics")]
            modules.graphics.attach(&raw).await?;
            Ok(modules)
        })
    }

    fn load(modules: Modules, window: &mut Window, size: Vec2) -> Result<Self> {
        Self::new(window, size, modules)
    }

    fn event(&mut self, event: WindowEvent) {
        self.push(event);
    }

    #[cfg_attr(not(feature = "input"), expect(unused_variables))]
    fn raw_event(&mut self, event: &raw::Event, scale_factor: f64) {
        // The browser lets audio play only from inside the handler of a key or button press,
        // which is where this runs on the web.
        #[cfg(feature = "audio")]
        if is_press(event) {
            self.modules.audio.resume();
        }
        #[cfg(feature = "input")]
        self.modules.input.on_event(event, scale_factor);
    }

    fn frame(&mut self, window: &mut Window) -> Result<Next> {
        self.run_frame_at(Instant::now(), window)
    }

    /// Polls the GPU recovery, if one is under way.
    #[cfg_attr(not(feature = "graphics"), expect(unused_variables))]
    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<Result<()>> {
        #[cfg(feature = "graphics")]
        if let Some(recovery) = &mut self.recovery {
            let Poll::Ready(graphics) = recovery.as_mut().poll(cx) else {
                return Poll::Pending;
            };
            self.recovery = None;
            self.modules.graphics = graphics?;
            // The game was paused, and the pause must not turn into catch-up steps.
            self.clock.restart();
        }
        Poll::Ready(Ok(()))
    }

    fn fail(error: &Self::Error) {
        error!(%error, "the game stopped");
    }
}
