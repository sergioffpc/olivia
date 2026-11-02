//! How audio reaches the game, driven frame by frame.
#![cfg(feature = "audio")]

use std::time::Duration;

use olivia::audio::Sound;
use olivia::math::Vec2;
use olivia::window::Window;
use olivia::{Config, Game, GameLoop, Modules, Result, Setup, Update};

const TONE: &[u8] = include_bytes!("../../olivia-audio/tests/fixtures/tone.wav");

/// Loads a sound in load and plays it in update, whether or not this machine has a device.
struct Player {
    tone: Sound,
}

impl Game for Player {
    fn load(s: Setup<'_>) -> Result<Self> {
        let tone = s.audio.sound(TONE.to_vec())?;
        s.audio.play(&tone);
        Ok(Self { tone })
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        u.audio.play(&self.tone);
        u.audio.set_master_volume(0.5);
        u.quit();
        Ok(())
    }
}

#[test]
fn load_and_update_play_sounds_and_the_game_tears_down() {
    let mut window = Window::headless();
    let modules = Modules::new(&Config::new("olivia-test")).unwrap();
    let mut game_loop = GameLoop::<Player>::new(&mut window, Vec2::ZERO, modules).unwrap();
    let keep_going = game_loop
        .run_frame(Duration::from_millis(17), &mut window)
        .unwrap();
    assert!(!keep_going);
}
