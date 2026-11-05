use olivia::{Result, animation::Animation, filesystem::Filesystem};

use super::{fallback::NO_ANIMATION, read};

#[derive(Debug)]
pub(crate) struct Animations {
    idle: Animation,
    walk: Animation,
    jump: Animation,
    double_jump: Animation,
    shoot: Animation,
    hurt: Animation,
    win: Animation,
    calling: Animation,
    freed: Animation,
}

impl Animations {
    pub(crate) fn load(filesystem: &Filesystem) -> Result<Self> {
        Ok(Self {
            idle: animation(filesystem, "hero_idle")?,
            walk: animation(filesystem, "hero_walk")?,
            jump: animation(filesystem, "hero_jump")?,
            double_jump: animation(filesystem, "hero_double_jump")?,
            shoot: animation(filesystem, "hero_shoot")?,
            hurt: animation(filesystem, "hero_hurt")?,
            win: animation(filesystem, "hero_win")?,
            calling: animation(filesystem, "princess_calling")?,
            freed: animation(filesystem, "princess_freed")?,
        })
    }
}

/// The animation `animations/{name}.json`.
fn animation(filesystem: &Filesystem, name: &str) -> Result<Animation> {
    let bytes = read(filesystem, &format!("animations/{name}.json"))?;
    Ok(Animation::parse(bytes.as_deref().unwrap_or(NO_ANIMATION))?)
}
