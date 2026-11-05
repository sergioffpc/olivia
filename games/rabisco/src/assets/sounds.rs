use olivia::{
    Result,
    audio::{Audio, Sound},
    filesystem::Filesystem,
};

use super::{fallback::silence, list};
use crate::random::Random;

#[derive(Debug)]
pub(crate) struct Sounds {
    jump: Takes,
    fart: Takes,
    shoot: Takes,
    hit: Takes,
    pickup: Takes,
    hurt: Takes,
    gameover: Takes,
    help: Takes,
    win: Takes,
}

impl Sounds {
    pub(crate) fn load(filesystem: &Filesystem, audio: &mut Audio) -> Result<Self> {
        let files = list(filesystem, "sounds")?;
        let mut takes = |name: &str| Takes::load(filesystem, audio, &files, name);
        Ok(Self {
            jump: takes("hero_jump")?,
            fart: takes("hero_fart")?,
            shoot: takes("hero_shoot")?,
            hit: takes("hit")?,
            pickup: takes("pickup")?,
            hurt: takes("hero_hurt")?,
            gameover: takes("gameover")?,
            help: takes("princess_help")?,
            win: takes("hero_win")?,
        })
    }
}

/// The recordings of one sound, `sounds/{name}_1.wav`, `sounds/{name}_2.wav` and on, as the sound
/// page numbers them, with any number missing: each time the sound plays, it is one of them at
/// random.
#[derive(Debug)]
struct Takes(Vec<Sound>);

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
    fn pick(&self, random: &mut Random) -> &Sound {
        &self.0[random.below(self.0.len())]
    }
}

/// Whether `path` is a take of the sound `name`: `sounds/{name}_` and a number from 1, without
/// leading zeros, then `.wav`.
fn is_take(path: &str, name: &str) -> bool {
    let number = path
        .strip_prefix("sounds/")
        .and_then(|rest| rest.strip_prefix(name))
        .and_then(|rest| rest.strip_prefix('_'))
        .and_then(|rest| rest.strip_suffix(".wav"));
    number.is_some_and(|n| {
        !n.is_empty() && !n.starts_with('0') && n.bytes().all(|b| b.is_ascii_digit())
    })
}
