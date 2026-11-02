use std::thread;
use std::time::{Duration, Instant};

use crate::timer::FIXED_STEP;

/// The wall clock, pacing frames one fixed step apart in place of vsync until graphics exists.
#[derive(Debug, Default)]
pub(crate) struct SystemClock {
    last: Option<Instant>,
}

impl SystemClock {
    /// Waits for the next frame and returns the time since the previous one.
    pub(crate) fn next_frame(&mut self) -> Duration {
        if let Some(last) = self.last {
            thread::sleep((last + FIXED_STEP).saturating_duration_since(Instant::now()));
        }
        let now = Instant::now();
        let frame_time = self.last.map_or(Duration::ZERO, |last| now - last);
        self.last = Some(now);
        frame_time
    }
}
