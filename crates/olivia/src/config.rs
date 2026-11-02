use std::path::PathBuf;

use olivia_math::Vec2;
use olivia_window::Attributes;

/// How a game starts, fixed before [`run`](crate::run).
///
/// An option for a disabled module is ignored.
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    identity: String,
    title: String,
    size: Vec2,
    resizable: bool,
    vsync: bool,
    fullscreen: bool,
    msaa: u32,
    assets: Option<PathBuf>,
}

impl Config {
    /// Creates a configuration for the game named `identity`, which also titles its window.
    #[must_use]
    pub fn new(identity: impl Into<String>) -> Self {
        let identity = identity.into();
        Self {
            title: identity.clone(),
            identity,
            size: Vec2::new(1280.0, 720.0),
            resizable: true,
            vsync: true,
            fullscreen: false,
            msaa: 4,
            assets: None,
        }
    }

    /// Sets the window title.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the window's logical size. The default is 1280 by 720.
    #[must_use]
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    /// Sets whether the player can resize the window. The default is `true`.
    #[must_use]
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Sets whether drawing follows vsync. The default is `true`.
    #[must_use]
    pub fn vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    /// Sets whether the window starts fullscreen. The default is `false`.
    #[must_use]
    pub fn fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    /// Sets the MSAA sample count. The default is 4; 1 disables it.
    #[must_use]
    pub fn msaa(mut self, samples: u32) -> Self {
        self.msaa = samples;
        self
    }

    /// Overrides where the game's assets are read from.
    #[must_use]
    pub fn assets(mut self, path: impl Into<PathBuf>) -> Self {
        self.assets = Some(path.into());
        self
    }

    /// How the window starts.
    pub(crate) fn window_attributes(self) -> Attributes {
        Attributes {
            title: self.title,
            size: self.size,
            resizable: self.resizable,
            fullscreen: self.fullscreen,
        }
    }
}
