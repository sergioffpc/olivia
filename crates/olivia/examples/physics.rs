//! Drops boxes kept as `hecs` entities onto the ground, and quits when asked to close.
//!
//! Click empty space to drop a box and click a box to remove it. Tab shows the frame rate, the
//! colliders and their bounding boxes, which the game draws itself.
//!
//! Run it with `cargo run --example physics`.

use hecs::Entity;
use olivia::graphics::{Camera, Draw, Font, View};
use olivia::input::{Key, Mouse};
use olivia::math::{Color, Rect, Vec2};
use olivia::physics::{ColliderBuilder, ColliderHandle, RigidBodyBuilder, RigidBodyHandle, World};
use olivia::window::WindowEvent;
use olivia::{Config, Frame, Game, Result, Setup, Update};

const GRAVITY: Vec2 = Vec2::new(0.0, 9.81);
const PIXELS_PER_METRE: f32 = 40.0;
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

/// A box entity's body in the physics world.
struct Body(RigidBodyHandle);

/// A box entity's colour.
struct Tint(Color);

struct Demo {
    physics: World,
    entities: hecs::World,
    font: Font,
    camera: Camera,
    viewport: Vec2,
    frame_rate: f32,
    diagnostics: bool,
    dropped: usize,
}

impl Demo {
    fn drop_box(&mut self, position: Vec2) {
        let body = self
            .physics
            .insert_body(RigidBodyBuilder::dynamic().translation(position));
        let tint = Tint(BOX_COLOURS[self.dropped % BOX_COLOURS.len()]);
        self.dropped += 1;
        let entity = self.entities.spawn((Body(body), tint));
        let collider = ColliderBuilder::cuboid(BOX_HALF_SIZE.x, BOX_HALF_SIZE.y)
            .user_data(entity.to_bits().get().into());
        self.physics.insert_collider(collider, Some(body));
    }

    /// The entity `collider` belongs to. The ground has none.
    fn entity_of(&self, collider: ColliderHandle) -> Option<Entity> {
        let bits = self.physics.collider(collider).user_data;
        Entity::from_bits(u64::try_from(bits).ok()?)
    }

    fn remove_box(&mut self, entity: Entity) -> Result<()> {
        let body = self.entities.get::<&Body>(entity)?.0;
        self.physics.remove_body(body);
        self.entities.despawn(entity)?;
        Ok(())
    }

    fn click(&mut self, position: Vec2) -> Result<()> {
        let colliders = self.physics.overlap_point(position);
        let Some(&collider) = colliders.first() else {
            self.drop_box(position);
            return Ok(());
        };
        if let Some(entity) = self.entity_of(collider) {
            self.remove_box(entity)?;
        }
        Ok(())
    }

    fn draw_bodies(&self, view: &mut View<'_>, factor: f32) {
        let corner = GROUND_CENTRE - GROUND_HALF_SIZE;
        let size = GROUND_HALF_SIZE * 2.0;
        let ground = Rect::new(corner.x, corner.y, size.x, size.y);
        view.rect(ground, Draw::new().color(Color::rgb(0.3, 0.3, 0.35)));

        let size = BOX_HALF_SIZE * 2.0;
        for (Body(body), Tint(colour)) in &mut self.entities.query::<(&Body, &Tint)>() {
            let position = self.physics.interpolated_position(*body, factor);
            let draw = Draw::new()
                .origin(Vec2::splat(0.5))
                .rotation(self.physics.interpolated_angle(*body, factor))
                .color(*colour);
            view.rect(Rect::new(position.x, position.y, size.x, size.y), draw);
        }
    }

    fn draw_debug_lines(&self, view: &mut View<'_>) {
        let width = 1.0 / self.camera.zoom;
        for [a, b] in self.physics.debug_lines() {
            view.line(a, b, width, Draw::new().color(DIAGNOSTICS_COLOUR));
        }
    }

    fn draw_frame_rate(&self, view: &mut View<'_>) {
        let text = format!("{:.0} FPS", self.frame_rate);
        let pos = Vec2::splat(8.0);
        let size = self.font.measure(&text, FONT_SIZE);
        let panel = Rect::new(pos.x, pos.y, size.x, size.y);
        view.rect_outline(panel, 1.0, Draw::new().color(DIAGNOSTICS_COLOUR));
        view.text(
            &self.font,
            FONT_SIZE,
            &text,
            pos,
            Draw::new().color(DIAGNOSTICS_COLOUR),
        );
    }
}

impl Game for Demo {
    fn load(s: Setup<'_>) -> Result<Self> {
        let bytes = s.filesystem.read("DejaVuSansMono.ttf")?;
        let mut physics = World::new(GRAVITY);
        physics.insert_collider(
            ColliderBuilder::cuboid(GROUND_HALF_SIZE.x, GROUND_HALF_SIZE.y)
                .translation(GROUND_CENTRE),
            None,
        );
        let mut demo = Self {
            physics,
            entities: hecs::World::new(),
            font: s.graphics.font(&bytes)?,
            camera: Camera {
                position: Vec2::ZERO,
                zoom: PIXELS_PER_METRE,
                rotation: 0.0,
            },
            viewport: Vec2::ZERO,
            frame_rate: 0.0,
            diagnostics: false,
            dropped: 0,
        };
        for position in FIRST_BOXES {
            demo.drop_box(position);
        }
        Ok(demo)
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        for event in u.events {
            match event {
                WindowEvent::Resized(size) => self.viewport = *size,
                WindowEvent::CloseRequested => u.quit(),
                _ => {}
            }
        }
        if u.input.pressed(Key::Tab) {
            self.diagnostics = !self.diagnostics;
        }
        if u.input.pressed(Mouse::Left) {
            self.click(self.camera.screen_to_world(u.input.cursor(), self.viewport))?;
        }
        self.frame_rate = u.timer.frame_rate();
        self.physics.step();
        Ok(())
    }

    fn draw(&self, f: &mut Frame<'_>) {
        f.clear(Color::rgb(0.1, 0.1, 0.12));
        let factor = f.interpolation_factor();
        let mut view = f.view(self.camera);
        self.draw_bodies(&mut view, factor);
        if self.diagnostics {
            self.draw_debug_lines(&mut view);
            let mut hud = f.view(Camera::screen(self.viewport));
            self.draw_frame_rate(&mut hud);
        }
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    olivia::run::<Demo>(
        Config::new("physics")
            .title("Olivia physics")
            .size(960.0, 540.0),
    )
}
