//! Loops an animation on a character, both under the assets folder, with an optional second
//! animation looping over it as an overlay and a marker at `hand_right`. Quits when asked to close.
//! This is how captures are reviewed.
//!
//! Run it with
//! `cargo run --example animation -- characters/stand-in animations/stand-in.json animations/stand-in-shoot.json`.

use std::env;

use olivia::animation::{Animation, Character, Play};
use olivia::ecs::{Camera, Clear, Entity, Sprite, Transform};
use olivia::graphics::Draw;
use olivia::math::{Color, Vec2};
use olivia::window::WindowEvent;
use olivia::{Config, Game, Result, Setup, Update};

const USAGE: &str = "usage: animation <character folder> <animation file> [overlay file]";

struct Demo {
    hero: Entity,
    marker: Entity,
    animation: Animation,
    overlay: Option<Animation>,
}

impl Game for Demo {
    fn load(s: Setup<'_>) -> Result<Self> {
        let mut args = env::args().skip(1);
        let folder = args.next().ok_or(USAGE)?;
        let file = args.next().ok_or(USAGE)?;
        let overlay = args.next();

        s.world.spawn((Clear(Color::rgb(0.1, 0.15, 0.2)),));
        s.world.spawn((Camera {
            position: Vec2::ZERO,
            zoom: 0.4,
            rotation: 0.0,
        },));

        let character = Character::load(s.filesystem, s.graphics, &folder)?;
        let hero = s.world.spawn((Transform::default(), character));
        let animation = Animation::parse(&s.filesystem.read(&file)?)?;
        let overlay = overlay
            .map(|file| Animation::parse(&s.filesystem.read(&file)?))
            .transpose()?;

        let dot = s.graphics.texture_rgba(1, 1, &[255, 255, 0, 255])?;
        let draw = Draw::new()
            .origin(Vec2::splat(0.5))
            .scale(Vec2::splat(16.0))
            .depth(-1.0);
        let marker = s
            .world
            .spawn((Transform::default(), Sprite::new(dot).draw(draw)));
        Ok(Self {
            hero,
            marker,
            animation,
            overlay,
        })
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        if u.events.contains(&WindowEvent::CloseRequested) {
            u.quit();
        }
        let mut hero = u.world.get::<&mut Character>(self.hero)?;
        hero.play(&self.animation, Play::new().looping(true))?;
        if let Some(overlay) = &self.overlay {
            hero.overlay(overlay, Play::new().looping(true).crossfade(0.15))?;
        }
        if let Some(hand) = hero.joint("hand_right") {
            u.world.get::<&mut Transform>(self.marker)?.position = hand.offset;
        }
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
