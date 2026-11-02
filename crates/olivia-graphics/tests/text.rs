//! What fonts measure and what text puts in a frame, on a `Graphics` with no GPU.

use std::f32::consts::FRAC_PI_2;

use olivia_graphics::{Blend, Camera, Config, Draw, Error, Filter, Font, Graphics, Mesh, Vertex};
use olivia_math::{Color, Vec2};

const MONOSPACE: &[u8] = include_bytes!("../../olivia/assets/DejaVuSansMono.ttf");
/// How wide every glyph of this font is, as a multiple of its size.
const ADVANCE: f32 = 1233.0 / 2048.0;
/// How high a line of this font is, as a multiple of its size.
const LINE: f32 = 2384.0 / 2048.0;

fn graphics() -> Graphics {
    Graphics::new(Config {
        vsync: true,
        msaa: 4,
    })
}

fn font(graphics: &mut Graphics) -> Font {
    graphics.font(MONOSPACE).unwrap()
}

fn assert_close(actual: Vec2, x: f32, y: f32) {
    assert!(
        (actual - Vec2::new(x, y)).length() < 1e-2,
        "{actual} != ({x}, {y})"
    );
}

#[test]
fn a_font_decodes_from_its_bytes() {
    font(&mut graphics());
}

#[test]
fn bytes_that_are_no_font_are_an_error() {
    let error = graphics().font(b"not a font").unwrap_err();
    assert!(matches!(error, Error::UndecodableFont));
}

#[test]
fn measure_is_the_widest_line_by_the_lines_at_that_size() {
    let font = font(&mut graphics());
    assert_close(
        font.measure("abcd", 20.0),
        4.0 * 20.0 * ADVANCE,
        20.0 * LINE,
    );
    assert_close(
        font.measure("abcd\nab", 10.0),
        4.0 * 10.0 * ADVANCE,
        2.0 * 10.0 * LINE,
    );
}

#[test]
fn measure_wrapped_breaks_between_words_to_fit_the_width() {
    let font = font(&mut graphics());
    let width = 6.0 * 20.0 * ADVANCE;
    assert_close(
        font.measure_wrapped("abcd efgh ij", 20.0, width),
        4.0 * 20.0 * ADVANCE,
        3.0 * 20.0 * LINE,
    );
}

#[test]
fn a_clone_measures_like_its_font() {
    let font = font(&mut graphics());
    assert_eq!(font.clone().measure("abc", 12.0), font.measure("abc", 12.0));
}

#[test]
#[should_panic(expected = "a font size must be positive")]
fn a_font_size_of_zero_is_a_bug() {
    let _ = font(&mut graphics()).measure("abc", 0.0);
}

/// Shows the game's coordinates as they are.
const WORLD: Camera = Camera {
    position: Vec2::ZERO,
    zoom: 1.0,
    rotation: 0.0,
};

/// What `text` at size 20 puts in a frame, through `camera` with `draw`.
fn drawn(camera: Camera, text: &str, pos: Vec2, draw: Draw) -> Mesh {
    let mut graphics = graphics();
    let font = font(&mut graphics);
    let mut target = graphics.target();
    target.view(camera).text(&font, 20.0, text, pos, draw);
    target.mesh()
}

/// The smallest and largest corner over `vertices`.
fn bounds(vertices: &[Vertex]) -> (Vec2, Vec2) {
    let positions = vertices.iter().map(|vertex| vertex.position);
    let min = positions.clone().fold(Vec2::INFINITY, Vec2::min);
    (min, positions.fold(Vec2::NEG_INFINITY, Vec2::max))
}

#[test]
fn each_visible_glyph_is_two_triangles() {
    let mesh = drawn(WORLD, "a b", Vec2::ZERO, Draw::new());
    assert_eq!(mesh.vertices.len(), 12);
}

#[test]
fn empty_text_draws_nothing() {
    let mesh = drawn(WORLD, "", Vec2::ZERO, Draw::new());
    assert_eq!(mesh.vertices, []);
}

#[test]
fn a_character_the_font_lacks_draws_no_other_font() {
    let mesh = drawn(WORLD, "\u{1F600}", Vec2::ZERO, Draw::new());
    assert!(mesh.vertices.len() <= 6);
}

