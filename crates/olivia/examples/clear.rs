//! Clears the window to a colour, and quits when asked to close.
//!
//! Run it with `cargo run --example clear`.

use olivia::math::Color;
use olivia::window::WindowEvent;
use olivia::{Config, Frame, Game, Result, Setup, Update};

struct Demo;

impl Game for Demo {
    fn load(_s: Setup<'_>) -> Result<Self> {
        Ok(Self)
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        if u.events.contains(&WindowEvent::CloseRequested) {
            u.quit();
        }
        Ok(())
    }

    fn draw(&self, f: &mut Frame<'_>) {
        f.clear(Color::rgb(0.2, 0.3, 0.5));
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    olivia::run::<Demo>(Config::new("clear").title("Olivia").size(640.0, 360.0))
}
