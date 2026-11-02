//! How sounds play from positions in the game's space, heard through a mock output.

use std::time::Duration;

use olivia_audio::{Audio, Emitter, Play, Sound};
use olivia_math::Vec2;

/// Lasts 0.2 s.
const TONE: &[u8] = include_bytes!("fixtures/tone.wav");

const SETTLED: Duration = Duration::from_millis(50);
const NEAR: f32 = 10.0;
const FAR: f32 = 100.0;

/// `kira`'s limit on sub-tracks, and on the sounds of each.
const LIMIT: usize = 128;

fn tone(audio: &mut Audio) -> Sound {
    audio.sound(TONE.to_vec()).unwrap()
}

fn looping() -> Play {
    Play::new().looping(true)
}

/// The loudest left and right samples once any transition is over.
fn settled(audio: &mut Audio) -> (f32, f32) {
    audio.advance(SETTLED);
    audio.advance(SETTLED)
}

/// What a looping tone sounds like from `pos`, with the listener at `(0, 0)`.
fn heard_from(pos: Vec2) -> (f32, f32) {
    let mut audio = Audio::mock(8);
    let sound = tone(&mut audio);
    let mut emitter = audio.emitter(pos, NEAR, FAR);
    emitter.play_with(&sound, looping());
    settled(&mut audio)
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-3,
        "{actual} is not {expected}"
    );
}

#[test]
fn a_sound_moves_with_its_emitter() {
    let mut audio = Audio::mock(8);
    let sound = tone(&mut audio);
    let mut emitter = audio.emitter(Vec2::ZERO, NEAR, FAR);
    let cue = emitter.play_with(&sound, looping());
    let (left, right) = settled(&mut audio);
    assert!(left > 0.0 && right > 0.0);

    emitter.set_position(Vec2::new(FAR, 0.0));

    assert_eq!(settled(&mut audio), (0.0, 0.0));
    assert!(cue.is_playing());
}

#[test]
fn dropping_an_emitter_lets_a_sound_just_played_finish() {
    let mut audio = Audio::mock(8);
    let sound = tone(&mut audio);
    let mut emitter = audio.emitter(Vec2::ZERO, NEAR, FAR);
    audio.advance(SETTLED);

    let cue = emitter.play(&sound);
    drop(emitter);

    let (left, right) = audio.advance(Duration::from_millis(100));
    assert!(left > 0.0 && right > 0.0);
    assert!(cue.is_playing());
    audio.advance(Duration::from_millis(300));
    assert!(!cue.is_playing());
}

#[test]
fn a_dropped_emitter_keeps_its_last_position() {
    let mut audio = Audio::mock(8);
    let sound = tone(&mut audio);
    let mut emitter = audio.emitter(Vec2::ZERO, NEAR, FAR);
    let mut cue = emitter.play_with(&sound, looping());
    emitter.set_position(Vec2::new(FAR, 0.0));
    drop(emitter);
    audio.set_listener(Vec2::new(FAR, 0.0));

    let (left, right) = settled(&mut audio);

    assert!(left > 0.0 && right > 0.0);
    cue.stop();
    audio.advance(SETTLED);
    assert!(!cue.is_playing());
}

#[test]
fn dropped_emitters_give_their_slots_back_once_silent() {
    let mut audio = Audio::mock(8);
    let sound = tone(&mut audio);
    for _ in 0..LIMIT * 2 {
        audio.emitter(Vec2::ZERO, NEAR, FAR).play(&sound).stop();
        audio.advance(SETTLED);
    }
    assert!(
        audio
            .emitter(Vec2::ZERO, NEAR, FAR)
            .play(&sound)
            .is_playing()
    );
}

#[test]
fn pan_follows_the_side_of_the_listener() {
    let (left, right) = heard_from(Vec2::new(-NEAR, 0.0));
    assert!(left > right);
    let (left, right) = heard_from(Vec2::new(NEAR, 0.0));
    assert!(right > left);
    let (left, right) = heard_from(Vec2::ZERO);
    assert_close(left, right);
}

#[test]
fn sounds_are_at_full_volume_up_to_near() {
    let (centre_left, centre_right) = heard_from(Vec2::ZERO);
    let (near_left, near_right) = heard_from(Vec2::new(0.0, NEAR));
    // Pan alone takes a little off, where distance would take about a quarter.
    assert!(near_left + near_right > (centre_left + centre_right) * 0.9);

    let (past_left, past_right) = heard_from(Vec2::new(0.0, NEAR * 2.0));
    assert!(past_left + past_right < near_left + near_right);
}

