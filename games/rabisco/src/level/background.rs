//! O cenário ao longe, atrás de tudo.
//!
//! Quando vamos de carro, as árvores ao pé da estrada passam a correr, mas as montanhas ao longe
//! parecem andar devagarinho. O fundo faz o mesmo: anda mais devagar do que o resto, e isso faz
//! parecer que está mesmo longe. Chama-se paralaxe.

use olivia::{
    Result,
    ecs::{CSprite, CTransform, Entity, World},
    graphics::Texture,
    math::Vec2,
};

use super::layout::WIDTH;
use crate::{camera::VIEW, depth, sprite::sized};

/// A rapidez com que o fundo passa, comparada com o resto do nível: 0.6 é um pouco mais de metade.
const PARALLAX: f32 = 0.6;

/// Um bocado do fundo. O fundo é feito de vários desenhos iguais, postos lado a lado.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Tile {
    /// Onde o bocado está guardado no mundo do jogo.
    pub(super) entity: Entity,
    /// O x do bocado quando a câmara está no começo do nível.
    pub(super) home: f32,
}

/// Põe o desenho `background` lado a lado, tantas vezes quantas as precisas para o fundo nunca
/// acabar, mesmo a andar devagar.
pub(super) fn spawn(world: &mut World, background: &Texture) -> Vec<Tile> {
    // Cada bocado fica da altura da câmara, e com a largura certa para não ficar esticado.
    let width = background.size().x * VIEW.y / background.size().y;
    let size = Vec2::new(width, VIEW.y);
    let mut tiles = Vec::new();
    let mut home = 0.0;
    // Põe bocados até passar o sítio mais longe que o fundo vai mostrar.
    while home < WIDTH * PARALLAX + VIEW.x + width {
        let draw = sized(background, size).depth(depth::BACKGROUND);
        let sprite = CSprite::new(background.clone()).draw(draw);
        let entity = world.spawn((CTransform::at(home, VIEW.y / 2.0), sprite));
        tiles.push(Tile { entity, home });
        home += width;
    }
    tiles
}

/// Põe cada bocado do fundo no sítio certo para a câmara que está em `camera`.
///
/// Quanto mais a câmara anda, mais os bocados andam com ela: assim passam mais devagar do que o
/// resto do nível.
pub(super) fn scroll(world: &mut World, tiles: &[Tile], camera: f32) -> Result<()> {
    for tile in tiles {
        world.get::<&mut CTransform>(tile.entity)?.position.x =
            tile.home + camera * (1.0 - PARALLAX);
    }
    Ok(())
}
