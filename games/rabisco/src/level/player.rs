use olivia::{
    Result, Update,
    animation::{Character, Play},
    ecs::{
        CBody, CSprite, CTransform, ColliderBuilder, Entity, RigidBodyBuilder, World, entity_of,
    },
    input::Key,
    math::{Color, Vec2},
    physics::{self, Cuboid, LockedAxes},
};

use super::{
    CActor,
    coins::CCoin,
    enemies::CEnemy,
    goal::{CEarshot, CGoal},
    layout::{CHECKPOINTS, GROUND},
    shots,
    terrain::{CPit, CSolid},
    touching,
};
use crate::{
    assets::{Assets, Characters, PART_SCALE, ROOT_HEIGHT},
    depth,
    random::Random,
    sprite::sized,
    status::{CStatus, Outcome},
};

/// The size of the hero's box.
const HERO_SIZE: Vec2 = Vec2::new(0.7, 1.6);
/// How fast the hero runs, in metres per second.
const RUN_SPEED: f32 = 3.4;
/// How much harder gravity pulls the hero than anything else: a jump as high as the level needs,
/// but quick, so that it does not float as on the moon.
const HERO_GRAVITY: f32 = 2.5;
/// How fast the hero leaves the ground on a jump, in metres per second: about 2.6 m up, over the
/// low platforms, 2.2 m above the ground, but short of the high ones, 3.4 m above it.
const JUMP_SPEED: f32 = 11.3;
/// How fast the hero goes up on the extra jump, in the air, in metres per second: about 1.3 m more,
/// enough for the high platforms.
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
/// How far along the launcher a shot appears, from the hand.
const MUZZLE: f32 = 0.6;
/// The side of the cloud the extra jump leaves.
const FART_SIZE: f32 = 1.5;
/// How long the cloud lasts.
const FART_TIME: f32 = 0.9;
/// How long one animation takes to give way to the next.
const CROSSFADE: f32 = 0.1;
/// How many times a second the hero blinks while nothing can hurt it.
const BLINKS_PER_SECOND: f64 = 8.0;
/// How long the hero shows it is hurt.
const HURT: f32 = 0.7;
/// How long after being hurt nothing can hurt the hero again, so that one touch costs one life.
const INVULNERABLE: f32 = 1.6;

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
        .gravity_scale(HERO_GRAVITY)
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

    shots::spawn(world, assets, centre, direction);
    Ok(())
}

/// Whatever the hero's box touches happens to it: coins are picked up, enemies hurt, the pit
/// swallows, the princess calls from earshot and the cage wins the game.
pub(super) fn meet(
    u: &mut Update<'_>,
    player: &Player,
    assets: &Assets,
    random: &mut Random,
    status: &mut CStatus,
) -> Result<()> {
    let root = u.world.get::<&CTransform>(player.entity)?.position;
    let centre = root + Vec2::new(0.0, ROOT_HEIGHT - HERO_SIZE.y / 2.0);
    let area = Cuboid::new(HERO_SIZE / 2.0);
    let mut is_hurt = false;
    let mut has_fallen = false;
    let mut has_won = false;
    for entity in touching(u.physics, &area, centre) {
        if u.world.satisfies::<&CCoin>(entity) {
            u.world.despawn(entity)?;
            status.score += 1;
            u.audio.play(assets.sounds.pickup.pick(random));
        }
        is_hurt |= u.world.satisfies::<&CEnemy>(entity);
        has_fallen |= u.world.satisfies::<&CPit>(entity);
        has_won |= u.world.satisfies::<&CGoal>(entity);
        // She calls only once a game.
        if u.world.satisfies::<&CEarshot>(entity) && !status.has_called {
            status.has_called = true;
            u.audio.play(assets.sounds.princess.help.pick(random));
        }
    }

    let invulnerable = u.world.get::<&CHero>(player.entity)?.invulnerable;
    if has_fallen {
        // A fall always costs a life, and the hero stands again at the last checkpoint passed.
        lose_life(u, player, assets, random, status)?;
        let reached = CHECKPOINTS.iter().rev().find(|x| **x <= root.x);
        let x = *reached.unwrap_or(&CHECKPOINTS[0]);
        let body = u.world.get::<&CBody>(player.entity)?.0;
        let body = u.physics.body_mut(body);
        body.set_translation(Vec2::new(x, GROUND - ROOT_HEIGHT), true);
        body.set_linvel(Vec2::ZERO, true);
    } else if is_hurt && invulnerable == 0.0 {
        lose_life(u, player, assets, random, status)?;
    }
    // Reaching the cage frees the princess: the cage is gone.
    if has_won && status.outcome == Outcome::Playing {
        status.outcome = Outcome::Won;
        u.audio.play(assets.sounds.hero.win.pick(random));
        let cage: Vec<Entity> = u.world.query::<Entity>().with::<&CGoal>().iter().collect();
        for entity in cage {
            u.world.despawn(entity)?;
        }
    }
    Ok(())
}