#[test]
fn sounds_fall_silent_at_far_and_not_before() {
    assert_eq!(heard_from(Vec2::new(FAR, 0.0)), (0.0, 0.0));
    let (left, right) = heard_from(Vec2::new(FAR * 0.9, 0.0));
    assert!(left > 0.0 && right > 0.0);
}

#[test]
fn the_listener_starts_at_the_origin_and_moves() {
    let mut audio = Audio::mock(8);
    let sound = tone(&mut audio);
    let pos = Vec2::new(FAR, 0.0);
    let mut emitter = audio.emitter(pos, NEAR, FAR);
    emitter.play_with(&sound, looping());
    assert_eq!(settled(&mut audio), (0.0, 0.0));

    audio.set_listener(pos);

    let (left, right) = settled(&mut audio);
    assert!(left > 0.0);
    assert_close(left, right);
}

#[test]
fn play_ignores_pan_and_volume_combines_with_distance() {
    let pos = Vec2::new(NEAR * 3.0, 0.0);
    let hear = |play: Play| {
        let mut audio = Audio::mock(8);
        let sound = tone(&mut audio);
        audio
            .emitter(pos, NEAR, FAR)
            .play_with(&sound, play.looping(true));
        settled(&mut audio)
    };
    let (left, right) = hear(Play::new());
    assert_eq!(hear(Play::new().pan(1.0)), (left, right));

    let (half_left, half_right) = hear(Play::new().volume(0.5));
    assert_close(half_left, left / 2.0);
    assert_close(half_right, right / 2.0);
}

#[test]
fn master_volume_reaches_emitters() {
    let mut audio = Audio::mock(8);
    let sound = tone(&mut audio);
    let mut emitter = audio.emitter(Vec2::ZERO, NEAR, FAR);
    emitter.play_with(&sound, looping());
    audio.set_master_volume(0.0);
    assert_eq!(settled(&mut audio), (0.0, 0.0));
}

#[test]
fn sounds_played_from_audio_stay_put() {
    let mut audio = Audio::mock(8);
    let sound = tone(&mut audio);
    audio.play_with(&sound, looping());
    let before = settled(&mut audio);

    audio.set_listener(Vec2::new(FAR * 10.0, 0.0));

    assert_eq!(settled(&mut audio), before);
}

#[test]
#[should_panic(expected = "near")]
fn near_at_far_panics() {
    Audio::mock(8).emitter(Vec2::ZERO, FAR, FAR);
}

#[test]
#[should_panic(expected = "near")]
fn near_beyond_far_panics_with_no_device_too() {
    Audio::without_device().emitter(Vec2::ZERO, FAR, NEAR);
}

fn assert_inert(mut emitter: Emitter, sound: &Sound) {
    emitter.set_position(Vec2::ONE);
    assert!(!emitter.play(sound).is_playing());
    assert!(!emitter.play_with(sound, looping()).is_playing());
}

#[test]
fn with_no_device_an_emitter_is_inert() {
    let mut audio = Audio::without_device();
    let sound = tone(&mut audio);
    audio.set_listener(Vec2::ONE);
    assert_inert(audio.emitter(Vec2::ZERO, NEAR, FAR), &sound);
}

#[test]
fn above_the_sub_track_limit_an_emitter_is_inert() {
    let mut audio = Audio::mock(8);
    let sound = tone(&mut audio);
    let emitters: Vec<_> = (0..LIMIT)
        .map(|_| audio.emitter(Vec2::ZERO, NEAR, FAR))
        .collect();
    assert_inert(audio.emitter(Vec2::ZERO, NEAR, FAR), &sound);
    drop(emitters);
}

#[test]
fn above_the_sound_limit_an_emitter_refuses_sounds_and_streams() {
    let mut audio = Audio::mock(8);
    let sound = tone(&mut audio);
    let stream = audio.stream(TONE.to_vec()).unwrap();
    let mut emitter = audio.emitter(Vec2::ZERO, NEAR, FAR);
    for _ in 0..LIMIT {
        assert!(emitter.play(&sound).is_playing());
    }
    assert!(!emitter.play(&sound).is_playing());
    assert!(!emitter.play(&stream).is_playing());
}

#[test]
fn a_streamed_sound_plays_from_an_emitter() {
    let mut audio = Audio::mock(8);
    let stream = audio.stream(TONE.to_vec()).unwrap();
    let mut emitter = audio.emitter(Vec2::ZERO, NEAR, FAR);
    let cue = emitter.play(&stream);
    audio.advance(SETTLED);
    assert!(cue.is_playing());
}

#[test]
fn audio_and_emitters_can_cross_threads() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Audio>();
    assert_send_sync::<Emitter>();
}
