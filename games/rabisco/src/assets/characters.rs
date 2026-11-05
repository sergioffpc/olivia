use olivia::{
    Result,
    animation::Character,
    filesystem::Filesystem,
    graphics::{Draw, Graphics},
    math::Vec2,
};

use super::{
    fallback::NO_SKELETON,
    read,
    textures::{path, texture},
};
use crate::depth;

/// Parts are 8 pixels per millimetre of sheet: this turns them into metres.
const PART_SCALE: f32 = 0.0011;
/// How far the hero's feet are under its root joint as drawn, in metres: measured on
/// `skeletons/hero.json` at [`PART_SCALE`], and so to measure again when either changes.
pub(crate) const HERO_ROOT_HEIGHT: f32 = 0.88;

#[derive(Debug)]
pub(crate) struct Characters {
    pub(crate) hero: Character,
    #[expect(dead_code, reason = "spawned once the level has the princess")]
    pub(crate) princess: Character,
}

impl Characters {
    pub(crate) fn load(filesystem: &Filesystem, graphics: &mut Graphics) -> Result<Self> {
        let parts = Draw::new().scale(Vec2::splat(PART_SCALE));
        let hero = character(filesystem, graphics, "hero")?.draw(parts.depth(depth::HERO));
        let princess =
            character(filesystem, graphics, "princess")?.draw(parts.depth(depth::PRINCESS));

        Ok(Self { hero, princess })
    }
}

/// The character `skeletons/{name}.json`, its parts the textures `{name}_{part}`.
fn character(filesystem: &Filesystem, graphics: &mut Graphics, name: &str) -> Result<Character> {
    let skeleton = read(filesystem, &format!("skeletons/{name}.json"))?;
    Character::parse(skeleton.as_deref().unwrap_or(NO_SKELETON), |part| {
        texture(filesystem, graphics, &path(&format!("{name}_{part}")))
    })
}
