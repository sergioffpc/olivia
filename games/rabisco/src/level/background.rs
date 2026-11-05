use olivia::{
    Result,
    ecs::{CSprite, CTransform, Entity, World},
    graphics::Texture,
    math::Vec2,
};

use super::layout::WIDTH;
use crate::{camera::VIEW, depth, sprite::sized};

/// How fast the background scrolls past, as a fraction of how fast the level does.
const PARALLAX: f32 = 0.6;

/// One tile of the background.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Tile {
    pub(super) entity: Entity,
    /// Its x when the camera is at the start.
    pub(super) home: f32,
}

/// Tiles `background` across the whole view, far enough for the parallax to never run out.
pub(super) fn spawn(world: &mut World, background: &Texture) -> Vec<Tile> {
    let width = background.size().x * VIEW.y / background.size().y;
    let size = Vec2::new(width, VIEW.y);
    let mut tiles = Vec::new();
    let mut home = 0.0;
    while home < WIDTH * PARALLAX + VIEW.x + width {
        let draw = sized(background, size).depth(depth::BACKGROUND);
        let sprite = CSprite::new(background.clone()).draw(draw);
        let entity = world.spawn((CTransform::at(home, VIEW.y / 2.0), sprite));
        tiles.push(Tile { entity, home });
        home += width;
    }
    tiles
}

/// Places every tile for the camera at `camera`: the further the camera has gone, the further the
/// tiles go with it, so that they scroll past slower than the level.
pub(super) fn scroll(world: &mut World, tiles: &[Tile], camera: f32) -> Result<()> {
    for tile in tiles {
        world.get::<&mut CTransform>(tile.entity)?.position.x =
            tile.home + camera * (1.0 - PARALLAX);
    }
    Ok(())
}
