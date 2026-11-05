//! Os desenhos do jogo, feitos pela turma nas folhas de desenho.

use olivia::{
    Result,
    filesystem::Filesystem,
    graphics::{Graphics, Texture},
};

use super::{fallback::no_texture, read};

/// Todos os desenhos do jogo, menos as partes das personagens.
#[derive(Debug)]
pub(crate) struct Textures {
    /// O cenário que se vê ao longe, atrás de tudo.
    pub(crate) background: Texture,
    /// O chão e as plataformas.
    pub(crate) platform: Texture,
    /// Uma vida que o herói ainda tem.
    pub(crate) life: Texture,
    /// Uma vida que o herói já perdeu.
    pub(crate) life_lost: Texture,
    /// Os algarismos de 0 a 9, cada um no seu lugar: o 3 está na posição 3.
    pub(crate) digits: [Texture; 10],
    /// O que o herói dispara.
    pub(crate) projectile: Texture,
    /// A nuvem que sai no salto extra.
    pub(crate) fart: Texture,
    /// O que o herói apanha pelo caminho.
    pub(crate) coin: Texture,
    /// A jaula onde a princesa está presa.
    pub(crate) goal: Texture,
    /// O mau que rasteja.
    pub(crate) crawl: Texture,
    /// O mau que rebola.
    pub(crate) roll: Texture,
    /// O mau que flutua.
    pub(crate) float: Texture,
}

impl Textures {
    /// Vai buscar à pasta `textures/` todos os desenhos.
    pub(crate) fn load(filesystem: &Filesystem, graphics: &mut Graphics) -> Result<Self> {
        let mut named = |name: &str| texture(filesystem, graphics, &path(name));
        Ok(Self {
            background: named("background")?,
            platform: named("platform")?,
            life: named("hud_life")?,
            life_lost: named("hud_life_lost")?,
            digits: [
                named("hud_digit_0")?,
                named("hud_digit_1")?,
                named("hud_digit_2")?,
                named("hud_digit_3")?,
                named("hud_digit_4")?,
                named("hud_digit_5")?,
                named("hud_digit_6")?,
                named("hud_digit_7")?,
                named("hud_digit_8")?,
                named("hud_digit_9")?,
            ],
            projectile: named("projectile")?,
            fart: named("fart")?,
            coin: named("coin")?,
            goal: named("goal")?,
            crawl: named("enemy_crawl")?,
            roll: named("enemy_roll")?,
            float: named("enemy_float")?,
        })
    }
}

/// O sítio do ficheiro do desenho `name`.
pub(super) fn path(name: &str) -> String {
    format!("textures/{name}.png")
}

/// O desenho do ficheiro `path`, ou um quadrado de uma cor só se ele ainda não estiver na pasta.
pub(super) fn texture(
    filesystem: &Filesystem,
    graphics: &mut Graphics,
    path: &str,
) -> Result<Texture> {
    match read(filesystem, path)? {
        Some(bytes) => Ok(graphics.texture(&bytes)?),
        None => no_texture(graphics, path),
    }
}
