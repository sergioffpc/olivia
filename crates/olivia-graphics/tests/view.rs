//! What sprites put in a frame, on a `Graphics` with no GPU, whose window has no size.

use std::f32::consts::FRAC_PI_2;

use olivia_graphics::{Camera, Config, Draw, Graphics, Target, Texture, Vertex};
use olivia_math::{Color, Rect, Vec2};

/// Shows the game's coordinates as they are.
const WORLD: Camera = Camera {
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

/// A blank texture, 20 wide and 10 high.
fn texture(graphics: &mut Graphics) -> Texture {
    graphics.texture_rgba(20, 10, &[255; 800]).unwrap()
}

/// The top-left, top-right, bottom-left and bottom-right corners of the sprite at `index`.
fn corners(target: &Target<'_>, index: usize) -> [Vertex; 4] {
    let sprite = &target.vertices()[index * 6..(index + 1) * 6];
    [sprite[0], sprite[1], sprite[2], sprite[5]]
}

fn positions(target: &Target<'_>) -> [Vec2; 4] {
    corners(target, 0).map(|corner| corner.position)
}

/// The corners of one sprite at `pos` with `draw`, seen through `camera`.
fn sprite(camera: Camera, pos: Vec2, draw: Draw) -> [Vec2; 4] {
    let mut graphics = graphics();
    let texture = texture(&mut graphics);
    let mut target = graphics.target();
    target.view(camera).sprite(&texture, pos, draw);
    positions(&target)
}

fn assert_close(actual: [Vec2; 4], expected: [(f32, f32); 4]) {
    for (actual, (x, y)) in actual.into_iter().zip(expected) {
        assert!(
            (actual - Vec2::new(x, y)).length() < 1e-3,
            "{actual} != ({x}, {y})"
        );
    }
}

#[test]
fn a_frame_with_no_view_draws_nothing() {
    assert_eq!(graphics().target().vertices(), []);
}

#[test]
fn by_default_a_sprite_hangs_from_its_top_left_corner_at_its_own_size() {
    let corners = sprite(WORLD, Vec2::new(8.0, 4.0), Draw::new());
    assert_close(
        corners,
        [(8.0, 4.0), (28.0, 4.0), (8.0, 14.0), (28.0, 14.0)],
    );
}

#[test]
fn every_corner_is_in_two_triangles_with_no_gaps() {
    let mut graphics = graphics();
    let texture = texture(&mut graphics);
    let mut target = graphics.target();
    target.view(WORLD).sprite(&texture, Vec2::ZERO, Draw::new());
    let vertices = target.vertices();
    assert_eq!(vertices.len(), 6);
    // Both triangles share the diagonal from top-right to bottom-left.
    assert_eq!(vertices[1], vertices[4]);
    assert_eq!(vertices[2], vertices[3]);
}

#[test]
fn origin_is_normalised_to_the_sprite() {
    let draw = Draw::new().origin(Vec2::splat(0.5));
    assert_close(
        sprite(WORLD, Vec2::ZERO, draw),
        [(-10.0, -5.0), (10.0, -5.0), (-10.0, 5.0), (10.0, 5.0)],
    );
}

#[test]
fn scale_grows_the_sprite_from_its_pivot() {
    let draw = Draw::new().scale(Vec2::new(2.0, 3.0));
    assert_close(
        sprite(WORLD, Vec2::new(1.0, 1.0), draw),
        [(1.0, 1.0), (41.0, 1.0), (1.0, 31.0), (41.0, 31.0)],
    );
}

#[test]
fn negative_scale_flips_around_the_pivot() {
    let draw = Draw::new()
        .origin(Vec2::splat(0.5))
        .scale(Vec2::new(-1.0, 1.0));
    assert_close(
        sprite(WORLD, Vec2::ZERO, draw),
        [(10.0, -5.0), (-10.0, -5.0), (10.0, 5.0), (-10.0, 5.0)],
    );
}

#[test]
fn rotation_turns_clockwise_around_the_pivot() {
    let draw = Draw::new().rotation(FRAC_PI_2);
    assert_close(
        sprite(WORLD, Vec2::new(5.0, 5.0), draw),
        [(5.0, 5.0), (5.0, 25.0), (-5.0, 5.0), (-5.0, 25.0)],
    );
}

#[test]
fn a_sprite_goes_through_the_camera_of_its_view() {
    let camera = Camera {
        position: Vec2::new(10.0, 0.0),
        zoom: 2.0,
        rotation: 0.0,
    };
    assert_close(
        sprite(camera, Vec2::ZERO, Draw::new()),
        [(-20.0, 0.0), (20.0, 0.0), (-20.0, 20.0), (20.0, 20.0)],
    );
}

#[test]
fn a_whole_texture_maps_edge_to_edge() {
    let mut graphics = graphics();
    let texture = texture(&mut graphics);
    let mut target = graphics.target();
    target.view(WORLD).sprite(&texture, Vec2::ZERO, Draw::new());
    let uv = corners(&target, 0).map(|corner| corner.uv);
    assert_eq!(uv, [Vec2::ZERO, Vec2::X, Vec2::Y, Vec2::ONE]);
}

#[test]
fn a_region_draws_at_its_own_size_and_samples_only_itself() {
    let mut graphics = graphics();
    let region = texture(&mut graphics).region(Rect::new(10.0, 5.0, 5.0, 5.0));
    let mut target = graphics.target();
    target.view(WORLD).sprite(&region, Vec2::ZERO, Draw::new());
    let [top_left, _, _, bottom_right] = corners(&target, 0);
    assert_eq!(bottom_right.position, Vec2::new(5.0, 5.0));
    assert_eq!(top_left.uv, Vec2::new(0.5, 0.5));
    assert_eq!(bottom_right.uv, Vec2::new(0.75, 1.0));
}

#[test]
fn a_sprite_is_white_by_default_so_its_texture_shows_unchanged() {
    let mut graphics = graphics();
    let texture = texture(&mut graphics);
    let mut target = graphics.target();
    target.view(WORLD).sprite(&texture, Vec2::ZERO, Draw::new());
    for vertex in target.vertices() {
        assert_eq!(vertex.color, [1.0; 4]);
    }
}

#[test]
fn color_reaches_every_corner_premultiplied() {
    let mut graphics = graphics();
    let texture = texture(&mut graphics);
    let draw = Draw::new().color(Color::new(1.0, 0.5, 0.0, 0.5));
    let mut target = graphics.target();
    target.view(WORLD).sprite(&texture, Vec2::ZERO, draw);
    for vertex in target.vertices() {
        assert_eq!(vertex.color, [0.5, 0.25, 0.0, 0.5]);
    }
}

#[test]
fn views_draw_in_creation_order() {
    let mut graphics = graphics();
    let texture = texture(&mut graphics);
    let mut target = graphics.target();
    let mut first = target.view(WORLD);
    first.sprite(&texture, Vec2::new(1.0, 0.0), Draw::new());
    let mut second = target.view(WORLD);
    second.sprite(&texture, Vec2::new(2.0, 0.0), Draw::new());
    let mut third = target.view(WORLD);
    third.sprite(&texture, Vec2::new(3.0, 0.0), Draw::new());

    let order = [0, 1, 2].map(|index| corners(&target, index)[0].position.x);
    assert_eq!(order, [1.0, 2.0, 3.0]);
}
