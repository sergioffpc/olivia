//! Loops a track from an emitter that moves back and forth past the listener, from left to right,
//! fading out at both ends. Space plays a sound from the emitter's position.
//!
//! Run it with `RUST_LOG=warn cargo run --example spatial`.

use olivia::audio::{Emitter, Play, Sound};
use olivia::input::Key;
use olivia::math::Vec2;
use olivia::window::WindowEvent;
use olivia::{Config, Frame, Game, Result, Setup, Update};

const NEAR: f32 = 50.0;
const FAR: f32 = 500.0;
const SWEEP: f32 = 600.0;
const SWEEP_SPEED: f32 = 0.5;

struct Demo {
    beep: Sound,
    emitter: Emitter,
    phase: f32,
}

impl Game for Demo {
    fn load(s: Setup<'_>) -> Result<Self> {
        let beep = s.audio.sound(s.filesystem.read("beep.wav")?)?;
        let theme = s.audio.stream(s.filesystem.read("theme.ogg")?)?;
        let mut emitter = s.audio.emitter(Vec2::ZERO, NEAR, FAR);
        emitter.play_with(&theme, Play::new().looping(true));
        Ok(Self {
            beep,
            emitter,
            phase: 0.0,
        })
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        for event in u.events {
            if let WindowEvent::CloseRequested = event {
                u.quit();
            }
        }

        self.phase += u.timer.dt() * SWEEP_SPEED;
        self.emitter
            .set_position(Vec2::new(SWEEP * self.phase.sin(), 0.0));
        if u.input.pressed(Key::Space) {
            self.emitter.play(&self.beep);
        }
        Ok(())
    }

    fn draw(&self, _f: &mut Frame<'_>) {}
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    olivia::run::<Demo>(Config::new("olivia-spatial").title("Spatial sound"))
}
