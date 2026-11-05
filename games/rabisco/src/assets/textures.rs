use olivia::{
    Result,
    filesystem::Filesystem,
    graphics::{Graphics, Texture},
};

use super::{fallback::no_texture, read};

#[derive(Debug)]
pub(crate) struct Textures {
    pub(crate) background: Texture,
    pub(crate) platform: Texture,
    pub(crate) life: Texture,
    pub(crate) life_lost: Texture,
    /// The digits 0 to 9, each at its own index.
    pub(crate) digits: [Texture; 10],
    pub(crate) projectile: Texture,
    pub(crate) fart: Texture,
    pub(crate) coin: Texture,
    pub(crate) goal: Texture,
    pub(crate) crawl: Texture,
    pub(crate) roll: Texture,
    pub(crate) float: Texture,
}

impl Textures {
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

/// Where the texture `name` is.
pub(super) fn path(name: &str) -> String {
    format!("textures/{name}.png")
}

/// The texture at `path`, or a square of a colour of its own if it is not there yet.
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
