//! As músicas que tocam por trás do jogo.

use olivia::{
    Result,
    audio::{Audio, Cue, Play},
    filesystem::Filesystem,
};

use super::{fallback::silence, is_audio, list};
use crate::random::Random;

/// As músicas da pasta `musics/`: toca uma de cada vez, escolhida à sorte.
///
/// Só se guarda o nome de cada ficheiro: a música só se lê quando for escolhida para tocar.
#[derive(Debug)]
pub(crate) struct Musics(Vec<String>);

impl Musics {
    /// Vê que músicas há na pasta `musics/`.
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

    /// Toca uma das músicas, à sorte, sempre outra vez do princípio, até alguém a parar.
    pub(crate) fn play(
        &self,
        filesystem: &Filesystem,
        audio: &mut Audio,
        random: &mut Random,
    ) -> Result<Cue> {
        // Escolhe a música à sorte, ou o silêncio se não houver nenhuma.
        let bytes = if self.0.is_empty() {
            silence()
        } else {
            filesystem.read(&self.0[random.below(self.0.len())])?
        };
        // Toca-a aos bocados, enquanto a lê, porque uma música comprida demora a ler toda.
        let music = audio.stream(bytes)?;
        Ok(audio.play_with(&music, Play::new().volume(0.5).looping(true)))
    }
}

/// Se o ficheiro `path` é uma música que o jogo sabe tocar, pela terminação do nome.
fn is_music(path: &str) -> bool {
    path.rsplit_once('.')
        .is_some_and(|(_, extension)| is_audio(extension))
}
