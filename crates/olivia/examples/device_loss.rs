//! Destroys the GPU device every few seconds and keeps drawing with the same texture and font, to check
//! recovery by hand. The sprite and the text must keep coming back, with the count going up.
//!
//! Run it with `RUST_LOG=olivia_graphics=debug cargo run --example device_loss`.

use olivia::graphics::{Camera, Draw, Font, Texture};
use olivia::math::{Color, Vec2};
use olivia::window::WindowEvent;
use olivia::{Config, Frame, Game, Result, Setup, Update};

const SECONDS_BETWEEN_LOSSES: f32 = 3.0;

struct Demo {
    hero: Texture,
    font: Font,
    viewport: Vec2,
    until_loss: f32,
    losses: u32,
}

impl Game for Demo {
    fn load(s: Setup<'_>) -> Result<Self> {
        let hero = s.filesystem.read("hero.png")?;
        let font = s.filesystem.read("DejaVuSansMono.ttf")?;
        Ok(Self {
            hero: s.graphics.texture(&hero)?,
            font: s.graphics.font(&font)?,
            viewport: Vec2::ZERO,
            until_loss: SECONDS_BETWEEN_LOSSES,
            losses: 0,
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
        self.until_loss -= u.timer.dt();
        if self.until_loss <= 0.0 {
            self.until_loss = SECONDS_BETWEEN_LOSSES;
            self.losses += 1;
            u.graphics.raw().0.destroy();
        }
        Ok(())
    }

    fn draw(&self, f: &mut Frame<'_>) {
        f.clear(Color::rgb(0.1, 0.15, 0.2));
        let mut hud = f.view(Camera::screen(self.viewport));
        let scaled = Draw::new().scale(Vec2::splat(4.0));
        hud.sprite(&self.hero, Vec2::new(16.0, 16.0), scaled);
        let text = format!("devices lost: {}", self.losses);
        hud.text(&self.font, 24.0, &text, Vec2::new(16.0, 120.0), Draw::new());
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    olivia::run::<Demo>(Config::new("device-loss").size(640.0, 360.0))
}
