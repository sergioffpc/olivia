//! Como está a correr o jogo: as vidas, os pontos e se já acabou.

use olivia::ecs::{Entity, World};

/// Quantas vidas o herói tem no começo do jogo.
pub(crate) const LIVES: u32 = 3;

/// Se o jogo ainda está a decorrer, ou como acabou.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// Ainda estamos a jogar.
    Playing,
    /// Ganhámos: a princesa foi salva!
    Won,
    /// Perdemos: o herói ficou sem vidas.
    Lost,
}

/// O placar do jogo, guardado numa coisa só para ele no mundo do jogo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CStatus {
    /// Quantas vidas faltam ao herói.
    pub(crate) lives: u32,
    /// Quantas moedas o herói já apanhou.
    pub(crate) score: u32,
    /// Se ainda estamos a jogar, ou se já ganhámos ou perdemos.
    pub(crate) outcome: Outcome,
    /// Se a princesa já pediu ajuda: ela só pede uma vez em cada jogo.
    pub(crate) has_called: bool,
}

impl CStatus {
    /// O placar de um jogo novo: todas as vidas, zero moedas, e a jogar.
    pub(crate) const fn new() -> Self {
        Self {
            lives: LIVES,
            score: 0,
            outcome: Outcome::Playing,
            has_called: false,
        }
    }
}

/// Põe no mundo do jogo o placar de um jogo novo, e diz onde ficou guardado.
pub(crate) fn spawn(world: &mut World) -> Entity {
    world.spawn((CStatus::new(),))
}
