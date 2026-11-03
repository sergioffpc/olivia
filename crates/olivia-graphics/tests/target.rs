//! The colour a target clears to, on a `Graphics` with no GPU.

use olivia_graphics::{Config, Graphics};
use olivia_math::Color;

fn graphics() -> Graphics {
    Graphics::new(Config {
        vsync: true,
        msaa: 4,
    })
}

#[test]
fn a_target_clears_to_black_unless_told_otherwise() {
    assert_eq!(graphics().target().clear_color(), Color::BLACK);
}

#[test]
fn the_last_clear_wins() {
    let mut graphics = graphics();
    let mut target = graphics.target();
    target.clear(Color::RED);
    target.clear(Color::BLUE);
    assert_eq!(target.clear_color(), Color::BLUE);
}

#[test]
fn a_target_with_no_gpu_presents_nothing() {
    assert!(!graphics().target().present());
}
