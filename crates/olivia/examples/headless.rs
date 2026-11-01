//! A game with no window: it updates for one second of game time and quits.
//!
//! Run it with `cargo run --example headless`.

use olivia::{Config, Frame, Game, Result, Setup, Update};

struct Countdown {
    seconds_left: f32,
}

impl Game for Countdown {
    fn load(_s: Setup<'_>) -> Result<Self> {
        Ok(Self { seconds_left: 1.0 })
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        self.seconds_left -= u.timer.dt();
        if self.seconds_left <= 0.0 {
            tracing::info!(elapsed = u.timer.elapsed(), "time is up");
            u.quit();
        }
        Ok(())
    }

    fn draw(&self, f: &mut Frame<'_>) {
        // A game with a window would draw here, between two update states.
        let _ = f.interpolation_factor();
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    olivia::run::<Countdown>(Config::new("headless"))
}
