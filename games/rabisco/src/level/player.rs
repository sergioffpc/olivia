use olivia::{
    Result, Update,
    animation::{Character, Play},
    ecs::{
        CBody, CSprite, CTransform, ColliderBuilder, Entity, RigidBodyBuilder, World, entity_of,
    },
    input::Key,
    math::Vec2,
    physics::{self, LockedAxes},
};

use super::{
    CActor,
    layout::{CHECKPOINTS, GROUND},
    terrain::CSolid,
};
use crate::{
    assets::{Assets, Characters, ROOT_HEIGHT},
    depth,
    random::Random,
    sprite::sized,
    status::{CStatus, Outcome},
};

/// The size of the hero's box.
const HERO_SIZE: Vec2 = Vec2::new(0.7, 1.6);
/// How fast the hero runs, in metres per second.
const RUN_SPEED: f32 = 3.4;
/// How fast the hero leaves the ground on a jump, in metres per second.
const JUMP_SPEED: f32 = 9.0;
/// How fast the hero goes up on the extra jump, in the air, in metres per second.
const EXTRA_JUMP_SPEED: f32 = 8.0;
/// How long the extra jump's spin lasts: its animation.
const SPIN: f32 = 0.6;
/// How far under the feet still counts as standing.
const FOOTING: f32 = 0.08;
/// Longer than the shoot animation, which must leave before it can start again.
const SHOT_COOLDOWN: f32 = 0.45;
/// How long after the key the arm is up and the shot leaves.
const SHOT_DELAY: f32 = 0.16;
/// How long the shoot animation takes to come in over what plays.
const OVERLAY_FADE: f32 = 0.06;
/// How fast a shot leaves, in metres per second.
const SHOT_SPEED: f32 = 7.2;
/// How much gravity pulls a shot: an arc, so that it comes down on what crawls along the ground.
const SHOT_GRAVITY: f32 = 0.4;
/// The side of a shot.
const SHOT_SIZE: f32 = 0.48;
/// How far along the launcher a shot appears, from the hand.
const MUZZLE: f32 = 0.6;
/// The side of the cloud the extra jump leaves.
const FART_SIZE: f32 = 1.5;
/// How long the cloud lasts.
const FART_TIME: f32 = 0.9;

/// A shot in flight, with how long it has flown.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct CShot {
    pub(super) time: f32,
}

/// The cloud the extra jump leaves, with this much time left.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct CFart(pub(super) f32);

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
    pub(super) entity: Entity,
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

/// Runs, jumps and shoots the hero one fixed step, as the keys say while the game is playing.
pub(super) fn update(
    u: &mut Update<'_>,
    player: &Player,
    assets: &Assets,
    random: &mut Random,
    status: CStatus,
) -> Result<()> {
    let dt = u.timer.dt();
    // Once the game has ended, the keys do nothing, but the hero still falls and comes to a stop.
    let is_playing = status.outcome == Outcome::Playing;
    let left = is_playing && u.input.down(Key::Left);
    let right = is_playing && u.input.down(Key::Right);
    let jump = is_playing && (u.input.pressed(Key::Space) || u.input.pressed(Key::Up));
    let shoot = is_playing && u.input.pressed(Key::X);

    let root = u.world.get::<&CTransform>(player.entity)?.position;
    let feet = root + Vec2::new(0.0, ROOT_HEIGHT);
    let is_standing = is_standing(u.physics, u.world, feet);
    let body = u.world.get::<&CBody>(player.entity)?.0;
    let mut hero = *u.world.get::<&CHero>(player.entity)?;

    // Every timer counts down to zero.
    hero.spin = (hero.spin - dt).max(0.0);
    hero.hurt = (hero.hurt - dt).max(0.0);
    hero.invulnerable = (hero.invulnerable - dt).max(0.0);
    hero.cooldown = (hero.cooldown - dt).max(0.0);

    // It runs only while one way is held: both, or neither, stop it at once.
    let mut velocity = u.physics.body(body).linvel();
    velocity.x = match (left, right) {
        (true, false) => -RUN_SPEED,
        (false, true) => RUN_SPEED,
        _ => 0.0,
    };
    // Standing still, it keeps looking where it last went.
    if velocity.x != 0.0 {
        hero.is_facing_left = velocity.x < 0.0;
    }
    // Going up past a platform's edge is not standing on it.
    hero.is_grounded = is_standing && velocity.y >= 0.0;
    if hero.is_grounded {
        hero.has_extra_jump = true;
    }
    // A jump from the ground, or else the one extra jump in the air, which spins and leaves a
    // cloud behind.
    if jump && hero.is_grounded {
        velocity.y = -JUMP_SPEED;
        hero.is_grounded = false;
        u.audio.play(assets.sounds.hero.jump.pick(random));
    } else if jump && hero.has_extra_jump {
        velocity.y = -EXTRA_JUMP_SPEED;
        hero.has_extra_jump = false;
        hero.spin = SPIN;
        u.audio.play(assets.sounds.hero.fart.pick(random));
        spawn_fart(u.world, assets, feet);
    }
    u.physics.body_mut(body).set_linvel(velocity, true);

    // The key raises the arm over whatever plays, and the shot leaves once the arm is up.
    if shoot && hero.cooldown == 0.0 {
        hero.cooldown = SHOT_COOLDOWN;
        hero.pending_shot = Some(SHOT_DELAY);
        u.audio.play(assets.sounds.hero.shoot.pick(random));
        let overlay = Play::new().crossfade(OVERLAY_FADE);
        let mut character = u.world.get::<&mut Character>(player.entity)?;
        character.overlay(&assets.animations.hero.shoot, overlay)?;
    }
    hero.pending_shot = hero.pending_shot.map(|left| left - dt);
    if hero.pending_shot.is_some_and(|left| left <= 0.0) {
        hero.pending_shot = None;
        fire(u.world, player, assets, &hero, root)?;
    }
    *u.world.get::<&mut CHero>(player.entity)? = hero;
    Ok(())
}

/// Whether there is something solid under either side of the feet at `feet`.
fn is_standing(physics: &physics::World, world: &World, feet: Vec2) -> bool {
    [-0.3, 0.3].iter().any(|side| {
        // Started under the box, or the ray would find the hero itself.
        let start = feet + Vec2::new(*side, 0.02);
        physics
            .ray_cast(start, Vec2::Y, FOOTING)
            .and_then(|(collider, _)| entity_of(physics, collider))
            .is_some_and(|entity| world.satisfies::<&CSolid>(entity))
    })
}

/// The cloud the extra jump leaves at the hero's feet.
fn spawn_fart(world: &mut World, assets: &Assets, feet: Vec2) {
    let texture = &assets.textures.fart;
    let draw = sized(texture, Vec2::splat(FART_SIZE)).depth(depth::THING);
    let sprite = CSprite::new(texture.clone()).draw(draw);
    world.spawn((
        CActor,
        CFart(FART_TIME),
        CTransform::at(feet.x, feet.y),
        sprite,
    ));
}

/// Sends a shot from the launcher, straight ahead of how the hero looks.
fn fire(
    world: &mut World,
    player: &Player,
    assets: &Assets,
    hero: &CHero,
    root: Vec2,
) -> Result<()> {
    let hand = world.get::<&Character>(player.entity)?.joint("hand_right");
    let hand = hand.ok_or("the hero has no hand_right")?;
    let direction = if hero.is_facing_left { -1.0 } else { 1.0 };
    let centre = root + hand.offset + Vec2::new(direction * MUZZLE, 0.0);

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
    Ok(())
}
