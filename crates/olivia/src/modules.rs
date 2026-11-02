#[cfg(feature = "filesystem")]
use olivia_filesystem::Filesystem;
#[cfg(feature = "graphics")]
use olivia_graphics::Graphics;

use crate::{Config, Result};

/// The modules a game's phases borrow, built from its [`Config`] before the window opens.
#[derive(Debug)]
pub struct Modules {
    #[cfg(feature = "filesystem")]
    pub(crate) filesystem: Filesystem,
    #[cfg(feature = "graphics")]
    pub(crate) graphics: Graphics,
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
            #[cfg(feature = "filesystem")]
            filesystem: config.filesystem()?,
            #[cfg(feature = "graphics")]
            graphics: config.graphics(),
        })
    }
}
