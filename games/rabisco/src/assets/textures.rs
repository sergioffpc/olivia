use olivia::{
    Result,
    filesystem::Filesystem,
    graphics::{Graphics, Texture},
};

use super::{fallback::no_texture, read};
use crate::random::Random;

#[derive(Debug)]
pub(crate) struct Textures {
    pub(crate) background: Texture,
    pub(crate) platform: Texture,
    pub(crate) life: Texture,
    pub(crate) life_lost: Texture,
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
    pub(crate) fn load(
        filesystem: &Filesystem,
        graphics: &mut Graphics,
        random: &mut Random,
    ) -> Result<Self> {
        let mut named = |name: &str| texture(filesystem, graphics, random, &path(name));
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

/// The texture at `path`, or a square of a random colour if it is not there yet.
pub(super) fn texture(
    filesystem: &Filesystem,
    graphics: &mut Graphics,
    random: &mut Random,
    path: &str,
) -> Result<Texture> {
    match read(filesystem, path)? {
        Some(bytes) => Ok(graphics.texture(&bytes)?),
        None => no_texture(graphics, random),
    }
}
