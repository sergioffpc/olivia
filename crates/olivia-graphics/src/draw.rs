use olivia_math::{Color, Vec2};

/// How to draw something, built with chained setters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Draw {
    pub(crate) origin: Vec2,
    pub(crate) rotation: f32,
    pub(crate) scale: Vec2,
    pub(crate) color: Color,
}

impl Default for Draw {
    fn default() -> Self {
        Self {
            origin: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
            color: Color::WHITE,
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
}
