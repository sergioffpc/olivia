use olivia::{
    ecs::{CHud, CSprite, CTransform, Entity, World},
    graphics::Texture,
    math::Vec2,
};

use crate::{assets::Textures, game::WINDOW, sprite::sized, status::LIVES};

/// How many digits the score shows.
const DIGITS: usize = 3;
/// The side of an icon, in logical pixels.
const HUD_SIZE: f32 = 52.0;
/// How far the icons are from the window's edges.
const HUD_MARGIN: f32 = 16.0;
/// The room between two lives.
const HUD_GAP: f32 = 6.0;
/// The y of every icon's centre.
const ROW_Y: f32 = HUD_MARGIN + HUD_SIZE / 2.0;

/// The lives at the top left and the score at the top right, showing the status they follow.
#[derive(Debug)]
#[expect(dead_code, reason = "read once the HUD follows the game")]
pub(crate) struct Hud {
    /// The lives, from the left.
    lives: Vec<Entity>,
    /// The digits of the score, from the most significant.
    digits: Vec<Entity>,
    /// The entity with the `CStatus` shown.
    status: Entity,
}

impl Hud {
    pub(crate) fn spawn(world: &mut World, textures: &Textures, status: Entity) -> Self {
        let first_life = HUD_MARGIN + HUD_SIZE / 2.0;
        let life_step = HUD_SIZE + HUD_GAP;
        let lives = spawn_row(world, &textures.life, LIVES as usize, first_life, life_step);
        #[allow(clippy::cast_precision_loss)] // a handful of digits
        let first_digit = WINDOW.x - first_life - (DIGITS - 1) as f32 * HUD_SIZE;
        let digits = spawn_row(world, &textures.digits[0], DIGITS, first_digit, HUD_SIZE);

        Self {
            lives,
            digits,
            status,
        }
    }
}

/// A row of `count` icons on the HUD, from `first` rightwards `step` apart, all showing `texture`.
fn spawn_row(
    world: &mut World,
    texture: &Texture,
    count: usize,
    first: f32,
    step: f32,
) -> Vec<Entity> {
    let mut x = first;
    let mut row = Vec::with_capacity(count);
    for _ in 0..count {
        let draw = sized(texture, Vec2::splat(HUD_SIZE));
        row.push(world.spawn((
            CHud,
            CTransform::at(x, ROW_Y),
            CSprite::new(texture.clone()).draw(draw),
        )));
        x += step;
    }
    row
}
