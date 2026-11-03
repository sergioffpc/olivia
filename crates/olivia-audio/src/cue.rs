use std::time::Duration;

use kira::Tween;
use kira::sound::static_sound::StaticSoundHandle;
use kira::sound::streaming::StreamingSoundHandle;
use kira::sound::{FromFileError, PlaybackState};
use kira::{Decibels, Panning, PlaybackRate};

/// How long setters and [`Cue::stop`] take, to avoid clicks.
pub(crate) const TRANSITION: Tween = tween(Duration::from_millis(10));

const fn tween(duration: Duration) -> Tween {
    Tween {
        start_time: kira::StartTime::Immediate,
        duration,
        easing: kira::Easing::Linear,
    }
}

/// Converts a linear `volume`, where `1.0` is unchanged, to `kira` decibels.
#[doc(hidden)]
#[must_use]
pub fn decibels(volume: f32) -> f32 {
    if volume > 0.0 {
        (20.0 * volume.log10()).max(Decibels::SILENCE.0)
    } else {
        Decibels::SILENCE.0
    }
}

pub(crate) fn panning(pan: f32) -> Panning {
    Panning(pan.clamp(-1.0, 1.0))
}

pub(crate) fn playback_rate(pitch: f32) -> PlaybackRate {
    PlaybackRate(f64::from(pitch))
}

/// One playing of a sound, and the thing the game controls while it lasts. Dropping it does not stop
/// the sound. After the sound ends, the cue stays valid, its controls do nothing and
/// [`is_playing`](Self::is_playing) is `false`. A cue that could not start behaves the same.
#[derive(Debug)]
pub struct Cue {
    playing: Option<Playing>,
}

#[derive(Debug)]
pub(crate) enum Playing {
    Decoded(StaticSoundHandle),
    Streamed(StreamingSoundHandle<FromFileError>),
}

/// Runs `$call` on the `kira` handle of `$cue`, whichever kind it is, if it has one.
macro_rules! on_playing {
    ($cue:ident, $handle:ident => $call:expr) => {
        match &mut $cue.playing {
            Some(Playing::Decoded($handle)) => $call,
            Some(Playing::Streamed($handle)) => $call,
            None => {}
        }
    };
}

impl Cue {
    pub(crate) fn new(playing: Playing) -> Self {
        Self {
            playing: Some(playing),
        }
    }

    /// A cue that is not playing anything.
    pub(crate) fn inert() -> Self {
        Self { playing: None }
    }

    /// Sets the volume: linear from `0.0` to `1.0`, and above `1.0` amplifies.
    pub fn set_volume(&mut self, volume: f32) {
        on_playing!(self, handle => handle.set_volume(decibels(volume), TRANSITION));
    }

    /// Sets the pan, from `-1.0` (left) to `1.0` (right). A sound from an [`Emitter`] takes its pan
    /// from its position, so leave its cue at the centre.
    ///
    /// [`Emitter`]: crate::Emitter
    pub fn set_pan(&mut self, pan: f32) {
        on_playing!(self, handle => handle.set_panning(panning(pan), TRANSITION));
    }

    /// Sets the playback rate, where `1.0` is normal. It also changes the speed.
    pub fn set_pitch(&mut self, pitch: f32) {
        on_playing!(self, handle => handle.set_playback_rate(playback_rate(pitch), TRANSITION));
    }

    /// Pauses the sound, keeping its place.
    pub fn pause(&mut self) {
        on_playing!(self, handle => handle.pause(TRANSITION));
    }

    /// Continues a paused sound.
    pub fn resume(&mut self) {
        on_playing!(self, handle => handle.resume(TRANSITION));
    }

    /// Stops the sound for good.
    pub fn stop(&mut self) {
        on_playing!(self, handle => handle.stop(TRANSITION));
    }

    /// Fades the sound out over `fade` and stops it for good.
    pub fn stop_after(&mut self, fade: Duration) {
        on_playing!(self, handle => handle.stop(tween(fade)));
    }

    /// Whether the sound is being heard: `false` while paused and once it has ended.
    #[must_use]
    pub fn is_playing(&self) -> bool {
        let state = match &self.playing {
            Some(Playing::Decoded(handle)) => handle.state(),
            Some(Playing::Streamed(handle)) => handle.state(),
            None => PlaybackState::Stopped,
        };
        !matches!(state, PlaybackState::Paused | PlaybackState::Stopped)
    }
}
