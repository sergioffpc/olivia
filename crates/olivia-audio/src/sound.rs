use std::io::Cursor;

use kira::sound::static_sound::StaticSoundData;
use kira::sound::{Region, SoundData};
use kira::track::{MainTrackHandle, SpatialTrackHandle};
use kira::{Decibels, PlaySoundError};

use crate::cue::{Cue, Playing, decibels, panning, playback_rate};
use crate::platform::Stream;
use crate::{Error, Play};

/// Audio loaded and ready to play. It holds no playing state, so one sound can play many times at
/// once. Cheap to clone.
#[derive(Clone, Debug)]
pub struct Sound {
    data: Data,
}

#[derive(Clone, Debug)]
enum Data {
    Decoded(Box<StaticSoundData>),
    Streamed(Stream),
}

/// A `kira` track that sounds can start on.
pub(crate) trait Track {
    fn play<D: SoundData>(&mut self, data: D) -> Result<D::Handle, PlaySoundError<D::Error>>;
    fn is_full(&self) -> bool;
}

/// Implements [`Track`] for `kira` handles, which share these methods but no trait.
macro_rules! impl_track {
    ($($handle:ty),+) => {$(
        impl Track for $handle {
            fn play<D: SoundData>(
                &mut self,
                data: D,
            ) -> Result<D::Handle, PlaySoundError<D::Error>> {
                <$handle>::play(self, data)
            }

            fn is_full(&self) -> bool {
                self.num_sounds() >= self.sound_capacity()
            }
        }
    )+};
}

impl_track!(MainTrackHandle, SpatialTrackHandle);

impl Sound {
    pub(crate) fn decoded(bytes: Vec<u8>) -> Result<Self, Error> {
        let data =
            StaticSoundData::from_cursor(Cursor::new(bytes)).map_err(Error::undecodable_audio)?;
        Ok(Self {
            data: Data::Decoded(Box::new(data)),
        })
    }

    pub(crate) fn streamed(bytes: Vec<u8>) -> Result<Self, Error> {
        Ok(Self {
            data: Data::Streamed(Stream::new(bytes)?),
        })
    }

    /// Starts this sound on `track` as `play` says.
    pub(crate) fn start(
        &self,
        track: &mut impl Track,
        play: Play,
    ) -> Result<Cue, Box<dyn std::error::Error>> {
        let volume = Decibels(decibels(play.volume));
        let panning = panning(play.pan);
        let playback_rate = playback_rate(play.pitch);
        let loop_region = play.looping.then(|| Region::from(..));
        let playing = match &self.data {
            Data::Decoded(data) => {
                let data = data
                    .volume(volume)
                    .panning(panning)
                    .playback_rate(playback_rate)
                    .loop_region(loop_region);
                Playing::Decoded(track.play(data)?)
            }
            Data::Streamed(stream) => {
                // `kira` starts the decoding thread before it refuses a sound, and never ends it.
                if track.is_full() {
                    return Err(PlaySoundError::<()>::SoundLimitReached.into());
                }
                let data = stream
                    .data()?
                    .volume(volume)
                    .panning(panning)
                    .playback_rate(playback_rate)
                    .loop_region(loop_region);
                Playing::Streamed(track.play(data)?)
            }
        };
        Ok(Cue::new(playing))
    }
}
