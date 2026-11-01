pub use glam::{Mat3, Vec2};

/// An axis-aligned rectangle: `pos` is its top-left corner, in Y-down coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub pos: Vec2,
    pub size: Vec2,
}

impl Rect {
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
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
    pub const RED: Self = Self::rgb(1.0, 0.0, 0.0);
    pub const GREEN: Self = Self::rgb(0.0, 1.0, 0.0);
    pub const BLUE: Self = Self::rgb(0.0, 0.0, 1.0);
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

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
