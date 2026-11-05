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
            idle: Animations::read(filesystem, "idle")?,
            walk: Animations::read(filesystem, "walk")?,
            jump: Animations::read(filesystem, "jump")?,
            double_jump: Animations::read(filesystem, "double_jump")?,
            shoot: Animations::read(filesystem, "shoot")?,
            hurt: Animations::read(filesystem, "hurt")?,
            win: Animations::read(filesystem, "win")?,
            calling: Animations::read(filesystem, "calling")?,
            freed: Animations::read(filesystem, "freed")?,
        })
    }

    fn read(filesystem: &Filesystem, name: &str) -> Result<Animation> {
        let bytes = read(filesystem, &format!("animations/{name}.json"))?;
        Ok(Animation::parse(bytes.as_deref().unwrap_or(NO_ANIMATION))?)
    }
}

pub struct Sounds {
    jump: Sound,
    fart: Sound,
    shoot: Sound,
    hit: Sound,
    pickup: Sound,
    hurt: Sound,
    gameover: Sound,
    help: Sound,
    win: Sound,
}

impl Sounds {
    fn load(filesystem: &Filesystem, audio: &mut Audio) -> Result<Self> {
        Ok(Self {
            jump: Sounds::read(filesystem, audio, "jump")?,
            fart: Sounds::read(filesystem, audio, "fart")?,
            shoot: Sounds::read(filesystem, audio, "shoot")?,
            hit: Sounds::read(filesystem, audio, "hit")?,
            pickup: Sounds::read(filesystem, audio, "pickup")?,
            hurt: Sounds::read(filesystem, audio, "hurt")?,
            gameover: Sounds::read(filesystem, audio, "gameover")?,
            help: Sounds::read(filesystem, audio, "help")?,
            win: Sounds::read(filesystem, audio, "win")?,
        })
    }

    fn read(filesystem: &Filesystem, audio: &mut Audio, name: &str) -> Result<Sound> {
        let bytes = read(filesystem, &format!("sounds/{name}.wav"))?;
        Ok(audio.sound(bytes.unwrap_or_else(silence))?)
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
            life: Textures::read(filesystem, graphics, "life")?,
            life_lost: Textures::read(filesystem, graphics, "life_lost")?,
            digits: [
                Textures::read(filesystem, graphics, "digit_0")?,
                Textures::read(filesystem, graphics, "digit_1")?,
                Textures::read(filesystem, graphics, "digit_2")?,
                Textures::read(filesystem, graphics, "digit_3")?,
                Textures::read(filesystem, graphics, "digit_4")?,
                Textures::read(filesystem, graphics, "digit_5")?,
                Textures::read(filesystem, graphics, "digit_6")?,
                Textures::read(filesystem, graphics, "digit_7")?,
                Textures::read(filesystem, graphics, "digit_8")?,
                Textures::read(filesystem, graphics, "digit_9")?,
            ],
            projectile: Textures::read(filesystem, graphics, "projectile")?,
            fart: Textures::read(filesystem, graphics, "fart")?,
            coin: Textures::read(filesystem, graphics, "coin")?,
            goal: Textures::read(filesystem, graphics, "goal")?,
            crawl: Textures::read(filesystem, graphics, "crawl")?,
            roll: Textures::read(filesystem, graphics, "roll")?,
            float: Textures::read(filesystem, graphics, "float")?,
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
