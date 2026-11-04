//! Loops a streamed track and plays a sound over it. Space plays the sound, P pauses or resumes the
//! track, the arrow keys change the track's pan and the master volume, and Enter fades the track out.
//! Dropping the sound's cue does not stop it.
//!
//! Run it with `RUST_LOG=warn cargo run --example audio`.

use std::time::Duration;

use olivia::audio::{Cue, Play, Sound};
use olivia::input::Key;
use olivia::window::WindowEvent;
use olivia::{Config, Game, Result, Setup, Update};

struct Demo {
    beep: Sound,
    theme: Cue,
    is_paused: bool,
    pan: f32,
    master_volume: f32,
}

impl Game for Demo {
    fn load(s: Setup<'_>) -> Result<Self> {
        let beep = s.audio.sound(s.filesystem.read("beep.wav")?)?;
        let theme = s.audio.stream(s.filesystem.read("theme.ogg")?)?;
        let theme = s
            .audio
            .play_with(&theme, Play::new().volume(0.5).looping(true));
        Ok(Self {
            beep,
            theme,
            is_paused: false,
            pan: 0.0,
            master_volume: 1.0,
        })
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        for event in u.events {
            if let WindowEvent::CloseRequested = event {
                u.quit();
            }
        }

        let input = u.input;
        if input.pressed(Key::Space) {
            u.audio.play(&self.beep);
        }
        if input.pressed(Key::P) {
            self.is_paused = !self.is_paused;
            if self.is_paused {
                self.theme.pause();
            } else {
                self.theme.resume();
            }
        }
        if input.pressed(Key::Enter) {
            self.theme.stop_after(Duration::from_secs(2));
        }

        let step = u.timer.dt();
        if input.down(Key::Left) {
            self.pan = (self.pan - step).max(-1.0);
            self.theme.set_pan(self.pan);
        }
        if input.down(Key::Right) {
            self.pan = (self.pan + step).min(1.0);
            self.theme.set_pan(self.pan);
        }
        if input.down(Key::Down) {
            self.master_volume = (self.master_volume - step).max(0.0);
            u.audio.set_master_volume(self.master_volume);
        }
        if input.down(Key::Up) {
            self.master_volume = (self.master_volume + step).min(1.0);
            u.audio.set_master_volume(self.master_volume);
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    olivia::run::<Demo>(Config::new("olivia-audio").title("Audio"))
}
