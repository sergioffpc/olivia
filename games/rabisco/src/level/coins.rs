use olivia::{
    ecs::{Sprite, Transform, World},
    graphics::Texture,
    math::Vec2,
};

use super::{Actor, THING_DEPTH, moved};
use crate::sprite::sized;

/// Where the things to pick up are.
const COINS: [Vec2; 14] = [
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

/// Every coin, each showing `coin`.
pub(super) fn spawn(world: &mut World, coin: &Texture) {
    for pos in COINS {
        let draw = sized(coin, Vec2::splat(1.0)).depth(THING_DEPTH);
        let (body, collider) = moved(Vec2::splat(1.0));
        let sprite = Sprite::new(coin.clone()).draw(draw);
        world.spawn((Actor, Transform::at(pos.x, pos.y), body, collider, sprite));
    }
}
