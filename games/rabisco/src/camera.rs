//! A câmara: o pedaço do nível que se vê na janela.

use olivia::{
    Result,
    ecs::{self, CTransform, Entity, World},
    math::Vec2,
};

use crate::{game::WINDOW, level::Level};

/// Quantos píxeis da janela ocupa um metro do jogo.
const PIXELS_PER_METRE: f32 = 100.0;
/// Quanto do nível a câmara vê de cada vez, em metros.
pub(crate) const VIEW: Vec2 = Vec2::new(WINDOW.x / PIXELS_PER_METRE, WINDOW.y / PIXELS_PER_METRE);

/// A câmara, que vai atrás do herói pelo nível fora.
#[derive(Debug)]
pub(crate) struct Camera {
    /// Onde a câmara está guardada no mundo do jogo.
    entity: Entity,
}

impl Camera {
    /// Põe a câmara no mundo do jogo, a olhar para o começo do nível.
    pub(crate) fn spawn(world: &mut World) -> Self {
        let entity = world.spawn((ecs::Camera {
            position: VIEW / 2.0,
            zoom: PIXELS_PER_METRE,
            rotation: 0.0,
        },));
        Self { entity }
    }

    /// Anda com a câmara atrás do herói, e o fundo do nível anda com ela.
    pub(crate) fn update(&self, world: &mut World, level: &Level) -> Result<()> {
        // Vê onde está o herói.
        let x = world.get::<&CTransform>(level.hero())?.position.x;
        // Não deixa a câmara passar das pontas do nível, para nunca se ver o que está para lá.
        let half = VIEW.x / 2.0;
        let x = x.clamp(half, Level::WIDTH - half);
        // Põe a câmara lá.
        world.get::<&mut ecs::Camera>(self.entity)?.position.x = x;
        // Pede ao nível que mexa o fundo.
        level.scroll(world, x)
    }
}
