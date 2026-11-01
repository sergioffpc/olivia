use std::cell::Cell;
use std::thread;
use std::time::{Duration, Instant};

use tracing::{info_span, trace_span, warn};

use crate::timer::STEP;
use crate::{Config, Frame, Game, Result, Setup, Timer, Update};

/// The most fixed steps one frame runs; any more are dropped.
const MAX_STEPS: u128 = 8;

/// Where the loop gets its time from.
pub trait Clock {
    /// Waits for the next frame and returns the time since the previous one.
    fn next_frame(&mut self) -> Duration;
}

/// The wall clock, sleeping for one fixed step in place of vsync until the window exists.
#[derive(Debug, Default)]
struct SystemClock {
    last: Option<Instant>,
}

impl Clock for SystemClock {
    fn next_frame(&mut self) -> Duration {
        thread::sleep(STEP);
        let now = Instant::now();
        let frame_time = self.last.map_or(Duration::ZERO, |last| now - last);
        self.last = Some(now);
        frame_time
    }
}

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
    // Seeded with one step, so an update always runs before the first draw.
    let mut accumulator = STEP;

    loop {
        let frame_time = clock.next_frame();
        timer.start_frame(frame_time);
        accumulator += frame_time;

        let steps = accumulator.as_nanos() / STEP.as_nanos();
        accumulator = Duration::from_nanos_u128(accumulator.as_nanos() % STEP.as_nanos());
        if steps > MAX_STEPS {
            warn!(dropped = steps - MAX_STEPS, "fixed steps dropped");
        }

        for _ in 0..steps.min(MAX_STEPS) {
            trace_span!("update").in_scope(|| game.update(Update::new(&timer, &quit)))?;
            timer.finish_step();
            if quit.get() {
                return Ok(());
            }
        }

        let mut frame = Frame::new(accumulator.div_duration_f32(STEP));
        trace_span!("draw").in_scope(|| game.draw(&mut frame));
    }
}
