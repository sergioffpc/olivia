use std::sync::Arc;

use olivia_math::Vec2;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Device,
    Extent3d, Origin3d, Queue, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor,
};

use crate::texture::Image;

/// The side of a page, unless one glyph needs more.
const PAGE: u32 = 1024;
/// The blank pixels around a glyph, so that linear filtering never reaches its neighbours.
const GUTTER: u32 = 1;

/// What an atlas needs of the GPU.
#[derive(Debug)]
pub(crate) struct AtlasGpu {
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) texture_layout: BindGroupLayout,
}

impl AtlasGpu {
    /// An empty page and what binds it to a draw.
    fn page(&self, side: u32) -> (Texture, BindGroup) {
        let texture = self.device.create_texture(&TextureDescriptor {
            label: Some("glyph atlas"),
            size: Extent3d {
                width: side,
                height: side,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&TextureViewDescriptor::default());
        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some("glyph atlas"),
            layout: &self.texture_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&view),
            }],
        });
        (texture, bind_group)
    }

    /// Copies premultiplied `rgba` into `texture` at `(x, y)`.
    fn write(
        &self,
        texture: &Texture,
        (x, y): (u32, u32),
        (width, height): (u32, u32),
        rgba: &[u8],
    ) {
        self.queue.write_texture(
            TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: Origin3d { x, y, z: 0 },
                aspect: TextureAspect::All,
            },
            rgba,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: None,
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
}

/// Where a glyph is in an atlas.
#[derive(Clone, Debug)]
pub(crate) struct Slot {
    pub(crate) image: Arc<Image>,
    /// The top-left and bottom-right corners of the glyph, normalised to the image.
    pub(crate) uv: (Vec2, Vec2),
}

/// The glyphs of one font, on pages that fill up row by row and last as long as the font. Pages
/// keep no CPU copy, as their glyphs can always be rasterised again.
#[derive(Debug)]
pub(crate) struct Atlas {
    /// None in a headless game.
    gpu: Option<AtlasGpu>,
    /// Only the last one has room.
    pages: Vec<Page>,
}

#[derive(Debug)]
struct Page {
    image: Arc<Image>,
    /// None in a headless game.
    texture: Option<Texture>,
    side: u32,
    /// Where the next glyph goes.
    x: u32,
    y: u32,
    /// The height of the tallest glyph in the row.
    row: u32,
}

impl Page {
    fn new(gpu: Option<&AtlasGpu>, side: u32) -> Self {
        let (texture, bind_group) = gpu.map(|gpu| gpu.page(side)).unzip();
        Self {
            image: Arc::new(Image::new(side, side, Vec::new(), bind_group)),
            texture,
            side,
            x: GUTTER,
            y: GUTTER,
            row: 0,
        }
    }

    /// Takes room for a glyph and returns its top-left corner, if it fits.
    fn place(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        if self.x + width + GUTTER > self.side {
            self.x = GUTTER;
            self.y += self.row + GUTTER;
            self.row = 0;
        }
        if self.x + width + GUTTER > self.side || self.y + height + GUTTER > self.side {
            return None;
        }
        let corner = (self.x, self.y);
        self.x += width + GUTTER;
        self.row = self.row.max(height);
        Some(corner)
    }
}

impl Atlas {
    pub(crate) fn new(gpu: Option<AtlasGpu>) -> Self {
        Self {
            gpu,
            pages: Vec::new(),
        }
    }

    /// Adds a glyph of premultiplied `rgba`.
    #[expect(
        clippy::cast_precision_loss,
        reason = "a GPU texture's side stays far below 2^24"
    )]
    pub(crate) fn add(&mut self, width: u32, height: u32, rgba: &[u8]) -> Slot {
        let placed = self.pages.last_mut().and_then(|page| {
            let corner = page.place(width, height)?;
            Some((page, corner))
        });
        let (page, corner) = if let Some(placed) = placed {
            placed
        } else {
            let side = PAGE.max(width.max(height) + 2 * GUTTER);
            self.pages.push(Page::new(self.gpu.as_ref(), side));
            let page = self.pages.last_mut().expect("a page was just added");
            let corner = page
                .place(width, height)
                .expect("a new page fits the glyph");
            (page, corner)
        };
        if let (Some(gpu), Some(texture)) = (&self.gpu, &page.texture) {
            gpu.write(texture, corner, (width, height), rgba);
        }
        let page_side = page.side as f32;
        let uv_start = Vec2::new(corner.0 as f32, corner.1 as f32) / page_side;
        let uv_size = Vec2::new(width as f32, height as f32) / page_side;
        Slot {
            image: Arc::clone(&page.image),
            uv: (uv_start, uv_start + uv_size),
        }
    }
}
