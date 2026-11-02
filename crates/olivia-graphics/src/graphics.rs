use olivia_math::Color;
use olivia_window::Window;

use crate::Error;
use crate::gpu::Gpu;

/// How drawing runs, fixed before the window opens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    /// Whether drawing follows vsync.
    pub vsync: bool,
    /// The MSAA sample count, where 1 disables it.
    pub msaa: u32,
}

/// The game's graphics.
#[derive(Debug)]
pub struct Graphics {
    config: Config,
    /// The GPU drawing to the window; none until [`attach`](Self::attach), so never in a headless
    /// game, whose frames draw nothing.
    gpu: Option<Gpu>,
}

impl Graphics {
    /// Creates graphics that draw nothing until attached to a window.
    #[doc(hidden)]
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config, gpu: None }
    }

    /// Starts drawing to `window`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoAdapter`] or [`Error::NoDevice`] if no GPU can draw to it.
    ///
    /// # Panics
    ///
    /// Panics on a headless window.
    #[doc(hidden)]
    pub fn attach(&mut self, window: &Window) -> Result<(), Error> {
        self.gpu = Some(Gpu::new(window.raw(), self.config)?);
        Ok(())
    }

    /// Starts a frame.
    #[doc(hidden)]
    pub fn target(&mut self) -> Target<'_> {
        Target {
            gpu: self.gpu.as_mut(),
            clear_color: Color::BLACK,
        }
    }

    /// The underlying `wgpu` device and queue. Unstable: they change with `wgpu`.
    ///
    /// # Panics
    ///
    /// Panics in a headless game.
    #[must_use]
    pub fn raw(&self) -> (&wgpu::Device, &wgpu::Queue) {
        let gpu = self.gpu.as_ref().expect("a headless game has no GPU");
        (&gpu.device, &gpu.queue)
    }
}

/// What one frame draws, until it is presented.
#[doc(hidden)]
#[derive(Debug)]
pub struct Target<'a> {
    gpu: Option<&'a mut Gpu>,
    clear_color: Color,
}

impl Target<'_> {
    /// The colour the frame clears to.
    #[must_use]
    pub fn clear_color(&self) -> Color {
        self.clear_color
    }

    /// Sets the colour the frame clears to.
    pub fn clear(&mut self, color: Color) {
        self.clear_color = color;
    }

    /// Shows the frame and returns whether it reached the window.
    #[must_use]
    pub fn present(self) -> bool {
        self.gpu.is_some_and(|gpu| gpu.present(self.clear_color))
    }
}
