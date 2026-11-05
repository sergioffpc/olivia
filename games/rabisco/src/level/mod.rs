//! The one level: its background, what the hero stands on, and who and what is on it.

mod background;
mod coins;
mod enemies;
mod goal;
mod layout;
mod player;
mod terrain;

use olivia::{
    Result,
    ecs::{ColliderBuilder, RigidBodyBuilder, World},
    math::Vec2,
};

use crate::assets::Assets;

/// What a new game replaces.
struct CActor;

#[derive(Debug)]
#[expect(dead_code, reason = "read once the update phase plays the level")]
pub(crate) struct Level {
    background: Vec<background::Tile>,
    player: player::Player,
    goal: goal::Goal,
}

impl Level {
    pub(crate) fn spawn(world: &mut World, assets: &Assets) -> Result<Self> {
        let Assets {
            characters,
            textures,
            ..
        } = assets;
        let background = background::spawn(world, &textures.background);
        terrain::spawn(world, &textures.platform);
        enemies::spawn(world, textures);
        coins::spawn(world, &textures.coin);
        let goal = goal::spawn(world, assets)?;
        let player = player::spawn(world, characters);

        Ok(Self {
            background,
            player,
            goal,
        })
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
