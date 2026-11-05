use olivia::{
    ecs::{CSprite, CTransform, ColliderBuilder, World},
    graphics::{Draw, Texture},
    math::{Rect, Vec2},
};

use super::{
    layout::{PIT, PLATFORMS, WIDTH},
    zone,
};
use crate::{camera::VIEW, depth};

/// The narrowest a platform's last tile is drawn, in metres: anything less is rounding.
const MIN_TILE: f32 = 1e-3;

/// A collider nothing passes through, shots included.
pub(super) struct CSolid;
/// What swallows a falling hero.
struct CPit;

/// The platforms, the walls at the ends and the pit under it all.
pub(super) fn spawn(world: &mut World, platform: &Texture) {
    for rect in PLATFORMS {
        spawn_platform(world, platform, rect);
    }
    spawn_walls(world);
    spawn_pit(world);
}

/// Lays `platform` along `rect`, from its top edge, cutting the last tile to fit.
fn spawn_platform(world: &mut World, platform: &Texture, rect: Rect) {
    let centre = rect.pos + rect.size / 2.0;
    world.spawn((
        CSolid,
        CTransform::at(centre.x, centre.y),
        ColliderBuilder::cuboid(rect.size.x / 2.0, rect.size.y / 2.0).friction(0.0),
    ));
    let scale = rect.size.y / platform.size().y;
    let tile = platform.size().x * scale;
    let end = rect.pos.x + rect.size.x;
    let mut x = rect.pos.x;
    while end - x > MIN_TILE {
        let width = (end - x).min(tile);
        let cut = platform.region(Rect::new(0.0, 0.0, width / scale, platform.size().y));
        let draw = Draw::new().scale(Vec2::splat(scale)).depth(depth::PLATFORM);
        world.spawn((CTransform::at(x, rect.pos.y), CSprite::new(cut).draw(draw)));
        x += tile;
    }
}

/// The walls at the ends of the level, which no one can leave by.
fn spawn_walls(world: &mut World) {
    for x in [-0.5, WIDTH + 0.5] {
        let wall = ColliderBuilder::cuboid(0.5, VIEW.y * 2.0).friction(0.0);
        world.spawn((CSolid, CTransform::at(x, VIEW.y / 2.0), wall));
    }
}

/// A sensor along the whole level, from [`PIT`] down.
fn spawn_pit(world: &mut World) {
    let pit = zone(Vec2::new(WIDTH, 1.0));
    world.spawn((CPit, CTransform::at(WIDTH / 2.0, PIT + 0.5), pit));
}
