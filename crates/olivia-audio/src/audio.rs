use std::fmt;
use std::time::{Duration, Instant};

use kira::backend::mock::{MockBackend, MockBackendSettings};
use kira::track::MainTrackBuilder;
use kira::{AudioManager, AudioManagerSettings, Decibels};
use tracing::warn;

use crate::cue::{TRANSITION, decibels};
use crate::{Cue, Error, Play, Sound};

/// The shortest time between two warnings about sounds that could not start.
const WARNING_INTERVAL: Duration = Duration::from_secs(1);

/// The sample rate of the mock output.
const MOCK_SAMPLE_RATE: u32 = 48_000;

/// How many frames the mock output renders at once, which is `kira`'s default.
const MOCK_FRAMES: u32 = 128;

/// The game's audio. With no audio device it stays silent, and everything still works.
pub struct Audio {
    output: Option<Output>,
    last_warning: Option<Instant>,
}

enum Output {
    Device(Box<AudioManager>),
    Mock(Box<AudioManager<MockBackend>>),
}

impl Default for Audio {
    /// Opens the default audio device, or stays silent and emits one `warn` if there is none.
    fn default() -> Self {
        match AudioManager::new(AudioManagerSettings::default()) {
            Ok(manager) => Self::with_output(Some(Output::Device(Box::new(manager)))),
            Err(error) => Self::silent(error),
        }
    }
}

impl fmt::Debug for Audio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Audio").finish_non_exhaustive()
    }
}

impl Audio {
    fn with_output(output: Option<Output>) -> Self {
        Self {
            output,
            last_warning: None,
        }
    }

    fn silent(reason: impl fmt::Display) -> Self {
        warn!(%reason, "no audio device, the game stays silent");
        Self::with_output(None)
    }

    /// Audio as it is on a machine with no audio device.
    #[doc(hidden)]
    #[must_use]
    pub fn without_device() -> Self {
        Self::silent("none was asked for")
    }

    /// Audio on a mock output with room for `sound_capacity` sounds, which only [`advance`] moves on.
    ///
    /// [`advance`]: Self::advance
    #[doc(hidden)]
    #[must_use]
    pub fn mock(sound_capacity: usize) -> Self {
        let manager = AudioManager::new(AudioManagerSettings {
            main_track_builder: MainTrackBuilder::new().sound_capacity(sound_capacity),
            backend_settings: MockBackendSettings {
                sample_rate: MOCK_SAMPLE_RATE,
            },
            ..AudioManagerSettings::default()
        })
        .expect("the mock output always starts");
        Self::with_output(Some(Output::Mock(Box::new(manager))))
    }

    /// Moves the mock output on by about `time`.
    #[doc(hidden)]
    pub fn advance(&mut self, time: Duration) {
        if let Some(Output::Mock(manager)) = &mut self.output {
            let blocks =
                time.as_millis() * u128::from(MOCK_SAMPLE_RATE) / (1000 * u128::from(MOCK_FRAMES));
            for _ in 0..=blocks {
                manager.backend_mut().on_start_processing();
                manager.backend_mut().process();
            }
        }
    }

    /// Decodes `bytes` up front, for short sounds.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UndecodableAudio`] if the bytes are not OGG Vorbis, MP3, FLAC or WAV audio.
    pub fn sound(&mut self, bytes: Vec<u8>) -> Result<Sound, Error> {
        Sound::decoded(bytes)
    }

    /// Keeps `bytes` and decodes them while playing, for long tracks.
    ///
    /// # Errors
    ///
    /// Returns [`Error::UndecodableAudio`] if the bytes are not OGG Vorbis, MP3, FLAC or WAV audio.
    pub fn stream(&mut self, bytes: Vec<u8>) -> Result<Sound, Error> {
        Sound::streamed(bytes)
    }

    /// Plays `sound` with the default options.
    pub fn play(&mut self, sound: &Sound) -> Cue {
        self.play_with(sound, Play::new())
    }

    /// Plays `sound` as `play` says. With no device, or above `kira`'s limits, the cue is inert.
    pub fn play_with(&mut self, sound: &Sound, play: Play) -> Cue {
        let started = match &mut self.output {
            Some(Output::Device(manager)) => sound.start(manager, play),
            Some(Output::Mock(manager)) => sound.start(manager, play),
            None => return Cue::inert(),
        };
        started.unwrap_or_else(|error| {
            self.warn_about_refusal(&*error);
            Cue::inert()
        })
    }

    /// Sets the volume of everything: linear from `0.0` to `1.0`, and above `1.0` amplifies.
    pub fn set_master_volume(&mut self, volume: f32) {
        let volume = Decibels(decibels(volume));
        match &mut self.output {
            Some(Output::Device(manager)) => manager.main_track().set_volume(volume, TRANSITION),
            Some(Output::Mock(manager)) => manager.main_track().set_volume(volume, TRANSITION),
            None => {}
        }
    }

    fn warn_about_refusal(&mut self, error: &dyn std::error::Error) {
        let now = Instant::now();
        if self
            .last_warning
            .is_none_or(|last| now - last >= WARNING_INTERVAL)
        {
            self.last_warning = Some(now);
            warn!(%error, "a sound could not start");
        }
    }
}
