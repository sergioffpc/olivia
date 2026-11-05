use olivia::{
    Game, Result, Setup, Update,
    audio::Cue,
    ecs::CClear,
    math::{Color, Vec2},
    window::WindowEvent,
};

use crate::{
    assets::{Assets, Musics},
    camera::Camera,
    hud::Hud,
    level::Level,
    random::Random,
    status,
};

/// The size of the window, in logical pixels.
pub(crate) const WINDOW: Vec2 = Vec2::new(1280.0, 720.0);
/// The gravity, in metres per second squared.
const GRAVITY: f32 = 9.81;

#[derive(Debug)]
#[expect(dead_code, reason = "read once the update phase plays the game")]
pub struct Rabisco {
    random: Random,
    assets: Assets,
    music: Cue,
    camera: Camera,
    level: Level,
    hud: Hud,
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

        let assets = Assets::load(s.filesystem, s.graphics, s.audio)?;
        let music = Musics::load(s.filesystem)?.play(s.filesystem, s.audio, &mut random)?;

        s.world.spawn((CClear(Color::BLUE),));
        let camera = Camera::spawn(s.world);
        let level = Level::spawn(s.world, &assets)?;
        let status = status::spawn(s.world);
        let hud = Hud::spawn(s.world, &assets.textures, status);

        Ok(Self {
            random,
            assets,
            music,
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
