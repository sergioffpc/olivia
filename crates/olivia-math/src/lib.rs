//! Math types shared across Olivia: vectors, matrices, rectangles and colours.

pub use glam::{Mat3, Vec2};

/// An axis-aligned rectangle: `pos` is its top-left corner, in Y-down coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    /// The top-left corner.
    pub pos: Vec2,
    /// The width and height.
    pub size: Vec2,
}

impl Rect {
    /// Creates a rectangle from its top-left corner and size.
    #[must_use]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            size: Vec2::new(width, height),
        }
    }
}

/// A plain colour with channels from `0.0` to `1.0`, not premultiplied.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    /// The red channel.
    pub r: f32,
    /// The green channel.
    pub g: f32,
    /// The blue channel.
    pub b: f32,
    /// The alpha channel, where `1.0` is opaque.
    pub a: f32,
}

impl Color {
    /// Opaque black.
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    /// Opaque white.
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
    /// Opaque red.
    pub const RED: Self = Self::rgb(1.0, 0.0, 0.0);
    /// Opaque green.
    pub const GREEN: Self = Self::rgb(0.0, 1.0, 0.0);
    /// Opaque blue.
    pub const BLUE: Self = Self::rgb(0.0, 0.0, 1.0);
    /// Fully transparent black.
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// Creates a colour from all four channels.
    #[must_use]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates an opaque colour.
    #[must_use]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rect_is_built_from_its_top_left_and_size() {
        let rect = Rect::new(10.0, 20.0, 30.0, 40.0);

        assert_eq!(rect.pos, Vec2::new(10.0, 20.0));
        assert_eq!(rect.size, Vec2::new(30.0, 40.0));
    }

    #[test]
    fn rgb_colors_are_opaque() {
        assert_eq!(
            Color::rgb(0.25, 0.5, 0.75),
            Color::new(0.25, 0.5, 0.75, 1.0)
        );
    }

    #[test]
    fn color_constants_have_the_expected_channels() {
        let channels = |c: Color| [c.r, c.g, c.b, c.a];

        assert_eq!(channels(Color::BLACK), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(channels(Color::WHITE), [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(channels(Color::RED), [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(channels(Color::GREEN), [0.0, 1.0, 0.0, 1.0]);
        assert_eq!(channels(Color::BLUE), [0.0, 0.0, 1.0, 1.0]);
        assert_eq!(channels(Color::TRANSPARENT), [0.0, 0.0, 0.0, 0.0]);
    }
}
