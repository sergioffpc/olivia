//! `resume`, which only the web needs, leaves the desktop's audio as it was.

use std::time::Duration;

use olivia_audio::Audio;

/// Lasts 0.2 s.
const TONE: &[u8] = include_bytes!("fixtures/tone.wav");

#[test]
fn resume_leaves_a_playing_sound_playing() {
    let mut audio = Audio::mock(8);
    let sound = audio.sound(TONE.to_vec()).unwrap();
    let cue = audio.play(&sound);
    audio.resume();
    let (left, right) = audio.advance(Duration::from_millis(100));
    assert!(cue.is_playing());
    assert!(left > 0.0 && right > 0.0);
}

#[test]
fn resume_does_nothing_without_a_device() {
    let mut audio = Audio::without_device();
    audio.resume();
    let sound = audio.sound(TONE.to_vec()).unwrap();
    assert!(!audio.play(&sound).is_playing());
}
