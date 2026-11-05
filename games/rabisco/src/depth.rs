//! O que se desenha à frente e o que se desenha atrás.
//!
//! É como fazer uma colagem: primeiro colamos o fundo, depois o chão, e por fim as personagens por
//! cima de tudo. Quanto maior o número, mais à frente fica o desenho.

pub(crate) use olivia::graphics::depth::BACKDROP as BACKGROUND;
use olivia::graphics::depth::PLAYFIELD;

/// O chão e as plataformas onde o herói anda.
pub(crate) const PLATFORM: f32 = PLAYFIELD + 1.0;
/// A princesa que o herói vem salvar.
pub(crate) const PRINCESS: f32 = PLAYFIELD + 2.0;
/// A jaula onde a princesa está presa: fica à frente dela, para se verem as grades.
pub(crate) const GOAL: f32 = PLAYFIELD + 4.0;
/// As coisas do nível: as moedas e os maus.
pub(crate) const THING: f32 = PLAYFIELD + 10.0;
/// O herói, à frente de tudo o que há no nível.
pub(crate) const HERO: f32 = PLAYFIELD + 20.0;
