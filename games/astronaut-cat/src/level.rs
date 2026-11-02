//! The fixed layout of the level, in metres with y down: the class changes how it looks, never this.

use olivia::math::{Rect, Vec2};

/// The size of the window, in logical pixels.
pub const WINDOW: Vec2 = Vec2::new(1280.0, 720.0);
/// How many logical pixels a metre takes.
pub const PIXELS_PER_METRE: f32 = 100.0;
/// How much the camera sees.
pub const VIEW: Vec2 = Vec2::new(12.8, 7.2);
/// Where the level ends.
pub const WIDTH: f32 = 44.0;
/// The height of the ground's top.
pub const GROUND: f32 = 6.0;
/// The gravity, in metres per second squared.
pub const GRAVITY: f32 = 22.0;

/// The size of the hero's box.
pub const HERO_SIZE: Vec2 = Vec2::new(0.7, 1.6);
/// How far the feet of a character standing as drawn are under its root joint.
pub const ROOT_HEIGHT: f32 = 0.88;
/// The size of the place the character to save is held in.
pub const GOAL_SIZE: Vec2 = Vec2::new(2.2, 3.0);
/// The size of what is picked up.
pub const COIN_SIZE: f32 = 0.56;
/// The size of a shot.
pub const SHOT_SIZE: f32 = 0.48;

/// The left end of the goal's cage, seen from afar: closer than this, who is held calls for help.
pub const HELP_DISTANCE: f32 = 5.2;
/// Where the cage stands, by its centre.
pub const GOAL: Vec2 = Vec2::new(41.5, GROUND - GOAL_SIZE.y / 2.0);
/// Where a fall begins to cost a life.
pub const PIT: f32 = 8.4;

/// Where the hero starts over after a fall: the last one passed.
pub const CHECKPOINTS: [f32; 4] = [1.2, 15.6, 26.2, 33.0];

/// What the hero stands on. They are solid, so anything over a walkway leaves room for the hero's
/// head, and the ones that do need the extra jump to reach.
pub const PLATFORMS: [Rect; 10] = [
    Rect::new(0.0, GROUND, 13.0, 1.2),
    Rect::new(15.0, GROUND, 9.0, 1.2),
    Rect::new(25.8, GROUND, 5.0, 1.2),
    Rect::new(32.6, GROUND, 11.4, 1.2),
    Rect::new(6.4, 3.8, 2.8, 0.34),
    Rect::new(10.2, 2.6, 2.6, 0.34),
    Rect::new(17.2, 3.8, 3.0, 0.34),
    Rect::new(21.4, 2.6, 2.6, 0.34),
    Rect::new(27.0, 3.8, 2.4, 0.34),
    Rect::new(34.4, 3.8, 3.0, 0.34),
];

/// How an enemy moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Long and low, slowly along the ground.
    Crawl,
    /// Round, quickly along the ground, turning as it goes.
    Roll,
    /// Through the air, bobbing.
    Float,
}

impl Kind {
    /// The size of its box.
    #[must_use]
    pub const fn size(self) -> Vec2 {
        match self {
            Self::Crawl => Vec2::new(1.5, 0.87),
            Self::Roll => Vec2::new(0.9, 0.9),
            Self::Float => Vec2::new(0.88, 1.2),
        }
    }

    /// How fast it goes, in metres per second.
    #[must_use]
    pub const fn speed(self) -> f32 {
        match self {
            Self::Crawl => 0.7,
            Self::Roll => 1.9,
            Self::Float => 1.0,
        }
    }
}

/// `(kind, left end of its walk, right end, height of its centre)`.
pub const ENEMIES: [(Kind, f32, f32, f32); 7] = [
    (Kind::Crawl, 5.0, 12.0, GROUND - 0.435),
    (Kind::Float, 10.4, 14.6, 4.0),
    (Kind::Roll, 16.0, 23.4, GROUND - 0.45),
    (Kind::Crawl, 19.5, 23.5, GROUND - 0.435),
    (Kind::Float, 26.4, 32.0, 4.0),
    (Kind::Roll, 33.2, 38.0, GROUND - 0.45),
    (Kind::Crawl, 35.0, 38.8, GROUND - 0.435),
];

/// Where the things to pick up are.
pub const COINS: [Vec2; 14] = [
    Vec2::new(3.0, 5.4),
    Vec2::new(4.2, 5.4),
    Vec2::new(7.2, 3.2),
    Vec2::new(8.4, 3.2),
    Vec2::new(11.5, 2.0),
    Vec2::new(14.0, 4.2),
    Vec2::new(18.0, 3.2),
    Vec2::new(19.4, 3.2),
    Vec2::new(22.7, 2.0),
    Vec2::new(24.9, 4.2),
    Vec2::new(28.2, 3.2),
    Vec2::new(31.7, 4.2),
    Vec2::new(35.2, 3.2),
    Vec2::new(36.6, 3.2),
];
