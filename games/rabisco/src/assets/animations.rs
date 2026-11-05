use olivia::{Result, animation::Animation, filesystem::Filesystem};

use super::{fallback::NO_ANIMATION, read};

/// The animations of every character, `animations/{character}_{name}.json`.
#[derive(Debug)]
pub(crate) struct Animations {
    pub(crate) hero: HeroAnimations,
    pub(crate) princess: PrincessAnimations,
}

#[derive(Debug)]
pub(crate) struct HeroAnimations {
    pub(crate) idle: Animation,
    pub(crate) walk: Animation,
    pub(crate) jump: Animation,
    pub(crate) double_jump: Animation,
    pub(crate) shoot: Animation,
    pub(crate) hurt: Animation,
    pub(crate) win: Animation,
}

#[derive(Debug)]
pub(crate) struct PrincessAnimations {
    pub(crate) calling: Animation,
    pub(crate) freed: Animation,
}

impl Animations {
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

/// The animation `animations/{name}.json`.
fn animation(filesystem: &Filesystem, name: &str) -> Result<Animation> {
    let bytes = read(filesystem, &format!("animations/{name}.json"))?;
    Ok(Animation::parse(bytes.as_deref().unwrap_or(NO_ANIMATION))?)
}
