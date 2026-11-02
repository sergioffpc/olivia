//! The audio module, a thin wrapper over `kira`. A [`Sound`] holds no playing state, and audio never
//! stops a game from running.

mod audio;
mod cue;
mod play;
mod sound;

pub use audio::Audio;
pub use cue::Cue;
#[doc(hidden)]
pub use cue::decibels;
pub use play::Play;
pub use sound::Sound;

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
