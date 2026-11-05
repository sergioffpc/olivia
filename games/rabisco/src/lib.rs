//! O Rabisco: um jogo de plataformas feito pela turma.
//!
//! Os desenhos, os sons e os movimentos das personagens foram feitos pelos alunos. Este código
//! junta tudo e põe o jogo a funcionar. Cada ficheiro trata de uma parte do jogo:
//!
//! - `game`: o jogo inteiro, que liga todas as outras partes.
//! - `assets`: os desenhos, os sons e os movimentos que o jogo vai buscar às pastas.
//! - `level`: o nível, com o chão, os maus, as moedas, o herói e a princesa.
//! - `camera`: a câmara, que segue o herói.
//! - `hud`: as vidas e os pontos, desenhados por cima do jogo.
//! - `status`: como está a correr o jogo: as vidas, os pontos e se já acabou.
//! - `depth`: o que se desenha à frente e o que se desenha atrás.
//! - `random`: números à sorte, para o jogo não ser sempre igual.
//! - `sprite`: uma ajuda para pôr um desenho do tamanho certo.

// É um jogo, não uma biblioteca: as suas partes não são usadas por outros programas.
#![allow(missing_docs, clippy::missing_errors_doc, clippy::missing_panics_doc)]

mod assets;
mod camera;
mod depth;
mod game;
mod hud;
mod level;
mod random;
mod sprite;
mod status;

pub use game::Rabisco;
