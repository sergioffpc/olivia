//! Os sons do jogo, gravados pela turma.

use olivia::{
    Result,
    audio::{Audio, Sound},
    filesystem::Filesystem,
};

use super::{fallback::silence, is_audio, list};
use crate::random::Random;

/// Todos os sons do jogo, os de cada personagem à parte.
#[derive(Debug)]
pub(crate) struct Sounds {
    /// Os sons do herói.
    pub(crate) hero: HeroSounds,
    /// Os sons da princesa.
    pub(crate) princess: PrincessSounds,
    /// Um tiro acertou num mau.
    pub(crate) hit: Takes,
    /// O herói apanhou uma moeda.
    pub(crate) pickup: Takes,
    /// O jogo acabou: perdemos.
    pub(crate) gameover: Takes,
}

/// Os sons do herói.
#[derive(Debug)]
pub(crate) struct HeroSounds {
    /// A saltar.
    pub(crate) jump: Takes,
    /// O salto extra, que sai com um pum.
    pub(crate) fart: Takes,
    /// A disparar.
    pub(crate) shoot: Takes,
    /// A ficar magoado.
    pub(crate) hurt: Takes,
    /// A festejar a vitória.
    pub(crate) win: Takes,
}

/// Os sons da princesa.
#[derive(Debug)]
pub(crate) struct PrincessSounds {
    /// A pedir ajuda.
    pub(crate) help: Takes,
}

impl Sounds {
    /// Vai buscar à pasta `sounds/` todas as gravações de cada som.
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

/// As gravações de um som: `sounds/{nome}_1.wav`, `sounds/{nome}_2.wav`, e por aí fora.
///
/// Cada aluno pode gravar o mesmo som à sua maneira. Pode faltar qualquer número, e o ficheiro pode
/// ser de qualquer tipo de som que o jogo saiba tocar. Cada vez que o som toca, sai uma das
/// gravações à sorte.
#[derive(Debug)]
pub(crate) struct Takes(Vec<Sound>);

impl Takes {
    /// As gravações do som `name`, procuradas entre os ficheiros `files` da pasta `sounds/`.
    fn load(
        filesystem: &Filesystem,
        audio: &mut Audio,
        files: &[String],
        name: &str,
    ) -> Result<Self> {
        // Lê cada gravação deste som.
        let mut takes = Vec::new();
        for path in files.iter().filter(|path| is_take(path, name)) {
            takes.push(audio.sound(filesystem.read(path)?)?);
        }
        // Se não houver nenhuma, avisa e usa o silêncio.
        if takes.is_empty() {
            let path = format!("sounds/{name}_1.wav");
            tracing::warn!(path, "missing asset, using a default");
            takes.push(audio.sound(silence())?);
        }
        Ok(Self(takes))
    }

    /// Uma das gravações, escolhida à sorte.
    pub(crate) fn pick(&self, random: &mut Random) -> &Sound {
        &self.0[random.below(self.0.len())]
    }
}

/// Se o ficheiro `path` é uma gravação do som `name`.
///
/// O nome tem de ser `sounds/{name}_`, depois um número a começar no 1, sem zeros à esquerda, e
/// por fim a terminação de um ficheiro de som.
fn is_take(path: &str, name: &str) -> bool {
    // Tira ao nome do ficheiro, por ordem, a pasta, o nome do som e o tracinho.
    let take = path
        .strip_prefix("sounds/")
        .and_then(|rest| rest.strip_prefix(name))
        .and_then(|rest| rest.strip_prefix('_'))
        // O que sobra separa-se no último ponto: o número de um lado e a terminação do outro.
        .and_then(|rest| rest.rsplit_once('.'));
    take.is_some_and(|(number, extension)| {
        is_audio(extension)
            && !number.is_empty()
            && !number.starts_with('0')
            && number.bytes().all(|b| b.is_ascii_digit())
    })
}
