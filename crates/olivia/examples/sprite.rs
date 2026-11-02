//! Draws a sprite through a turning world camera, with a HUD on top, and quits when asked to close.
//!
//! Run it with `cargo run --example sprite`.

use olivia::graphics::{Camera, Draw, Texture};
use olivia::math::{Color, Rect, Vec2};
use olivia::window::WindowEvent;
use olivia::{Config, Frame, Game, Result, Setup, Update};

struct Demo {
    hero: Texture,
    /// The hero's head, a region sharing its pixels.
    head: Texture,
    viewport: Vec2,
    time: f32,
}

impl Game for Demo {
    fn load(s: Setup<'_>) -> Result<Self> {
        let bytes = s.filesystem.read("hero.png")?;
        let hero = s.graphics.texture(&bytes)?;
        Ok(Self {
            head: hero.region(Rect::new(3.0, 0.0, 10.0, 8.0)),
            hero,
            viewport: Vec2::ZERO,
            time: 0.0,
        })
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        for event in u.events {
            match event {
                WindowEvent::Resized(size) => self.viewport = *size,
                WindowEvent::CloseRequested => u.quit(),
                _ => {}
            }
        }
        self.time += u.timer.dt();
        Ok(())
    }

    fn draw(&self, f: &mut Frame<'_>) {
        f.clear(Color::rgb(0.1, 0.15, 0.2));

        let mut world = f.view(Camera {
            position: Vec2::new(self.time.sin() * 40.0, 0.0),
            zoom: 4.0,
            rotation: self.time * 0.25,
        });
        let centred = Draw::new().origin(Vec2::splat(0.5));
        world.sprite(&self.hero, Vec2::new(-30.0, 0.0), centred);
        let flipped = centred.scale(Vec2::new(-1.0, 1.0));
        world.sprite(&self.hero, Vec2::ZERO, flipped);
        let tinted = centred
            .rotation(self.time)
            .color(Color::new(0.4, 1.0, 0.4, 0.6));
        world.sprite(&self.hero, Vec2::new(30.0, 0.0), tinted);

        let mut hud = f.view(Camera::screen(self.viewport));
        let scaled = Draw::new().scale(Vec2::splat(3.0));
        hud.sprite(&self.head, Vec2::new(8.0, 8.0), scaled);
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    olivia::run::<Demo>(Config::new("sprite").title("Olivia").size(640.0, 360.0))
}
