use olivia::{
    Result,
    audio::{Audio, Cue, Play, Sound},
    filesystem::Filesystem,
};

use super::{fallback::silence, list};
use crate::random::Random;

/// The music in `musics/`, every file of a kind the game can play, of which one plays at a time.
#[derive(Debug)]
pub(crate) struct Musics(Vec<Sound>);

impl Musics {
    pub(crate) fn load(filesystem: &Filesystem, audio: &mut Audio) -> Result<Self> {
        let mut musics = Vec::new();
        for path in list(filesystem, "musics")?
            .iter()
            .filter(|path| is_music(path))
        {
            // Streamed, as a long track would take long to decode up front.
            musics.push(audio.stream(filesystem.read(path)?)?);
        }
        if musics.is_empty() {
            tracing::warn!(path = "musics/", "no music, using silence");
            musics.push(audio.stream(silence())?);
        }
        Ok(Self(musics))
    }

    /// Plays one of the musics, at random, over and over until the returned cue stops it.
    pub(crate) fn play(&self, audio: &mut Audio, random: &mut Random) -> Cue {
        let music = &self.0[random.below(self.0.len())];
        audio.play_with(music, Play::new().volume(0.5).looping(true))
    }
}

/// Whether `path` is a kind of file the game can play as music, by its extension.
fn is_music(path: &str) -> bool {
    let extension = path
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase());
    matches!(extension.as_deref(), Some("wav" | "ogg" | "mp3" | "flac"))
}
