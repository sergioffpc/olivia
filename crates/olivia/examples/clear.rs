//! Clears the window to a colour, and quits when asked to close.
//!
//! Run it with `cargo run --example clear`.

use olivia::ecs::CClear;
use olivia::math::Color;
use olivia::window::WindowEvent;
use olivia::{Config, Game, Result, Setup, Update};

struct Demo;

impl Game for Demo {
    fn load(s: Setup<'_>) -> Result<Self> {
        s.world.spawn((CClear(Color::rgb(0.0, 0.0, 1.0)),));
        Ok(Self)
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        if u.events.contains(&WindowEvent::CloseRequested) {
            u.quit();
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    olivia::run::<Demo>(Config::new("clear").title("Olivia").size(640.0, 360.0))
}