/// Takes a life from the hero, hurting it for a while, and ends the game on the last one.
fn lose_life(
    u: &mut Update<'_>,
    player: &Player,
    assets: &Assets,
    random: &mut Random,
    status: &mut CStatus,
) -> Result<()> {
    status.lives -= 1;
    let mut hero = u.world.get::<&mut CHero>(player.entity)?;
    hero.hurt = HURT;
    hero.invulnerable = INVULNERABLE;
    drop(hero);
    if status.lives == 0 {
        status.outcome = Outcome::Lost;
        u.audio.play(assets.sounds.gameover.pick(random));
    } else {
        u.audio.play(assets.sounds.hero.hurt.pick(random));
    }
    Ok(())
}

/// Shows the hero as it stands: the animation of what it does, facing where it looks, blinking
/// while nothing can hurt it.
pub(super) fn act(
    u: &mut Update<'_>,
    player: &Player,
    assets: &Assets,
    status: CStatus,
) -> Result<()> {
    let moves = &assets.animations.hero;
    let hero = *u.world.get::<&CHero>(player.entity)?;
    let body = u.world.get::<&CBody>(player.entity)?.0;
    let is_moving = u.physics.body(body).linvel().x != 0.0;
    // The first that holds wins: how the game ended, then being hurt, the extra jump's spin, being
    // in the air, and last walking or standing, the only ones that loop.
    let (chosen, looping) = if status.outcome == Outcome::Won {
        (&moves.win, false)
    } else if status.outcome == Outcome::Lost || hero.hurt > 0.0 {
        (&moves.hurt, false)
    } else if hero.spin > 0.0 {
        (&moves.double_jump, false)
    } else if !hero.is_grounded {
        (&moves.jump, false)
    } else if is_moving {
        (&moves.walk, true)
    } else {
        (&moves.idle, true)
    };
    let mut character = u.world.get::<&mut Character>(player.entity)?;
    // Asking for what already plays does nothing, so this can run every step.
    character.play(chosen, Play::new().looping(looping).crossfade(CROSSFADE))?;
    // The shot's arm comes down once its animation is over.
    if character.finished(&moves.shoot) {
        character.remove_overlay(&moves.shoot, OVERLAY_FADE);
    }

    let scale = if hero.is_facing_left {
        Vec2::new(-PART_SCALE, PART_SCALE)
    } else {
        Vec2::splat(PART_SCALE)
    };
    // Shown and hidden by turns, on the game's whole time, only while the game is playing.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // time is never negative
    let is_blinking = hero.invulnerable > 0.0
        && status.outcome == Outcome::Playing
        && ((u.timer.elapsed() * BLINKS_PER_SECOND) as u64).is_multiple_of(2);
    let alpha = if is_blinking { 0.0 } else { 1.0 };
    character.draw = character
        .draw
        .scale(scale)
        .color(Color::new(1.0, 1.0, 1.0, alpha));
    Ok(())
}

/// Clouds grow to twice their size, thin out and go.
pub(super) fn thin_farts(u: &mut Update<'_>) -> Result<()> {
    let dt = u.timer.dt();
    // Gathered first, as nothing can go while the query runs.
    let mut gone = Vec::new();
    for (entity, CFart(left), sprite) in u.world.query_mut::<(Entity, &mut CFart, &mut CSprite)>() {
        *left -= dt;
        let life = (*left / FART_TIME).max(0.0);
        let size = FART_SIZE * (2.0 - life);
        let scale = Vec2::splat(size) / sprite.texture.size();
        sprite.draw = sprite
            .draw
            .scale(scale)
            .color(Color::new(1.0, 1.0, 1.0, life));
        if *left <= 0.0 {
            gone.push(entity);
        }
    }
    for entity in gone {
        u.world.despawn(entity)?;
    }
    Ok(())
}
