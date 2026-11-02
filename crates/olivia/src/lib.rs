//! Olivia, a 2D game library.

mod clock;
mod config;
pub mod ecs;
mod r#loop;
mod modules;
mod phase;
#[cfg(feature = "graphics")]
mod renderer;
#[cfg(feature = "physics")]
mod simulation;
mod timer;

#[cfg(feature = "animation")]
pub use olivia_animation as animation;
#[cfg(feature = "audio")]
pub use olivia_audio as audio;
#[cfg(feature = "filesystem")]
pub use olivia_filesystem as filesystem;
#[cfg(feature = "graphics")]
pub use olivia_graphics as graphics;
#[cfg(feature = "input")]
pub use olivia_input as input;
pub use olivia_math as math;
#[cfg(feature = "physics")]
pub use olivia_physics as physics;
pub use olivia_window as window;

pub use config::Config;
#[doc(hidden)]
pub use r#loop::GameLoop;
pub use r#loop::run;
#[doc(hidden)]
pub use modules::Modules;
pub use phase::{Setup, Update};
pub use timer::Timer;

/// The result of a phase and of [`run`], boxing any error by default.
pub type Result<T = (), E = Box<dyn std::error::Error + Send + Sync>> = std::result::Result<T, E>;

/// A game: the phases Olivia drives around the [world](ecs) it simulates and draws.
pub trait Game: Sized {
    /// Builds the game. Runs once, after the window exists and before anything else.
    ///
    /// # Errors
    ///
    /// An `Err` stops the loop and returns from [`run`].
    fn load(s: Setup<'_>) -> Result<Self>;

    /// Advances the game by one fixed step.
    ///
    /// # Errors
    ///
    /// An `Err` stops the loop and returns from [`run`].
    fn update(&mut self, u: Update<'_>) -> Result<()>;
}
