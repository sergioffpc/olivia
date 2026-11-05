//! As moedas que o herói apanha pelo caminho.

use olivia::{
    ecs::{CSprite, CTransform, World},
    graphics::Texture,
    math::Vec2,
};

use super::{CActor, layout::COINS, zone};
use crate::{depth, sprite::sized};

/// Uma etiqueta para o que o herói apanha quando lhe toca.
pub(super) struct CCoin;

/// O tamanho de uma moeda, para se ver e para se tocar: um pouco mais pequena que meio metro.
const COIN_SIZE: Vec2 = Vec2::splat(0.56);

/// Põe no nível todas as moedas, cada uma com o desenho `coin`.
pub(super) fn spawn(world: &mut World, coin: &Texture) {
    for pos in COINS {
        let draw = sized(coin, COIN_SIZE).depth(depth::THING);
        // A moeda nunca sai do sítio, por isso não precisa de corpo: chega uma zona onde tocar.
        let collider = zone(COIN_SIZE);
        let sprite = CSprite::new(coin.clone()).draw(draw);
        world.spawn((
            CActor,
            CCoin,
            CTransform::at(pos.x, pos.y),
            collider,
            sprite,
        ));
    }
}
