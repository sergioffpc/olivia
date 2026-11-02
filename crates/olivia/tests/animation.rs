//! Characters in the world, loaded from the committed stand-in, in a headless game with no GPU.
#![cfg(feature = "animation")]

use std::path::Path;
use std::time::Duration;

use olivia::animation::{Animation, Character, Play};
use olivia::ecs::{Camera, Entity, Hud, Transform};
use olivia::graphics::Draw;
use olivia::math::Vec2;
use olivia::window::Window;
use olivia::{Config, Game, GameLoop, Modules, Result, Setup, Update};

const STEP: Duration = Duration::from_nanos(16_666_666);
const PARTS: usize = 11;
/// Where the torso, the part of the root joint, comes in the drawing order.
const TORSO: usize = 6;
/// Where the upper right arm, the part of `shoulder_right`, comes in the drawing order.
const ARM: usize = 7;
/// The top-left corner of the torso from the root joint.
const TORSO_CORNER: Vec2 = Vec2::new(-120.0, -210.0);
const CAMERA: Camera = Camera {
    position: Vec2::ZERO,
    zoom: 1.0,
    rotation: 0.0,
};

/// Spawns the stand-in at the origin.
struct Stage;

impl Game for Stage {
    fn load(s: Setup<'_>) -> Result<Self> {
        let character = Character::load(s.filesystem, s.graphics, "characters/stand-in")?;
        s.world.spawn((Transform::default(), character));
        Ok(Self)
    }

    fn update(&mut self, _u: Update<'_>) -> Result<()> {
        Ok(())
    }
}

/// A game that has spawned the stand-in, its window and the stand-in's entity.
fn start() -> (GameLoop<Stage>, Window, Entity) {
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let mut window = Window::headless();
    let modules = Modules::new(&Config::new("test").assets(assets)).unwrap();
    let mut game_loop = GameLoop::<Stage>::new(&mut window, Vec2::ZERO, modules).unwrap();
    let heroes = game_loop.world().query_mut::<Entity>();
    let hero = heroes.with::<&Character>().into_iter().next().unwrap();
    (game_loop, window, hero)
}

/// The top-left corner of each part drawn, in drawing order.
fn corners(game_loop: &mut GameLoop<Stage>) -> Vec<Vec2> {
    let mesh = game_loop.mesh();
    let corners = mesh.vertices.iter().step_by(6);
    corners.map(|vertex| vertex.position).collect()
}

fn assert_close(actual: Vec2, expected: Vec2) {
    assert!(
        (actual - expected).length() < 1e-3,
        "{actual} != {expected}"
    );
}

#[test]
fn nothing_outside_the_hud_is_drawn_without_a_camera() {
    let (mut game_loop, ..) = start();
    assert_eq!(corners(&mut game_loop), []);
}

#[test]
fn every_camera_sees_every_part() {
    let (mut game_loop, ..) = start();
    game_loop.world().spawn((CAMERA,));
    game_loop.world().spawn((CAMERA,));
    assert_eq!(corners(&mut game_loop).len(), PARTS * 2);
}

#[test]
fn a_character_is_drawn_at_its_entity() {
    let (mut game_loop, _window, hero) = start();
    let world = game_loop.world();
    world.spawn((CAMERA,));
    *world.get::<&mut Transform>(hero).unwrap() = Transform::at(300.0, 500.0);
    let corners = corners(&mut game_loop);
    assert_close(corners[TORSO], Vec2::new(300.0, 500.0) + TORSO_CORNER);
}

#[test]
fn the_game_can_change_the_draw_of_a_spawned_character() {
    let (mut game_loop, _window, hero) = start();
    let world = game_loop.world();
    world.spawn((CAMERA,));
    world.get::<&mut Character>(hero).unwrap().draw = Draw::new().scale(Vec2::splat(0.5));
    assert_close(corners(&mut game_loop)[TORSO], TORSO_CORNER * 0.5);
}

