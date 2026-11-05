//! Shows `depth`, `filter` and `blend`, and quits when asked to close.
//!
//! Left to right: squares drawn back to front whatever their spawn order, a small checker scaled up
//! with each filter, and overlapping squares with each blend.
//!
//! Run it with `cargo run --example draw_options`.

use olivia::ecs::{CClear, CHud, CSprite, CTransform, World};
use olivia::graphics::{Blend, Draw, Filter, Texture};
use olivia::math::{Color, Vec2};
use olivia::window::WindowEvent;
use olivia::{Config, Game, Result, Setup, Update};

const RED: Color = Color::new(1.0, 0.2, 0.2, 0.8);
const GREEN: Color = Color::new(0.2, 1.0, 0.2, 0.8);
const BLUE: Color = Color::new(0.2, 0.2, 1.0, 0.8);

struct Demo;

fn sprite(world: &mut World, texture: &Texture, x: f32, y: f32, draw: Draw) {
    let sprite = CSprite::new(texture.clone()).draw(draw);
    world.spawn((CHud, CTransform::at(x, y), sprite));
}

/// Three overlapping squares of `draw`, red, green then blue from `(x, y)`.
fn squares(world: &mut World, white: &Texture, x: f32, y: f32, draw: Draw) {
    let square = draw.scale(Vec2::splat(80.0));
    sprite(world, white, x, y, square.color(RED));
    sprite(world, white, x + 40.0, y + 20.0, square.color(GREEN));
    sprite(world, white, x + 20.0, y + 50.0, square.color(BLUE));
}

impl Game for Demo {
    fn load(s: Setup<'_>) -> Result<Self> {
        let white = s.graphics.texture_rgba(1, 1, &[255; 4])?;
        #[rustfmt::skip]
        let checker = s.graphics.texture_rgba(2, 2, &[
            255, 255, 255, 255,   30, 30, 30, 255,
            30, 30, 30, 255,      255, 255, 255, 255,
        ])?;
        let world = s.world;
        world.spawn((CClear(Color::rgb(0.1, 0.1, 0.12)),));

        // Spawned red first, yet red ends on top.
        let square = Draw::new().scale(Vec2::splat(80.0));
        sprite(world, &white, 40.0, 40.0, square.color(RED).depth(2.0));
        sprite(world, &white, 70.0, 60.0, square.color(GREEN).depth(1.0));
        sprite(world, &white, 50.0, 80.0, square.color(BLUE).depth(-1.0));

        let scaled = Draw::new().scale(Vec2::splat(50.0));
        sprite(world, &checker, 220.0, 40.0, scaled);
        sprite(
            world,
            &checker,
            220.0,
            170.0,
            scaled.filter(Filter::Nearest),
        );

        squares(world, &white, 370.0, 40.0, Draw::new());
        squares(world, &white, 370.0, 180.0, Draw::new().blend(Blend::Add));
        Ok(Self)
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        if u.events.contains(&WindowEvent::CloseRequested) {
            u.quit();
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    olivia::run::<Demo>(
        Config::new("draw_options")
            .title("Olivia")
            .size(640.0, 360.0),
    )
}
