use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use cosmic_text::{
    Align, Attrs, AttrsOwned, Buffer, CacheKey, Family, FontSystem, Metrics, Shaping, SwashCache,
    SwashContent, SwashImage, fontdb,
};
use olivia_math::{Rect, Vec2};
use tracing::trace_span;

use crate::Error;
use crate::atlas::{Atlas, AtlasGpu, Slot};
use crate::pipeline::premultiplied;

/// How many layouts a font keeps before starting over, so ever-changing text cannot fill memory.
const MAX_LAYOUTS: usize = 1024;

/// A typeface to draw text with. Cheap to clone, and released when the last clone is dropped.
#[derive(Clone, Debug)]
pub struct Font {
    typesetter: Arc<Mutex<Typesetter>>,
}

/// What a layout is cached by, with the size and the width as bits.
type LayoutKey = (u32, Option<u32>, String);

/// One font and what turns text into its glyphs.
struct Typesetter {
    /// Holds this font alone, so no text can fall back to another.
    system: FontSystem,
    attrs: AttrsOwned,
    /// The height of a line, as a multiple of the font's size.
    line_height: f32,
    layouts: HashMap<LayoutKey, Arc<Layout>>,
    rasteriser: SwashCache,
    atlas: Atlas,
    /// Every glyph rasterised so far, or none for a blank one such as a space.
    glyphs: HashMap<CacheKey, Option<Glyph>>,
}

/// Text arranged at one size and width.
#[derive(Debug)]
pub(crate) struct Layout {
    /// The box around the text.
    pub(crate) size: Vec2,
    pub(crate) glyphs: Vec<Glyph>,
}

/// A rasterised glyph.
#[derive(Clone, Debug)]
pub(crate) struct Glyph {
    /// Where it is, from the top-left corner of the text, or from the pen while it is not in a
    /// [`Layout`].
    pub(crate) rect: Rect,
    pub(crate) slot: Slot,
}

impl fmt::Debug for Typesetter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Typesetter").finish_non_exhaustive()
    }
}

impl Typesetter {
    /// `text` at `size`, wrapped to `width` if there is one, from the cache if it is there.
    fn layout(&mut self, size: f32, width: Option<f32>, text: &str) -> Arc<Layout> {
        let key = (size.to_bits(), width.map(f32::to_bits), text.to_owned());
        if let Some(layout) = self.layouts.get(&key) {
            return Arc::clone(layout);
        }
        if self.layouts.len() == MAX_LAYOUTS {
            self.layouts.clear();
        }
        let layout = Arc::new(self.shape(size, width, text));
        self.layouts.insert(key, Arc::clone(&layout));
        layout
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "a pen stays far below 2^24 pixels from the text's corner"
    )]
    fn shape(&mut self, size: f32, width: Option<f32>, text: &str) -> Layout {
        let _span = trace_span!("text layout").entered();
        let metrics = Metrics::new(size, size * self.line_height);
        let mut buffer = Buffer::new(&mut self.system, metrics);
        buffer.set_size(width, None);
        let attrs = self.attrs.as_attrs();
        // Left even for right-to-left text, which would otherwise end at the wrapping width and
        // leave the box that `measure` reports.
        buffer.set_text(text, &attrs, Shaping::Advanced, Some(Align::Left));
        buffer.shape_until_scroll(&mut self.system, false);

        let mut bottom_right = Vec2::ZERO;
        let mut glyphs = Vec::new();
        for run in buffer.layout_runs() {
            bottom_right.x = bottom_right.x.max(run.line_w);
            bottom_right.y = run.line_top + run.line_height;
            for glyph in run.glyphs {
                let glyph = glyph.physical((0.0, run.line_y), 1.0);
                if let Some(mut placed) = self.glyph(glyph.cache_key) {
                    placed.rect.pos += Vec2::new(glyph.x as f32, glyph.y as f32);
                    glyphs.push(placed);
                }
            }
        }
        Layout {
            size: bottom_right,
            glyphs,
        }
    }

    /// The glyph of `key`, rasterised the first time it is asked for.
    fn glyph(&mut self, key: CacheKey) -> Option<Glyph> {
        if let Some(glyph) = self.glyphs.get(&key) {
            return glyph.clone();
        }
        let image = self.rasteriser.get_image_uncached(&mut self.system, key);
        let glyph = image
            .filter(|image| image.placement.width > 0 && image.placement.height > 0)
            .map(|image| self.rasterised(&image));
        self.glyphs.insert(key, glyph.clone());
        glyph
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "a glyph stays far below 2^24 pixels"
    )]
    fn rasterised(&mut self, image: &SwashImage) -> Glyph {
        let placement = image.placement;
        let rgba = match image.content {
            SwashContent::Mask => image.data.iter().flat_map(|&alpha| [alpha; 4]).collect(),
            SwashContent::Color | SwashContent::SubpixelMask => premultiplied(&image.data),
        };
        Glyph {
            rect: Rect::new(
                placement.left as f32,
                -placement.top as f32,
                placement.width as f32,
                placement.height as f32,
            ),
            slot: self.atlas.add(placement.width, placement.height, &rgba),
        }
    }
}