#[test]
fn one_marked_hud_is_drawn_after_the_world_and_ignores_the_camera() {
    let (mut game_loop, _window, hero) = start();
    let world = game_loop.world();
    let in_hud = Character::clone(&world.get::<&Character>(hero).unwrap());
    world.spawn((Hud, Transform::default(), in_hud));
    world.spawn((Camera {
        position: Vec2::new(1000.0, 0.0),
        ..CAMERA
    },));

    let corners = corners(&mut game_loop);
    assert_eq!(corners.len(), PARTS * 2);
    assert_close(corners[TORSO], Vec2::new(-1000.0, 0.0) + TORSO_CORNER);
    assert_close(corners[PARTS + TORSO], TORSO_CORNER);
}

#[test]
fn a_game_with_a_character_runs_a_frame() {
    let (mut game_loop, mut window, _) = start();
    game_loop.world().spawn((CAMERA,));
    assert!(game_loop.run_frame(STEP, &mut window).unwrap());
    assert_eq!(corners(&mut game_loop).len(), PARTS);
}

/// Plays a wave on `hero` and runs one frame of `steps` fixed steps, which also runs the step every
/// first frame starts with. Returns the angle the frame draws the right arm at.
fn wave(game_loop: &mut GameLoop<Stage>, window: &mut Window, hero: Entity, steps: f32) -> f32 {
    game_loop.world().spawn((CAMERA,));
    let rows: Vec<[f32; 1]> = (0..20_u8).map(|row| [f32::from(row) * 0.1]).collect();
    let json =
        format!(r#"{{ "version": 1, "fps": 60, "joints": ["shoulder_right"], "rows": {rows:?} }}"#);
    let wave = Animation::parse(json.as_bytes()).unwrap();
    let world = game_loop.world();
    let mut character = world.get::<&mut Character>(hero).unwrap();
    character.play(&wave, Play::new()).unwrap();
    drop(character);

    game_loop.run_frame(STEP.mul_f32(steps), window).unwrap();
    let mesh = game_loop.mesh();
    let top_left = mesh.vertices[ARM * 6].position;
    let top_right = mesh.vertices[ARM * 6 + 1].position;
    (top_right - top_left).to_angle()
}

#[test]
fn a_character_advances_each_fixed_step_by_the_timers_dt() {
    let (mut game_loop, mut window, hero) = start();
    let angle = wave(&mut game_loop, &mut window, hero, 3.0);
    assert!((angle - 0.3).abs() < 1e-3, "{angle}");
}

#[test]
fn the_interpolation_factor_reaches_a_character_with_no_body() {
    let (mut game_loop, mut window, hero) = start();
    let angle = wave(&mut game_loop, &mut window, hero, 3.5);
    assert!((game_loop.interpolation_factor() - 0.5).abs() < 1e-3);
    assert!((angle - 0.35).abs() < 1e-3, "{angle}");
}

#[test]
fn the_stand_in_animation_plays_on_the_stand_in() {
    let (mut game_loop, _window, hero) = start();
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/animations/stand-in.json");
    let animation = Animation::parse(&std::fs::read(path).unwrap()).unwrap();
    let world = game_loop.world();
    let mut character = world.get::<&mut Character>(hero).unwrap();
    character
        .play(&animation, Play::new().looping(true))
        .unwrap();
}

#[cfg(feature = "physics")]
#[test]
fn a_body_is_drawn_between_its_last_two_steps() {
    use olivia::ecs::{ColliderBuilder, RigidBodyBuilder};

    let (mut game_loop, mut window, hero) = start();
    let world = game_loop.world();
    world
        .insert(
            hero,
            (RigidBodyBuilder::dynamic(), ColliderBuilder::ball(0.5)),
        )
        .unwrap();
    world.spawn((CAMERA,));
    game_loop.run_frame(STEP * 4, &mut window).unwrap();
    game_loop.run_frame(STEP / 2, &mut window).unwrap();

    let drawn = corners(&mut game_loop)[TORSO] - TORSO_CORNER;
    let world = game_loop.world();
    let stepped = world.get::<&Transform>(hero).unwrap().position;
    assert!(
        drawn.y < stepped.y,
        "{} is not above {}",
        drawn.y,
        stepped.y
    );
}
