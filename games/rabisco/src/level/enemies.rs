use olivia::{
    ecs::{CSprite, CTransform, World},
    math::Vec2,
};

use super::{
    CActor,
    layout::{ENEMIES, GROUND},
    moved,
};
use crate::{assets::Textures, depth, sprite::sized};

/// How an enemy moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    /// Long and low, slowly along the ground.
    Crawl,
    /// Round, quickly along the ground, turning as it goes.
    Roll,
    /// Through the air, bobbing.
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
    #[expect(dead_code, reason = "used once the update phase moves the enemies")]
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
    #[expect(
        dead_code,
        reason = "turned to once the update phase moves the enemies"
    )]
    Right,
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
struct CEnemy {
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
