use olivia::{
    Result,
    audio::{Audio, Sound},
    filesystem::Filesystem,
};

use super::{fallback::silence, is_audio, list};
use crate::random::Random;

/// The sounds of the game, those of each character apart.
#[derive(Debug)]
#[expect(dead_code, reason = "played once the update phase plays the game")]
pub(crate) struct Sounds {
    pub(crate) hero: HeroSounds,
    pub(crate) princess: PrincessSounds,
    pub(crate) hit: Takes,
    pub(crate) pickup: Takes,
    pub(crate) gameover: Takes,
}

#[derive(Debug)]
#[expect(dead_code, reason = "played once the update phase moves the hero")]
pub(crate) struct HeroSounds {
    pub(crate) jump: Takes,
    pub(crate) fart: Takes,
    pub(crate) shoot: Takes,
    pub(crate) hurt: Takes,
    pub(crate) win: Takes,
}

#[derive(Debug)]
#[expect(dead_code, reason = "played once the level has the princess")]
pub(crate) struct PrincessSounds {
    pub(crate) help: Takes,
}

impl Sounds {
    pub(crate) fn load(filesystem: &Filesystem, audio: &mut Audio) -> Result<Self> {
        let files = list(filesystem, "sounds")?;
        let mut takes = |name: &str| Takes::load(filesystem, audio, &files, name);
        Ok(Self {
            hero: HeroSounds {
                jump: takes("hero_jump")?,
                fart: takes("hero_fart")?,
                shoot: takes("hero_shoot")?,
                hurt: takes("hero_hurt")?,
                win: takes("hero_win")?,
            },
            princess: PrincessSounds {
                help: takes("princess_help")?,
            },
            hit: takes("hit")?,
            pickup: takes("pickup")?,
            gameover: takes("gameover")?,
        })
    }
}

/// The recordings of one sound, `sounds/{name}_1.wav`, `sounds/{name}_2.wav` and on, as the sound
/// page numbers them, with any number missing and in any kind of audio file the game can play:
/// each time the sound plays, it is one of them at random.
#[derive(Debug)]
pub(crate) struct Takes(Vec<Sound>);

impl Takes {
    /// The takes of the sound `name` among `files`, the ones in `sounds/`.
    fn load(
        filesystem: &Filesystem,
        audio: &mut Audio,
        files: &[String],
        name: &str,
    ) -> Result<Self> {
        let mut takes = Vec::new();
        for path in files.iter().filter(|path| is_take(path, name)) {
            takes.push(audio.sound(filesystem.read(path)?)?);
        }
        if takes.is_empty() {
            let path = format!("sounds/{name}_1.wav");
            tracing::warn!(path, "missing asset, using a default");
            takes.push(audio.sound(silence())?);
        }
        Ok(Self(takes))
    }

    /// One of the recordings, at random.
    #[expect(dead_code, reason = "played once the update phase plays the game")]
    pub(crate) fn pick(&self, random: &mut Random) -> &Sound {
        &self.0[random.below(self.0.len())]
    }
}

/// Whether `path` is a take of the sound `name`: `sounds/{name}_` and a number from 1, without
/// leading zeros, then the extension of an audio file.
fn is_take(path: &str, name: &str) -> bool {
    let take = path
        .strip_prefix("sounds/")
        .and_then(|rest| rest.strip_prefix(name))
        .and_then(|rest| rest.strip_prefix('_'))
        .and_then(|rest| rest.rsplit_once('.'));
    take.is_some_and(|(number, extension)| {
        is_audio(extension)
            && !number.is_empty()
            && !number.starts_with('0')
            && number.bytes().all(|b| b.is_ascii_digit())
    })
}
