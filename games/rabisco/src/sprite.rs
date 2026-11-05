//! Uma ajuda para desenhar imagens do tamanho que queremos.

use olivia::{
    graphics::{Draw, Texture},
    math::Vec2,
};

/// Diz como desenhar a imagem `texture` com o tamanho `size`, em metros, presa pelo meio.
///
/// Os desenhos dos alunos têm cada um o seu tamanho em píxeis. Esta função calcula quanto é
/// preciso esticar ou encolher cada desenho para ficar com o tamanho certo no jogo.
pub(crate) fn sized(texture: &Texture, size: Vec2) -> Draw {
    Draw::new()
        // O desenho fica preso pelo meio, como um quadro pendurado num prego ao centro.
        .origin(Vec2::splat(0.5))
        // Dividir o tamanho que queremos pelo tamanho do desenho diz quanto o esticar.
        .scale(size / texture.size())
}
