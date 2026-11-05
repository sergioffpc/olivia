//! How deep each layer of the game is drawn, from back to front.

pub(crate) use olivia::graphics::depth::BACKDROP as BACKGROUND;
use olivia::graphics::depth::PLAYFIELD;

/// What the hero stands on.
pub(crate) const PLATFORM: f32 = PLAYFIELD + 1.0;
/// Who the hero comes to save.
pub(crate) const PRINCESS: f32 = PLAYFIELD + 2.0;
/// The cage the princess is held in, in front of her.
pub(crate) const GOAL: f32 = PLAYFIELD + 4.0;
/// The things on the level: coins and enemies.
pub(crate) const THING: f32 = PLAYFIELD + 10.0;
/// The hero, in front of everything on the level.
pub(crate) const HERO: f32 = PLAYFIELD + 20.0;
