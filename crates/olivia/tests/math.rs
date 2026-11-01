use olivia::math::{Color, Rect, Vec2};

#[test]
fn math_is_reachable_through_the_umbrella() {
    assert_eq!(Rect::new(1.0, 2.0, 3.0, 4.0).pos, Vec2::new(1.0, 2.0));
    assert_eq!(Color::BLACK, Color::rgb(0.0, 0.0, 0.0));
}
