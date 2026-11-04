//! Draws sprites through a turning world camera, with a HUD on top, and quits when asked to close.
//!
//! Run it with `cargo run --example sprite`.

use olivia::ecs::{Camera, Clear, Entity, Hud, Sprite, Transform};
use olivia::graphics::Draw;
use olivia::math::{Color, Rect, Vec2};
use olivia::window::WindowEvent;
use olivia::{Config, Game, Result, Setup, Update};

/// A sprite that turns with the clock.
struct Spinning;

struct Demo {
    camera: Entity,
    time: f32,
}

impl Game for Demo {
    fn load(s: Setup<'_>) -> Result<Self> {
        let bytes = s.filesystem.read("hero.png")?;
        let hero = s.graphics.texture(&bytes)?;
        let head = hero.region(Rect::new(3.0, 0.0, 10.0, 8.0));

        s.world.spawn((Clear(Color::rgb(0.1, 0.15, 0.2)),));
        let camera = s.world.spawn((Camera {
            position: Vec2::ZERO,
            zoom: 4.0,
            rotation: 0.0,
        },));

        let centred = Draw::new().origin(Vec2::splat(0.5));
        let sprite = Sprite::new(hero).draw(centred);
        s.world.spawn((Transform::at(-30.0, 0.0), sprite.clone()));
        let flipped = centred.scale(Vec2::new(-1.0, 1.0));
        s.world
            .spawn((Transform::default(), sprite.clone().draw(flipped)));
        let tinted = centred.color(Color::new(0.4, 1.0, 0.4, 0.6));
        s.world
            .spawn((Spinning, Transform::at(30.0, 0.0), sprite.draw(tinted)));

        let scaled = Draw::new().scale(Vec2::splat(3.0));
        s.world
            .spawn((Hud, Transform::at(8.0, 8.0), Sprite::new(head).draw(scaled)));
        Ok(Self { camera, time: 0.0 })
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        if u.events.contains(&WindowEvent::CloseRequested) {
            u.quit();
        }
        self.time += u.timer.dt();
        let mut camera = u.world.get::<&mut Camera>(self.camera)?;
        camera.position.x = self.time.sin() * 40.0;
        camera.rotation = self.time * 0.25;
        drop(camera);
        for (_, transform) in u.world.query_mut::<(&Spinning, &mut Transform)>() {
            transform.rotation = self.time;
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    olivia::run::<Demo>(Config::new("sprite").title("Olivia").size(640.0, 360.0))
}
