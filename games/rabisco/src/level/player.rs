use olivia::{
    ecs::{CTransform, ColliderBuilder, Entity, RigidBodyBuilder, World},
    math::Vec2,
    physics::LockedAxes,
};

use super::{
    CActor,
    layout::{CHECKPOINTS, GROUND},
};
use crate::assets::{Characters, ROOT_HEIGHT};

/// The size of the hero's box.
const HERO_SIZE: Vec2 = Vec2::new(0.7, 1.6);

/// The hero's state, beside its body and its character.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct CHero {
    /// Whether it looks left.
    is_facing_left: bool,
    /// Whether it stands on something.
    is_grounded: bool,
    has_extra_jump: bool,
    spin: f32,
    hurt: f32,
    invulnerable: f32,
    cooldown: f32,
    pending_shot: Option<f32>,
}

impl CHero {
    /// A hero just come in: looking right, in the air, with its extra jump.
    const fn new() -> Self {
        Self {
            is_facing_left: false,
            is_grounded: false,
            has_extra_jump: true,
            spin: 0.0,
            hurt: 0.0,
            invulnerable: 0.0,
            cooldown: 0.0,
            pending_shot: None,
        }
    }
}

#[derive(Debug)]
pub(super) struct Player {
    #[expect(dead_code, reason = "read once the update phase moves the hero")]
    entity: Entity,
}

/// The hero, standing on the ground at the first checkpoint.
pub(super) fn spawn(world: &mut World, characters: &Characters) -> Player {
    let body = RigidBodyBuilder::dynamic()
        .locked_axes(LockedAxes::ROTATION_LOCKED)
        .can_sleep(false);
    // The root joint is the entity's position, and the box must end at the feet.
    let centre = ROOT_HEIGHT - HERO_SIZE.y / 2.0;
    let collider = ColliderBuilder::cuboid(HERO_SIZE.x / 2.0, HERO_SIZE.y / 2.0)
        .translation(Vec2::new(0.0, centre))
        .friction(0.0);
    let entity = world.spawn((
        CActor,
        CHero::new(),
        CTransform::at(CHECKPOINTS[0], GROUND - ROOT_HEIGHT),
        body,
        collider,
        characters.hero.clone(),
    ));

    Player { entity }
}
