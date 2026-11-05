// A game, not a library: its code needs no documentation.
#![allow(missing_docs, clippy::missing_errors_doc, clippy::missing_panics_doc)]

use olivia::animation::Animation;
use olivia::audio::{Audio, Play, Sound};
use olivia::ecs::{Camera, Clear, Entity};
use olivia::filesystem::{self, Filesystem};
use olivia::graphics::{Graphics, Texture};
use olivia::math::{Color, Vec2};
use olivia::window::WindowEvent;
use olivia::{Game, Result, Setup, Update};

/// The size of the window, in logical pixels.
pub const WINDOW: Vec2 = Vec2::new(1280.0, 720.0);
/// How many logical pixels a metre takes.
pub const PIXELS_PER_METRE: f32 = 100.0;
/// How much the camera sees.
pub const VIEW: Vec2 = Vec2::new(12.8, 7.2);
/// The gravity, in metres per second squared.
pub const GRAVITY: f32 = 9.81;

pub struct Rabisco {
    animations: Animations,
    sounds: Sounds,
    random: Random,
    textures: Textures,

    camera: Entity,
}

impl Rabisco {
    #[must_use]
    pub fn config() -> olivia::Config {
        olivia::Config::new("rabisco")
            .title("Rabisco")
            .size(WINDOW.x, WINDOW.y)
            .resizable(false)
            .gravity(0.0, GRAVITY)
    }
}

impl Game for Rabisco {
    fn load(s: Setup<'_>) -> Result<Self> {
        let animations = Animations::load(s.filesystem)?;
        let sounds = Sounds::load(s.filesystem, s.audio)?;
        let textures = Textures::load(s.filesystem, s.graphics)?;

        let music = match read(s.filesystem, "musics/music.wav")? {
            Some(bytes) => s.audio.stream(bytes)?,
            None => s.audio.stream(silence())?,
        };
        let theme = Play::new().volume(0.5).looping(true);
        s.audio.play_with(&music, theme);

        s.world.spawn((Clear(Color::BLUE),));
        let camera = s.world.spawn((Camera {
            position: VIEW / 2.0,
            zoom: PIXELS_PER_METRE,
            rotation: 0.0,
        },));

        Ok(Self {
            animations,
            sounds,
            random: Random::from_clock(),
            textures,
            camera,
        })
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        if u.events.contains(&WindowEvent::CloseRequested) {
            u.quit();
        }
        Ok(())
    }
}

pub struct Animations {
    idle: Animation,
    walk: Animation,
    jump: Animation,
    double_jump: Animation,
    shoot: Animation,
    hurt: Animation,
    win: Animation,
    calling: Animation,
    freed: Animation,
}

impl Animations {
    fn load(filesystem: &Filesystem) -> Result<Self> {
        Ok(Self {
            idle: Animations::read(filesystem, "hero_idle")?,
            walk: Animations::read(filesystem, "hero_walk")?,
            jump: Animations::read(filesystem, "hero_jump")?,
            double_jump: Animations::read(filesystem, "hero_double_jump")?,
            shoot: Animations::read(filesystem, "hero_shoot")?,
            hurt: Animations::read(filesystem, "hero_hurt")?,
            win: Animations::read(filesystem, "hero_win")?,
            calling: Animations::read(filesystem, "princess_calling")?,
            freed: Animations::read(filesystem, "princess_freed")?,
        })
    }

    fn read(filesystem: &Filesystem, name: &str) -> Result<Animation> {
        let bytes = read(filesystem, &format!("animations/{name}.json"))?;
        Ok(Animation::parse(bytes.as_deref().unwrap_or(NO_ANIMATION))?)
    }
}

pub struct Sounds {
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
    fn load(filesystem: &Filesystem, audio: &mut Audio) -> Result<Self> {
        Ok(Self {
            jump: Takes::read(filesystem, audio, "hero_jump")?,
            fart: Takes::read(filesystem, audio, "hero_fart")?,
            shoot: Takes::read(filesystem, audio, "hero_shoot")?,
            hit: Takes::read(filesystem, audio, "hit")?,
            pickup: Takes::read(filesystem, audio, "pickup")?,
            hurt: Takes::read(filesystem, audio, "hero_hurt")?,
            gameover: Takes::read(filesystem, audio, "gameover")?,
            help: Takes::read(filesystem, audio, "princess_help")?,
            win: Takes::read(filesystem, audio, "hero_win")?,
        })
    }
}

/// The recordings of one sound, `sounds/{name}_1.wav`, `sounds/{name}_2.wav` and on, as the sound
/// page numbers them, with any number missing: each time the sound plays, it is one of them at
/// random.
#[derive(Debug)]
pub struct Takes(Vec<Sound>);

impl Takes {
    fn read(filesystem: &Filesystem, audio: &mut Audio, name: &str) -> Result<Self> {
        let files = match filesystem.list("sounds") {
            Ok(files) => files,
            Err(filesystem::Error::NotFound(_)) => Vec::new(),
            Err(error) => return Err(error.into()),
        };
        let mut takes = Vec::new();
        for path in files.iter().filter(|path| is_take(path, name)) {
            takes.push(audio.sound(filesystem.read(path)?)?);
        }
        if takes.is_empty() {
            tracing::warn!(path = take_path(name, 1), "missing asset, using a default");
            takes.push(audio.sound(silence())?);
        }
        Ok(Self(takes))
    }

    /// One of the recordings, at random.
    #[must_use]
    pub fn pick(&self, random: &mut Random) -> &Sound {
        &self.0[random.below(self.0.len())]
    }
}

