//! As vidas e os pontos, desenhados por cima do jogo.
//!
//! Chama-se HUD, das palavras inglesas "heads-up display": o que se vê sem tirar os olhos do jogo,
//! como os números no painel de um carro.

use olivia::{
    Result,
    ecs::{CHud, CSprite, CTransform, Entity, World},
    graphics::Texture,
    math::Vec2,
};

use crate::{
    assets::Textures,
    game::WINDOW,
    sprite::sized,
    status::{CStatus, LIVES},
};

/// Quantos algarismos tem o número dos pontos.
const DIGITS: usize = 3;
/// O tamanho de cada desenho do HUD, em píxeis.
const HUD_SIZE: f32 = 52.0;
/// O espaço entre os desenhos e as bordas da janela.
const HUD_MARGIN: f32 = 16.0;
/// O espaço entre duas vidas.
const HUD_GAP: f32 = 6.0;
/// A altura do meio de todos os desenhos do HUD.
const ROW_Y: f32 = HUD_MARGIN + HUD_SIZE / 2.0;

/// As vidas em cima à esquerda e os pontos em cima à direita, a mostrar o placar.
#[derive(Debug)]
pub(crate) struct Hud {
    /// Os desenhos das vidas, da esquerda para a direita.
    lives: Vec<Entity>,
    /// Os algarismos dos pontos, pela ordem em que se leem.
    digits: Vec<Entity>,
    /// Onde está guardado o placar que o HUD mostra.
    status: Entity,
}

impl Hud {
    /// Põe no mundo do jogo os desenhos das vidas e dos pontos.
    pub(crate) fn spawn(world: &mut World, textures: &Textures, status: Entity) -> Self {
        // As vidas, a começar junto à borda da esquerda, com um espacinho entre elas.
        let first_life = HUD_MARGIN + HUD_SIZE / 2.0;
        let life_step = HUD_SIZE + HUD_GAP;
        let lives = spawn_row(world, &textures.life, LIVES as usize, first_life, life_step);
        // Os algarismos, encostados uns aos outros, a acabar junto à borda da direita.
        #[allow(clippy::cast_precision_loss)] // são poucos algarismos
        let first_digit = WINDOW.x - first_life - (DIGITS - 1) as f32 * HUD_SIZE;
        let digits = spawn_row(world, &textures.digits[0], DIGITS, first_digit, HUD_SIZE);

        Self {
            lives,
            digits,
            status,
        }
    }

    /// Mostra o placar como está agora: as vidas e os pontos.
    pub(crate) fn update(&self, world: &World, textures: &Textures) -> Result<()> {
        let status = *world.get::<&CStatus>(self.status)?;
        self.show_lives(world, textures, status.lives)?;
        self.show_score(world, textures, status.score)
    }

    /// Mostra as `lives` vidas que faltam, e risca as que já se perderam.
    fn show_lives(&self, world: &World, textures: &Textures, lives: u32) -> Result<()> {
        // Conta as vidas da esquerda para a direita: as primeiras ainda as temos.
        for (index, entity) in (0u32..).zip(&self.lives) {
            let life = if index < lives {
                &textures.life
            } else {
                &textures.life_lost
            };
            world.get::<&mut CSprite>(*entity)?.texture = life.clone();
        }
        Ok(())
    }

    /// Mostra os pontos `score`, um algarismo em cada desenho.
    fn show_score(&self, world: &World, textures: &Textures, score: u32) -> Result<()> {
        // Começa pelas unidades, o último algarismo, e vai andando para a esquerda: o resto da
        // divisão por 10 é o algarismo, e dividir por 10 passa ao seguinte.
        let mut score = score as usize;
        for entity in self.digits.iter().rev() {
            world.get::<&mut CSprite>(*entity)?.texture = textures.digits[score % 10].clone();
            score /= 10;
        }
        Ok(())
    }
}

/// Põe uma fila de `count` desenhos iguais, `texture`, a começar em `first` e a andar `step`
/// para a direita de cada vez.
fn spawn_row(
    world: &mut World,
    texture: &Texture,
    count: usize,
    first: f32,
    step: f32,
) -> Vec<Entity> {
    let mut x = first;
    let mut row = Vec::with_capacity(count);
    for _ in 0..count {
        // Põe um desenho no sítio.
        let draw = sized(texture, Vec2::splat(HUD_SIZE));
        row.push(world.spawn((
            CHud,
            CTransform::at(x, ROW_Y),
            CSprite::new(texture.clone()).draw(draw),
        )));
        // Anda para a direita, para o desenho seguinte.
        x += step;
    }
    row
}
