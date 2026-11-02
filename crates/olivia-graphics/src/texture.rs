use std::fmt;
use std::sync::Arc;

use olivia_math::{Rect, Vec2};
use wgpu::BindGroup;

/// An image to draw, or a region of one. Cheap to clone, and released when the last clone and
/// region are dropped.
#[derive(Clone, Debug)]
pub struct Texture {
    pub(crate) image: Arc<Image>,
    /// The part of the image this draws, in pixels.
    region: Rect,
}

/// The pixels a texture and its regions share, and their copy on the GPU, or a page of a glyph
/// atlas, which has only the copy on the GPU.
pub(crate) struct Image {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    /// What a draw binds; none in a headless game.
    pub(crate) bind_group: Option<BindGroup>,
}

impl Image {
    pub(crate) fn new(
        width: u32,
        height: u32,
        rgba: Vec<u8>,
        bind_group: Option<BindGroup>,
    ) -> Self {
        Self {
            width,
            height,
            rgba,
            bind_group,
        }
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "a GPU texture's side stays far below 2^24"
    )]
    fn size(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }
}

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Image")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish_non_exhaustive()
    }
}

impl Texture {
    pub(crate) fn new(
        width: u32,
        height: u32,
        rgba: Vec<u8>,
        bind_group: Option<BindGroup>,
    ) -> Self {
        let image = Image::new(width, height, rgba, bind_group);
        Self {
            region: Rect {
                pos: Vec2::ZERO,
                size: image.size(),
            },
            image: Arc::new(image),
        }
    }

    /// The part of this texture inside `rect`, in pixels from its top-left corner. It shares this
    /// texture's pixels, so regions of one image batch together.
    ///
    /// # Panics
    ///
    /// Panics if `rect` is not inside this texture.
    #[must_use]
    pub fn region(&self, rect: Rect) -> Self {
        let inside = rect.pos.cmpge(Vec2::ZERO).all()
            && rect.size.cmpge(Vec2::ZERO).all()
            && (rect.pos + rect.size).cmple(self.region.size).all();
        assert!(
            inside,
            "{rect:?} is outside the texture of size {}",
            self.region.size
        );
        Self {
            image: Arc::clone(&self.image),
            region: Rect {
                pos: self.region.pos + rect.pos,
                size: rect.size,
            },
        }
    }

    /// The size this draws at, in pixels.
    #[doc(hidden)]
    #[must_use]
    pub fn size(&self) -> Vec2 {
        self.region.size
    }

    /// The pixels of the whole image, even for a region.
    #[doc(hidden)]
    #[must_use]
    pub fn pixels(&self) -> &[u8] {
        &self.image.rgba
    }

    /// The top-left and bottom-right corners of this texture, normalised to the image.
    pub(crate) fn uv(&self) -> (Vec2, Vec2) {
        let size = self.image.size();
        let start = self.region.pos / size;
        (start, start + self.region.size / size)
    }
}
