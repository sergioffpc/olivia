//! The defaults that stand in for missing assets.

use olivia::{
    Result,
    graphics::{Graphics, Texture},
};

use crate::random::Random;

/// The side of the texture that stands in for a missing one, in pixels.
const NO_TEXTURE_SIDE: u32 = 64;

/// A square of a random colour, for a missing texture, so that each missing one stands apart.
pub(super) fn no_texture(graphics: &mut Graphics, random: &mut Random) -> Result<Texture> {
    let [red, green, blue, ..] = random.next_u64().to_le_bytes();
    let pixels = [red, green, blue, 255].repeat((NO_TEXTURE_SIDE * NO_TEXTURE_SIDE) as usize);
    Ok(graphics.texture_rgba(NO_TEXTURE_SIDE, NO_TEXTURE_SIDE, &pixels)?)
}

/// An animation on no joints, for a missing one: it leaves every joint as it is.
pub(super) const NO_ANIMATION: &[u8] = br#"{"version":1,"fps":1,"joints":[],"rows":[[]]}"#;

/// A WAV of a hundredth of a second of silence, for a missing sound or music.
pub(super) fn silence() -> Vec<u8> {
    const RATE: u32 = 44_100;
    const DATA: u32 = RATE / 100 * 2; // mono, 16 bits per sample
    let mut wav = Vec::with_capacity(44 + DATA as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + DATA).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // size of the fmt chunk
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // channels
    wav.extend_from_slice(&RATE.to_le_bytes());
    wav.extend_from_slice(&(RATE * 2).to_le_bytes()); // bytes per second
    wav.extend_from_slice(&2u16.to_le_bytes()); // bytes per frame
    wav.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&DATA.to_le_bytes());
    wav.resize(44 + DATA as usize, 0);
    wav
}
