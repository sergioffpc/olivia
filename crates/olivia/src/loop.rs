use std::cell::Cell;

use tracing::{info_span, trace_span};

use crate::clock::{Clock, SystemClock};
use crate::{Config, Frame, Game, Result, Setup, Timer, Update};

/// Runs `G` until it quits or a phase fails.
///
/// # Errors
///
/// Returns the first `Err` from [`Game::load`] or [`Game::update`].
pub fn run<G: Game>(config: Config) -> Result<()> {
    run_with::<G, _>(config, SystemClock::default())
}

/// Runs `G` on the given clock.
///
/// # Errors
///
/// Returns the first `Err` from [`Game::load`] or [`Game::update`].
pub fn run_with<G: Game, C: Clock>(_config: Config, mut clock: C) -> Result<()> {
    let mut game = info_span!("load").in_scope(|| G::load(Setup::new()))?;
    let mut timer = Timer::new();
    let quit = Cell::new(false);

    loop {
        for _ in 0..timer.start_frame(clock.next_frame()) {
            trace_span!("update").in_scope(|| game.update(Update::new(&timer, &quit)))?;
            timer.finish_step();
            if quit.get() {
                return Ok(());
            }
        }

        let mut frame = Frame::new(timer.interpolation_factor());
        trace_span!("draw").in_scope(|| game.draw(&mut frame));
    }
}
