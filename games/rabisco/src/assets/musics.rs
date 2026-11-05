use olivia::{
    Result,
    audio::{Audio, Cue, Play},
    filesystem::Filesystem,
};

use super::{fallback::silence, is_audio, list};
use crate::random::Random;

/// The music in `musics/`, every file of a kind the game can play, of which one plays at a time.
/// Only the one that plays is read.
#[derive(Debug)]
pub(crate) struct Musics(Vec<String>);

impl Musics {
    pub(crate) fn load(filesystem: &Filesystem) -> Result<Self> {
        let musics: Vec<String> = list(filesystem, "musics")?
            .into_iter()
            .filter(|path| is_music(path))
            .collect();
        if musics.is_empty() {
            tracing::warn!(path = "musics/", "no music, using silence");
        }
        Ok(Self(musics))
    }

    /// Plays one of the musics, at random, over and over until the returned cue stops it.
    pub(crate) fn play(
        &self,
        filesystem: &Filesystem,
        audio: &mut Audio,
        random: &mut Random,
    ) -> Result<Cue> {
        let bytes = if self.0.is_empty() {
            silence()
        } else {
            filesystem.read(&self.0[random.below(self.0.len())])?
        };
        // Streamed, as a long track would take long to decode up front.
        let music = audio.stream(bytes)?;
        Ok(audio.play_with(&music, Play::new().volume(0.5).looping(true)))
    }
}

/// Whether `path` is a kind of file the game can play as music, by its extension.
fn is_music(path: &str) -> bool {
    path.rsplit_once('.')
        .is_some_and(|(_, extension)| is_audio(extension))
}
