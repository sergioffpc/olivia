use std::time::Duration;

use tracing::warn;

/// The interval between updates, at 60 Hz.
pub(crate) const STEP: Duration = Duration::from_nanos(1_000_000_000 / 60);

/// The most fixed steps one frame runs; any more are dropped.
const MAX_STEPS: u128 = 8;

/// The game's time, read-only.
#[derive(Debug)]
pub struct Timer {
    elapsed: Duration,
    accumulator: Duration,
    behind: bool,
    fps: f32,
    sample_frames: f32,
    sample_time: Duration,
}

impl Timer {
    /// The fixed step, in seconds.
    #[must_use]
    pub fn dt(&self) -> f32 {
        STEP.as_secs_f32()
    }

    /// The time the game has advanced through, in seconds: one fixed step per finished update.
    ///
    /// Fixed steps dropped when the loop falls behind do not count.
    #[must_use]
    pub fn elapsed(&self) -> f64 {
        self.elapsed.as_secs_f64()
    }

    /// The frames per second measured over the last full second, 0 before the first one ends.
    #[must_use]
    pub fn fps(&self) -> f32 {
        self.fps
    }

    pub(crate) fn new() -> Self {
        Self {
            elapsed: Duration::ZERO,
            // Seeded with one step, so an update always runs before the first draw.
            accumulator: STEP,
            behind: false,
            fps: 0.0,
            sample_frames: 0.0,
            sample_time: Duration::ZERO,
        }
    }

    /// Accounts for a frame and returns how many fixed steps it runs.
    pub(crate) fn start_frame(&mut self, frame_time: Duration) -> u128 {
        self.sample_fps(frame_time);
        self.accumulator += frame_time;

        let steps = self.accumulator.as_nanos() / STEP.as_nanos();
        self.accumulator = Duration::from_nanos_u128(self.accumulator.as_nanos() % STEP.as_nanos());

        // Warns once per lag, not on every frame of it.
        let behind = steps > MAX_STEPS;
        if behind && !self.behind {
            warn!(dropped = steps - MAX_STEPS, "fixed steps dropped");
        }
        self.behind = behind;

        steps.min(MAX_STEPS)
    }

    pub(crate) fn finish_step(&mut self) {
        self.elapsed += STEP;
    }

    pub(crate) fn interpolation_factor(&self) -> f32 {
        self.accumulator.div_duration_f32(STEP)
    }

    fn sample_fps(&mut self, frame_time: Duration) {
        if frame_time.is_zero() {
            return;
        }
        self.sample_frames += 1.0;
        self.sample_time += frame_time;
        if self.sample_time >= Duration::from_secs(1) {
            self.fps = self.sample_frames / self.sample_time.as_secs_f32();
            self.sample_frames = 0.0;
            self.sample_time = Duration::ZERO;
        }
    }
}
