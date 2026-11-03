//! Rectangles and colours are built with the expected values.

use olivia_math::{Color, Rect, Vec2};

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
