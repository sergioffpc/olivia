//! As personagens, feitas de partes desenhadas presas umas às outras, como um boneco articulado.

use olivia::{
    Result,
    animation::Character,
    filesystem::Filesystem,
    graphics::{Draw, Graphics},
    math::Vec2,
};

use super::{
    fallback::NO_SKELETON,
    read,
    textures::{path, texture},
};
use crate::depth;

/// Quanto encolher as partes desenhadas para ficarem do tamanho certo, em metros.
///
/// As folhas de desenho foram fotografadas com 8 píxeis por milímetro.
pub(crate) const PART_SCALE: f32 = 0.0011;
/// A distância dos pés ao ponto de onde a personagem se segura, a meio do tronco, em metros.
///
/// O herói e a princesa foram desenhados na mesma folha de corpo, `tools/make_sheets.py`, por isso
/// a distância é a mesma para os dois. Foi medida em `skeletons/hero.json` com [`PART_SCALE`]:
/// se algum deles mudar, é preciso medir outra vez.
pub(crate) const ROOT_HEIGHT: f32 = 0.88;

/// As personagens do jogo.
#[derive(Debug)]
pub(crate) struct Characters {
    /// O herói, que tu controlas.
    pub(crate) hero: Character,
    /// A princesa, que o herói vem salvar.
    pub(crate) princess: Character,
}

impl Characters {
    /// Vai buscar às pastas as personagens e põe cada uma à sua profundidade.
    pub(crate) fn load(filesystem: &Filesystem, graphics: &mut Graphics) -> Result<Self> {
        let parts = Draw::new().scale(Vec2::splat(PART_SCALE));
        let hero = character(filesystem, graphics, "hero")?.draw(parts.depth(depth::HERO));
        let princess =
            character(filesystem, graphics, "princess")?.draw(parts.depth(depth::PRINCESS));

        Ok(Self { hero, princess })
    }
}

/// A personagem `name`: o esqueleto `skeletons/{name}.json`, e um desenho `{name}_{parte}` para
/// cada parte do corpo.
fn character(filesystem: &Filesystem, graphics: &mut Graphics, name: &str) -> Result<Character> {
    // O esqueleto diz que partes há e onde se prendem umas às outras.
    let skeleton = read(filesystem, &format!("skeletons/{name}.json"))?;
    // Depois vai buscar o desenho de cada parte, à medida que o esqueleto as pede.
    Character::parse(skeleton.as_deref().unwrap_or(NO_SKELETON), |part| {
        texture(filesystem, graphics, &path(&format!("{name}_{part}")))
    })
}