#[test]
fn by_default_text_hangs_from_its_top_left_corner() {
    let pos = Vec2::new(100.0, 50.0);
    let (min, max) = bounds(&drawn(WORLD, "ab", pos, Draw::new()).vertices);
    let size = Vec2::new(2.0 * 20.0 * ADVANCE, 20.0 * LINE);
    assert!(min.cmpge(pos).all() && max.cmple(pos + size).all());
}

#[test]
fn origin_is_normalised_to_the_box_of_the_text() {
    let pos = Vec2::new(100.0, 50.0);
    let draw = Draw::new().origin(Vec2::ONE);
    let (min, max) = bounds(&drawn(WORLD, "ab", pos, draw).vertices);
    let size = Vec2::new(2.0 * 20.0 * ADVANCE, 20.0 * LINE);
    assert!(min.cmpge(pos - size).all() && max.cmple(pos).all());
}

#[test]
fn scale_rotation_and_the_camera_move_text_like_a_sprite() {
    let plain = bounds(&drawn(WORLD, "a", Vec2::ZERO, Draw::new()).vertices);
    let scaled = Draw::new().scale(Vec2::splat(2.0));
    let (min, max) = bounds(&drawn(WORLD, "a", Vec2::ZERO, scaled).vertices);
    assert_close(min, plain.0.x * 2.0, plain.0.y * 2.0);
    assert_close(max, plain.1.x * 2.0, plain.1.y * 2.0);

    let turned = Draw::new().rotation(FRAC_PI_2);
    let (min, max) = bounds(&drawn(WORLD, "a", Vec2::ZERO, turned).vertices);
    assert_close(min, -plain.1.y, plain.0.x);
    assert_close(max, -plain.0.y, plain.1.x);

    let zoomed = Camera { zoom: 2.0, ..WORLD };
    let (min, max) = bounds(&drawn(zoomed, "a", Vec2::ZERO, Draw::new()).vertices);
    assert_close(min, plain.0.x * 2.0, plain.0.y * 2.0);
    assert_close(max, plain.1.x * 2.0, plain.1.y * 2.0);
}

#[test]
fn color_reaches_every_glyph_premultiplied() {
    let draw = Draw::new().color(Color::new(1.0, 0.5, 0.0, 0.5));
    for vertex in drawn(WORLD, "ab", Vec2::ZERO, draw).vertices {
        assert_eq!(vertex.color, [0.5, 0.25, 0.0, 0.5]);
    }
}

#[test]
fn a_batch_of_text_carries_its_filter_and_blend() {
    let draw = Draw::new().filter(Filter::Nearest).blend(Blend::Add);
    let mesh = drawn(WORLD, "ab", Vec2::ZERO, draw);
    assert_eq!(mesh.batches.len(), 1);
    assert_eq!(mesh.batches[0].filter, Filter::Nearest);
    assert_eq!(mesh.batches[0].blend, Blend::Add);
}

#[test]
fn depth_orders_text_among_sprites() {
    let mut graphics = graphics();
    let font = font(&mut graphics);
    let texture = graphics.texture_rgba(1, 1, &[255; 4]).unwrap();
    let mut target = graphics.target();
    let mut view = target.view(WORLD);
    let on_top = Draw::new().depth(1.0);
    view.text(&font, 20.0, "a", Vec2::new(500.0, 0.0), on_top);
    view.sprite(&texture, Vec2::ZERO, Draw::new());
    let vertices = target.mesh().vertices;
    assert_eq!(vertices[0].position, Vec2::ZERO);
    assert!(vertices[6].position.x >= 500.0);
}

#[test]
fn a_repeated_glyph_samples_one_place_and_another_glyph_another() {
    // The same fraction of a pixel into each cell, as the glyphs stand a whole pixel apart.
    let size = 10.0 / ADVANCE;
    let mut graphics = graphics();
    let font = font(&mut graphics);
    let mut target = graphics.target();
    let draw = Draw::new();
    target
        .view(WORLD)
        .text(&font, size, "aab", Vec2::ZERO, draw);
    let vertices = target.mesh().vertices;
    assert_eq!(vertices[0].uv, vertices[6].uv);
    assert_ne!(vertices[0].uv, vertices[12].uv);
    for vertex in &vertices {
        assert!(vertex.uv.cmpge(Vec2::ZERO).all() && vertex.uv.cmple(Vec2::ONE).all());
    }
}

