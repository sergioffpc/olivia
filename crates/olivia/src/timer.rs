use std::time::Duration;

/// The interval between updates, at 60 Hz.
pub(crate) const STEP: Duration = Duration::from_nanos(1_000_000_000 / 60);

/// The game's time, read-only.
#[derive(Debug)]
pub struct Timer {
    elapsed: Duration,
    fps: f32,
    frames: f32,
    sample: Duration,
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
            fps: 0.0,
            frames: 0.0,
            sample: Duration::ZERO,
        }
    }

    pub(crate) fn finish_step(&mut self) {
        self.elapsed += STEP;
    }

    pub(crate) fn start_frame(&mut self, frame_time: Duration) {
        self.frames += 1.0;
        self.sample += frame_time;
        if self.sample >= Duration::from_secs(1) {
            self.fps = self.frames / self.sample.as_secs_f32();
            self.frames = 0.0;
            self.sample = Duration::ZERO;
        }
    }
}
