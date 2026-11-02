use olivia::animation::{Animation, Character, Play};
use olivia::audio::{Audio, Play as PlaySound, Sound};
use olivia::ecs::{
    Body, Camera, Clear, ColliderBuilder, ColliderLines, Entity, Hud, RigidBodyBuilder, Sprite,
    Transform, World, entity_of,
};
use olivia::filesystem::Filesystem;
use olivia::graphics::{Draw, Graphics, Texture, depth};
use olivia::input::Key;
use olivia::math::{Color, Rect, Vec2};
use olivia::physics::{self, Ball, Cuboid, LockedAxes, Pose, Shape};
use olivia::window::WindowEvent;
use olivia::{Config, Game, Result, Setup, Update};

use crate::level::{
    CHECKPOINTS, COIN_SIZE, COINS, ENEMIES, GOAL, GOAL_SIZE, GRAVITY, GROUND, HELP_DISTANCE,
    HERO_SIZE, Kind, PIT, PIXELS_PER_METRE, PLATFORMS, ROOT_HEIGHT, SHOT_SIZE, VIEW, WIDTH, WINDOW,
};

/// The most lives the hero has.
const LIVES: u32 = 3;
/// Parts are 8 pixels per millimetre of sheet: this turns them into metres.
const PART_SCALE: f32 = 0.0011;
const JUMP_SPEED: f32 = 9.0;
const EXTRA_JUMP_SPEED: f32 = 8.0;
const RUN_SPEED: f32 = 3.4;
const SHOT_SPEED: f32 = 7.2;
/// How much gravity pulls a shot: an arc, so that it comes down on what crawls along the ground.
const SHOT_GRAVITY: f32 = 0.4;
const SHOT_TIME: f32 = 1.2;
/// Longer than the shoot animation, which must leave before it can start again.
const SHOT_COOLDOWN: f32 = 0.45;
/// How long after the key the arm is up and the shot leaves.
const SHOT_DELAY: f32 = 0.16;
/// How far along the launcher a shot appears, from the hand.
const MUZZLE: f32 = 0.6;
/// How long the extra jump's spin lasts: its animation.
const SPIN: f32 = 0.6;
const HURT: f32 = 0.7;
const INVULNERABLE: f32 = 1.6;
const BOB_SPEED: f32 = 2.5;
const BOB_HEIGHT: f32 = 0.5;
/// How far under the feet still counts as standing.
const FOOTING: f32 = 0.08;
const CROSSFADE: f32 = 0.1;
const OVERLAY_FADE: f32 = 0.06;

const HERO_DEPTH: f32 = depth::PLAYFIELD + 20.0;
const THING_DEPTH: f32 = depth::PLAYFIELD + 10.0;
const GOAL_DEPTH: f32 = depth::PLAYFIELD + 4.0;
const CAPTIVE_DEPTH: f32 = depth::PLAYFIELD + 2.0;
const PLATFORM_DEPTH: f32 = depth::PLAYFIELD + 1.0;
const FART_TIME: f32 = 0.9;
const FART_SIZE: f32 = 1.5;
/// How much slower the background moves than the camera.
const PARALLAX: f32 = 0.6;
const BLINKS_PER_SECOND: f64 = 8.0;
const DIGITS: usize = 3;
const HUD_SIZE: f32 = 52.0;
const HUD_MARGIN: f32 = 16.0;
const DIAGNOSTICS_COLOUR: Color = Color::rgb(1.0, 0.9, 0.2);

/// How the game stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Still running.
    Playing,
    /// Saved.
    Won,
    /// No lives left.
    Lost,
}

/// What a game has come to, on an entity of its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Status {
    /// Lives left.
    pub lives: u32,
    /// Things picked up.
    pub score: u32,
    /// How it stands.
    pub phase: Phase,
    /// Whether who is held has called for help.
    pub has_called: bool,
}

/// The hero's state, beside its body and its character.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Hero {
    /// Whether it looks left.
    pub is_facing_left: bool,
    /// Whether it stands on something.
    pub is_grounded: bool,
    has_extra_jump: bool,
    spin: f32,
    hurt: f32,
    invulnerable: f32,
    cooldown: f32,
    pending_shot: Option<f32>,
}

