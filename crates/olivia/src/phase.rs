use std::cell::Cell;
use std::fmt;

#[cfg(feature = "audio")]
use olivia_audio::Audio;
#[cfg(feature = "filesystem")]
use olivia_filesystem::Filesystem;
#[cfg(feature = "graphics")]
use olivia_graphics::Graphics;
#[cfg(feature = "input")]
use olivia_input::Input;
#[cfg(feature = "physics")]
use olivia_physics::World;
use olivia_window::{Window, WindowEvent};

use crate::{Modules, Timer, ecs};

/// The handles available to [`Game::load`](crate::Game::load).
#[non_exhaustive]
pub struct Setup<'a> {
    /// The game's window.
    pub window: &'a mut Window,
    /// The game's audio.
    #[cfg(feature = "audio")]
    pub audio: &'a mut Audio,
    /// The game's assets and saves.
    #[cfg(feature = "filesystem")]
    pub filesystem: &'a mut Filesystem,
    /// The game's graphics.
    #[cfg(feature = "graphics")]
    pub graphics: &'a mut Graphics,
    /// The game's entities.
    pub world: &'a mut ecs::World,
}

impl<'a> Setup<'a> {
    pub(crate) fn new(window: &'a mut Window, modules: &'a mut Modules) -> Self {
        Self {
            window,
            #[cfg(feature = "audio")]
            audio: &mut modules.audio,
            #[cfg(feature = "filesystem")]
            filesystem: &mut modules.filesystem,
            #[cfg(feature = "graphics")]
            graphics: &mut modules.graphics,
            world: &mut modules.world,
        }
    }
}

/// The handles available to [`Game::update`](crate::Game::update).
#[non_exhaustive]
pub struct Update<'a> {
    /// The game's window.
    pub window: &'a mut Window,
    /// The game's audio.
    #[cfg(feature = "audio")]
    pub audio: &'a mut Audio,
    /// The game's assets and saves.
    #[cfg(feature = "filesystem")]
    pub filesystem: &'a mut Filesystem,
    /// The game's graphics.
    #[cfg(feature = "graphics")]
    pub graphics: &'a mut Graphics,
    /// The game's entities. Olivia steps physics right after this update.
    pub world: &'a mut ecs::World,
    /// The physics simulation, for queries, forces and the events of the last step.
    #[cfg(feature = "physics")]
    pub physics: &'a mut World,
    /// What happened to the window since the last update. Only the first step of a frame has any.
    pub events: &'a [WindowEvent],
    /// The keyboard and mouse.
    #[cfg(feature = "input")]
    pub input: &'a Input,
    /// The fixed step, elapsed time and frame rate.
    pub timer: &'a Timer,
    quit: &'a Cell<bool>,
}

impl<'a> Update<'a> {
    pub(crate) fn new(
        window: &'a mut Window,
        modules: &'a mut Modules,
        events: &'a [WindowEvent],
        timer: &'a Timer,
        quit: &'a Cell<bool>,
    ) -> Self {
        Self {
            window,
            #[cfg(feature = "audio")]
            audio: &mut modules.audio,
            #[cfg(feature = "filesystem")]
            filesystem: &mut modules.filesystem,
            #[cfg(feature = "graphics")]
            graphics: &mut modules.graphics,
            world: &mut modules.world,
            #[cfg(feature = "physics")]
            physics: &mut modules.simulation.physics,
            events,
            #[cfg(feature = "input")]
            input: &modules.input,
            timer,
            quit,
        }
    }

    /// Asks the loop to stop once this update returns.
    pub fn quit(&self) {
        self.quit.set(true);
    }
}

impl fmt::Debug for Setup<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Setup").finish_non_exhaustive()
    }
}

impl fmt::Debug for Update<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Update")
            .field("events", &self.events)
            .finish_non_exhaustive()
    }
}
