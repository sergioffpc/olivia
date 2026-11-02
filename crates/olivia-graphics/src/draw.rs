use olivia_math::{Color, Vec2};

/// How a texture is sampled when it is drawn at another size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Filter {
    /// Blends neighbouring pixels, which looks smooth.
    #[default]
    Linear,
    /// Takes the nearest pixel, which keeps pixel art sharp.
    Nearest,
}

/// How something drawn combines with what is under it.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Blend {
    /// Covers what is under it, as far as its alpha says.
    #[default]
    Alpha,
    /// Adds to what is under it, as far as its alpha says, which suits light and fire.
    Add,
}

/// How to draw something, built with chained setters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Draw {
    pub(crate) origin: Vec2,
    pub(crate) rotation: f32,
    pub(crate) scale: Vec2,
    pub(crate) color: Color,
    pub(crate) depth: f32,
    pub(crate) filter: Filter,
    pub(crate) blend: Blend,
}

impl Default for Draw {
    fn default() -> Self {
        Self {
            origin: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
            color: Color::WHITE,
            depth: 0.0,
            filter: Filter::Linear,
            blend: Blend::Alpha,
        }
    }
}

impl Draw {
    /// The default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the pivot, normalised to the object's box, where `(0, 0)` is its top-left corner and
    /// `(1, 1)` its bottom-right one. The default is the top-left corner.
    #[must_use]
    pub fn origin(mut self, origin: Vec2) -> Self {
        self.origin = origin;
        self
    }

    /// Sets the clockwise turn around the pivot, in radians. The default is `0.0`.
    #[must_use]
    pub fn rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// Sets the scale around the pivot, where a negative axis flips. The default is `(1, 1)`.
    #[must_use]
    pub fn scale(mut self, scale: Vec2) -> Self {
        self.scale = scale;
        self
    }

    /// Sets the colour multiplied into the object. The default is white, which leaves it unchanged.
    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the order within a [`View`](crate::View), with no fixed range, where larger is on top
    /// and ties keep call order. The default is `0.0`.
    #[must_use]
    pub fn depth(mut self, depth: f32) -> Self {
        self.depth = depth;
        self
    }

    /// Sets how the texture is sampled. The default is [`Filter::Linear`].
    #[must_use]
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filter = filter;
        self
    }

    /// Sets how the object combines with what is under it. The default is [`Blend::Alpha`].
    #[must_use]
    pub fn blend(mut self, blend: Blend) -> Self {
        self.blend = blend;
        self
    }
}
