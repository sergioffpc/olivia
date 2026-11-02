//! How a camera maps the game's coordinates to the window and back.

use std::f32::consts::FRAC_PI_2;

use olivia_graphics::Camera;
use olivia_math::Vec2;

const VIEWPORT: Vec2 = Vec2::new(800.0, 600.0);

fn assert_close(actual: Vec2, expected: Vec2) {
    assert!(
        (actual - expected).length() < 1e-3,
        "{actual} != {expected}"
    );
}

#[test]
fn a_screen_camera_maps_each_point_to_itself() {
    let camera = Camera::screen(VIEWPORT);
    let point = Vec2::new(8.0, 500.0);
    assert_close(camera.world_to_screen(point, VIEWPORT), point);
    assert_close(camera.world_to_screen(Vec2::ZERO, VIEWPORT), Vec2::ZERO);
}

#[test]
fn the_camera_position_is_the_centre_of_the_screen() {
    let camera = Camera {
        position: Vec2::new(100.0, -50.0),
        zoom: 3.0,
        rotation: 1.0,
    };
    assert_close(
        camera.world_to_screen(camera.position, VIEWPORT),
        Vec2::new(400.0, 300.0),
    );
}

#[test]
fn zoom_scales_distances_from_the_centre() {
    let camera = Camera {
        position: Vec2::ZERO,
        zoom: 2.0,
        rotation: 0.0,
    };
    assert_close(
        camera.world_to_screen(Vec2::new(10.0, 20.0), VIEWPORT),
        Vec2::new(420.0, 340.0),
    );
}

#[test]
fn a_camera_turned_clockwise_shows_the_world_turned_anticlockwise() {
    let camera = Camera {
        position: Vec2::ZERO,
        zoom: 1.0,
        rotation: FRAC_PI_2,
    };
    // A point to the right of the camera ends up above the centre, with Y down.
    assert_close(
        camera.world_to_screen(Vec2::new(10.0, 0.0), VIEWPORT),
        Vec2::new(400.0, 290.0),
    );
}

#[test]
fn screen_to_world_undoes_world_to_screen() {
    let camera = Camera {
        position: Vec2::new(30.0, 40.0),
        zoom: 0.5,
        rotation: 0.7,
    };
    let point = Vec2::new(-12.0, 99.0);
    let screen = camera.world_to_screen(point, VIEWPORT);
    assert_close(camera.screen_to_world(screen, VIEWPORT), point);
}

#[test]
fn screen_to_world_finds_what_is_under_a_screen_point() {
    let camera = Camera {
        position: Vec2::new(100.0, 100.0),
        zoom: 2.0,
        rotation: 0.0,
    };
    assert_close(
        camera.screen_to_world(Vec2::ZERO, VIEWPORT),
        Vec2::new(-100.0, -50.0),
    );
}
