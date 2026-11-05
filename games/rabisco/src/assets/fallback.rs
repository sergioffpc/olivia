//! O que o jogo usa no lugar dos trabalhos da turma que ainda não estão nas pastas.

use olivia::{
    Result,
    graphics::{Graphics, Texture},
};

use crate::random::Random;

/// O lado do quadrado que aparece no lugar de um desenho que falta, em píxeis.
const NO_TEXTURE_SIDE: u32 = 64;

/// Um quadrado de uma cor só, no lugar do desenho `path` que falta.
///
/// A cor sai do nome do ficheiro: o mesmo desenho em falta tem sempre a mesma cor, e desenhos
/// diferentes têm quase sempre cores diferentes. Assim vê-se logo qual é qual.
pub(super) fn no_texture(graphics: &mut Graphics, path: &str) -> Result<Texture> {
    // Transforma o nome num número, e o número em três cores: vermelho, verde e azul.
    let [red, green, blue, ..] = Random::from_seed(fnv1a(path)).next_u64().to_le_bytes();
    // Pinta todos os píxeis do quadrado com essa cor, sem transparência.
    let pixels = [red, green, blue, 255].repeat((NO_TEXTURE_SIDE * NO_TEXTURE_SIDE) as usize);
    Ok(graphics.texture_rgba(NO_TEXTURE_SIDE, NO_TEXTURE_SIDE, &pixels)?)
}

/// Transforma o texto `text` num número, sempre o mesmo para o mesmo texto (chama-se FNV-1a).
fn fnv1a(text: &str) -> u64 {
    const OFFSET: u64 = 0xCBF2_9CE4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01B3;
    // Letra a letra, mistura a letra no número e multiplica-o por um número especial.
    text.bytes().fold(OFFSET, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(PRIME)
    })
}

/// Um esqueleto de uma parte só, `body`, presa pelo meio, no lugar de um que falta.
///
/// A personagem aparece como o desenho `{nome}_body`, ou o quadrado que está no lugar dele.
pub(super) const NO_SKELETON: &[u8] =
    br#"{"version":1,"joints":[{"name":"root","part":"body","point":[32,32]}]}"#;

/// Um movimento que não mexe nada, no lugar de um que falta.
pub(super) const NO_ANIMATION: &[u8] = br#"{"version":1,"fps":1,"joints":[],"rows":[[]]}"#;

/// Um som de silêncio, com um centésimo de segundo, no lugar de um som ou música que falta.
///
/// É um ficheiro WAV feito à mão: primeiro um cabeçalho que diz como o som é, depois o som, que
/// aqui são só zeros, ou seja, silêncio.
pub(super) fn silence() -> Vec<u8> {
    const RATE: u32 = 44_100;
    const DATA: u32 = RATE / 100 * 2; // um canal, com 16 bits por amostra
    let mut wav = Vec::with_capacity(44 + DATA as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + DATA).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // o tamanho da parte "fmt"
    wav.extend_from_slice(&1u16.to_le_bytes()); // som sem compressão (PCM)
    wav.extend_from_slice(&1u16.to_le_bytes()); // canais
    wav.extend_from_slice(&RATE.to_le_bytes());
    wav.extend_from_slice(&(RATE * 2).to_le_bytes()); // bytes por segundo
    wav.extend_from_slice(&2u16.to_le_bytes()); // bytes por amostra
    wav.extend_from_slice(&16u16.to_le_bytes()); // bits por amostra
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&DATA.to_le_bytes());
    // O som propriamente dito: só zeros.
    wav.resize(44 + DATA as usize, 0);
    wav
}
