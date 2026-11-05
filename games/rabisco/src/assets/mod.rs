//! Os desenhos, os sons e os movimentos que o jogo vai buscar às pastas.
//!
//! A turma vai pondo os seus trabalhos nas pastas aos poucos. Enquanto um ainda não está lá, o
//! jogo usa outra coisa no lugar dele, para poder funcionar na mesma: um quadrado de uma cor, um
//! som calado, ou um boneco que não se mexe.

mod animations;
mod characters;
mod fallback;
mod musics;
mod sounds;
mod textures;

pub(crate) use animations::{Animations, HeroAnimations};
pub(crate) use characters::{Characters, PART_SCALE, ROOT_HEIGHT};
pub(crate) use musics::Musics;
pub(crate) use sounds::Sounds;
pub(crate) use textures::Textures;

use olivia::{
    Result,
    audio::Audio,
    filesystem::{self, Filesystem},
    graphics::Graphics,
};

/// Tudo o que o jogo carregou das pastas, guardado junto como numa mochila.
///
/// Carrega-se uma vez só, no começo, e depois empresta-se a quem precisar. A música não está cá
/// dentro: ela é o que está a tocar, não uma coisa que se vai buscar de vez em quando.
#[derive(Debug)]
pub(crate) struct Assets {
    /// Os movimentos das personagens.
    pub(crate) animations: Animations,
    /// As personagens, feitas de partes desenhadas.
    pub(crate) characters: Characters,
    /// Os sons gravados pela turma.
    pub(crate) sounds: Sounds,
    /// Os desenhos.
    pub(crate) textures: Textures,
}

impl Assets {
    /// Vai buscar às pastas tudo o que o jogo precisa.
    pub(crate) fn load(
        filesystem: &Filesystem,
        graphics: &mut Graphics,
        audio: &mut Audio,
    ) -> Result<Self> {
        Ok(Self {
            animations: Animations::load(filesystem)?,
            characters: Characters::load(filesystem, graphics)?,
            sounds: Sounds::load(filesystem, audio)?,
            textures: Textures::load(filesystem, graphics)?,
        })
    }
}

/// Lê o ficheiro `path`, ou avisa e não dá nada se ele ainda não estiver na pasta, para se usar
/// outra coisa no lugar dele. Um ficheiro que está lá mas não se consegue ler é um erro a sério.
fn read(filesystem: &Filesystem, path: &str) -> Result<Option<Vec<u8>>> {
    match filesystem.read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(filesystem::Error::NotFound(_)) => {
            tracing::warn!(path, "missing asset, using a default");
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}

/// Os ficheiros que há na pasta `path`, ou nenhum se a pasta ainda não existir. Quem pede é que
/// avisa, se não encontrar nada que sirva.
fn list(filesystem: &Filesystem, path: &str) -> Result<Vec<String>> {
    match filesystem.list(path) {
        Ok(files) => Ok(files),
        Err(filesystem::Error::NotFound(_)) => Ok(Vec::new()),
        Err(error) => Err(error.into()),
    }
}

/// Se a terminação `extension` é de um ficheiro de som que o jogo sabe tocar.
fn is_audio(extension: &str) -> bool {
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "wav" | "ogg" | "mp3" | "flac"
    )
}
