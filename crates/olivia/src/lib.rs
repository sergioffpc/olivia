//! Olivia, a 2D game library.

mod config;
mod game_loop;
mod phase;
mod timer;

pub use olivia_math as math;

pub use config::Config;
pub use game_loop::run;
#[doc(hidden)]
pub use game_loop::{Clock, run_with};
pub use phase::{Frame, Setup, Update};
pub use timer::Timer;

/// The result of a phase and of [`run`], boxing any error by default.
pub type Result<T = (), E = Box<dyn std::error::Error + Send + Sync>> = std::result::Result<T, E>;

/// A game: the state it owns and the phases Olivia drives.
pub trait Game: Sized {
    /// Builds the game. Runs once, before anything else.
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

    /// Draws the current state. One or more updates always run before the first draw.
    fn draw(&self, f: &mut Frame<'_>);
}
