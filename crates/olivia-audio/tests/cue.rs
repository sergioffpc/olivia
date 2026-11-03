//! How a cue lives and ends, with a mock output standing in for the audio device.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use olivia_audio::{Audio, Cue, Play, Sound, decibels};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::{Context, SubscriberExt};

/// Lasts 0.2 s.
const TONE: &[u8] = include_bytes!("fixtures/tone.wav");

const LONG_AFTER_THE_END: Duration = Duration::from_millis(400);
const SETTLED: Duration = Duration::from_millis(50);

fn tone(audio: &mut Audio) -> Sound {
    audio.sound(TONE.to_vec()).unwrap()
}

/// Counts `warn` events.
struct Warnings(Arc<AtomicUsize>);

impl<S: Subscriber> Layer<S> for Warnings {
    fn on_event(&self, event: &Event<'_>, _context: Context<'_, S>) {
        if *event.metadata().level() == Level::WARN {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// How many `warn` events `f` emits.
fn warnings_in(f: impl FnOnce()) -> usize {
    let count = Arc::new(AtomicUsize::new(0));
    let subscriber = tracing_subscriber::registry().with(Warnings(Arc::clone(&count)));
    tracing::subscriber::with_default(subscriber, f);
    count.load(Ordering::Relaxed)
}

fn use_every_control(cue: &mut Cue) {
    cue.set_volume(0.5);
    cue.set_pan(-1.0);
    cue.set_pitch(2.0);
    cue.pause();
    cue.resume();
    cue.stop();
    cue.stop_after(Duration::from_secs(1));
}

#[test]
fn a_cue_plays_until_its_sound_ends() {
    let mut audio = Audio::mock(8);
    let sound = tone(&mut audio);
    let cue = audio.play(&sound);
    audio.advance(Duration::from_millis(100));
    assert!(cue.is_playing());
    audio.advance(LONG_AFTER_THE_END);
    assert!(!cue.is_playing());
}

#[test]
fn after_a_cue_ends_its_controls_do_nothing() {
    let mut audio = Audio::mock(8);
    let sound = tone(&mut audio);
    let mut cue = audio.play(&sound);
    audio.advance(LONG_AFTER_THE_END);

    use_every_control(&mut cue);
    audio.advance(SETTLED);

    assert!(!cue.is_playing());
}

#[test]
fn dropping_a_cue_leaves_its_sound_playing() {
    let mut audio = Audio::mock(1);
    let sound = tone(&mut audio);
    drop(audio.play(&sound));
    audio.advance(Duration::from_millis(100));

    // The only slot is still taken, so the dropped cue's sound has not stopped.
    assert!(!audio.play(&sound).is_playing());

    audio.advance(LONG_AFTER_THE_END);
    assert!(audio.play(&sound).is_playing());
}

#[test]
fn one_sound_plays_many_times_at_once() {
    let mut audio = Audio::mock(8);
    let sound = tone(&mut audio);
    let first = audio.play(&sound);
    let second = audio.play(&sound);
    audio.advance(SETTLED);
    assert!(first.is_playing() && second.is_playing());
}

#[test]
fn stop_ends_a_cue_after_a_short_transition() {
    let mut audio = Audio::mock(8);
    let sound = tone(&mut audio);
    let mut cue = audio.play(&sound);
    cue.stop();
    audio.advance(SETTLED);
    assert!(!cue.is_playing());
}

#[test]
fn stop_after_keeps_playing_through_the_fade() {
    let mut audio = Audio::mock(8);
    let play = Play::new().looping(true);
    let sound = tone(&mut audio);
    let mut cue = audio.play_with(&sound, play);
    cue.stop_after(Duration::from_millis(200));
    audio.advance(Duration::from_millis(100));
    assert!(cue.is_playing());
    audio.advance(Duration::from_millis(200));
    assert!(!cue.is_playing());
}

#[test]
fn a_paused_cue_is_not_playing_until_it_resumes() {
    let mut audio = Audio::mock(8);
    let sound = tone(&mut audio);
    let mut cue = audio.play(&sound);
    cue.pause();
    audio.advance(LONG_AFTER_THE_END);
    assert!(!cue.is_playing());

    cue.resume();
    audio.advance(SETTLED);
    assert!(cue.is_playing());
}

#[test]
fn a_looping_cue_outlives_its_sound() {
    let mut audio = Audio::mock(8);
    let sound = tone(&mut audio);
    let cue = audio.play_with(&sound, Play::new().looping(true));
    audio.advance(LONG_AFTER_THE_END);
    assert!(cue.is_playing());
}

#[test]
fn pitch_is_the_playback_rate() {
    let mut audio = Audio::mock(8);
    let sound = tone(&mut audio);
    let cue = audio.play_with(&sound, Play::new().pitch(2.0));
    audio.advance(Duration::from_millis(150));
    assert!(!cue.is_playing());
}

#[test]
fn a_streamed_sound_plays_and_stops_like_a_decoded_one() {
    let mut audio = Audio::mock(8);
    let sound = audio.stream(TONE.to_vec()).unwrap();
    let mut first = audio.play_with(&sound, Play::new().looping(true));
    let second = audio.play(&sound);
    audio.advance(SETTLED);
    assert!(first.is_playing() && second.is_playing());

    first.stop();
    audio.advance(SETTLED);
    assert!(!first.is_playing());
}

#[test]
fn above_the_sound_limit_play_is_inert_and_warns_once_in_a_burst() {
    let mut audio = Audio::mock(1);
    let sound = tone(&mut audio);
    let playing = audio.play(&sound);

    let warnings = warnings_in(|| {
        for _ in 0..10 {
            let mut cue = audio.play(&sound);
            assert!(!cue.is_playing());
            use_every_control(&mut cue);
        }
    });

    assert_eq!(warnings, 1);
    assert!(playing.is_playing());
}

#[test]
fn above_the_sound_limit_a_stream_is_refused_until_a_slot_frees() {
    let mut audio = Audio::mock(1);
    let tone = tone(&mut audio);
    let stream = audio.stream(TONE.to_vec()).unwrap();
    let _playing = audio.play(&tone);

    assert!(!audio.play(&stream).is_playing());

    audio.advance(LONG_AFTER_THE_END);
    assert!(audio.play(&stream).is_playing());
}

#[test]
fn with_no_device_play_is_inert_and_only_the_start_warns() {
    let warnings = warnings_in(|| {
        let mut audio = Audio::without_device();
        let sound = tone(&mut audio);
        for _ in 0..3 {
            let mut cue = audio.play(&sound);
            assert!(!cue.is_playing());
            use_every_control(&mut cue);
        }
        let stream = audio.stream(TONE.to_vec()).unwrap();
        assert!(!audio.play_with(&stream, Play::new()).is_playing());
        audio.set_master_volume(0.5);
    });
    assert_eq!(warnings, 1);
}

#[test]
fn master_volume_leaves_cues_playing() {
    let mut audio = Audio::mock(8);
    let sound = tone(&mut audio);
    let cue = audio.play(&sound);
    audio.set_master_volume(0.0);
    audio.advance(SETTLED);
    assert!(cue.is_playing());
}

#[test]
fn volume_is_linear_and_goes_silent_at_zero() {
    for (volume, expected) in [
        (1.0, 0.0),
        (0.1, -20.0),
        (2.0, 6.0206),
        (0.0, -60.0),
        (-1.0, -60.0),
    ] {
        assert!((decibels(volume) - expected).abs() < 1e-3, "{volume}");
    }
}
