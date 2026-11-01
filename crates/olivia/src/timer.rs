use std::time::Duration;

use tracing::warn;

/// The interval between updates, at 60 Hz.
pub(crate) const FIXED_STEP: Duration = Duration::from_nanos(1_000_000_000 / 60);

/// The most fixed steps one frame runs; any more are dropped.
const MAX_STEPS_PER_FRAME: u128 = 8;

/// How long the frame rate is measured over before it is updated.
const FPS_WINDOW: Duration = Duration::from_secs(1);

/// The game's time, read-only.
#[derive(Debug)]
pub struct Timer {
    /// The game time: one fixed step per finished update.
    elapsed: Duration,
    /// Real time not yet consumed by a fixed step, less than one step after each frame.
    unstepped: Duration,
    /// Whether the last frame had to drop fixed steps.
    was_behind: bool,
    /// The frame rate measured over the last full window.
    fps: f32,
    /// The frames counted in the window being measured.
    window_frames: u32,
    /// The time the window being measured has covered so far.
    window_time: Duration,
}

impl Timer {
    /// The fixed step, in seconds.
    #[must_use]
    pub fn dt(&self) -> f32 {
        FIXED_STEP.as_secs_f32()
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
            unstepped: FIXED_STEP,
            was_behind: false,
            fps: 0.0,
            window_frames: 0,
            window_time: Duration::ZERO,
        }
    }

    /// Accounts for a frame that took `frame_time` and returns how many fixed steps it runs.
    pub(crate) fn steps_for_frame(&mut self, frame_time: Duration) -> u128 {
        self.measure_fps(frame_time);
        self.unstepped += frame_time;

        let step_nanos = FIXED_STEP.as_nanos();
        let due = self.unstepped.as_nanos() / step_nanos;
        self.unstepped = Duration::from_nanos_u128(self.unstepped.as_nanos() % step_nanos);

        // Warns once per lag, not on every frame of it.
        let is_behind = due > MAX_STEPS_PER_FRAME;
        if is_behind && !self.was_behind {
            warn!(dropped = due - MAX_STEPS_PER_FRAME, "fixed steps dropped");
        }
        self.was_behind = is_behind;

        due.min(MAX_STEPS_PER_FRAME)
    }

    /// Advances the game time by one fixed step, after an update finishes.
    pub(crate) fn finish_step(&mut self) {
        self.elapsed += FIXED_STEP;
    }

    /// How far the unstepped time has advanced into the next fixed step, from 0 to 1.
    pub(crate) fn interpolation_factor(&self) -> f32 {
        self.unstepped.div_duration_f32(FIXED_STEP)
    }

    /// Counts a frame, and updates the frame rate once a full window has passed.
    fn measure_fps(&mut self, frame_time: Duration) {
        // The first frame takes no time and would skew the rate.
        if frame_time.is_zero() {
            return;
        }
        self.window_frames += 1;
        self.window_time += frame_time;
        if self.window_time >= FPS_WINDOW {
            #[expect(
                clippy::cast_precision_loss,
                reason = "a frame count stays far below 2^24"
            )]
            let frames = self.window_frames as f32;
            self.fps = frames / self.window_time.as_secs_f32();
            self.window_frames = 0;
            self.window_time = Duration::ZERO;
        }
    }
}
