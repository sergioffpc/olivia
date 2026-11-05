//! As moedas que o herói apanha pelo caminho.

use olivia::{
    ecs::{CSprite, CTransform, World},
    graphics::Texture,
    math::Vec2,
};

use super::{CActor, layout::COINS, moved};
use crate::{depth, sprite::sized};

/// Uma etiqueta para o que o herói apanha quando lhe toca.
pub(super) struct CCoin;

/// O tamanho de uma moeda, para se ver e para se tocar.
const COIN_SIZE: Vec2 = Vec2::splat(1.0);

/// Põe no nível todas as moedas, cada uma com o desenho `coin`.
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
