use std::fmt;
#[cfg(feature = "filesystem")]
use std::mem;

#[cfg(feature = "audio")]
use olivia_audio::Audio;
#[cfg(feature = "filesystem")]
use olivia_filesystem::Filesystem;
#[cfg(feature = "graphics")]
use olivia_graphics::Graphics;
#[cfg(feature = "graphics")]
use olivia_graphics::Target;
#[cfg(feature = "input")]
use olivia_input::Input;

#[cfg(feature = "graphics")]
use crate::renderer::{self, Poses};
#[cfg(feature = "physics")]
use crate::simulation::Simulation;
use crate::{Config, Result, ecs};

/// The modules a game's phases borrow, built from its [`Config`] before the window opens.
pub struct Modules {
    #[cfg(feature = "audio")]
    pub(crate) audio: Audio,
    #[cfg(feature = "filesystem")]
    pub(crate) filesystem: Filesystem,
    #[cfg(feature = "graphics")]
    pub(crate) graphics: Graphics,
    #[cfg(feature = "input")]
    pub(crate) input: Input,
    #[cfg(feature = "physics")]
    pub(crate) simulation: Simulation,
    pub(crate) world: ecs::World,
    #[cfg(any(feature = "filesystem", feature = "graphics"))]
    pub(crate) config: Config,
}

/// What a game leaves behind in the modules, and a hot reload starts over.
#[cfg(feature = "filesystem")]
pub(crate) struct Stage {
    #[cfg(feature = "audio")]
    audio: Audio,
    #[cfg(feature = "physics")]
    simulation: Simulation,
    world: ecs::World,
}

impl Modules {
    /// Builds the modules `config` asks for.
    ///
    /// # Errors
    ///
    /// Returns the failure of a module that cannot start.
    #[cfg_attr(
        not(any(feature = "filesystem", feature = "graphics", feature = "physics")),
        expect(unused_variables)
    )]
    pub fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            #[cfg(feature = "audio")]
            audio: Audio::default(),
            #[cfg(feature = "filesystem")]
            filesystem: config.filesystem()?,
            #[cfg(feature = "graphics")]
            graphics: config.graphics(),
            #[cfg(feature = "input")]
            input: Input::default(),
            #[cfg(feature = "physics")]
            simulation: Simulation::new(config.gravity_value()),
            world: ecs::World::new(),
            #[cfg(any(feature = "filesystem", feature = "graphics"))]
            config: config.clone(),
        })
    }

    /// A stage as it is before a game loads.
    #[cfg(feature = "filesystem")]
    #[cfg_attr(not(feature = "physics"), expect(clippy::unused_self))]
    pub(crate) fn empty_stage(&self) -> Stage {
        Stage {
            #[cfg(feature = "audio")]
            audio: Audio::default(),
            #[cfg(feature = "physics")]
            simulation: Simulation::new(self.config.gravity_value()),
            world: ecs::World::new(),
        }
    }

    /// Trades the stage in use for `stage`.
    #[cfg(feature = "filesystem")]
    pub(crate) fn swap_stage(&mut self, stage: &mut Stage) {
        #[cfg(feature = "audio")]
        mem::swap(&mut self.audio, &mut stage.audio);
        #[cfg(feature = "physics")]
        mem::swap(&mut self.simulation, &mut stage.simulation);
        mem::swap(&mut self.world, &mut stage.world);
    }

    /// Draws the world `factor` of the way into the next fixed step.
    #[cfg(feature = "graphics")]
    #[cfg_attr(
        not(any(feature = "animation", feature = "physics")),
        expect(unused_variables)
    )]
    pub(crate) fn render(&mut self, factor: f32) -> Target<'_> {
        let poses = Poses {
            #[cfg(feature = "physics")]
            physics: &self.simulation.physics,
            #[cfg(any(feature = "animation", feature = "physics"))]
            factor,
            #[cfg(not(feature = "physics"))]
            world: std::marker::PhantomData,
        };
        let mut target = self.graphics.target();
        renderer::render(&self.world, poses, &mut target);
        target
    }
}

impl fmt::Debug for Modules {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Modules").finish_non_exhaustive()
    }
}
