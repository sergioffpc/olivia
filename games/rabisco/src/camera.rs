use olivia::{
    Result,
    ecs::{self, CTransform, Entity, World},
    math::Vec2,
};

use crate::{game::WINDOW, level::Level};

/// How many logical pixels a metre takes.
const PIXELS_PER_METRE: f32 = 100.0;
/// How much the camera sees, in metres.
pub(crate) const VIEW: Vec2 = Vec2::new(WINDOW.x / PIXELS_PER_METRE, WINDOW.y / PIXELS_PER_METRE);

/// The camera on the level, following the hero along it.
#[derive(Debug)]
pub(crate) struct Camera {
    entity: Entity,
}

impl Camera {
    pub(crate) fn spawn(world: &mut World) -> Self {
        let entity = world.spawn((ecs::Camera {
            position: VIEW / 2.0,
            zoom: PIXELS_PER_METRE,
            rotation: 0.0,
        },));
        Self { entity }
    }

    /// Follows the hero across `level`, and the level's background with it.
    pub(crate) fn update(&self, world: &mut World, level: &Level) -> Result<()> {
        // Kept half a view from either end, so that it never shows past the level.
        let half = VIEW.x / 2.0;
        let x = world.get::<&CTransform>(level.hero())?.position.x;
        let x = x.clamp(half, Level::WIDTH - half);
        world.get::<&mut ecs::Camera>(self.entity)?.position.x = x;
        level.scroll(world, x)
    }
}
