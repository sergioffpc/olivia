use std::time::Duration;

use olivia_window::Next;
use web_time::Instant;

use crate::timer::FIXED_STEP;

/// How much later than due a frame can start before the wait counts as a pause: the window was not
/// redrawn, as while minimised.
const PAUSE: Duration = Duration::from_millis(250);

/// The wall clock, measuring frames and pacing those that presenting did not pace.
#[derive(Debug, Default)]
pub(crate) struct SystemClock {
    /// When the last frame started.
    last: Option<Instant>,
    /// When the next frame was due.
    due: Option<Instant>,
}

impl SystemClock {
    /// Starts a frame at `now` and returns the time since the last one started: zero for the first
    /// frame and for the first after a pause, so a pause never turns into catch-up steps.
    pub(crate) fn start_frame(&mut self, now: Instant) -> Duration {
        if self.due.is_some_and(|due| now > due + PAUSE) {
            self.restart();
        }
        let frame_time = self
            .last
            .map_or(Duration::ZERO, |last| now.saturating_duration_since(last));
        self.last = Some(now);
        frame_time
    }

    /// Ends the frame, `presented` to the window or not, and returns when the next one is due.
    pub(crate) fn end_frame(&mut self, presented: bool) -> Next {
        let now = Instant::now();
        let Some(last) = self.last.filter(|_| !presented) else {
            self.due = Some(now);
            return Next::Redraw;
        };
        let next = last + FIXED_STEP;
        self.due = Some(next.max(now));
        Next::At(next)
    }

    /// Forgets the frames so far, so the next one starts from zero.
    fn restart(&mut self) {
        *self = Self::default();
    }
}
