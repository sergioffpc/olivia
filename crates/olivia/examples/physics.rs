//! Drops boxes onto the ground, and quits when asked to close.
//!
//! Click empty space to drop a box and click a box to remove it. Tab shows the frame rate, the
//! colliders and their bounding boxes.
//!
//! Run it with `cargo run --example physics`.

use olivia::ecs::{
    CClear, CColliderLines, CHud, CShape, CText, CTransform, Camera, ColliderBuilder, Entity,
    Geometry, RigidBodyBuilder, World, entity_of,
};
use olivia::graphics::Draw;
use olivia::input::{Key, Mouse};
use olivia::math::{Color, Vec2};
use olivia::window::WindowEvent;
use olivia::{Config, Game, Result, Setup, Update};

const PIXELS_PER_METRE: f32 = 40.0;
const CAMERA: Camera = Camera {
    position: Vec2::ZERO,
    zoom: PIXELS_PER_METRE,
    rotation: 0.0,
};
const GROUND_CENTRE: Vec2 = Vec2::new(0.0, 5.5);
const GROUND_HALF_SIZE: Vec2 = Vec2::new(10.0, 0.5);
const BOX_HALF_SIZE: Vec2 = Vec2::new(0.5, 0.5);
const FIRST_BOXES: [Vec2; 3] = [
    Vec2::new(-1.0, 0.0),
    Vec2::new(-0.4, -2.0),
    Vec2::new(0.2, -4.0),
];
const BOX_COLOURS: [Color; 3] = [
    Color::rgb(0.9, 0.4, 0.3),
    Color::rgb(0.3, 0.7, 0.5),
    Color::rgb(0.3, 0.5, 0.9),
];
const DIAGNOSTICS_COLOUR: Color = Color::rgb(1.0, 0.9, 0.2);
const FONT_SIZE: f32 = 18.0;

/// A box the player can remove.
struct CCrate;

struct Demo {
    frame_rate: Entity,
    diagnostics: Option<Entity>,
    viewport: Vec2,
    dropped: usize,
}

impl Demo {
    fn drop_box(&mut self, world: &mut World, position: Vec2) {
        let tint = BOX_COLOURS[self.dropped % BOX_COLOURS.len()];
        self.dropped += 1;
        let size = BOX_HALF_SIZE * 2.0;
        let centred = Draw::new().origin(Vec2::splat(0.5)).color(tint);
        world.spawn((
            CCrate,
            CTransform::at(position.x, position.y),
            RigidBodyBuilder::dynamic(),
            ColliderBuilder::cuboid(BOX_HALF_SIZE.x, BOX_HALF_SIZE.y),
            CShape::new(Geometry::Rect(size)).draw(centred),
        ));
    }

    fn click(&mut self, u: &mut Update<'_>, position: Vec2) -> Result<()> {
        let colliders = u.physics.overlap_point(position);
        let Some(&collider) = colliders.first() else {
            self.drop_box(u.world, position);
            return Ok(());
        };
        if let Some(entity) = entity_of(u.physics, collider)
            && u.world.satisfies::<&CCrate>(entity)
        {
            u.world.despawn(entity)?;
        }
        Ok(())
    }

    fn toggle_diagnostics(&mut self, world: &mut World) -> Result<()> {
        if let Some(entity) = self.diagnostics.take() {
            world.despawn(entity)?;
        } else {
            let lines = world.spawn((CColliderLines(DIAGNOSTICS_COLOUR),));
            self.diagnostics = Some(lines);
        }
        Ok(())
    }
}

impl Game for Demo {
    fn load(s: Setup<'_>) -> Result<Self> {
        let bytes = s.filesystem.read("DejaVuSansMono.ttf")?;
        let font = s.graphics.font(&bytes)?;
        s.world.spawn((CClear(Color::rgb(0.1, 0.1, 0.12)),));
        s.world.spawn((CAMERA,));

        let centred = Draw::new()
            .origin(Vec2::splat(0.5))
            .color(Color::rgb(0.3, 0.3, 0.35));
        s.world.spawn((
            CTransform::at(GROUND_CENTRE.x, GROUND_CENTRE.y),
            ColliderBuilder::cuboid(GROUND_HALF_SIZE.x, GROUND_HALF_SIZE.y),
            CShape::new(Geometry::Rect(GROUND_HALF_SIZE * 2.0)).draw(centred),
        ));

        let colour = Draw::new().color(DIAGNOSTICS_COLOUR);
        let frame_rate = s.world.spawn((
            CHud,
            CTransform::at(8.0, 8.0),
            CText::new(font, FONT_SIZE, "").draw(colour),
        ));

        let mut demo = Self {
            frame_rate,
            diagnostics: None,
            viewport: Vec2::ZERO,
            dropped: 0,
        };
        for position in FIRST_BOXES {
            demo.drop_box(s.world, position);
        }
        Ok(demo)
    }

    fn update(&mut self, mut u: Update<'_>) -> Result<()> {
        for event in u.events {
            match event {
                WindowEvent::Resized(size) => self.viewport = *size,
                WindowEvent::CloseRequested => u.quit(),
                _ => {}
            }
        }
        if u.input.pressed(Key::Tab) {
            self.toggle_diagnostics(u.world)?;
        }
        if u.input.pressed(Mouse::Left) {
            let cursor = CAMERA.screen_to_world(u.input.cursor(), self.viewport);
            self.click(&mut u, cursor)?;
        }
        let mut frame_rate = u.world.get::<&mut CText>(self.frame_rate)?;
        frame_rate.text = match self.diagnostics {
            Some(_) => format!("{:.0} FPS", u.timer.frame_rate()),
            None => String::new(),
        };
        Ok(())
    }
}

fn main() -> Result<()> {
    olivia::run::<Demo>(
        Config::new("physics")
            .title("Olivia physics")
            .size(960.0, 540.0),
    )
}
