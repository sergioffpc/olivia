//! The logging `run` sets up when the game sets up none. The global subscriber can be installed only
//! once per process, so this file holds a single test.

use olivia::{Config, Game, Result, Setup, Update};
use tracing::Level;

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
fn run_logs_at_info_by_default() {
    #[expect(unsafe_code, reason = "the only way to clear an environment variable")]
    // SAFETY: this is the process's only test, so no other thread reads the environment.
    unsafe {
        std::env::remove_var("RUST_LOG");
    }
    // An invalid identity makes `run` return before any window opens.
    assert!(olivia::run::<Idle>(Config::new("")).is_err());
    assert!(tracing::enabled!(Level::INFO));
    assert!(!tracing::enabled!(Level::DEBUG));
}
