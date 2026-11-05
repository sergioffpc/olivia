use olivia::{
    ecs::{self, Entity, World},
    math::Vec2,
};

use crate::game::WINDOW;

/// How many logical pixels a metre takes.
const PIXELS_PER_METRE: f32 = 100.0;
/// How much the camera sees, in metres.
pub(crate) const VIEW: Vec2 = Vec2::new(WINDOW.x / PIXELS_PER_METRE, WINDOW.y / PIXELS_PER_METRE);

/// The camera on the level, at its start.
#[derive(Debug)]
pub(crate) struct Camera {
    #[expect(dead_code, reason = "read once the camera follows the hero")]
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
}
