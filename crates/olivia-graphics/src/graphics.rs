use std::sync::{Arc, Mutex, Weak};

use olivia_math::{Color, Vec2};
use olivia_window::Window;
use tracing::{debug, warn};
use wgpu::ErrorFilter;

use crate::Error;
use crate::camera::Camera;
use crate::font::{Font, Typesetter};
use crate::gpu::Gpu;
use crate::texture::{Image, Texture};
use crate::view::{DrawCalls, Mesh, View};

const WHITE: [u8; 4] = [255; 4];

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
    /// A white pixel, which shapes sample.
    white: Texture,
    /// Every image with a copy on the GPU, to rebuild it when the device is lost.
    images: Vec<Weak<Image>>,
    /// Every font, to rebuild its atlas when the device is lost.
    fonts: Vec<Weak<Mutex<Typesetter>>>,
}

/// Adds `resource` to `tracked`, first forgetting dropped ones if that would grow the list, so a
/// game that keeps creating and dropping resources does not grow it forever.
fn track<T>(tracked: &mut Vec<Weak<T>>, resource: &Arc<T>) {
    if tracked.len() == tracked.capacity() {
        tracked.retain(|weak| weak.strong_count() > 0);
    }
    tracked.push(Arc::downgrade(resource));
}

/// Calls `rebuild` on what is still alive in `tracked` and forgets the rest.
fn rebuild_alive<T>(tracked: &mut Vec<Weak<T>>, rebuild: impl Fn(&T)) {
    tracked.retain(|weak| {
        weak.upgrade()
            .inspect(|resource| rebuild(resource))
            .is_some()
    });
}

impl Graphics {
    /// Creates graphics that draw nothing until attached to a window.
    #[doc(hidden)]
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            config,
            gpu: None,
            white: Texture::new(1, 1, WHITE.to_vec(), None),
            images: Vec::new(),
            fonts: Vec::new(),
        }
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
        self.white = self.upload(1, 1, WHITE.to_vec());
        Ok(())
    }

    /// Brings back whatever the GPU lost, so that the next frame has a valid surface.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoAdapter`] or [`Error::NoDevice`] if no GPU can draw to the window any
    /// more, or [`Error::NoDevice`] if the new device cannot hold what the old one did.
    #[doc(hidden)]
    pub fn recover(&mut self) -> Result<(), Error> {
        let Some(gpu) = &mut self.gpu else {
            return Ok(());
        };
        if !gpu.is_device_lost() {
            return gpu.recreate_lost_surface();
        }
        let window = Arc::clone(&gpu.window);
        // The old swapchain goes first: a window takes only one at a time.
        self.gpu = None;
        let gpu = Gpu::new(&window, self.config)?;
        let device = gpu.device.clone();
        self.gpu = Some(gpu);

        // A new adapter may be weaker than the old one, and `wgpu` panics on an uncaught error.
        let scopes = [
            ErrorFilter::Validation,
            ErrorFilter::OutOfMemory,
            ErrorFilter::Internal,
        ]
        .map(|filter| device.push_error_scope(filter));
        self.rebuild();
        let mut failure = None;
        for scope in scopes.into_iter().rev() {
            failure = pollster::block_on(scope.pop()).or(failure);
        }
        if let Some(error) = failure {
            return Err(Error::NoDevice(error.into()));
        }
        warn!("GPU device lost and recreated");
        Ok(())
    }

    /// Copies every image to the current GPU and empties every font's atlas.
    #[doc(hidden)]
    pub fn rebuild(&mut self) {
        let gpu = self.gpu.as_ref();
        rebuild_alive(&mut self.images, |image| image.rebuild(gpu));
        rebuild_alive(&mut self.fonts, |typesetter| {
            Typesetter::rebuild(typesetter, gpu.map(Gpu::atlas_gpu));
        });
        debug!(
            images = self.images.len(),
            fonts = self.fonts.len(),
            "GPU resources rebuilt"
        );
    }

    /// How many images and fonts a rebuild would look at.
    #[doc(hidden)]
    #[must_use]
    pub fn tracked_resources(&self) -> usize {
        self.images.len() + self.fonts.len()
    }

    /// Starts a frame.
    #[doc(hidden)]
    pub fn target(&mut self) -> Target<'_> {
        Target {
            viewport: self.gpu.as_ref().map_or(Vec2::ZERO, Gpu::viewport),
            gpu: self.gpu.as_mut(),
            clear_color: Color::BLACK,
            calls: DrawCalls::new(Arc::clone(&self.white.image)),
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

    fn upload(&mut self, width: u32, height: u32, rgba: Vec<u8>) -> Texture {
        let bind_group = self
            .gpu
            .as_ref()
            .map(|gpu| gpu.upload(width, height, &rgba));
        let texture = Texture::new(width, height, rgba, bind_group);
        track(&mut self.images, &texture.image);
        texture
    }

    /// Decodes a TrueType or OpenType font.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UndecodableFont`] if `bytes` are not a font.
    pub fn font(&mut self, bytes: &[u8]) -> Result<Font, Error> {
        let font = Font::new(bytes, self.gpu.as_ref().map(Gpu::atlas_gpu))?;
        track(&mut self.fonts, &font.typesetter);
        Ok(font)
    }

    /// The underlying `wgpu` device and queue. Unstable: they change with `wgpu`, and with the
    /// device itself when it is lost, so what is made from them does not survive that.
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
