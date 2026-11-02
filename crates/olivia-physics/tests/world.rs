//! What a game sees of a simulation through its `World`.

use std::f32::consts::PI;

use olivia_math::Vec2;
use olivia_physics::{
    ActiveEvents, ColliderBuilder, ColliderHandle, Cuboid, Event, Pose, RigidBodyBuilder,
    RigidBodyHandle, World,
};

/// A dynamic ball, which has mass only because it has a collider.
fn ball(world: &mut World) -> RigidBodyHandle {
    let body = world.insert_body(RigidBodyBuilder::dynamic());
    world.insert_collider(ColliderBuilder::ball(0.5), Some(body));
    body
}

#[test]
fn one_step_advances_a_body_by_a_sixtieth_of_a_second() {
    let mut world = World::new(Vec2::new(0.0, -60.0));
    let body = ball(&mut world);

    world.step();

    let speed = world.body(body).linvel().y;
    assert!((speed + 1.0).abs() < 1e-4, "speed was {speed}");
}

/// A collider on a dynamic body, because two fixed colliders never touch.
fn reporting_ball(world: &mut World) -> (RigidBodyHandle, ColliderHandle) {
    let body = world.insert_body(RigidBodyBuilder::dynamic());
    let collider = ColliderBuilder::ball(1.0).active_events(ActiveEvents::COLLISION_EVENTS);
    (body, world.insert_collider(collider, Some(body)))
}

#[test]
fn contacts_are_reported_once_and_last_until_the_next_step() {
    let mut world = World::new(Vec2::ZERO);
    let fixed = world.insert_collider(ColliderBuilder::ball(1.0), None);
    let (body, collider) = reporting_ball(&mut world);

    world.step();
    let [Event::ContactStart(first, second)] = *world.events() else {
        panic!("expected one start, got {:?}", world.events());
    };
    assert!([(fixed, collider), (collider, fixed)].contains(&(first, second)));

    world.body_mut(body).set_linvel(Vec2::ZERO, false);
    world.step();
    assert_eq!(world.events(), []);

    world
        .body_mut(body)
        .set_translation(Vec2::new(5.0, 0.0), true);
    world.step();
    let [Event::ContactStop(first, second)] = *world.events() else {
        panic!("expected one stop, got {:?}", world.events());
    };
    assert!([(fixed, collider), (collider, fixed)].contains(&(first, second)));
}

#[test]
fn removing_a_touching_collider_reports_a_stop_with_its_stale_handle() {
    let mut world = World::new(Vec2::ZERO);
    world.insert_collider(ColliderBuilder::ball(1.0), None);
    let (_, gone) = reporting_ball(&mut world);
    world.step();

    world.remove_collider(gone);
    world.step();

    let [Event::ContactStop(first, second)] = *world.events() else {
        panic!("expected one stop, got {:?}", world.events());
    };
    assert!(first == gone || second == gone);
}

#[test]
fn interpolation_runs_from_the_pose_before_the_step_to_the_pose_after_it() {
    let mut world = World::new(Vec2::ZERO);
    let body = world.insert_body(
        RigidBodyBuilder::dynamic()
            .linvel(Vec2::new(60.0, 0.0))
            .angvel(6.0),
    );

    world.step();

    assert!(
        world
            .interpolated_position(body, 0.0)
            .abs_diff_eq(Vec2::ZERO, 1e-4)
    );
    assert!(
        world
            .interpolated_position(body, 0.5)
            .abs_diff_eq(Vec2::new(0.5, 0.0), 1e-4)
    );
    assert!(
        world
            .interpolated_position(body, 1.0)
            .abs_diff_eq(Vec2::new(1.0, 0.0), 1e-4)
    );
    assert!(world.interpolated_angle(body, 0.0).abs() < 1e-4);
    assert!((world.interpolated_angle(body, 0.5) - 0.05).abs() < 1e-4);
}

#[test]
fn interpolation_takes_the_short_way_across_the_half_turn() {
    let mut world = World::new(Vec2::ZERO);
    let body = world.insert_body(RigidBodyBuilder::dynamic().rotation(PI - 0.05).angvel(6.0));

    world.step();

    let halfway = world.interpolated_angle(body, 0.5);
    assert!((halfway.abs() - PI).abs() < 1e-3, "halfway was {halfway}");
}

#[test]
fn a_body_not_yet_stepped_interpolates_to_where_it_is() {
    let mut world = World::new(Vec2::ZERO);
    let position = Vec2::new(2.0, 3.0);
    let body = world.insert_body(RigidBodyBuilder::fixed().translation(position));

    assert_eq!(world.interpolated_position(body, 0.0), position);
}

#[test]
fn a_move_before_the_step_is_part_of_the_snapshot() {
    let mut world = World::new(Vec2::ZERO);
    let body = world.insert_body(RigidBodyBuilder::dynamic());
    world.step();

    let teleported = Vec2::new(10.0, 0.0);
    world.body_mut(body).set_translation(teleported, true);
    world.step();

    assert_eq!(world.interpolated_position(body, 0.0), teleported);
}

#[test]
#[should_panic(expected = "stale")]
fn a_stale_body_handle_panics() {
    let mut world = World::new(Vec2::ZERO);
    let body = ball(&mut world);
    world.remove_body(body);
    world.remove_body(body);
}

