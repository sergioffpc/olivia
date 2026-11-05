use olivia::{
    Result,
    animation::Character,
    filesystem::Filesystem,
    graphics::{Draw, Graphics, depth},
    math::Vec2,
};

use super::textures::{path, texture};
use crate::random::Random;

/// Parts are 8 pixels per millimetre of sheet: this turns them into metres.
const PART_SCALE: f32 = 0.0011;
const HERO_DEPTH: f32 = depth::PLAYFIELD + 20.0;
const PRINCESS_DEPTH: f32 = depth::PLAYFIELD + 2.0;

#[derive(Debug)]
pub(crate) struct Characters {
    hero: Character,
    princess: Character,
}

impl Characters {
    pub(crate) fn load(
        filesystem: &Filesystem,
        graphics: &mut Graphics,
        random: &mut Random,
    ) -> Result<Self> {
        let parts = Draw::new().scale(Vec2::splat(PART_SCALE));
        let hero = character(filesystem, graphics, random, "hero")?.draw(parts.depth(HERO_DEPTH));
        let princess =
            character(filesystem, graphics, random, "princess")?.draw(parts.depth(PRINCESS_DEPTH));

        Ok(Self { hero, princess })
    }
}

/// The character `skeletons/{name}.json`, its parts the textures `{name}_{part}`.
fn character(
    filesystem: &Filesystem,
    graphics: &mut Graphics,
    random: &mut Random,
    name: &str,
) -> Result<Character> {
    let skeleton = filesystem.read(&format!("skeletons/{name}.json"))?;
    Character::parse(&skeleton, |part| {
        texture(
            filesystem,
            graphics,
            random,
            &path(&format!("{name}_{part}")),
        )
    })
}
