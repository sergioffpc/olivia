use olivia::{
    Result, Update,
    audio::Audio,
    ecs::{CSprite, CTransform, ColliderBuilder, Entity, RigidBodyBuilder, World},
    math::Vec2,
    physics::{self, Ball},
};

use super::{CActor, enemies::CEnemy, terrain::CSolid, touching};
use crate::{assets::Assets, depth, random::Random, sprite::sized};

/// How fast a shot leaves, in metres per second.
const SHOT_SPEED: f32 = 7.2;
/// How much gravity pulls a shot: an arc, so that it comes down on what crawls along the ground.
const SHOT_GRAVITY: f32 = 0.4;
/// The side of a shot.
const SHOT_SIZE: f32 = 0.48;
/// How long a shot flies before it is gone, in seconds.
const SHOT_TIME: f32 = 1.2;

/// A shot in flight, with how long it has flown.
#[derive(Clone, Copy, Debug, PartialEq)]
struct CShot {
    time: f32,
}

/// A shot leaving `centre`, flying rightwards if `direction` is 1 or leftwards if it is -1.
pub(super) fn spawn(world: &mut World, assets: &Assets, centre: Vec2, direction: f32) {
    // Mirrored with the hero, so that it points where it flies.
    let texture = &assets.textures.projectile;
    let draw = sized(texture, Vec2::splat(SHOT_SIZE));
    let draw = draw
        .scale(draw.scale_value() * Vec2::new(direction, 1.0))
        .depth(depth::THING);
    let body = RigidBodyBuilder::dynamic()
        .gravity_scale(SHOT_GRAVITY)
        .linvel(Vec2::new(direction * SHOT_SPEED, 0.0));
    world.spawn((
        CActor,
        CShot { time: 0.0 },
        CTransform::at(centre.x, centre.y),
        body,
        ColliderBuilder::ball(SHOT_SIZE / 2.0).sensor(true),
        CSprite::new(texture.clone()).draw(draw),
    ));
}

/// Flies every shot one fixed step: what it hits, and whether it is gone.
pub(super) fn update(u: &mut Update<'_>, assets: &Assets, random: &mut Random) -> Result<()> {
    let dt = u.timer.dt();
    // Collected first, as the world changes while the shots land.
    let shots: Vec<(Entity, Vec2, f32)> = u
        .world
        .query_mut::<(Entity, &CTransform, &mut CShot)>()
        .into_iter()
        .map(|(entity, transform, shot)| {
            shot.time += dt;
            (entity, transform.position, shot.time)
        })
        .collect();
    for (shot, position, time) in shots {
        land(
            u.world, u.physics, u.audio, assets, random, shot, position, time,
        )?;
    }
    Ok(())
}

/// What the shot `shot` at `position`, `time` seconds into its flight, does this step.
#[allow(clippy::too_many_arguments)]
fn land(
    world: &mut World,
    physics: &physics::World,
    audio: &mut Audio,
    assets: &Assets,
    random: &mut Random,
    shot: Entity,
    position: Vec2,
    time: f32,
) -> Result<()> {
    let touched = touching(physics, &Ball::new(SHOT_SIZE / 2.0), position);
    // An enemy it touches is gone, and so is the shot.
    let enemy = touched.iter().find(|e| world.satisfies::<&CEnemy>(**e));
    if let Some(enemy) = enemy {
        world.despawn(*enemy)?;
        audio.play(assets.sounds.hit.pick(random));
    }
    // It also stops at anything solid, and once it has flown long enough.
    let is_stopped = touched.iter().any(|e| world.satisfies::<&CSolid>(*e));
    if enemy.is_some() || is_stopped || time > SHOT_TIME {
        world.despawn(shot)?;
    }
    Ok(())
}
