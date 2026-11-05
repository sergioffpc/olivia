use olivia::{
    ecs::{CBody, CSprite, CTransform, World},
    math::Vec2,
    physics,
};

use super::{
    CActor,
    layout::{ENEMIES, GROUND},
    moved,
};
use crate::{assets::Textures, depth, sprite::sized};

/// How fast a floating enemy sways up and down, in radians per second.
const SWAY_SPEED: f32 = 2.5;
/// How far a floating enemy sways above and below where it started, in metres.
const SWAY_HEIGHT: f32 = 0.5;

/// How an enemy moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    /// Long and low, slowly along the ground.
    Crawl,
    /// Round, quickly along the ground, turning as it goes.
    Roll,
    /// Through the air, swaying up and down.
    Float,
}

impl Kind {
    /// The size of its box.
    const fn size(self) -> Vec2 {
        match self {
            Self::Crawl => Vec2::new(1.5, 0.87),
            Self::Roll => Vec2::new(0.9, 0.9),
            Self::Float => Vec2::new(0.88, 1.2),
        }
    }

    /// How fast it goes, in metres per second.
    const fn speed(self) -> f32 {
        match self {
            Self::Crawl => 0.7,
            Self::Roll => 1.9,
            Self::Float => 1.0,
        }
    }
}

/// The height an enemy walks at.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Lane {
    /// Standing on the ground.
    Ground,
    /// In the air, its centre at this y.
    Air(f32),
}

/// The stretch an enemy walks, from its left end to its right one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Span {
    pub(super) left: f32,
    pub(super) right: f32,
}

/// Which way an enemy walks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Direction {
    Left,
    Right,
}

impl Direction {
    /// The sign of its x: -1 for left, 1 for right.
    const fn sign(self) -> f32 {
        match self {
            Self::Left => -1.0,
            Self::Right => 1.0,
        }
    }
}

/// Where an enemy of a kind walks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Patrol {
    kind: Kind,
    span: Span,
    lane: Lane,
}

impl Patrol {
    pub(super) const fn new(kind: Kind, left: f32, right: f32, lane: Lane) -> Self {
        Self {
            kind,
            span: Span { left, right },
            lane,
        }
    }

    /// The y of its centre.
    fn y(self) -> f32 {
        match self.lane {
            Lane::Ground => GROUND - self.kind.size().y / 2.0,
            Lane::Air(y) => y,
        }
    }
}

/// An enemy walking its stretch.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct CEnemy {
    kind: Kind,
    span: Span,
    direction: Direction,
    time: f32,
}

/// Every enemy, each at the right end of its walk, walking left.
pub(super) fn spawn(world: &mut World, textures: &Textures) {
    for patrol in ENEMIES {
        spawn_enemy(world, textures, patrol);
    }
}

fn spawn_enemy(world: &mut World, textures: &Textures, patrol: Patrol) {
    let Patrol { kind, span, .. } = patrol;
    let texture = match kind {
        Kind::Crawl => &textures.crawl,
        Kind::Roll => &textures.roll,
        Kind::Float => &textures.float,
    };
    let draw = sized(texture, kind.size()).depth(depth::THING);
    let enemy = CEnemy {
        kind,
        span,
        direction: Direction::Left,
        time: 0.0,
    };
    let (body, collider) = moved(kind.size());
    let sprite = CSprite::new(texture.clone()).draw(draw);
    world.spawn((
        CActor,
        enemy,
        CTransform::at(span.right, patrol.y()),
        body,
        collider,
        sprite,
    ));
}

/// Moves every enemy one fixed step of `dt` seconds, back and forth along its span.
pub(super) fn update(world: &mut World, physics: &mut physics::World, dt: f32) {
    for (CBody(body), transform, enemy, sprite) in
        world.query_mut::<(&CBody, &CTransform, &mut CEnemy, &mut CSprite)>()
    {
        // Its own clock, which only the floater's sway reads.
        enemy.time += dt;
        // It turns round only once past an end, and keeps on as it was in between, so it always
        // walks back and forth within its span.
        if transform.position.x < enemy.span.left {
            enemy.direction = Direction::Right;
        } else if transform.position.x > enemy.span.right {
            enemy.direction = Direction::Left;
        }
        let sign = enemy.direction.sign();
        let speed = enemy.kind.speed();
        // The derivative of `sin(t · SWAY_SPEED) · SWAY_HEIGHT`, so that the floater's height
        // swings smoothly by `SWAY_HEIGHT` either way.
        let sway = match enemy.kind {
            Kind::Float => (enemy.time * SWAY_SPEED).cos() * SWAY_SPEED * SWAY_HEIGHT,
            Kind::Crawl | Kind::Roll => 0.0,
        };
        // Its body is kinematic and moved by velocity, so the game sets it every step.
        let body = physics.body_mut(*body);
        body.set_linvel(Vec2::new(sign * speed, sway), true);
        match enemy.kind {
            // At `v / r` it rolls without slipping, backwards when it goes left.
            Kind::Roll => {
                let radius = enemy.kind.size().x / 2.0;
                body.set_angvel(sign * speed / radius, true);
            }
            // The only one with a front and a back: it mirrors around its centre to face where it
            // goes.
            Kind::Crawl => {
                let scale = enemy.kind.size() / sprite.texture.size();
                sprite.draw = sprite.draw.scale(scale * Vec2::new(sign, 1.0));
            }
            Kind::Float => {}
        }
    }
}
