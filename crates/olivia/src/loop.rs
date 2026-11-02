use std::cell::Cell;
use std::time::Duration;

#[cfg(feature = "animation")]
use olivia_animation::Character;
#[cfg(feature = "filesystem")]
use olivia_filesystem::Watcher;
#[cfg(feature = "input")]
use olivia_input::Input;
use olivia_math::Vec2;
use olivia_window::{Driver, Window, WindowEvent, raw};
#[cfg(feature = "filesystem")]
use tracing::{error, info, warn};
use tracing::{info_span, trace_span};

use crate::clock::SystemClock;
use crate::{Config, Game, Modules, Result, Setup, Timer, Update};

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
    /// The window's logical size, which a reloaded game has to be told.
    #[cfg(feature = "filesystem")]
    size: Vec2,
    /// Present when hot reload is on.
    #[cfg(feature = "filesystem")]
    watcher: Option<Watcher>,
}

/// Runs [`Game::load`] on `modules`, and gives its entities their bodies.
fn load<G: Game>(window: &mut Window, modules: &mut Modules) -> Result<G> {
    let game = info_span!("load").in_scope(|| G::load(Setup::new(window, modules)))?;
    #[cfg(feature = "physics")]
    modules.simulation.sync(&mut modules.world);
    Ok(game)
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
            is_minimized: false,
            presented: false,
            #[cfg(feature = "filesystem")]
            size,
            #[cfg(feature = "filesystem")]
            watcher,
        })
    }

    /// Receives an event from the window.
    pub fn push(&mut self, event: WindowEvent) {
        if let WindowEvent::Minimized(is_minimized) = event {
            self.is_minimized = is_minimized;
        }
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

    /// Runs a frame that took `frame_time` and returns whether to keep going.
    ///
    /// # Errors
    ///
    /// Returns the first `Err` from [`Game::update`], or a `graphics::Error` if the GPU is lost and
    /// cannot be brought back.
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

        self.presented = !self.is_minimized && self.present()?;
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

    /// Draws the world and returns whether the frame reached the window.
    #[cfg(feature = "graphics")]
    fn present(&mut self) -> Result<bool> {
        self.modules.graphics.recover()?;
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

    #[cfg_attr(not(feature = "graphics"), expect(unused_mut))]
    fn start(mut modules: Modules, window: &mut Window, size: Vec2) -> Result<Self> {
        #[cfg(feature = "graphics")]
        modules.graphics.attach(window)?;
        Self::new(window, size, modules)
    }

    fn event(&mut self, event: WindowEvent) {
        self.push(event);
    }

    #[cfg_attr(not(feature = "input"), expect(unused_variables))]
    fn raw_event(&mut self, event: &raw::Event, scale_factor: f64) {
        #[cfg(feature = "input")]
        self.modules.input.on_event(event, scale_factor);
    }

    fn frame(&mut self, window: &mut Window) -> Result<bool> {
        let frame_time = self.clock.next_frame(self.presented);
        self.run_frame(frame_time, window)
    }
}
