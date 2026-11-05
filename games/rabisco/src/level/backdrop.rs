use olivia::{
    ecs::{CSprite, CTransform, Entity, World},
    graphics::{Texture, depth},
    math::Vec2,
};

use super::WIDTH;
use crate::{game::VIEW, sprite::sized};

/// How much slower the background moves than the camera.
const PARALLAX: f32 = 0.6;

/// Tiles `backdrop` across the whole view, far enough for the parallax to never run out, each
/// tile with where it sits when the camera is at the start.
pub(super) fn spawn(world: &mut World, backdrop: &Texture) -> Vec<(Entity, f32)> {
    let width = backdrop.size().x * VIEW.y / backdrop.size().y;
    let size = Vec2::new(width, VIEW.y);
    let mut tiles = Vec::new();
    let mut home = 0.0;
    while home < WIDTH * PARALLAX + VIEW.x + width {
        let draw = sized(backdrop, size).depth(depth::BACKDROP);
        let sprite = CSprite::new(backdrop.clone()).draw(draw);
        let tile = world.spawn((CTransform::at(home, VIEW.y / 2.0), sprite));
        tiles.push((tile, home));
        home += width;
    }
    tiles
}