#[test]
#[should_panic(expected = "stale")]
fn a_stale_handle_panics_when_interpolating() {
    let mut world = World::new(Vec2::ZERO);
    let body = ball(&mut world);
    world.step();
    world.remove_body(body);
    let _ = world.interpolated_position(body, 0.5);
}

#[test]
#[should_panic(expected = "outside 0 to 1")]
fn a_factor_past_one_panics() {
    let mut world = World::new(Vec2::ZERO);
    let body = ball(&mut world);
    let _ = world.interpolated_angle(body, 1.5);
}

/// A fixed ball of radius 1 at `centre`, known to queries once the world has stepped.
fn stepped_ball(world: &mut World, centre: Vec2) -> ColliderHandle {
    let collider = world.insert_collider(ColliderBuilder::ball(1.0).translation(centre), None);
    world.step();
    collider
}

#[test]
fn a_ray_reports_the_nearest_collider_and_its_distance_in_metres() {
    let mut world = World::new(Vec2::ZERO);
    stepped_ball(&mut world, Vec2::new(10.0, 0.0));
    let near = stepped_ball(&mut world, Vec2::new(5.0, 0.0));

    let hit = world.ray_cast(Vec2::ZERO, Vec2::new(2.0, 0.0), 100.0);

    assert_eq!(hit, Some((near, 4.0)));
}

#[test]
fn a_ray_starting_inside_a_collider_hits_it_at_zero() {
    let mut world = World::new(Vec2::ZERO);
    let ball = stepped_ball(&mut world, Vec2::ZERO);

    assert_eq!(
        world.ray_cast(Vec2::ZERO, Vec2::X, 100.0),
        Some((ball, 0.0))
    );
}

#[test]
fn a_ray_stops_at_its_maximum_distance() {
    let mut world = World::new(Vec2::ZERO);
    stepped_ball(&mut world, Vec2::new(5.0, 0.0));

    assert_eq!(world.ray_cast(Vec2::ZERO, Vec2::X, 3.9), None);
}

#[test]
fn a_ray_misses_a_collider_removed_since_the_last_step() {
    let mut world = World::new(Vec2::ZERO);
    let ball = stepped_ball(&mut world, Vec2::new(5.0, 0.0));

    world.remove_collider(ball);

    assert_eq!(world.ray_cast(Vec2::ZERO, Vec2::X, 100.0), None);
}

#[test]
#[should_panic(expected = "zero ray direction")]
fn a_ray_without_a_direction_panics() {
    let world = World::new(Vec2::ZERO);
    let _ = world.ray_cast(Vec2::ZERO, Vec2::ZERO, 1.0);
}

#[test]
fn a_point_overlaps_every_collider_containing_it() {
    let mut world = World::new(Vec2::ZERO);
    let left = stepped_ball(&mut world, Vec2::new(-0.5, 0.0));
    let right = stepped_ball(&mut world, Vec2::new(0.5, 0.0));
    let far = stepped_ball(&mut world, Vec2::new(5.0, 0.0));

    let found = world.overlap_point(Vec2::ZERO);

    assert_eq!(found.len(), 2);
    assert!(found.contains(&left) && found.contains(&right) && !found.contains(&far));
}

#[test]
fn a_shape_overlaps_the_colliders_it_touches_at_its_pose() {
    let mut world = World::new(Vec2::ZERO);
    let touched = stepped_ball(&mut world, Vec2::new(3.0, 0.0));
    stepped_ball(&mut world, Vec2::new(-3.0, 0.0));

    let found = world.overlap_shape(
        &Cuboid::new(Vec2::splat(1.5)),
        Pose::from_translation(Vec2::new(1.0, 0.0)),
    );

    assert_eq!(found, [touched]);
}

/// Whether every point of `lines` is a corner of a box around `centre`.
fn outlines_box_at(lines: &[[Vec2; 2]], centre: Vec2, half_size: Vec2) -> bool {
    let mut corners = lines.iter().flatten();
    corners.all(|corner| (*corner - centre).abs().abs_diff_eq(half_size, 1e-5))
}

#[test]
fn debug_lines_outline_a_collider_and_its_bounding_box_in_metres() {
    let mut world = World::new(Vec2::ZERO);
    let centre = Vec2::new(3.0, 4.0);
    world.insert_collider(ColliderBuilder::cuboid(1.0, 2.0).translation(centre), None);
    world.step();

    let lines = world.debug_lines();

    assert_eq!(lines.len(), 8, "four edges each: {lines:?}");
    for corner in lines.into_iter().flatten() {
        assert!(
            (corner - centre)
                .abs()
                .abs_diff_eq(Vec2::new(1.0, 2.0), 1e-5)
        );
    }
}

#[test]
fn debug_lines_keep_the_poses_of_the_last_step() {
    let mut world = World::new(Vec2::ZERO);
    let body = world.insert_body(RigidBodyBuilder::dynamic());
    world.insert_collider(ColliderBuilder::cuboid(1.0, 1.0), Some(body));
    world.step();

    world
        .body_mut(body)
        .set_translation(Vec2::new(9.0, 0.0), true);
    assert!(outlines_box_at(&world.debug_lines(), Vec2::ZERO, Vec2::ONE));

    world.step();
    assert!(outlines_box_at(
        &world.debug_lines(),
        Vec2::new(9.0, 0.0),
        Vec2::ONE
    ));
}
