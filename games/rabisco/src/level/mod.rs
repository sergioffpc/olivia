//! The one level: its backdrop, what the hero stands on, and who and what is on it.

mod backdrop;
mod coins;
mod enemies;
mod terrain;

use olivia::{
    ecs::{ColliderBuilder, Entity, RigidBodyBuilder, World},
    graphics::depth,
    math::Vec2,
};

use crate::assets::Textures;

/// The height of the ground's top.
const GROUND: f32 = 6.0;
/// Where the level ends.
const WIDTH: f32 = 44.0;
/// How deep the things on the level are drawn.
const THING_DEPTH: f32 = depth::PLAYFIELD + 10.0;

/// What a new game replaces.
struct Actor;

#[derive(Debug)]
pub(crate) struct LevelManager {
    backgrounds: Vec<(Entity, f32)>,
}

impl LevelManager {
    pub(crate) fn load(textures: &Textures, world: &mut World) -> Self {
        let backgrounds = backdrop::spawn(world, &textures.background);
        terrain::spawn(world, &textures.platform);
        enemies::spawn(world, textures);
        coins::spawn(world, &textures.coin);

        Self { backgrounds }
    }
}

/// A sensor box that sits still: what it touches passes through.
fn zone(size: Vec2) -> ColliderBuilder {
    ColliderBuilder::cuboid(size.x / 2.0, size.y / 2.0).sensor(true)
}

/// A sensor box on a body the game moves by its velocity.
fn moved(size: Vec2) -> (RigidBodyBuilder, ColliderBuilder) {
    let body = RigidBodyBuilder::kinematic_velocity_based();
    (body, zone(size))
}
