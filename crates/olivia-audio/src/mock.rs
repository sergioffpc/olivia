use std::sync::Mutex;
use std::time::Duration;

use kira::backend::{Backend, Renderer};

const SAMPLE_RATE: u32 = 48_000;

/// How many frames are rendered at once, which is `kira`'s default.
const FRAMES: usize = 128;

/// An output with no device, which renders only when told to and reports what it rendered.
pub(crate) struct Mock {
    /// In a `Mutex` only because `Renderer` is not `Sync` and `Audio` is.
    renderer: Option<Mutex<Renderer>>,
}

impl Mock {
    /// Renders about `time` and returns the loudest left and right samples.
    pub(crate) fn render(&mut self, time: Duration) -> (f32, f32) {
        let renderer = self
            .renderer
            .as_mut()
            .expect("the manager started this")
            .get_mut()
            .expect("nothing panics holding this");
        let blocks = time.as_millis() * u128::from(SAMPLE_RATE) / (1000 * FRAMES as u128);
        let mut samples = [0.0; FRAMES * 2];
        let mut peak = (0.0_f32, 0.0_f32);
        for _ in 0..=blocks {
            renderer.on_start_processing();
            renderer.process(&mut samples, 2);
            for [left, right] in samples.as_chunks().0 {
                peak.0 = peak.0.max(left.abs());
                peak.1 = peak.1.max(right.abs());
            }
        }
        peak
    }
}

impl Backend for Mock {
    type Settings = ();
    type Error = ();

    fn setup((): (), _internal_buffer_size: usize) -> Result<(Self, u32), ()> {
        Ok((Self { renderer: None }, SAMPLE_RATE))
    }

    fn start(&mut self, renderer: Renderer) -> Result<(), ()> {
        self.renderer = Some(Mutex::new(renderer));
        Ok(())
    }
}
