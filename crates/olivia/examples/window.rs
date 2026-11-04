//! Opens a window, logs what happens to it, and quits when asked to close.
//!
//! Run it with `cargo run --example window`.

use olivia::window::WindowEvent;
use olivia::{Config, Game, Result, Setup, Update};

struct Demo;

impl Game for Demo {
    fn load(_s: Setup<'_>) -> Result<Self> {
        Ok(Self)
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        for event in u.events {
            tracing::info!(?event, "window event");
            if let WindowEvent::CloseRequested = event {
                u.quit();
            }
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    olivia::run::<Demo>(Config::new("window").title("Olivia").size(640.0, 360.0))
}
