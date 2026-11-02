//! The world Olivia owns: entities drawn and simulated, in a headless game with no GPU.
#![cfg(all(feature = "graphics", feature = "physics"))]

use std::cell::Cell;
use std::time::Duration;

use olivia::ecs::{
    Body, Camera, ColliderBuilder, Entity, Geometry, Hud, RigidBodyBuilder, Shape, Transform,
    World, entity_of,
};
use olivia::graphics::Draw;
use olivia::math::Vec2;
use olivia::window::Window;
use olivia::{Config, Game, GameLoop, Modules, Result, Setup, Update};

const STEP: Duration = Duration::from_nanos(16_666_666);
const CAMERA: Camera = Camera {
    position: Vec2::ZERO,
    zoom: 1.0,
    rotation: 0.0,
};

thread_local! {
    static OVERLAPS: Cell<usize> = const { Cell::new(0) };
    static OWNER: Cell<Option<Entity>> = const { Cell::new(None) };
}

/// Records what the physics world finds at the origin.
struct Probe;

impl Game for Probe {
    fn load(_s: Setup<'_>) -> Result<Self> {
        Ok(Self)
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        let colliders = u.physics.overlap_point(Vec2::ZERO);
        OVERLAPS.set(colliders.len());
        OWNER.set(colliders.first().and_then(|c| entity_of(u.physics, *c)));
        Ok(())
    }
}

fn start() -> (GameLoop<Probe>, Window) {
    let mut window = Window::headless();
    let modules = Modules::new(&Config::new("test")).unwrap();
    let game_loop = GameLoop::new(&mut window, Vec2::ZERO, modules).unwrap();
    (game_loop, window)
}

fn square() -> Shape {
    Shape::new(Geometry::Rect(Vec2::ONE))
}

fn transform(world: &World, entity: Entity) -> Transform {
    *world.get::<&Transform>(entity).unwrap()
}

#[test]
fn a_rigid_body_falls_and_moves_its_transform() {
    let (mut game_loop, mut window) = start();
    let entity = game_loop.world().spawn((
        Transform::at(3.0, 1.0),
        RigidBodyBuilder::dynamic(),
        ColliderBuilder::ball(0.5),
    ));
    game_loop.run_frame(STEP * 5, &mut window).unwrap();

    let world = game_loop.world();
    assert!(world.satisfies::<&Body>(entity));
    assert!(!world.satisfies::<&RigidBodyBuilder>(entity));
    let moved = transform(world, entity);
    assert!((moved.position.x - 3.0).abs() < 1e-3);
    assert!(moved.position.y > 1.0);
}

#[test]
fn a_collider_alone_gets_a_fixed_body() {
    let (mut game_loop, mut window) = start();
    let entity = game_loop
        .world()
        .spawn((Transform::at(0.0, 1.0), ColliderBuilder::ball(0.5)));
    game_loop.run_frame(STEP * 5, &mut window).unwrap();
    assert_eq!(
        transform(game_loop.world(), entity),
        Transform::at(0.0, 1.0)
    );
}

#[test]
fn a_collider_leads_back_to_its_entity() {
    let (mut game_loop, mut window) = start();
    let entity = game_loop
        .world()
        .spawn((Transform::default(), ColliderBuilder::ball(0.5)));
    game_loop.run_frame(STEP * 3, &mut window).unwrap();
    assert_eq!(OWNER.get(), Some(entity));
}

#[test]
fn despawning_an_entity_removes_its_body() {
    let (mut game_loop, mut window) = start();
    let entity = game_loop
        .world()
        .spawn((Transform::default(), ColliderBuilder::ball(0.5)));
    game_loop.run_frame(STEP * 3, &mut window).unwrap();
    assert_eq!(OVERLAPS.get(), 1);

    game_loop.world().despawn(entity).unwrap();
    game_loop.run_frame(STEP * 3, &mut window).unwrap();
    assert_eq!(OVERLAPS.get(), 0);
}

#[test]
fn nothing_outside_the_hud_is_drawn_without_a_camera() {
    let (mut game_loop, _window) = start();
    game_loop.world().spawn((Transform::default(), square()));
    assert_eq!(game_loop.mesh().vertices.len(), 0);
}

#[test]
fn every_camera_sees_the_world() {
    let (mut game_loop, _window) = start();
    let world = game_loop.world();
    world.spawn((Transform::default(), square()));
    world.spawn((CAMERA,));
    world.spawn((CAMERA,));
    assert_eq!(game_loop.mesh().vertices.len(), 12);
}

#[test]
fn the_hud_is_drawn_after_the_world_and_ignores_the_camera() {
    let (mut game_loop, _window) = start();
    let world = game_loop.world();
    world.spawn((Hud, Transform::at(5.0, 5.0), square()));
    world.spawn((
        Transform::at(5.0, 5.0),
        square().draw(Draw::new().depth(9.0)),
    ));
    world.spawn((Camera {
        position: Vec2::new(100.0, 100.0),
        ..CAMERA
    },));

    let mesh = game_loop.mesh();
    assert_eq!(mesh.vertices.len(), 12);
    assert!(mesh.vertices[0].position.x < 0.0);
    assert!(mesh.vertices[6].position.x >= 5.0);
}

#[test]
fn a_body_is_drawn_between_its_last_two_steps() {
    let (mut game_loop, mut window) = start();
    let world = game_loop.world();
    let entity = world.spawn((
        Transform::default(),
        RigidBodyBuilder::dynamic(),
        ColliderBuilder::ball(0.5),
        square(),
    ));
    world.spawn((CAMERA,));
    game_loop.run_frame(STEP * 4, &mut window).unwrap();
    game_loop.run_frame(STEP / 2, &mut window).unwrap();

    let drawn = game_loop.mesh().vertices[0].position.y;
    assert!(drawn < transform(game_loop.world(), entity).position.y);
}
