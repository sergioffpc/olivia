use olivia::{
    ecs::{Entity, Hud, Sprite, Transform, World},
    graphics::Texture,
    math::Vec2,
};

use crate::{
    assets::Textures,
    game::WINDOW,
    sprite::sized,
    status::{LIVES, Status},
};

const POINTS: usize = 3;
const HUD_SIZE: f32 = 52.0;
const HUD_MARGIN: f32 = 16.0;

#[derive(Debug)]
pub(crate) struct HudManager {
    lives: Vec<Entity>,
    digits: Vec<Entity>,
    status: Entity,
}

impl HudManager {
    pub(crate) fn load(textures: &Textures, world: &mut World) -> Self {
        let first = HUD_MARGIN + HUD_SIZE / 2.0;
        let lives = spawn_row(world, LIVES as usize, first, HUD_SIZE + 6.0, &textures.life);
        let last = WINDOW.x - first;
        let digits = spawn_row(world, POINTS, last, -HUD_SIZE, &textures.digits[0]);
        let status = world.spawn((Status::new(),));

        Self {
            lives,
            digits,
            status,
        }
    }
}

/// A row of icons on the HUD, `step` apart from `first`, all showing `texture`.
fn spawn_row(
    world: &mut World,
    count: usize,
    first: f32,
    step: f32,
    texture: &Texture,
) -> Vec<Entity> {
    let y = HUD_MARGIN + HUD_SIZE / 2.0;
    let mut x = first;
    let mut row = Vec::with_capacity(count);
    for _ in 0..count {
        let draw = sized(texture, Vec2::splat(HUD_SIZE));
        row.push(world.spawn((
            Hud,
            Transform::at(x, y),
            Sprite::new(texture.clone()).draw(draw),
        )));
        x += step;
    }
    row
}
