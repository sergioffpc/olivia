use olivia::{
    ecs::{CSprite, CTransform, ColliderBuilder, World},
    graphics::{Draw, Texture, depth},
    math::{Rect, Vec2},
};

use super::{GROUND, WIDTH, zone};
use crate::game::VIEW;

/// Where a fall begins to cost a life.
const PIT: f32 = 8.4;

/// What the hero stands on. They are solid, so anything over a walkway leaves room for the hero's
/// head, and the ones that do need the extra jump to reach.
const PLATFORMS: [Rect; 10] = [
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
const PLATFORM_DEPTH: f32 = depth::PLAYFIELD + 1.0;

/// A collider that stops shots.
struct CSolid;
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
    while x < end {
        let width = (end - x).min(tile);
        let cut = platform.region(Rect::new(0.0, 0.0, width / scale, platform.size().y));
        let draw = Draw::new().scale(Vec2::splat(scale)).depth(PLATFORM_DEPTH);
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

fn spawn_pit(world: &mut World) {
    let pit = zone(Vec2::new(WIDTH, 1.0));
    world.spawn((CPit, CTransform::at(WIDTH / 2.0, PIT + 0.5), pit));
}
