//! Os movimentos das personagens, gravados pela turma com a câmara.

use olivia::{Result, animation::Animation, filesystem::Filesystem};

use super::{fallback::NO_ANIMATION, read};

/// Os movimentos de todas as personagens, cada um no ficheiro
/// `animations/{personagem}_{movimento}.json`.
#[derive(Debug)]
pub(crate) struct Animations {
    /// Os movimentos do herói.
    pub(crate) hero: HeroAnimations,
    /// Os movimentos da princesa.
    pub(crate) princess: PrincessAnimations,
}

/// Os movimentos do herói.
#[derive(Debug)]
pub(crate) struct HeroAnimations {
    /// Parado, à espera.
    pub(crate) idle: Animation,
    /// A andar.
    pub(crate) walk: Animation,
    /// A saltar.
    pub(crate) jump: Animation,
    /// A dar o salto extra, no ar, às voltas.
    pub(crate) double_jump: Animation,
    /// A disparar: só mexe o braço, por cima do que estiver a fazer.
    pub(crate) shoot: Animation,
    /// Magoado.
    pub(crate) hurt: Animation,
    /// A festejar a vitória.
    pub(crate) win: Animation,
}

/// Os movimentos da princesa.
#[derive(Debug)]
pub(crate) struct PrincessAnimations {
    /// Presa, a pedir ajuda.
    pub(crate) calling: Animation,
    /// Livre, contente.
    pub(crate) freed: Animation,
}

impl Animations {
    /// Vai buscar às pastas os movimentos de todas as personagens.
    pub(crate) fn load(filesystem: &Filesystem) -> Result<Self> {
        let hero = |name: &str| animation(filesystem, &format!("hero_{name}"));
        let princess = |name: &str| animation(filesystem, &format!("princess_{name}"));
        Ok(Self {
            hero: HeroAnimations {
                idle: hero("idle")?,
                walk: hero("walk")?,
                jump: hero("jump")?,
                double_jump: hero("double_jump")?,
                shoot: hero("shoot")?,
                hurt: hero("hurt")?,
                win: hero("win")?,
            },
            princess: PrincessAnimations {
                calling: princess("calling")?,
                freed: princess("freed")?,
            },
        })
    }
}

/// O movimento do ficheiro `animations/{name}.json`, ou um que não mexe nada se ele faltar.
fn animation(filesystem: &Filesystem, name: &str) -> Result<Animation> {
    let bytes = read(filesystem, &format!("animations/{name}.json"))?;
    Ok(Animation::parse(bytes.as_deref().unwrap_or(NO_ANIMATION))?)
}
