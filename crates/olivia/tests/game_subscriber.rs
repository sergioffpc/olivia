//! A subscriber the game installs before `run`. The global subscriber can be installed only once per
//! process, so this file holds a single test.

use olivia::{Config, Game, Result, Setup, Update};
use tracing::Level;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

struct Idle;

impl Game for Idle {
    fn load(_s: Setup<'_>) -> Result<Self> {
        Ok(Self)
    }

    fn update(&mut self, _u: Update<'_>) -> Result<()> {
        Ok(())
    }
}

#[test]
fn run_keeps_a_subscriber_the_game_installed() {
    tracing_subscriber::registry()
        .with(LevelFilter::TRACE)
        .init();
    // An invalid identity makes `run` return before any window opens.
    assert!(olivia::run::<Idle>(Config::new("")).is_err());
    assert!(tracing::enabled!(Level::TRACE));
}
