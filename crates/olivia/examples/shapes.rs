//! Draws every shape, sorted with a sprite, and quits when asked to close.
//!
//! The top row is filled and the bottom row outlined: a rectangle turning around its centre, a
//! circle, and a concave polygon turning around its position. Lines turn around their shared end on
//! the right, and at the far left a sprite sits between two rectangles by `depth`.
//!
//! Run it with `cargo run --example shapes`.

use olivia::ecs::{Clear, Geometry, Hud, Shape, Sprite, Transform, World};
use olivia::graphics::Draw;
use olivia::math::{Color, Vec2};
use olivia::window::WindowEvent;
use olivia::{Config, Game, Result, Setup, Update};

const RED: Color = Color::rgb(1.0, 0.3, 0.3);
const GREEN: Color = Color::rgb(0.3, 1.0, 0.4);
const BLUE: Color = Color::rgb(0.4, 0.5, 1.0);
const YELLOW: Color = Color::rgb(1.0, 0.9, 0.3);

/// An arrow head with a notch in its back, which only a concave fill draws right.
const ARROW: [Vec2; 4] = [
    Vec2::new(0.0, -40.0),
    Vec2::new(35.0, 40.0),
    Vec2::new(0.0, 15.0),
    Vec2::new(-35.0, 40.0),
];

/// A shape that turns with the clock.
struct Turning;

struct Demo;

fn still(world: &mut World, x: f32, y: f32, geometry: Geometry, draw: Draw) {
    let shape = Shape::new(geometry).draw(draw);
    world.spawn((Hud, Transform::at(x, y), shape));
}

fn turning(world: &mut World, x: f32, y: f32, geometry: Geometry, draw: Draw) {
    let shape = Shape::new(geometry).draw(draw);
    world.spawn((Hud, Turning, Transform::at(x, y), shape));
}

impl Game for Demo {
    fn load(s: Setup<'_>) -> Result<Self> {
        let pixel = s.graphics.texture_rgba(1, 1, &[255; 4])?;
        let world = s.world;
        world.spawn((Clear(Color::rgb(0.1, 0.1, 0.12)),));

        // Spawned top first, yet depth puts the sprite between the rectangles.
        let square = Geometry::Rect(Vec2::splat(60.0));
        still(
            world,
            50.0,
            110.0,
            square.clone(),
            Draw::new().color(RED).depth(1.0),
        );
        let sprite = Draw::new().color(YELLOW).scale(Vec2::splat(60.0));
        world.spawn((
            Hud,
            Transform::at(30.0, 90.0),
            Sprite::new(pixel).draw(sprite),
        ));
        still(
            world,
            10.0,
            70.0,
            square,
            Draw::new().color(BLUE).depth(-1.0),
        );

        let centred = Draw::new().origin(Vec2::splat(0.5));
        let size = Vec2::new(100.0, 60.0);
        turning(world, 210.0, 90.0, Geometry::Rect(size), centred.color(RED));
        still(
            world,
            350.0,
            90.0,
            Geometry::Circle(45.0),
            Draw::new().color(GREEN),
        );
        let arrow = Geometry::Polygon(ARROW.to_vec());
        turning(world, 470.0, 90.0, arrow, Draw::new().color(BLUE));

        let outline = Geometry::RectOutline(size, 4.0);
        turning(world, 210.0, 270.0, outline, centred.color(RED));
        let circle = Geometry::CircleOutline(45.0, 2.0);
        still(world, 350.0, 270.0, circle, Draw::new().color(GREEN));
        let arrow = Geometry::PolygonOutline(ARROW.to_vec(), 3.0);
        turning(world, 470.0, 270.0, arrow, Draw::new().color(BLUE));

        for (width, spread) in [(1.0, -100.0), (2.0, -33.0), (4.0, 33.0), (8.0, 100.0)] {
            let line = Geometry::Line(Vec2::new(50.0, spread), width);
            turning(world, 570.0, 180.0, line, Draw::new().color(YELLOW));
        }
        Ok(Self)
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        if u.events.contains(&WindowEvent::CloseRequested) {
            u.quit();
        }
        for (_, transform) in u.world.query_mut::<(&Turning, &mut Transform)>() {
            transform.rotation += 0.01;
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    olivia::run::<Demo>(Config::new("shapes").title("Olivia").size(640.0, 360.0))
}
