#[cfg(feature = "audio")]
use olivia_audio::Audio;
#[cfg(feature = "filesystem")]
use olivia_filesystem::Filesystem;
#[cfg(feature = "graphics")]
use olivia_graphics::Graphics;
#[cfg(feature = "input")]
use olivia_input::Input;

use crate::{Config, Result};

/// The modules a game's phases borrow, built from its [`Config`] before the window opens.
#[derive(Debug)]
pub struct Modules {
    #[cfg(feature = "audio")]
    pub(crate) audio: Audio,
    #[cfg(feature = "filesystem")]
    pub(crate) filesystem: Filesystem,
    #[cfg(feature = "graphics")]
    pub(crate) graphics: Graphics,
    #[cfg(feature = "input")]
    pub(crate) input: Input,
}

impl Modules {
    /// Builds the modules `config` asks for.
    ///
    /// # Errors
    ///
    /// Returns the failure of a module that cannot start.
    #[cfg_attr(
        not(any(feature = "filesystem", feature = "graphics")),
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
        })
    }
}
