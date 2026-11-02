//! How bytes become a sound, which is the same on a machine with no audio device.

use olivia_audio::{Audio, Error};

const FORMATS: [&[u8]; 4] = [
    include_bytes!("fixtures/tone.ogg"),
    include_bytes!("fixtures/tone.mp3"),
    include_bytes!("fixtures/tone.flac"),
    include_bytes!("fixtures/tone.wav"),
];

#[test]
fn every_format_loads_as_a_sound_and_as_a_stream() {
    let mut audio = Audio::without_device();
    for bytes in FORMATS {
        audio.sound(bytes.to_vec()).unwrap();
        audio.stream(bytes.to_vec()).unwrap();
    }
}

#[test]
fn bytes_that_are_not_audio_fail_at_load() {
    let mut audio = Audio::without_device();
    let garbage = b"not audio at all".to_vec();
    assert!(matches!(
        audio.sound(garbage.clone()),
        Err(Error::UndecodableAudio(_))
    ));
    assert!(matches!(
        audio.stream(garbage),
        Err(Error::UndecodableAudio(_))
    ));
}
