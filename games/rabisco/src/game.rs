use olivia::{
    Game, Result, Setup, Update,
    ecs::{CClear, Camera, Entity},
    math::{Color, Vec2},
    window::WindowEvent,
};

use crate::{
    assets::{Animations, Characters, Musics, Sounds, Textures},
    hud::HudManager,
    level::LevelManager,
    random::Random,
};

/// The size of the window, in logical pixels.
pub(crate) const WINDOW: Vec2 = Vec2::new(1280.0, 720.0);
/// How much the camera sees.
pub(crate) const VIEW: Vec2 = Vec2::new(12.8, 7.2);
/// How many logical pixels a metre takes.
const PIXELS_PER_METRE: f32 = 100.0;
/// The gravity, in metres per second squared.
const GRAVITY: f32 = 9.81;

#[derive(Debug)]
pub struct Rabisco {
    random: Random,

    animations: Animations,
    characters: Characters,
    sounds: Sounds,
    textures: Textures,

    camera: Entity,
    level: LevelManager,
    hud: HudManager,
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
        let mut random = Random::from_clock();

        let animations = Animations::load(s.filesystem)?;
        let characters = Characters::load(s.filesystem, s.graphics, &mut random)?;
        let sounds = Sounds::load(s.filesystem, s.audio)?;
        let textures = Textures::load(s.filesystem, s.graphics, &mut random)?;

        let musics = Musics::load(s.filesystem, s.audio)?;
        musics.play(s.audio, &mut random);

        s.world.spawn((CClear(Color::BLUE),));
        let camera = s.world.spawn((Camera {
            position: VIEW / 2.0,
            zoom: PIXELS_PER_METRE,
            rotation: 0.0,
        },));

        let level = LevelManager::load(&textures, s.world);
        let hud = HudManager::load(&textures, s.world);

        Ok(Self {
            random,
            animations,
            characters,
            sounds,
            textures,
            camera,
            level,
            hud,
        })
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        if u.events.contains(&WindowEvent::CloseRequested) {
            u.quit();
        }
        Ok(())
    }
}