#[test]
fn text_of_one_font_shares_a_batch_across_calls() {
    let mut graphics = graphics();
    let font = font(&mut graphics);
    let mut target = graphics.target();
    let mut view = target.view(WORLD);
    view.text(&font, 20.0, "a", Vec2::ZERO, Draw::new());
    view.text(&font, 20.0, "b", Vec2::ZERO, Draw::new());
    assert_eq!(target.mesh().batches.len(), 1);
}

#[test]
fn another_font_breaks_a_batch() {
    let mut graphics = graphics();
    let first = font(&mut graphics);
    let second = font(&mut graphics);
    let mut target = graphics.target();
    let mut view = target.view(WORLD);
    view.text(&first, 20.0, "a", Vec2::ZERO, Draw::new());
    view.text(&second, 20.0, "a", Vec2::ZERO, Draw::new());
    assert_eq!(target.mesh().batches.len(), 2);
}

#[test]
fn text_wrapped_continues_on_the_next_line() {
    let mut graphics = graphics();
    let font = font(&mut graphics);
    let mut target = graphics.target();
    let width = 2.0 * 20.0 * ADVANCE;
    let draw = Draw::new();
    target
        .view(WORLD)
        .text_wrapped(&font, 20.0, width, "ab cd", Vec2::ZERO, draw);
    let vertices = target.mesh().vertices;
    assert_eq!(vertices.len(), 24);
    assert!(vertices[12].position.y > vertices[0].position.y);
    assert!(vertices[12].position.x < vertices[6].position.x);
}

#[test]
fn glyphs_that_fill_a_page_of_the_atlas_go_on_to_another() {
    let text: String = ('a'..='z').chain('A'..='Z').collect();
    let mut graphics = graphics();
    let font = font(&mut graphics);
    let mut target = graphics.target();
    let draw = Draw::new();
    target
        .view(WORLD)
        .text(&font, 300.0, &text, Vec2::ZERO, draw);
    let mesh = target.mesh();
    assert!(mesh.batches.len() > 1);
    assert_eq!(mesh.vertices.len(), text.len() * 6);
}

#[test]
fn a_glyph_larger_than_a_page_of_the_atlas_still_draws() {
    let mut graphics = graphics();
    let font = font(&mut graphics);
    let mut target = graphics.target();
    let draw = Draw::new();
    target
        .view(WORLD)
        .text(&font, 3000.0, "W", Vec2::ZERO, draw);
    assert_eq!(target.mesh().vertices.len(), 6);
}

#[test]
fn layout_is_cached_by_size_width_and_text() {
    let font = font(&mut graphics());
    let _ = font.measure("abc", 20.0);
    let _ = font.measure("abc", 20.0);
    assert_eq!(font.cached_layouts(), 1);
    let _ = font.measure("abc", 21.0);
    let _ = font.measure("abd", 20.0);
    let _ = font.measure_wrapped("abc", 20.0, 100.0);
    assert_eq!(font.cached_layouts(), 4);
}

#[test]
fn ever_changing_text_does_not_grow_the_cache_without_limit() {
    let font = font(&mut graphics());
    for count in 0..1500 {
        let _ = font.measure(&count.to_string(), 20.0);
    }
    assert!(font.cached_layouts() <= 1024);
}

#[test]
fn wrapped_right_to_left_text_stays_inside_its_measured_box() {
    let mut graphics = graphics();
    let font = font(&mut graphics);
    let size = font.measure_wrapped("سلام", 20.0, 300.0);
    let mut target = graphics.target();
    let draw = Draw::new();
    target
        .view(WORLD)
        .text_wrapped(&font, 20.0, 300.0, "سلام", Vec2::ZERO, draw);
    let (_, max) = bounds(&target.mesh().vertices);
    assert!(max.x <= size.x + 1.0, "{max} is outside {size}");
}

#[test]
#[should_panic(expected = "a wrapping width must be positive")]
fn a_wrapping_width_of_zero_is_a_bug() {
    let _ = font(&mut graphics()).measure_wrapped("abc", 20.0, 0.0);
}

#[test]
#[should_panic(expected = "a font size must be positive")]
fn an_infinite_font_size_is_a_bug() {
    let _ = font(&mut graphics()).measure("abc", f32::INFINITY);
}
