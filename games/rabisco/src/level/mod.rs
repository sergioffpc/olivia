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
    ecs::{CBody, ColliderBuilder, Entity, RigidBodyBuilder, World},
    input::{Input, Key},
    math::Vec2,
};

use crate::{
    assets::Assets,
    status::{CStatus, Outcome},
};

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

    pub(crate) fn restart(&mut self, world: &mut World, assets: &Assets) -> Result<()> {
        let actors: Vec<Entity> = world.query::<Entity>().with::<&CActor>().iter().collect();
        for actor in actors {
            world.despawn(actor)?;
        }
        self.player = player::spawn(world, &assets.characters);
        Ok(())
    }

    /// The status to play this update on, or `None` while there is nothing to play yet. When the
    /// last game has ended and Enter is pressed, it starts a new one first, and gives `None` until
    /// the new hero has its body, which the physics adds on the step after its entity.
    pub(crate) fn ready_to_play(
        &mut self,
        world: &mut World,
        input: &Input,
        assets: &Assets,
        status: Entity,
    ) -> Result<Option<CStatus>> {
        let mut current = *world.get::<&CStatus>(status)?;
        if current.outcome != Outcome::Playing && input.pressed(Key::Enter) {
            self.restart(world, assets)?;
            current = CStatus::new();
            *world.get::<&mut CStatus>(status)? = current;
        }
        if !world.satisfies::<&CBody>(self.player.entity) {
            return Ok(None);
        }
        Ok(Some(current))
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
