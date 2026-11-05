use olivia::{
    ecs::{CSprite, CTransform, World},
    math::Vec2,
};

use super::{CActor, GROUND, THING_DEPTH, moved};
use crate::{assets::Textures, sprite::sized};

/// How an enemy moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
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
    const fn speed(self) -> f32 {
        match self {
            Self::Crawl => 0.7,
            Self::Roll => 1.9,
            Self::Float => 1.0,
        }
    }
}

/// `(kind, left end of its walk, right end, height of its centre)`.
const ENEMIES: [(Kind, f32, f32, f32); 7] = [
    (Kind::Crawl, 5.0, 12.0, GROUND - 0.435),
    (Kind::Float, 10.4, 14.6, 4.0),
    (Kind::Roll, 16.0, 23.4, GROUND - 0.45),
    (Kind::Crawl, 19.5, 23.5, GROUND - 0.435),
    (Kind::Float, 26.4, 32.0, 4.0),
    (Kind::Roll, 33.2, 38.0, GROUND - 0.45),
    (Kind::Crawl, 35.0, 38.8, GROUND - 0.435),
];

/// An enemy walking its stretch.
#[derive(Clone, Copy, Debug, PartialEq)]
struct CEnemy {
    kind: Kind,
    span: (f32, f32),
    direction: f32,
    time: f32,
}

/// Every enemy, each at the right end of its walk.
pub(super) fn spawn(world: &mut World, textures: &Textures) {
    for &(kind, left, right, height) in &ENEMIES {
        spawn_enemy(world, textures, kind, left, right, height);
    }
}

fn spawn_enemy(
    world: &mut World,
    textures: &Textures,
    kind: Kind,
    left: f32,
    right: f32,
    height: f32,
) {
    let texture = match kind {
        Kind::Crawl => &textures.crawl,
        Kind::Roll => &textures.roll,
        Kind::Float => &textures.float,
    };
    let draw = sized(texture, kind.size()).depth(THING_DEPTH);
    let enemy = CEnemy {
        kind,
        span: (left, right),
        direction: -1.0,
        time: 0.0,
    };
    let (body, collider) = moved(kind.size());
    let sprite = CSprite::new(texture.clone()).draw(draw);
    world.spawn((
        CActor,
        enemy,
        CTransform::at(right, height),
        body,
        collider,
        sprite,
    ));
}