/// An enemy walking its stretch.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Enemy {
    kind: Kind,
    span: (f32, f32),
    direction: f32,
    time: f32,
}

/// A shot in flight.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shot {
    time: f32,
}

/// A collider that stops shots.
struct Solid;
/// What the hero picks up by touching it.
struct Coin;
/// The place that saves who is held.
struct Goal;
/// The stretch close enough to hear who is held.
struct Earshot;
/// What swallows a falling hero.
struct Pit;
/// Who is held.
struct Captive;
/// A cloud with this much time left.
struct Fart(f32);
/// What a new game replaces.
struct Actor;

/// The sprite's draw, scaled to `size` and centred.
fn sized(texture: &Texture, size: Vec2) -> Draw {
    Draw::new()
        .origin(Vec2::splat(0.5))
        .scale(size / texture.size())
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

fn read_texture(filesystem: &Filesystem, graphics: &mut Graphics, path: &str) -> Result<Texture> {
    Ok(graphics.texture(&filesystem.read(path)?)?)
}

fn read_animation(filesystem: &Filesystem, path: &str) -> Result<Animation> {
    Ok(Animation::parse(&filesystem.read(path)?)?)
}

struct Textures {
    life: Texture,
    life_lost: Texture,
    digits: Vec<Texture>,
    projectile: Texture,
    fart: Texture,
    coin: Texture,
    goal: Texture,
    crawl: Texture,
    roll: Texture,
    float: Texture,
}

impl Textures {
    fn load(filesystem: &Filesystem, graphics: &mut Graphics) -> Result<Self> {
        let mut read = |path: &str| read_texture(filesystem, graphics, path);
        let digits = (0..10)
            .map(|digit| read(&format!("hud/digit_{digit}.png")))
            .collect::<Result<_>>()?;
        Ok(Self {
            life: read("hud/life.png")?,
            life_lost: read("hud/life_lost.png")?,
            digits,
            projectile: read("fx/projectile.png")?,
            fart: read("fx/fart.png")?,
            coin: read("level/coin.png")?,
            goal: read("level/goal.png")?,
            crawl: read("enemy_crawl.png")?,
            roll: read("enemy_roll.png")?,
            float: read("enemy_float.png")?,
        })
    }

    fn enemy(&self, kind: Kind) -> &Texture {
        match kind {
            Kind::Crawl => &self.crawl,
            Kind::Roll => &self.roll,
            Kind::Float => &self.float,
        }
    }
}

struct Sounds {
    jump: Sound,
    fart: Sound,
    shoot: Sound,
    hit: Sound,
    pickup: Sound,
    hurt: Sound,
    gameover: Sound,
    help: Sound,
    win: Sound,
}

impl Sounds {
    fn load(filesystem: &Filesystem, audio: &mut Audio) -> Result<Self> {
        let mut read = |name: &str| -> Result<Sound> {
            Ok(audio.sound(filesystem.read(&format!("sfx/{name}.wav"))?)?)
        };
        Ok(Self {
            jump: read("jump")?,
            fart: read("fart")?,
            shoot: read("shoot")?,
            hit: read("hit")?,
            pickup: read("pickup")?,
            hurt: read("hurt")?,
            gameover: read("gameover")?,
            help: read("help")?,
            win: read("win")?,
        })
    }
}

struct Moves {
    idle: Animation,
    walk: Animation,
    jump: Animation,
    double_jump: Animation,
    shoot: Animation,
    hurt: Animation,
    win: Animation,
    calling: Animation,
    freed: Animation,
}

impl Moves {
    fn load(filesystem: &Filesystem) -> Result<Self> {
        let hero = |name: &str| read_animation(filesystem, &format!("hero/anim_{name}.json"));
        Ok(Self {
            idle: hero("idle")?,
            walk: hero("walk")?,
            jump: hero("jump")?,
            double_jump: hero("double_jump")?,
            shoot: hero("shoot")?,
            hurt: hero("hurt")?,
            win: hero("win")?,
            calling: read_animation(filesystem, "princess/anim_help.json")?,
            freed: read_animation(filesystem, "princess/anim_win.json")?,
        })
    }

    /// What the hero is doing, and whether that repeats.
    fn of_hero(&self, hero: &Hero, phase: Phase, is_moving: bool) -> (&Animation, bool) {
        if phase == Phase::Won {
            (&self.win, false)
        } else if phase == Phase::Lost || hero.hurt > 0.0 {
            (&self.hurt, false)
        } else if hero.spin > 0.0 {
            (&self.double_jump, false)
        } else if !hero.is_grounded {
            (&self.jump, false)
        } else if is_moving {
            (&self.walk, true)
        } else {
            (&self.idle, true)
        }
    }
}

/// Lays `platform` along `rect`, from its top edge, cutting the last tile to fit.
fn spawn_platform(world: &mut World, platform: &Texture, rect: Rect) {
    let centre = rect.pos + rect.size / 2.0;
    world.spawn((
        Solid,
        Transform::at(centre.x, centre.y),
        ColliderBuilder::cuboid(rect.size.x / 2.0, rect.size.y / 2.0).friction(0.0),
    ));
    let scale = rect.size.y / platform.size().y;
    let tile = platform.size().x * scale;
    let end = rect.pos.x + rect.size.x;
    let mut x = rect.pos.x;
    while x < end {
        let width = (end - x).min(tile);
        let cut = platform.region(Rect::new(0.0, 0.0, width / scale, platform.size().y));
        let draw = Draw::new().scale(Vec2::splat(scale)).depth(PLATFORM_DEPTH);
        world.spawn((Transform::at(x, rect.pos.y), Sprite::new(cut).draw(draw)));
        x += tile;
    }
}

/// The walls at the ends of the level, which no one can leave by.
fn spawn_walls(world: &mut World) {
    for x in [-0.5, WIDTH + 0.5] {
        let wall = ColliderBuilder::cuboid(0.5, VIEW.y * 2.0).friction(0.0);
        world.spawn((Solid, Transform::at(x, VIEW.y / 2.0), wall));
    }
}

fn spawn_backgrounds(world: &mut World, backdrop: &Texture) -> Vec<(Entity, f32)> {
    let width = backdrop.size().x * VIEW.y / backdrop.size().y;
    let size = Vec2::new(width, VIEW.y);
    let mut tiles = Vec::new();
    let mut home = 0.0;
    while home < WIDTH * PARALLAX + VIEW.x + width {
        let draw = sized(backdrop, size).depth(depth::BACKDROP);
        let sprite = Sprite::new(backdrop.clone()).draw(draw);
        let tile = world.spawn((Transform::at(home, VIEW.y / 2.0), sprite));
        tiles.push((tile, home));
        home += width;
    }
    tiles
}

/// A row of icons on the HUD, `step` apart from `first`, all showing `texture`.
fn spawn_row(
    world: &mut World,
    count: usize,
    first: f32,
    step: f32,
    texture: &Texture,
) -> Vec<Entity> {
    let y = HUD_MARGIN + HUD_SIZE / 2.0;
    let mut x = first;
    let mut row = Vec::with_capacity(count);
    for _ in 0..count {
        let draw = sized(texture, Vec2::splat(HUD_SIZE));
        row.push(world.spawn((
            Hud,
            Transform::at(x, y),
            Sprite::new(texture.clone()).draw(draw),
        )));
        x += step;
    }
    row
}

/// The game.
pub struct Platformer {
    textures: Textures,
    sounds: Sounds,
    moves: Moves,
    hero_character: Character,
    captive_character: Character,
    hero: Entity,
    status: Entity,
    camera: Entity,
    backgrounds: Vec<(Entity, f32)>,
    lives: Vec<Entity>,
    digits: Vec<Entity>,
    diagnostics: Option<Entity>,
}

impl Platformer {
    /// What the game fixes before it starts.
    #[must_use]
    pub fn config() -> Config {
        Config::new("astronaut-cat")
            .title("The astronaut cat")
            .size(WINDOW.x, WINDOW.y)
            .resizable(false)
            .gravity(0.0, GRAVITY)
    }

    /// Puts everything a new game replaces into the world, and returns the hero.
    fn spawn_actors(&self, world: &mut World) -> Entity {
        let textures = &self.textures;
        for &(kind, left, right, height) in &ENEMIES {
            let texture = textures.enemy(kind);
            let draw = sized(texture, kind.size()).depth(THING_DEPTH);
            let enemy = Enemy {
                kind,
                span: (left, right),
                direction: -1.0,
                time: 0.0,
            };
            let (body, collider) = moved(kind.size());
            let sprite = Sprite::new(texture.clone()).draw(draw);
            world.spawn((
                Actor,
                enemy,
                Transform::at(right, height),
                body,
                collider,
                sprite,
            ));
        }
        for coin in COINS {
            let draw = sized(&textures.coin, Vec2::splat(COIN_SIZE)).depth(THING_DEPTH);
            let sprite = Sprite::new(textures.coin.clone()).draw(draw);
            let collider = zone(Vec2::splat(COIN_SIZE));
            world.spawn((Actor, Coin, Transform::at(coin.x, coin.y), collider, sprite));
        }

        let draw = sized(&textures.goal, GOAL_SIZE).depth(GOAL_DEPTH);
        let sprite = Sprite::new(textures.goal.clone()).draw(draw);
        world.spawn((
            Actor,
            Goal,
            Transform::at(GOAL.x, GOAL.y),
            zone(GOAL_SIZE),
            sprite,
        ));
        let earshot = zone(Vec2::new(HELP_DISTANCE * 2.0, VIEW.y));
        world.spawn((Actor, Earshot, Transform::at(GOAL.x, VIEW.y / 2.0), earshot));

        let mut captive = self.captive_character.clone();
        // A character made a moment ago has nothing playing, so this cannot name a missing joint.
        let looped = Play::new().looping(true);
        captive.play(&self.moves.calling, looped).ok();
        let feet = GROUND - ROOT_HEIGHT;
        world.spawn((Actor, Captive, Transform::at(GOAL.x, feet), captive));

        let body = RigidBodyBuilder::dynamic()
            .locked_axes(LockedAxes::ROTATION_LOCKED)
            .can_sleep(false);
        // The root joint is the entity's position, and the box must end at the feet.
        let centre = ROOT_HEIGHT - HERO_SIZE.y / 2.0;
        let collider = ColliderBuilder::cuboid(HERO_SIZE.x / 2.0, HERO_SIZE.y / 2.0)
            .translation(Vec2::new(0.0, centre))
            .friction(0.0);
        let hero = Hero {
            has_extra_jump: true,
            ..Hero::default()
        };
        world.spawn((
            Actor,
            hero,
            Transform::at(CHECKPOINTS[0], GROUND - ROOT_HEIGHT),
            body,
            collider,
            self.hero_character.clone(),
        ))
    }

    fn restart(&mut self, world: &mut World) -> Result<()> {
        let actors: Vec<Entity> = world.query::<Entity>().with::<&Actor>().iter().collect();
        for actor in actors {
            world.despawn(actor)?;
        }
        self.hero = self.spawn_actors(world);
        Ok(())
    }

    fn drive_enemies(u: &mut Update<'_>) {
        let dt = u.timer.dt();
        for (Body(body), transform, enemy, sprite) in
            u.world
                .query_mut::<(&Body, &Transform, &mut Enemy, &mut Sprite)>()
        {
            enemy.time += dt;
            if transform.position.x < enemy.span.0 {
                enemy.direction = 1.0;
            } else if transform.position.x > enemy.span.1 {
                enemy.direction = -1.0;
            }
            let speed = enemy.kind.speed();
            let body = u.physics.body_mut(*body);
            let bob = match enemy.kind {
                Kind::Float => (enemy.time * BOB_SPEED).cos() * BOB_SPEED * BOB_HEIGHT,
                _ => 0.0,
            };
            body.set_linvel(Vec2::new(enemy.direction * speed, bob), true);
            match enemy.kind {
                Kind::Roll => {
                    let radius = enemy.kind.size().x / 2.0;
                    body.set_angvel(enemy.direction * speed / radius, true);
                }
                // Only the crawler has a front to turn round: the others look at the player.
                Kind::Crawl => {
                    let scale = enemy.kind.size() / sprite.texture.size();
                    sprite.draw = sprite.draw.scale(scale * Vec2::new(enemy.direction, 1.0));
                }
                Kind::Float => {}
            }
        }
    }

    /// Whether there is ground under either side of feet at `feet`.
    fn is_standing(physics: &physics::World, world: &World, feet: Vec2) -> bool {
        // Started under the box, or the ray would find the hero itself.
        [-0.3, 0.3].iter().any(|side| {
            let start = feet + Vec2::new(*side, 0.02);
            physics
                .ray_cast(start, Vec2::Y, FOOTING)
                .and_then(|(collider, _)| entity_of(physics, collider))
                .is_some_and(|entity| world.satisfies::<&Solid>(entity))
        })
    }

    fn drive_hero(&mut self, u: &mut Update<'_>, status: Status) -> Result<()> {
        let dt = u.timer.dt();
        let is_playing = status.phase == Phase::Playing;
        let left = is_playing && u.input.down(Key::Left);
        let right = is_playing && u.input.down(Key::Right);
        let jump = is_playing && (u.input.pressed(Key::Space) || u.input.pressed(Key::Up));
        let shoot = is_playing && u.input.pressed(Key::X);

        let root = u.world.get::<&Transform>(self.hero)?.position;
        let feet = root + Vec2::new(0.0, ROOT_HEIGHT);
        let is_grounded = Self::is_standing(u.physics, u.world, feet);
        let body = u.world.get::<&Body>(self.hero)?.0;
        let mut hero = *u.world.get::<&Hero>(self.hero)?;

        hero.spin = (hero.spin - dt).max(0.0);
        hero.hurt = (hero.hurt - dt).max(0.0);
        hero.invulnerable = (hero.invulnerable - dt).max(0.0);
        hero.cooldown = (hero.cooldown - dt).max(0.0);
        let mut velocity = u.physics.body(body).linvel();
        velocity.x = match (left, right) {
            (true, false) => -RUN_SPEED,
            (false, true) => RUN_SPEED,
            _ => 0.0,
        };
        if velocity.x != 0.0 {
            hero.is_facing_left = velocity.x < 0.0;
        }
        hero.is_grounded = is_grounded && velocity.y >= 0.0;
        if hero.is_grounded {
            hero.has_extra_jump = true;
        }
        if jump && hero.is_grounded {
            velocity.y = -JUMP_SPEED;
            hero.is_grounded = false;
            u.audio.play(&self.sounds.jump);
        } else if jump && hero.has_extra_jump {
            velocity.y = -EXTRA_JUMP_SPEED;
            hero.has_extra_jump = false;
            hero.spin = SPIN;
            u.audio.play(&self.sounds.fart);
            self.spawn_fart(u.world, feet);
        }
        u.physics.body_mut(body).set_linvel(velocity, true);

        if shoot && hero.cooldown == 0.0 {
            hero.cooldown = SHOT_COOLDOWN;
            hero.pending_shot = Some(SHOT_DELAY);
            u.audio.play(&self.sounds.shoot);
            let overlay = Play::new().crossfade(OVERLAY_FADE);
            let mut character = u.world.get::<&mut Character>(self.hero)?;
            character.overlay(&self.moves.shoot, overlay)?;
        }
        hero.pending_shot = hero.pending_shot.map(|left| left - dt);
        if hero.pending_shot.is_some_and(|left| left <= 0.0) {
            hero.pending_shot = None;
            self.fire(u.world, &hero, root)?;
        }
        *u.world.get::<&mut Hero>(self.hero)? = hero;
        Ok(())
    }

    fn spawn_fart(&self, world: &mut World, feet: Vec2) {
        let texture = &self.textures.fart;
        let draw = sized(texture, Vec2::splat(FART_SIZE)).depth(THING_DEPTH);
        let sprite = Sprite::new(texture.clone()).draw(draw);
        world.spawn((
            Actor,
            Fart(FART_TIME),
            Transform::at(feet.x, feet.y),
            sprite,
        ));
    }

    /// Sends a shot from the launcher, straight ahead of how the hero looks.
    fn fire(&self, world: &mut World, hero: &Hero, root: Vec2) -> Result<()> {
        let hand = world.get::<&Character>(self.hero)?.joint("hand_right");
        let hand = hand.ok_or("the hero has no hand_right")?;
        let direction = if hero.is_facing_left { -1.0 } else { 1.0 };
        let centre = root + hand.offset + Vec2::new(direction * MUZZLE, 0.0);

        let texture = &self.textures.projectile;
        let size = Vec2::splat(SHOT_SIZE);
        let draw = sized(texture, size);
        let draw = draw
            .scale(draw.scale_value() * Vec2::new(direction, 1.0))
            .depth(THING_DEPTH);
        let body = RigidBodyBuilder::dynamic()
            .gravity_scale(SHOT_GRAVITY)
            .linvel(Vec2::new(direction * SHOT_SPEED, 0.0));
        world.spawn((
            Actor,
            Shot { time: 0.0 },
            Transform::at(centre.x, centre.y),
            body,
            ColliderBuilder::ball(SHOT_SIZE / 2.0).sensor(true),
            Sprite::new(texture.clone()).draw(draw),
        ));
        Ok(())
    }

    /// The entities with a collider touching `shape` at `position`.
    fn touching(u: &Update<'_>, shape: &dyn Shape, position: Vec2) -> Vec<Entity> {
        u.physics
            .overlap_shape(shape, Pose::from_translation(position))
            .into_iter()
            .filter_map(|collider| entity_of(u.physics, collider))
            .collect()
    }

    fn fly_shots(&self, u: &mut Update<'_>) -> Result<()> {
        let dt = u.timer.dt();
        let shots: Vec<(Entity, Vec2, f32)> = u
            .world
            .query_mut::<(Entity, &Transform, &mut Shot)>()
            .into_iter()
            .map(|(entity, transform, shot)| {
                shot.time += dt;
                (entity, transform.position, shot.time)
            })
            .collect();
        for (shot, position, time) in shots {
            let area = Ball::new(SHOT_SIZE / 2.0);
            let touched = Self::touching(u, &area, position);
            let enemy = touched.iter().find(|e| u.world.satisfies::<&Enemy>(**e));
            if let Some(enemy) = enemy {
                u.world.despawn(*enemy)?;
                u.audio.play(&self.sounds.hit);
            }
            let is_stopped = touched.iter().any(|e| u.world.satisfies::<&Solid>(*e));
            if enemy.is_some() || is_stopped || time > SHOT_TIME {
                u.world.despawn(shot)?;
            }
        }
        Ok(())
    }

    /// Whatever the hero's box touches happens to it.
    fn meet(&mut self, u: &mut Update<'_>, status: &mut Status) -> Result<()> {
        let root = u.world.get::<&Transform>(self.hero)?.position;
        let centre = root + Vec2::new(0.0, ROOT_HEIGHT - HERO_SIZE.y / 2.0);
        let area = Cuboid::new(HERO_SIZE / 2.0);
        let mut is_hurt = false;
        let mut has_fallen = false;
        let mut has_won = false;
        for entity in Self::touching(u, &area, centre) {
            if u.world.satisfies::<&Coin>(entity) {
                u.world.despawn(entity)?;
                status.score += 1;
                u.audio.play(&self.sounds.pickup);
            }
            is_hurt |= u.world.satisfies::<&Enemy>(entity);
            has_fallen |= u.world.satisfies::<&Pit>(entity);
            has_won |= u.world.satisfies::<&Goal>(entity);
            if u.world.satisfies::<&Earshot>(entity) && !status.has_called {
                status.has_called = true;
                u.audio.play(&self.sounds.help);
            }
        }

        let invulnerable = u.world.get::<&Hero>(self.hero)?.invulnerable;
        if has_fallen {
            self.lose_life(u, status)?;
            let reached = CHECKPOINTS.iter().rev().find(|x| **x <= root.x);
            let x = *reached.unwrap_or(&CHECKPOINTS[0]);
            let body = u.world.get::<&Body>(self.hero)?.0;
            let body = u.physics.body_mut(body);
            body.set_translation(Vec2::new(x, GROUND - ROOT_HEIGHT), true);
            body.set_linvel(Vec2::ZERO, true);
        } else if is_hurt && invulnerable == 0.0 {
            self.lose_life(u, status)?;
        }
        if has_won && status.phase == Phase::Playing {
            status.phase = Phase::Won;
            u.audio.play(&self.sounds.win);
            let cage: Vec<Entity> = u.world.query::<Entity>().with::<&Goal>().iter().collect();
            for entity in cage {
                u.world.despawn(entity)?;
            }
        }
        Ok(())
    }

    fn lose_life(&self, u: &mut Update<'_>, status: &mut Status) -> Result<()> {
        status.lives -= 1;
        let mut hero = u.world.get::<&mut Hero>(self.hero)?;
        hero.hurt = HURT;
        hero.invulnerable = INVULNERABLE;
        drop(hero);
        if status.lives == 0 {
            status.phase = Phase::Lost;
            u.audio.play(&self.sounds.gameover);
        } else {
            u.audio.play(&self.sounds.hurt);
        }
        Ok(())
    }

    fn act(&self, u: &mut Update<'_>, status: Status) -> Result<()> {
        let hero = *u.world.get::<&Hero>(self.hero)?;
        let body = u.world.get::<&Body>(self.hero)?.0;
        let is_moving = u.physics.body(body).linvel().x != 0.0;
        let (chosen, looping) = self.moves.of_hero(&hero, status.phase, is_moving);
        let mut character = u.world.get::<&mut Character>(self.hero)?;
        let play = Play::new().looping(looping).crossfade(CROSSFADE);
        character.play(chosen, play)?;
        if character.finished(&self.moves.shoot) {
            character.remove_overlay(&self.moves.shoot, OVERLAY_FADE);
        }

        let scale = if hero.is_facing_left {
            Vec2::new(-PART_SCALE, PART_SCALE)
        } else {
            Vec2::splat(PART_SCALE)
        };
        let is_blinking = hero.invulnerable > 0.0
            && status.phase == Phase::Playing
            && ((u.timer.elapsed() * BLINKS_PER_SECOND) as u64).is_multiple_of(2);
        let alpha = if is_blinking { 0.0 } else { 1.0 };
        character.draw = character
            .draw
            .scale(scale)
            .color(Color::new(1.0, 1.0, 1.0, alpha));
        drop(character);

        if status.phase == Phase::Won {
            let freed = Play::new().crossfade(CROSSFADE);
            for character in u.world.query_mut::<&mut Character>().with::<&Captive>() {
                character.play(&self.moves.freed, freed)?;
            }
        }
        Ok(())
    }

    /// Clouds grow, thin out and go.
    fn thin_farts(u: &mut Update<'_>) -> Result<()> {
        let dt = u.timer.dt();
        let mut gone = Vec::new();
        for (entity, Fart(left), sprite) in u.world.query_mut::<(Entity, &mut Fart, &mut Sprite)>()
        {
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

    fn show_status(&self, world: &World, status: Status) -> Result<()> {
        let textures = &self.textures;
        for (index, entity) in (0u32..).zip(&self.lives) {
            let life = if index < status.lives {
                &textures.life
            } else {
                &textures.life_lost
            };
            world.get::<&mut Sprite>(*entity)?.texture = life.clone();
        }
        let mut score = status.score as usize;
        for entity in &self.digits {
            world.get::<&mut Sprite>(*entity)?.texture = textures.digits[score % 10].clone();
            score /= 10;
        }
        Ok(())
    }

    fn follow(&self, world: &World) -> Result<()> {
        let half = VIEW.x / 2.0;
        let x = world.get::<&Transform>(self.hero)?.position.x;
        let x = x.clamp(half, WIDTH - half);
        world.get::<&mut Camera>(self.camera)?.position.x = x;
        for (entity, home) in &self.backgrounds {
            world.get::<&mut Transform>(*entity)?.position.x = home + x * (1.0 - PARALLAX);
        }
        Ok(())
    }

    fn toggle_diagnostics(&mut self, world: &mut World) -> Result<()> {
        if let Some(lines) = self.diagnostics.take() {
            world.despawn(lines)?;
        } else {
            self.diagnostics = Some(world.spawn((ColliderLines(DIAGNOSTICS_COLOUR),)));
        }
        Ok(())
    }
}

impl Game for Platformer {
    fn load(s: Setup<'_>) -> Result<Self> {
        let textures = Textures::load(s.filesystem, s.graphics)?;
        let sounds = Sounds::load(s.filesystem, s.audio)?;
        let moves = Moves::load(s.filesystem)?;
        let music = s.audio.stream(s.filesystem.read("sfx/music.wav")?)?;
        let theme = PlaySound::new().volume(0.4).looping(true);
        s.audio.play_with(&music, theme);

        let parts = Draw::new().scale(Vec2::splat(PART_SCALE));
        let hero_character = Character::load(s.filesystem, s.graphics, "hero")?;
        let hero_character = hero_character.draw(parts.depth(HERO_DEPTH));
        let captive_character = Character::load(s.filesystem, s.graphics, "princess")?;
        let captive_character = captive_character.draw(parts.depth(CAPTIVE_DEPTH));

        let world = &mut *s.world;
        world.spawn((Clear(Color::rgb(0.18, 0.14, 0.43)),));
        let camera = world.spawn((Camera {
            position: VIEW / 2.0,
            zoom: PIXELS_PER_METRE,
            rotation: 0.0,
        },));
        let backdrop = read_texture(s.filesystem, s.graphics, "level/background.png")?;
        let backgrounds = spawn_backgrounds(world, &backdrop);
        let platform = read_texture(s.filesystem, s.graphics, "level/platform.png")?;
        for rect in PLATFORMS {
            spawn_platform(world, &platform, rect);
        }
        spawn_walls(world);
        let pit = zone(Vec2::new(WIDTH, 1.0));
        world.spawn((Pit, Transform::at(WIDTH / 2.0, PIT + 0.5), pit));

        let first = HUD_MARGIN + HUD_SIZE / 2.0;
        let lives = spawn_row(world, LIVES as usize, first, HUD_SIZE + 6.0, &textures.life);
        let last = WINDOW.x - first;
        let digits = spawn_row(world, DIGITS, last, -HUD_SIZE, &textures.digits[0]);
        let status = world.spawn((Status {
            lives: LIVES,
            score: 0,
            phase: Phase::Playing,
            has_called: false,
        },));

        let mut game = Self {
            textures,
            sounds,
            moves,
            hero_character,
            captive_character,
            hero: Entity::DANGLING,
            status,
            camera,
            backgrounds,
            lives,
            digits,
            diagnostics: None,
        };
        game.hero = game.spawn_actors(world);
        Ok(game)
    }

    fn update(&mut self, mut u: Update<'_>) -> Result<()> {
        if u.events.contains(&WindowEvent::CloseRequested) {
            u.quit();
        }
        if u.input.pressed(Key::Tab) {
            self.toggle_diagnostics(u.world)?;
        }
        let mut status = *u.world.get::<&Status>(self.status)?;
        if status.phase != Phase::Playing && u.input.pressed(Key::Enter) {
            self.restart(u.world)?;
            status = Status {
                lives: LIVES,
                score: 0,
                phase: Phase::Playing,
                has_called: false,
            };
            *u.world.get::<&mut Status>(self.status)? = status;
        }
        // A new game's bodies appear on the step after their entities.
        if !u.world.satisfies::<&Body>(self.hero) {
            return Ok(());
        }

        Self::drive_enemies(&mut u);
        self.drive_hero(&mut u, status)?;
        if status.phase == Phase::Playing {
            self.fly_shots(&mut u)?;
            self.meet(&mut u, &mut status)?;
        }
        self.act(&mut u, status)?;
        Self::thin_farts(&mut u)?;
        self.show_status(u.world, status)?;
        self.follow(u.world)?;
        *u.world.get::<&mut Status>(self.status)? = status;
        Ok(())
    }
}
