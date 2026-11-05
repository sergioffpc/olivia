use olivia::{
    ecs::{CSprite, CTransform, World},
    graphics::Texture,
    math::Vec2,
};

use super::{CActor, layout::COINS, moved};
use crate::{depth, sprite::sized};

/// What the hero picks up by touching it.
pub(super) struct CCoin;

/// The size of a coin, as drawn and as touched.
const COIN_SIZE: Vec2 = Vec2::splat(1.0);

/// Every coin, each showing `coin`.
pub(super) fn spawn(world: &mut World, coin: &Texture) {
    for pos in COINS {
        let draw = sized(coin, COIN_SIZE).depth(depth::THING);
        let (body, collider) = moved(COIN_SIZE);
        let sprite = CSprite::new(coin.clone()).draw(draw);
        world.spawn((
            CActor,
            CCoin,
            CTransform::at(pos.x, pos.y),
            body,
            collider,
            sprite,
        ));
    }
}