impl Font {
    pub(crate) fn new(bytes: &[u8], gpu: Option<AtlasGpu>) -> Result<Self, Error> {
        let mut database = fontdb::Database::new();
        let source = fontdb::Source::Binary(Arc::new(bytes.to_vec()));
        let ids = database.load_font_source(source);
        let id = *ids.first().ok_or(Error::UndecodableFont)?;
        let face = database.face(id).ok_or(Error::UndecodableFont)?;
        let (family, _) = face.families.first().ok_or(Error::UndecodableFont)?;
        let attrs = Attrs::new()
            .family(Family::Name(family))
            .style(face.style)
            .weight(face.weight)
            .stretch(face.stretch);
        let attrs = AttrsOwned::new(&attrs);
        let weight = face.weight;

        let mut system = FontSystem::new_with_locale_and_db("en-US".to_owned(), database);
        let font = system.get_font(id, weight);
        let metrics = font.as_ref().ok_or(Error::UndecodableFont)?.metrics();
        let line_height =
            (metrics.ascent - metrics.descent + metrics.leading) / f32::from(metrics.units_per_em);
        Ok(Self {
            typesetter: Arc::new(Mutex::new(Typesetter {
                system,
                attrs,
                line_height,
                layouts: HashMap::new(),
                rasteriser: SwashCache::new(),
                atlas: Atlas::new(gpu),
                glyphs: HashMap::new(),
            })),
        })
    }

    /// The box `text` fills at `size`, with a line for each line break in it.
    ///
    /// # Panics
    ///
    /// Panics if `size` is not positive and finite, or if a glyph is larger than the GPU supports.
    #[must_use]
    pub fn measure(&self, text: &str, size: f32) -> Vec2 {
        self.layout(size, None, text).size
    }

    /// The box `text` fills at `size`, wrapped to lines no wider than `width`.
    ///
    /// # Panics
    ///
    /// Panics if `size` is not positive and finite, if `width` is not positive, or if a glyph is
    /// larger than the GPU supports.
    #[must_use]
    pub fn measure_wrapped(&self, text: &str, size: f32, width: f32) -> Vec2 {
        self.layout(size, Some(width), text).size
    }

    /// How many layouts are cached.
    #[doc(hidden)]
    #[must_use]
    pub fn cached_layouts(&self) -> usize {
        self.typesetter().layouts.len()
    }

    pub(crate) fn layout(&self, size: f32, width: Option<f32>, text: &str) -> Arc<Layout> {
        assert!(
            size > 0.0 && size.is_finite(),
            "a font size must be positive and finite, not {size}"
        );
        if let Some(width) = width {
            assert!(
                width > 0.0,
                "a wrapping width must be positive, not {width}"
            );
        }
        self.typesetter().layout(size, width, text)
    }

    fn typesetter(&self) -> MutexGuard<'_, Typesetter> {
        self.typesetter
            .lock()
            .expect("an earlier panic left this font unusable")
    }
}
