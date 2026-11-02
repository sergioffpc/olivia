use olivia_math::{Color, Vec2};
use olivia_window::Window;

use crate::Error;
use crate::camera::Camera;
use crate::gpu::Gpu;
use crate::texture::Texture;
use crate::view::{DrawCalls, Mesh, View};

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
            viewport: self.gpu.as_ref().map_or(Vec2::ZERO, Gpu::viewport),
            gpu: self.gpu.as_mut(),
            clear_color: Color::BLACK,
            calls: DrawCalls::default(),
        }
    }

    /// Decodes a PNG or JPEG image into a texture.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UndecodableImage`] if `bytes` are not a PNG or JPEG image.
    ///
    /// # Panics
    ///
    /// Panics if the image is larger than the GPU supports.
    pub fn texture(&mut self, bytes: &[u8]) -> Result<Texture, Error> {
        let image = image::load_from_memory(bytes)
            .map_err(|error| Error::UndecodableImage(error.into()))?
            .into_rgba8();
        let (width, height) = image.dimensions();
        Ok(self.upload(width, height, image.into_raw()))
    }

    /// Creates a texture from plain RGBA pixels, row by row from the top-left corner.
    ///
    /// # Errors
    ///
    /// Never fails today; the `Result` matches [`texture`](Self::texture).
    ///
    /// # Panics
    ///
    /// Panics if the texture is empty, if `pixels` does not hold 4 bytes per pixel, or if the
    /// texture is larger than the GPU supports.
    pub fn texture_rgba(
        &mut self,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Result<Texture, Error> {
        assert!(width > 0 && height > 0, "a texture cannot be empty");
        assert_eq!(
            u128::from(width) * u128::from(height) * 4,
            pixels.len() as u128,
            "a {width} by {height} texture needs 4 bytes per pixel"
        );
        Ok(self.upload(width, height, pixels.to_vec()))
    }

    fn upload(&self, width: u32, height: u32, rgba: Vec<u8>) -> Texture {
        let bind_group = self
            .gpu
            .as_ref()
            .map(|gpu| gpu.upload(width, height, &rgba));
        Texture::new(width, height, rgba, bind_group)
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
    /// The window's logical size when the frame started; zero with no GPU.
    viewport: Vec2,
    calls: DrawCalls,
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

    /// Starts drawing through `camera`, after anything drawn so far.
    pub fn view(&mut self, camera: Camera) -> View<'_> {
        View::new(&mut self.calls, camera, self.viewport)
    }

    /// What the frame has drawn so far, sorted and batched.
    #[must_use]
    pub fn mesh(&self) -> Mesh {
        self.calls.mesh()
    }

    /// Shows the frame and returns whether it reached the window.
    #[must_use]
    pub fn present(self) -> bool {
        self.gpu
            .is_some_and(|gpu| gpu.present(self.clear_color, &self.calls.mesh(), self.viewport))
    }
}
