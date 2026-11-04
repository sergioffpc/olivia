//! The audio module, a thin wrapper over `kira`. A [`Sound`] holds no playing state, and audio never
//! stops a game from running.

mod audio;
mod cue;
mod emitter;
mod mock;
mod platform;
mod play;
mod sound;

pub use audio::Audio;
pub use cue::Cue;
#[doc(hidden)]
pub use cue::decibels;
pub use emitter::Emitter;
pub use play::Play;
pub use sound::Sound;

/// Suspends all audio on the web, for a panic hook, which cannot reach [`Audio`]. Does nothing on
/// the desktop.
#[doc(hidden)]
pub fn suspend_all() {
    platform::suspend_all();
}

/// A failure of this module.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The bytes are not OGG Vorbis, MP3, FLAC or WAV audio.
    #[error("failed to decode the audio")]
    UndecodableAudio(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    fn undecodable_audio(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        Self::UndecodableAudio(error.into())
    }
}
