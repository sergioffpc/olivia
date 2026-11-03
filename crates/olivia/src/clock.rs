use std::thread;
use std::time::{Duration, Instant};

use crate::timer::FIXED_STEP;

/// The wall clock, pacing frames one fixed step apart when presenting did not pace them.
#[derive(Debug, Default)]
pub(crate) struct SystemClock {
    last: Option<Instant>,
}

impl SystemClock {
    /// Waits for the next frame, unless the last one was `presented`, and returns the time since
    /// the previous one.
    pub(crate) fn next_frame(&mut self, presented: bool) -> Duration {
        if let Some(last) = self.last.filter(|_| !presented) {
            thread::sleep((last + FIXED_STEP).saturating_duration_since(Instant::now()));
        }
        let now = Instant::now();
        let frame_time = self.last.map_or(Duration::ZERO, |last| now - last);
        self.last = Some(now);
        frame_time
    }
}
