use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use kira::backend::Backend;
use kira::listener::ListenerHandle;
use kira::track::{MainTrackBuilder, SpatialTrackHandle};
use kira::{AudioManager, AudioManagerSettings, Decibels};
use olivia_math::Vec2;
use tracing::warn;
use web_time::Instant;

use crate::cue::{TRANSITION, decibels};
use crate::mock::Mock;
use crate::platform;
use crate::{Cue, Emitter, Error, Play, Sound, emitter};

/// The shortest time between two warnings about sounds that could not start.
const WARNING_INTERVAL: Duration = Duration::from_secs(1);

/// `x`, `y`, `z` and `s` of a listener with no orientation.
const NO_ORIENTATION: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

/// The game's audio. With no audio device it stays silent, and everything still works.
pub struct Audio {
    output: Option<Output>,
    shared: Arc<Mutex<Shared>>,
}

struct Output {
    manager: Manager,
    listener: ListenerHandle,
}

enum Manager {
    Device(Box<AudioManager<platform::Backend>>),
    Mock(Box<AudioManager<Mock>>),
}

/// Runs `$call` on the `kira` manager in `$manager`, whichever kind it is.
macro_rules! on_manager {
    ($manager:expr, $handle:ident => $call:expr) => {
        match $manager {
            Manager::Device($handle) => $call,
            Manager::Mock($handle) => $call,
        }
    };
}

/// What `Audio` shares with its emitters.
#[derive(Debug, Default)]
pub(crate) struct Shared {
    last_warning: Option<Instant>,
    /// The tracks of dropped emitters with sounds left. `kira` can remove a track, whatever it was
    /// told to persist for, before the sounds just played on it arrive, so these stay held instead.
    retired: Vec<SpatialTrackHandle>,
}

fn lock(shared: &Mutex<Shared>) -> MutexGuard<'_, Shared> {
    shared.lock().expect("nothing panics holding this")
}

/// Holds the track of a dropped emitter until its sounds are over.
pub(crate) fn retire(shared: &Mutex<Shared>, track: SpatialTrackHandle) {
    lock(shared).retired.push(track);
}

/// Emits a `warn` about something `kira` refused, at most once per [`WARNING_INTERVAL`].
pub(crate) fn warn_about_refusal(shared: &Mutex<Shared>, error: &dyn fmt::Display) {
    let now = Instant::now();
    let mut shared = lock(shared);
    let is_due = shared
        .last_warning
        .is_none_or(|last| now - last >= WARNING_INTERVAL);
    if is_due {
        shared.last_warning = Some(now);
        drop(shared);
        warn!(%error, "audio is over its limits, this stays silent");
    }
}

/// Adds the one listener, at `(0, 0)`.
fn listener<B: Backend>(manager: &mut AudioManager<B>) -> ListenerHandle {
    manager
        .add_listener([0.0, 0.0, 0.0], NO_ORIENTATION)
        .expect("a new manager has room for a listener")
}

impl Default for Audio {
    /// Opens the default audio device, or stays silent and emits one `warn` if there is none.
    fn default() -> Self {
        match AudioManager::<platform::Backend>::new(AudioManagerSettings::default()) {
            Ok(mut manager) => {
                let listener = listener(&mut manager);
                Self::with_output(Some(Output {
                    manager: Manager::Device(Box::new(manager)),
                    listener,
                }))
            }
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
            shared: Arc::default(),
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
        let mut manager = AudioManager::<Mock>::new(AudioManagerSettings {
            main_track_builder: MainTrackBuilder::new().sound_capacity(sound_capacity),
            ..AudioManagerSettings::default()
        })
        .expect("the mock output always starts");
        let listener = listener(&mut manager);
        Self::with_output(Some(Output {
            manager: Manager::Mock(Box::new(manager)),
            listener,
        }))
    }

    /// Moves the mock output on by about `time` and returns the loudest left and right samples.
    #[doc(hidden)]
    pub fn advance(&mut self, time: Duration) -> (f32, f32) {
        match &mut self.output {
            Some(Output {
                manager: Manager::Mock(manager),
                ..
            }) => manager.backend_mut().render(time),
            _ => (0.0, 0.0),
        }
    }

    /// Lets the audio play on the web, where the browser holds it back until the player presses a
    /// key or a button. Must be called inside the handler of that press. Does nothing on the
    /// desktop, and nothing once the audio plays.
    #[doc(hidden)]
    pub fn resume(&mut self) {
        if let Some(Output {
            manager: Manager::Device(manager),
            ..
        }) = &mut self.output
        {
            platform::resume(manager.backend_mut());
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
    /// On the web it decodes them up front, as [`sound`](Self::sound) does. For a long track that
    /// can take hundreds of milliseconds, and the decoded sound holds about 10 MB per stereo
    /// minute.
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
        let Some(output) = &mut self.output else {
            return Cue::inert();
        };
        let started =
            on_manager!(&mut output.manager, manager => sound.start(manager.main_track(), play));
        started.unwrap_or_else(|error| {
            warn_about_refusal(&self.shared, &error);
            Cue::inert()
        })
    }

    /// Creates an [`Emitter`] at `pos`, whose sounds are at full volume up to `near` from the
    /// listener and silent from `far`, in the game's own unit. With no device, or above `kira`'s
    /// limits, the emitter is inert.
    ///
    /// # Panics
    ///
    /// Panics if `near >= far`.
    pub fn emitter(&mut self, pos: Vec2, near: f32, far: f32) -> Emitter {
        assert!(near < far, "an emitter's near must be below its far");
        let Some(output) = &mut self.output else {
            return Emitter::inert();
        };
        lock(&self.shared)
            .retired
            .retain(|track| track.num_sounds() > 0);
        let position = emitter::position(pos, near);
        let builder = emitter::track_builder(near, far);
        let listener = &output.listener;
        let added = on_manager!(
            &mut output.manager,
            manager => manager.add_spatial_sub_track(listener, position, builder)
        );
        match added {
            Ok(track) => Emitter::new(track, near, Arc::clone(&self.shared)),
            Err(error) => {
                warn_about_refusal(&self.shared, &error);
                Emitter::inert()
            }
        }
    }

    /// Moves the one listener, which has no orientation and starts at `(0, 0)`, to `pos`.
    pub fn set_listener(&mut self, pos: Vec2) {
        if let Some(output) = &mut self.output {
            output
                .listener
                .set_position([pos.x, pos.y, 0.0], TRANSITION);
        }
    }

    /// Sets the volume of everything: linear from `0.0` to `1.0`, and above `1.0` amplifies.
    pub fn set_master_volume(&mut self, volume: f32) {
        let volume = Decibels(decibels(volume));
        if let Some(output) = &mut self.output {
            on_manager!(
                &mut output.manager,
                manager => manager.main_track().set_volume(volume, TRANSITION)
            );
        }
    }
}
