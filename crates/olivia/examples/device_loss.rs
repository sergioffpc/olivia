//! Destroys the GPU device every few seconds and keeps drawing with the same texture and font, to check
//! recovery by hand. The sprite and the text must keep coming back, with the count going up.
//!
//! Run it with `RUST_LOG=olivia_graphics=debug cargo run --example device_loss`.

use olivia::ecs::{Clear, Entity, Hud, Sprite, Text, Transform};
use olivia::graphics::Draw;
use olivia::math::{Color, Vec2};
use olivia::window::WindowEvent;
use olivia::{Config, Game, Result, Setup, Update};

const SECONDS_BETWEEN_LOSSES: f32 = 3.0;

struct Demo {
    count: Entity,
    until_loss: f32,
    losses: u32,
}

impl Game for Demo {
    fn load(s: Setup<'_>) -> Result<Self> {
        let hero = s.filesystem.read("hero.png")?;
        let font = s.filesystem.read("DejaVuSansMono.ttf")?;
        let hero = s.graphics.texture(&hero)?;
        let font = s.graphics.font(&font)?;

        s.world.spawn((Clear(Color::rgb(0.1, 0.15, 0.2)),));
        let scaled = Draw::new().scale(Vec2::splat(4.0));
        s.world.spawn((
            Hud,
            Transform::at(16.0, 16.0),
            Sprite::new(hero).draw(scaled),
        ));
        let count = s.world.spawn((
            Hud,
            Transform::at(16.0, 120.0),
            Text::new(font, 24.0, "devices lost: 0"),
        ));
        Ok(Self {
            count,
            until_loss: SECONDS_BETWEEN_LOSSES,
            losses: 0,
        })
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        if u.events.contains(&WindowEvent::CloseRequested) {
            u.quit();
        }
        self.until_loss -= u.timer.dt();
        if self.until_loss <= 0.0 {
            self.until_loss = SECONDS_BETWEEN_LOSSES;
            self.losses += 1;
            u.world.get::<&mut Text>(self.count)?.text = format!("devices lost: {}", self.losses);
            u.graphics.raw().0.destroy();
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    olivia::run::<Demo>(Config::new("device-loss").size(640.0, 360.0))
}
