//! What a game sees of a simulation through its `World`.

use std::f32::consts::PI;

use olivia_math::Vec2;
use olivia_physics::{
    ActiveEvents, ColliderBuilder, ColliderHandle, Event, RigidBodyBuilder, RigidBodyHandle, World,
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
