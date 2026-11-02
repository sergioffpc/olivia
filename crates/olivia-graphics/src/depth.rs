//! Named [`depth`](crate::Draw::depth) values for the usual parts of a game's picture, with room
//! to order things within each, as in `depth::BACKDROP + 1.0`.

/// Scenery behind the game, which nothing in it collides with.
pub const BACKDROP: f32 = -100.0;

/// Where the game plays out: the player and the obstacles it meets. It is the default depth.
pub const PLAYFIELD: f32 = 0.0;

/// Effects over the game, such as rain or snow, which nothing in it interacts with.
pub const ATMOSPHERE: f32 = 100.0;
