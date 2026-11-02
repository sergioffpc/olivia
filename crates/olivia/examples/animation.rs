//! Loops an animation on a character, both under the assets folder, and quits when asked to close.
//! This is how captures are reviewed.
//!
//! Run it with
//! `cargo run --example animation -- characters/stand-in animations/stand-in.json`.

use std::env;

use olivia::animation::{Animation, Character, Play};
use olivia::ecs::{Camera, Clear, Entity, Transform};
use olivia::math::{Color, Vec2};
use olivia::window::WindowEvent;
use olivia::{Config, Game, Result, Setup, Update};

const USAGE: &str = "usage: animation <character folder> <animation file>";

struct Demo {
    hero: Entity,
    animation: Animation,
}

impl Game for Demo {
    fn load(s: Setup<'_>) -> Result<Self> {
        let mut args = env::args().skip(1);
        let folder = args.next().ok_or(USAGE)?;
        let file = args.next().ok_or(USAGE)?;

        s.world.spawn((Clear(Color::rgb(0.1, 0.15, 0.2)),));
        s.world.spawn((Camera {
            position: Vec2::ZERO,
            zoom: 0.4,
            rotation: 0.0,
        },));

        let character = Character::load(s.filesystem, s.graphics, &folder)?;
        let hero = s.world.spawn((Transform::default(), character));
        let animation = Animation::parse(&s.filesystem.read(&file)?)?;
        Ok(Self { hero, animation })
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        if u.events.contains(&WindowEvent::CloseRequested) {
            u.quit();
        }
        let mut hero = u.world.get::<&mut Character>(self.hero)?;
        hero.play(&self.animation, Play::new().looping(true))?;
        Ok(())
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    olivia::run::<Demo>(
        Config::new("animation")
            .title("Olivia animation")
            .size(960.0, 540.0),
    )
}
