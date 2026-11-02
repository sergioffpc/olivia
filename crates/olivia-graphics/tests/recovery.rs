//! What rebuilding the GPU resources does to a game's textures and fonts, on a `Graphics` with no GPU.

use olivia_graphics::{Camera, Config, Draw, Font, Graphics, Vertex};
use olivia_math::{Rect, Vec2};

const MONOSPACE: &[u8] = include_bytes!("../../olivia/assets/DejaVuSansMono.ttf");
const PIXELS: [u8; 16] = [200; 16];
const SCREEN: Camera = Camera {
    position: Vec2::ZERO,
    zoom: 1.0,
    rotation: 0.0,
};

fn graphics() -> Graphics {
    Graphics::new(Config {
        vsync: true,
        msaa: 4,
    })
}

/// The vertices of `text` drawn with `font` as the only thing in a frame.
fn text(graphics: &mut Graphics, font: &Font) -> Vec<Vertex> {
    let mut target = graphics.target();
    target
        .view(SCREEN)
        .text(font, 16.0, "Olivia", Vec2::ZERO, Draw::new());
    target.mesh().vertices
}

#[test]
fn a_texture_and_its_regions_survive_a_rebuild() {
    let mut graphics = graphics();
    let texture = graphics.texture_rgba(2, 2, &PIXELS).unwrap();
    let region = texture.region(Rect::new(1.0, 0.0, 1.0, 2.0));
    graphics.rebuild();
    assert_eq!(texture.pixels(), PIXELS);
    assert_eq!(region.size(), Vec2::new(1.0, 2.0));
    assert!(std::ptr::eq(texture.pixels(), region.pixels()));

    let mut target = graphics.target();
    let mut view = target.view(SCREEN);
    view.sprite(&texture, Vec2::ZERO, Draw::new());
    view.sprite(&region, Vec2::ZERO, Draw::new());
    assert_eq!(target.mesh().batches.len(), 1);
}

#[test]
fn a_font_forgets_its_layouts_and_draws_the_same_after_a_rebuild() {
    let mut graphics = graphics();
    let font = graphics.font(MONOSPACE).unwrap();
    let before = text(&mut graphics, &font);
    let size = font.measure("Olivia", 16.0);
    graphics.rebuild();
    assert_eq!(font.cached_layouts(), 0);
    assert_eq!(text(&mut graphics, &font), before);
    assert_eq!(font.measure("Olivia", 16.0), size);
}

#[test]
fn a_rebuild_forgets_what_the_game_dropped() {
    let mut graphics = graphics();
    let kept = graphics.texture_rgba(2, 2, &PIXELS).unwrap();
    let dropped = graphics.texture_rgba(2, 2, &PIXELS).unwrap();
    let font = graphics.font(MONOSPACE).unwrap();
    assert_eq!(graphics.tracked_resources(), 3);
    drop(dropped);
    drop(font);
    graphics.rebuild();
    assert_eq!(graphics.tracked_resources(), 1);
    assert_eq!(kept.pixels(), PIXELS);
}

#[test]
fn what_the_game_dropped_does_not_pile_up_between_rebuilds() {
    let mut graphics = graphics();
    for _ in 0..1000 {
        graphics.texture_rgba(2, 2, &PIXELS).unwrap();
    }
    assert!(graphics.tracked_resources() < 1000);
}

#[test]
fn a_clone_keeps_a_texture_tracked() {
    let mut graphics = graphics();
    let texture = graphics.texture_rgba(2, 2, &PIXELS).unwrap();
    let clone = texture.clone();
    drop(texture);
    graphics.rebuild();
    assert_eq!(graphics.tracked_resources(), 1);
    assert_eq!(clone.pixels(), PIXELS);
}

#[test]
fn a_game_with_no_gpu_has_nothing_to_recover() {
    assert!(graphics().recover().is_ok());
}
