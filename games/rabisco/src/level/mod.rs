//! The one level: its background, what the hero stands on, and who and what is on it.

mod background;
mod coins;
mod enemies;
mod goal;
mod layout;
mod player;
mod shots;
mod terrain;

use olivia::{
    Result, Update,
    ecs::{CBody, ColliderBuilder, Entity, RigidBodyBuilder, World, entity_of},
    input::{Input, Key},
    math::Vec2,
    physics::{self, Pose, Shape},
};

use crate::{
    assets::Assets,
    random::Random,
    status::{CStatus, Outcome},
};

/// What a new game replaces.
struct CActor;

#[derive(Debug)]
pub(crate) struct Level {
    background: Vec<background::Tile>,
    player: player::Player,
    goal: goal::Goal,
}

impl Level {
    /// Where the level ends.
    pub(crate) const WIDTH: f32 = layout::WIDTH;

    /// The level as a game starts: what stays for good, the background and the terrain, and the
    /// actors a new game replaces.
    pub(crate) fn spawn(world: &mut World, assets: &Assets) -> Result<Self> {
        let background = background::spawn(world, &assets.textures.background);
        terrain::spawn(world, &assets.textures.platform);
        let (player, goal) = spawn_actors(world, assets)?;

        Ok(Self {
            background,
            player,
            goal,
        })
    }

    /// Starts a new game: every actor, shots and clouds too, makes way for those of a new one.
    pub(crate) fn restart(&mut self, world: &mut World, assets: &Assets) -> Result<()> {
        let actors: Vec<Entity> = world.query::<Entity>().with::<&CActor>().iter().collect();
        for actor in actors {
            world.despawn(actor)?;
        }
        (self.player, self.goal) = spawn_actors(world, assets)?;
        Ok(())
    }

    /// Plays the level one fixed step: the enemies walk their spans and the hero does as the keys
    /// say. While the game is playing, the shots land and the hero meets what it touches, which
    /// changes `status`. Then everyone shows as they now stand.
    pub(crate) fn update(
        &self,
        u: &mut Update<'_>,
        assets: &Assets,
        random: &mut Random,
        status: &mut CStatus,
    ) -> Result<()> {
        enemies::update(u.world, u.physics, u.timer.dt());
        player::update(u, &self.player, assets, random, *status)?;
        if status.outcome == Outcome::Playing {
            shots::update(u, assets, random)?;
            player::meet(u, &self.player, assets, random, status)?;
        }
        // Only after what the hero met, so that it shows this step's hurt or win.
        player::act(u, &self.player, assets, *status)?;
        player::thin_farts(u)?;
        goal::update(u.world, &self.goal, assets, *status)
    }

    /// The hero's entity.
    pub(crate) const fn hero(&self) -> Entity {
        self.player.entity
    }

    /// Scrolls the background for the camera at `camera`, slower than the level.
    pub(crate) fn scroll(&self, world: &mut World, camera: f32) -> Result<()> {
        background::scroll(world, &self.background, camera)
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

/// The actors of a new game, all marked with [`CActor`]: the enemies, the coins, the goal with the
/// princess, and the hero.
fn spawn_actors(world: &mut World, assets: &Assets) -> Result<(player::Player, goal::Goal)> {
    enemies::spawn(world, &assets.textures);
    coins::spawn(world, &assets.textures.coin);
    let goal = goal::spawn(world, assets)?;
    let player = player::spawn(world, &assets.characters);
    Ok((player, goal))
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

/// The entities with a collider touching `shape` at `position`.
fn touching(physics: &physics::World, shape: &dyn Shape, position: Vec2) -> Vec<Entity> {
    physics
        .overlap_shape(shape, Pose::from_translation(position))
        .into_iter()
        .filter_map(|collider| entity_of(physics, collider))
        .collect()
}
