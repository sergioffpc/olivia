use hecs::Component;
#[cfg(feature = "animation")]
use olivia_animation::Character;
use olivia_graphics::{Camera, Target, View};
use olivia_math::{Rect, Vec2};
#[cfg(feature = "physics")]
use olivia_physics::World;

use crate::ecs::{self, CClear, CHud, CShape, CSprite, CText, CTransform, Entity, Geometry};
#[cfg(feature = "physics")]
use crate::ecs::{CBody, CColliderLines};

/// Where the entities are for this frame.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Poses<'a> {
    #[cfg(feature = "physics")]
    pub(crate) physics: &'a World,
    #[cfg(any(feature = "animation", feature = "physics"))]
    pub(crate) factor: f32,
    #[cfg(not(feature = "physics"))]
    pub(crate) world: std::marker::PhantomData<&'a ()>,
}

impl Poses<'_> {
    /// `transform`, moved between the last two steps if `entity` has a body.
    #[cfg_attr(not(feature = "physics"), expect(clippy::unused_self))]
    #[cfg_attr(not(feature = "physics"), expect(unused_variables))]
    fn of(self, world: &ecs::World, entity: Entity, transform: CTransform) -> CTransform {
        #[cfg(feature = "physics")]
        if let Ok(body) = world.get::<&CBody>(entity) {
            return CTransform {
                position: self.physics.interpolated_position(body.0, self.factor),
                rotation: self.physics.interpolated_angle(body.0, self.factor),
            };
        }
        transform
    }
}

/// Draws a frame: everything through each camera in spawn order, then the HUD.
pub(crate) fn render(world: &ecs::World, poses: Poses<'_>, target: &mut Target<'_>) {
    if let Some(CClear(color)) = world.query::<&CClear>().iter().next() {
        target.clear(*color);
    }

    let mut cameras: Vec<(Entity, Camera)> = world
        .query::<(Entity, &Camera)>()
        .iter()
        .map(|(entity, camera)| (entity, *camera))
        .collect();
    cameras.sort_by_key(|(entity, _)| entity.id());
    for (_, camera) in cameras {
        let mut view = target.view(camera);
        draw_entities(world, poses, &mut view, false);
        #[cfg(feature = "physics")]
        draw_collider_lines(world, poses, &mut view, camera);
    }

    let viewport = target.viewport();
    let mut hud = target.view(Camera::screen(viewport));
    draw_entities(world, poses, &mut hud, true);
}

/// Draws the entities that are part of the HUD, or the ones that are not.
fn draw_entities(world: &ecs::World, poses: Poses<'_>, view: &mut View<'_>, hud: bool) {
    each_entity(world, poses, hud, |sprite: &CSprite, pose| {
        let draw = sprite.draw.rotation(pose.rotation);
        view.sprite(&sprite.texture, pose.position, draw);
    });

    each_entity(world, poses, hud, |text: &CText, pose| {
        let draw = text.draw.rotation(pose.rotation);
        let (font, size, string) = (&text.font, text.size, &text.text);
        match text.width {
            Some(width) => view.text_wrapped(font, size, width, string, pose.position, draw),
            None => view.text(font, size, string, pose.position, draw),
        }
    });

    each_entity(world, poses, hud, |shape: &CShape, pose| {
        draw_shape(view, shape, pose);
    });

    #[cfg(feature = "animation")]
    each_entity(world, poses, hud, |character: &Character, pose| {
        character.render(view, pose.position, pose.rotation, poses.factor);
    });
}

/// Calls `draw` with the component `C` and the pose of each entity that is part of the HUD, or of
/// each that is not.
fn each_entity<C: Component>(
    world: &ecs::World,
    poses: Poses<'_>,
    hud: bool,
    mut draw: impl FnMut(&C, CTransform),
) {
    let mut entities = world.query::<(Entity, &CTransform, &C, Option<&CHud>)>();
    for (entity, transform, component, marker) in &mut entities {
        if marker.is_some() == hud {
            draw(component, poses.of(world, entity, *transform));
        }
    }
}

fn draw_shape(view: &mut View<'_>, shape: &CShape, pose: CTransform) {
    let draw = shape.draw.rotation(pose.rotation);
    let pos = pose.position;
    let rect = |size: Vec2| Rect::new(pos.x, pos.y, size.x, size.y);
    match &shape.geometry {
        Geometry::Rect(size) => view.rect(rect(*size), draw),
        Geometry::RectOutline(size, width) => view.rect_outline(rect(*size), *width, draw),
        Geometry::Circle(radius) => view.circle(pos, *radius, draw),
        Geometry::CircleOutline(radius, width) => view.circle_outline(pos, *radius, *width, draw),
        Geometry::Polygon(points) => view.polygon(pos, points, draw),
        Geometry::PolygonOutline(points, width) => view.polygon_outline(pos, points, *width, draw),
        Geometry::Line(to, width) => view.line(pos, pos + *to, *width, draw),
    }
}

#[cfg(feature = "physics")]
fn draw_collider_lines(world: &ecs::World, poses: Poses<'_>, view: &mut View<'_>, camera: Camera) {
    let mut lines = world.query::<&CColliderLines>();
    let Some(CColliderLines(color)) = lines.iter().next() else {
        return;
    };
    let draw = olivia_graphics::Draw::new().color(*color);
    for [a, b] in poses.physics.debug_lines() {
        view.line(a, b, 1.0 / camera.zoom, draw);
    }
}
