//! What shapes put in a frame, on a `Graphics` with no GPU, whose window has no size.

use std::f32::consts::{FRAC_PI_2, PI};

use olivia_graphics::{Blend, Camera, Config, Draw, Filter, Graphics, View};
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

/// An L, 20 across and with a notch of 10 cut from its bottom-right corner.
const L: [Vec2; 6] = [
    Vec2::new(0.0, 0.0),
    Vec2::new(20.0, 0.0),
    Vec2::new(20.0, 10.0),
    Vec2::new(10.0, 10.0),
    Vec2::new(10.0, 20.0),
    Vec2::new(0.0, 20.0),
];

const SQUARE: [Vec2; 4] = [
    Vec2::new(0.0, 0.0),
    Vec2::new(10.0, 0.0),
    Vec2::new(10.0, 10.0),
    Vec2::new(0.0, 10.0),
];

/// The corners of the triangles that `shape` draws through `camera`.
fn triangles(camera: Camera, shape: impl FnOnce(&mut View<'_>)) -> Vec<Vec2> {
    let mut graphics = graphics();
    let mut target = graphics.target();
    shape(&mut target.view(camera));
    let vertices = target.mesh().vertices;
    vertices.iter().map(|vertex| vertex.position).collect()
}

/// The smallest and largest X and Y of `points`.
fn bounds(points: &[Vec2]) -> (Vec2, Vec2) {
    let start = (Vec2::INFINITY, Vec2::NEG_INFINITY);
    points
        .iter()
        .fold(start, |(min, max), &point| (min.min(point), max.max(point)))
}

/// The total area of the triangles in `points`.
fn area(points: &[Vec2]) -> f32 {
    let triangles = points.as_chunks::<3>().0.iter();
    triangles
        .map(|[a, b, c]| (*b - *a).perp_dot(*c - *a).abs() / 2.0)
        .sum()
}

fn assert_close(actual: Vec2, x: f32, y: f32) {
    assert!(
        (actual - Vec2::new(x, y)).length() < 0.05,
        "{actual} != ({x}, {y})"
    );
}

fn assert_bounds(points: &[Vec2], min: (f32, f32), max: (f32, f32)) {
    let (actual_min, actual_max) = bounds(points);
    assert_close(actual_min, min.0, min.1);
    assert_close(actual_max, max.0, max.1);
}

fn assert_area(points: &[Vec2], expected: f32) {
    let actual = area(points);
    assert!(
        (actual - expected).abs() < expected * 0.02,
        "{actual} != {expected}"
    );
}

#[test]
fn a_rect_covers_its_area_from_its_top_left_corner() {
    let rect = Rect::new(10.0, 20.0, 30.0, 40.0);
    let points = triangles(WORLD, |view| view.rect(rect, Draw::new()));
    assert_bounds(&points, (10.0, 20.0), (40.0, 60.0));
    assert_eq!(area(&points), 1200.0);
}

#[test]
fn a_rect_is_two_triangles() {
    let rect = Rect::new(0.0, 0.0, 30.0, 40.0);
    let points = triangles(WORLD, |view| view.rect(rect, Draw::new()));
    assert_eq!(points.len(), 6);
}

#[test]
fn origin_is_normalised_to_a_rect_and_lands_on_its_position() {
    let rect = Rect::new(10.0, 10.0, 20.0, 40.0);
    let draw = Draw::new().origin(Vec2::splat(0.5));
    let points = triangles(WORLD, |view| view.rect(rect, draw));
    assert_bounds(&points, (0.0, -10.0), (20.0, 30.0));
}

#[test]
fn a_rect_turns_around_its_pivot() {
    let rect = Rect::new(5.0, 5.0, 20.0, 10.0);
    let draw = Draw::new().rotation(FRAC_PI_2);
    let points = triangles(WORLD, |view| view.rect(rect, draw));
    assert_bounds(&points, (-5.0, 5.0), (5.0, 25.0));
}

#[test]
fn a_rect_outline_straddles_the_edge_of_the_rect() {
    let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
    let points = triangles(WORLD, |view| view.rect_outline(rect, 2.0, Draw::new()));
    assert_bounds(&points, (-1.0, -1.0), (11.0, 11.0));
    assert_area(&points, 80.0);
}

#[test]
fn a_circle_is_centred_on_its_position() {
    let draw = Draw::new();
    let points = triangles(WORLD, |view| view.circle(Vec2::new(30.0, 40.0), 10.0, draw));
    assert_bounds(&points, (20.0, 30.0), (40.0, 50.0));
    assert_area(&points, PI * 100.0);
}

#[test]
fn an_origin_replaces_the_default_pivot_of_a_circle() {
    let draw = Draw::new().origin(Vec2::ZERO);
    let points = triangles(WORLD, |view| view.circle(Vec2::ZERO, 10.0, draw));
    assert_bounds(&points, (0.0, 0.0), (20.0, 20.0));
}

#[test]
fn a_circle_outline_straddles_the_circle() {
    let draw = Draw::new();
    let points = triangles(WORLD, |view| {
        view.circle_outline(Vec2::ZERO, 10.0, 2.0, draw);
    });
    assert_bounds(&points, (-11.0, -11.0), (11.0, 11.0));
    assert_area(&points, PI * 40.0);
}

#[test]
fn a_circle_stays_round_when_a_camera_zooms_in() {
    let camera = Camera {
        zoom: 20.0,
        ..WORLD
    };
    let points = triangles(camera, |view| view.circle(Vec2::ZERO, 1.0, Draw::new()));
    assert_area(&points, PI * 400.0);
}

#[test]
fn a_circle_stays_round_when_scaled_up() {
    let draw = Draw::new().scale(Vec2::splat(-20.0));
    let points = triangles(WORLD, |view| view.circle(Vec2::ZERO, 1.0, draw));
    assert_area(&points, PI * 400.0);
}

#[test]
fn a_polygon_may_be_concave_and_its_points_are_relative_to_its_position() {
    let pos = Vec2::new(5.0, 5.0);
    let points = triangles(WORLD, |view| view.polygon(pos, &L, Draw::new()));
    assert_bounds(&points, (5.0, 5.0), (25.0, 25.0));
    assert_eq!(area(&points), 300.0);
}

#[test]
fn a_polygon_pivots_on_its_position_even_outside_its_points() {
    let shifted = SQUARE.map(|point| point + Vec2::new(10.0, 0.0));
    let draw = Draw::new().rotation(FRAC_PI_2);
    let points = triangles(WORLD, |view| view.polygon(Vec2::ZERO, &shifted, draw));
    assert_bounds(&points, (-10.0, 10.0), (0.0, 20.0));
}

#[test]
fn origin_is_normalised_to_the_box_of_a_polygon() {
    let shifted = SQUARE.map(|point| point + Vec2::new(10.0, 0.0));
    let draw = Draw::new().origin(Vec2::ONE);
    let points = triangles(WORLD, |view| view.polygon(Vec2::ZERO, &shifted, draw));
    assert_bounds(&points, (-10.0, -10.0), (0.0, 0.0));
}

#[test]
fn a_polygon_outline_closes_the_polygon() {
    let points = triangles(WORLD, |view| {
        view.polygon_outline(Vec2::ZERO, &SQUARE, 2.0, Draw::new());
    });
    assert_bounds(&points, (-1.0, -1.0), (11.0, 11.0));
    assert_area(&points, 80.0);
}

#[test]
fn a_line_runs_between_its_ends_with_its_width_around_them() {
    let (a, b) = (Vec2::new(1.0, 2.0), Vec2::new(11.0, 2.0));
    let points = triangles(WORLD, |view| view.line(a, b, 4.0, Draw::new()));
    assert_bounds(&points, (1.0, 0.0), (11.0, 4.0));
    assert_area(&points, 40.0);
}

#[test]
fn a_line_pivots_on_its_first_end() {
    let (a, b) = (Vec2::new(1.0, 2.0), Vec2::new(11.0, 2.0));
    let draw = Draw::new().rotation(FRAC_PI_2);
    let points = triangles(WORLD, |view| view.line(a, b, 4.0, draw));
    assert_bounds(&points, (-1.0, 2.0), (3.0, 12.0));
}

#[test]
fn line_width_is_in_world_units() {
    let camera = Camera { zoom: 2.0, ..WORLD };
    let (a, b) = (Vec2::ZERO, Vec2::new(10.0, 0.0));
    let points = triangles(camera, |view| view.line(a, b, 3.0, Draw::new()));
    assert_bounds(&points, (0.0, -3.0), (20.0, 3.0));
}

#[test]
fn scale_moves_a_line_but_leaves_its_width_alone() {
    let (a, b) = (Vec2::ZERO, Vec2::new(10.0, 0.0));
    let draw = Draw::new().scale(Vec2::new(2.0, 5.0));
    let points = triangles(WORLD, |view| view.line(a, b, 3.0, draw));
    assert_bounds(&points, (0.0, -1.5), (20.0, 1.5));
}

#[test]
fn scale_keeps_an_outline_even_all_the_way_round() {
    let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
    let draw = Draw::new().scale(Vec2::new(3.0, 1.0));
    let points = triangles(WORLD, |view| view.rect_outline(rect, 2.0, draw));
    assert_bounds(&points, (-1.0, -1.0), (31.0, 11.0));
}

#[test]
fn color_reaches_every_corner_of_a_shape_premultiplied() {
    let mut graphics = graphics();
    let draw = Draw::new().color(Color::new(1.0, 0.5, 0.0, 0.5));
    let mut target = graphics.target();
    target.view(WORLD).circle(Vec2::ZERO, 5.0, draw);
    let vertices = target.mesh().vertices;
    assert_ne!(vertices, []);
    for vertex in vertices {
        assert_eq!(vertex.color, [0.5, 0.25, 0.0, 0.5]);
    }
}

#[test]
fn shapes_batch_together_whatever_their_kind_or_filter() {
    let mut graphics = graphics();
    let mut target = graphics.target();
    let mut view = target.view(WORLD);
    view.rect(Rect::new(0.0, 0.0, 1.0, 1.0), Draw::new());
    let nearest = Draw::new().filter(Filter::Nearest);
    view.line(Vec2::ZERO, Vec2::X, 1.0, nearest);
    view.circle(Vec2::ZERO, 1.0, nearest);
    assert_eq!(target.mesh().batches.len(), 1);
}

#[test]
fn another_blend_breaks_a_batch_of_shapes() {
    let mut graphics = graphics();
    let mut target = graphics.target();
    let mut view = target.view(WORLD);
    view.rect(Rect::new(0.0, 0.0, 1.0, 1.0), Draw::new());
    let add = Draw::new().blend(Blend::Add);
    view.rect(Rect::new(0.0, 0.0, 1.0, 1.0), add);
    assert_eq!(target.mesh().batches.len(), 2);
}

#[test]
fn shapes_and_sprites_interleave_by_depth() {
    let mut graphics = graphics();
    let texture = graphics.texture_rgba(1, 1, &[255; 4]).unwrap();
    let mut target = graphics.target();
    let mut view = target.view(WORLD);
    view.sprite(&texture, Vec2::new(3.0, 0.0), Draw::new().depth(2.0));
    let rect = Rect::new(2.0, 0.0, 1.0, 1.0);
    view.rect(rect, Draw::new().depth(1.0));
    view.sprite(&texture, Vec2::new(1.0, 0.0), Draw::new().depth(0.0));

    let mesh = target.mesh();
    let lefts = mesh.vertices.chunks(6).map(|quad| {
        let corners = quad.iter().map(|vertex| vertex.position.x);
        corners.fold(f32::INFINITY, f32::min)
    });
    let order: Vec<f32> = lefts.collect();
    assert_eq!(order, [1.0, 2.0, 3.0]);
    assert_eq!(mesh.batches.len(), 3);
}

#[test]
fn a_shape_with_nothing_finite_or_solid_to_draw_adds_nothing() {
    let draws: [fn(&mut View<'_>); 13] = [
        |view| view.polygon(Vec2::ZERO, &[], Draw::new()),
        |view| view.polygon(Vec2::ZERO, &[], Draw::new().origin(Vec2::ONE)),
        |view| view.polygon(Vec2::ZERO, &[Vec2::X], Draw::new()),
        |view| view.polygon(Vec2::ZERO, &[Vec2::X, Vec2::Y], Draw::new()),
        |view| view.circle(Vec2::ZERO, f32::NAN, Draw::new()),
        |view| view.circle_outline(Vec2::ZERO, 1.0, f32::NAN, Draw::new()),
        |view| view.line(Vec2::ZERO, Vec2::X, 0.0, Draw::new()),
        |view| view.line(Vec2::ZERO, Vec2::X, -1.0, Draw::new()),
        |view| view.polygon_outline(Vec2::ZERO, &[], 1.0, Draw::new()),
        |view| view.polygon(Vec2::ZERO, &[Vec2::NAN; 3], Draw::new()),
        |view| view.circle(Vec2::ZERO, -1.0, Draw::new()),
        |view| view.circle(Vec2::ZERO, 1.0, Draw::new().scale(Vec2::NAN)),
        |view| view.circle(Vec2::ZERO, 1.0, Draw::new().origin(Vec2::NAN)),
    ];
    for draw in draws {
        assert_eq!(triangles(WORLD, draw), []);
    }
}

#[test]
fn a_circle_of_no_radius_does_not_break_tessellation() {
    let points = triangles(WORLD, |view| view.circle(Vec2::ZERO, 0.0, Draw::new()));
    assert_eq!(area(&points), 0.0);
}

#[test]
fn a_camera_or_scale_of_zero_does_not_break_tessellation() {
    let camera = Camera { zoom: 0.0, ..WORLD };
    let points = triangles(camera, |view| view.circle(Vec2::ZERO, 1.0, Draw::new()));
    assert_eq!(area(&points), 0.0);
    let flat = Draw::new().scale(Vec2::ZERO);
    let points = triangles(WORLD, |view| view.circle(Vec2::ZERO, 1.0, flat));
    assert_eq!(area(&points), 0.0);
}

#[test]
fn a_camera_that_is_not_finite_does_not_break_tessellation() {
    for zoom in [f32::NAN, f32::INFINITY] {
        let camera = Camera { zoom, ..WORLD };
        triangles(camera, |view| view.circle(Vec2::ZERO, 1.0, Draw::new()));
    }
}