fn take_path(name: &str, n: usize) -> String {
    format!("sounds/{name}_{n}.wav")
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

/// A small generator of numbers that only look random, enough to vary what the game plays:
/// xorshift64*, seeded from the clock.
#[derive(Debug)]
pub struct Random(u64);

impl Random {
    #[must_use]
    pub fn from_clock() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_nanos());
        #[allow(clippy::cast_possible_truncation)] // the low bits vary the most
        Self::new(nanos as u64)
    }

    /// The same numbers for the same seed.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        // Zero would give only zeros.
        Self(seed | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// A number from 0 up to, but not including, `n`, which must not be 0.
    pub fn below(&mut self, n: usize) -> usize {
        assert!(n > 0, "no number is below 0");
        // The high bits are the best ones; the bias over 64 bits is too small to hear.
        #[allow(clippy::cast_possible_truncation)]
        let pick = ((u128::from(self.next_u64()) * n as u128) >> 64) as usize;
        pick
    }
}

pub struct Textures {
    life: Texture,
    life_lost: Texture,
    digits: [Texture; 10],
    projectile: Texture,
    fart: Texture,
    coin: Texture,
    goal: Texture,
    crawl: Texture,
    roll: Texture,
    float: Texture,
}

impl Textures {
    fn load(filesystem: &Filesystem, graphics: &mut Graphics) -> Result<Self> {
        Ok(Self {
            life: Textures::read(filesystem, graphics, "hud_life")?,
            life_lost: Textures::read(filesystem, graphics, "hud_life_lost")?,
            digits: [
                Textures::read(filesystem, graphics, "hud_digit_0")?,
                Textures::read(filesystem, graphics, "hud_digit_1")?,
                Textures::read(filesystem, graphics, "hud_digit_2")?,
                Textures::read(filesystem, graphics, "hud_digit_3")?,
                Textures::read(filesystem, graphics, "hud_digit_4")?,
                Textures::read(filesystem, graphics, "hud_digit_5")?,
                Textures::read(filesystem, graphics, "hud_digit_6")?,
                Textures::read(filesystem, graphics, "hud_digit_7")?,
                Textures::read(filesystem, graphics, "hud_digit_8")?,
                Textures::read(filesystem, graphics, "hud_digit_9")?,
            ],
            projectile: Textures::read(filesystem, graphics, "projectile")?,
            fart: Textures::read(filesystem, graphics, "fart")?,
            coin: Textures::read(filesystem, graphics, "coin")?,
            goal: Textures::read(filesystem, graphics, "goal")?,
            crawl: Textures::read(filesystem, graphics, "enemy_crawl")?,
            roll: Textures::read(filesystem, graphics, "enemy_roll")?,
            float: Textures::read(filesystem, graphics, "enemy_float")?,
        })
    }

    fn read(filesystem: &Filesystem, graphics: &mut Graphics, name: &str) -> Result<Texture> {
        match read(filesystem, &format!("textures/{name}.png"))? {
            Some(bytes) => Ok(graphics.texture(&bytes)?),
            None => Ok(no_texture(graphics)?),
        }
    }
}

/// Reads an asset, or warns and gives `None` if it is not there yet, so that a default can stand
/// in for it. An asset that is there but cannot be read is still an error.
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

/// The side of the texture that stands in for a missing one, in pixels.
const NO_TEXTURE_SIDE: u32 = 64;

/// A green square, for a missing texture.
fn no_texture(graphics: &mut Graphics) -> Result<Texture> {
    let pixels = [0, 255, 0, 255].repeat((NO_TEXTURE_SIDE * NO_TEXTURE_SIDE) as usize);
    Ok(graphics.texture_rgba(NO_TEXTURE_SIDE, NO_TEXTURE_SIDE, &pixels)?)
}

/// An animation on no joints, for a missing one: it leaves every joint as it is.
const NO_ANIMATION: &[u8] = br#"{"version":1,"fps":1,"joints":[],"rows":[[]]}"#;

/// A WAV of a hundredth of a second of silence, for a missing sound or music.
fn silence() -> Vec<u8> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn takes_are_numbered_from_one() {
        assert_eq!(take_path("hero_fart", 1), "sounds/hero_fart_1.wav");
        assert_eq!(take_path("hero_fart", 12), "sounds/hero_fart_12.wav");
    }

    #[test]
    fn takes_are_only_numbered_files_of_their_sound() {
        assert!(is_take("sounds/hero_jump_1.wav", "hero_jump"));
        assert!(is_take("sounds/hero_jump_12.wav", "hero_jump"));
        assert!(!is_take("sounds/hero_jump.wav", "hero_jump"));
        assert!(!is_take("sounds/hero_jump_01.wav", "hero_jump"));
        assert!(!is_take("sounds/hero_jump_x.wav", "hero_jump"));
        assert!(!is_take("sounds/hero_jump_extra_1.wav", "hero_jump"));
        assert!(!is_take("sounds/hero_jump_1.wav", "hero"));
    }

    #[test]
    fn random_picks_every_take_and_stays_in_range() {
        let mut random = Random::new(7);
        let mut seen = [0; 3];
        for _ in 0..3000 {
            seen[random.below(3)] += 1;
        }
        assert!(seen.iter().all(|&n| n > 800), "{seen:?}");
    }

    #[test]
    fn random_repeats_for_the_same_seed() {
        let a: Vec<_> = std::iter::repeat_with({
            let mut r = Random::new(42);
            move || r.next_u64()
        })
        .take(5)
        .collect();
        let mut r = Random::new(42);
        assert_eq!(a, (0..5).map(|_| r.next_u64()).collect::<Vec<_>>());
    }
}
