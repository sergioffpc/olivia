use std::cell::Cell;
#[cfg(not(feature = "graphics"))]
use std::marker::PhantomData;

#[cfg(feature = "audio")]
use olivia_audio::Audio;
#[cfg(feature = "filesystem")]
use olivia_filesystem::Filesystem;
#[cfg(feature = "graphics")]
use olivia_graphics::{Camera, Graphics, Target, View};
#[cfg(feature = "input")]
use olivia_input::Input;
#[cfg(feature = "graphics")]
use olivia_math::Color;
use olivia_window::{Window, WindowEvent};

use crate::{Modules, Timer};

/// The handles available to [`Game::load`](crate::Game::load).
#[non_exhaustive]
#[derive(Debug)]
pub struct Setup<'a> {
    /// The game's window.
    pub window: &'a mut Window,
    /// The game's audio.
    #[cfg(feature = "audio")]
    pub audio: &'a mut Audio,
    /// The game's assets and saves.
    #[cfg(feature = "filesystem")]
    pub filesystem: &'a mut Filesystem,
    /// The game's graphics.
    #[cfg(feature = "graphics")]
    pub graphics: &'a mut Graphics,
}

impl<'a> Setup<'a> {
    #[cfg_attr(
        not(any(feature = "audio", feature = "filesystem", feature = "graphics")),
        expect(unused_variables)
    )]
    pub(crate) fn new(window: &'a mut Window, modules: &'a mut Modules) -> Self {
        Self {
            window,
            #[cfg(feature = "audio")]
            audio: &mut modules.audio,
            #[cfg(feature = "filesystem")]
            filesystem: &mut modules.filesystem,
            #[cfg(feature = "graphics")]
            graphics: &mut modules.graphics,
        }
    }
}

/// The handles available to [`Game::update`](crate::Game::update).
#[non_exhaustive]
#[derive(Debug)]
pub struct Update<'a> {
    /// The game's window.
    pub window: &'a mut Window,
    /// The game's audio.
    #[cfg(feature = "audio")]
    pub audio: &'a mut Audio,
    /// The game's assets and saves.
    #[cfg(feature = "filesystem")]
    pub filesystem: &'a mut Filesystem,
    /// The game's graphics.
    #[cfg(feature = "graphics")]
    pub graphics: &'a mut Graphics,
    /// What happened to the window since the last update. Only the first step of a frame has any.
    pub events: &'a [WindowEvent],
    /// The keyboard and mouse.
    #[cfg(feature = "input")]
    pub input: &'a Input,
    /// The fixed step, elapsed time and frame rate.
    pub timer: &'a Timer,
    quit: &'a Cell<bool>,
}

impl<'a> Update<'a> {
    #[cfg_attr(
        not(any(
            feature = "audio",
            feature = "filesystem",
            feature = "graphics",
            feature = "input"
        )),
        expect(unused_variables)
    )]
    pub(crate) fn new(
        window: &'a mut Window,
        modules: &'a mut Modules,
        events: &'a [WindowEvent],
        timer: &'a Timer,
        quit: &'a Cell<bool>,
    ) -> Self {
        Self {
            window,
            #[cfg(feature = "audio")]
            audio: &mut modules.audio,
            #[cfg(feature = "filesystem")]
            filesystem: &mut modules.filesystem,
            #[cfg(feature = "graphics")]
            graphics: &mut modules.graphics,
            events,
            #[cfg(feature = "input")]
            input: &modules.input,
            timer,
            quit,
        }
    }

    /// Asks the loop to stop once this update returns.
    pub fn quit(&self) {
        self.quit.set(true);
    }
}

/// The handle [`Game::draw`](crate::Game::draw) receives to produce one frame.
#[derive(Debug)]
pub struct Frame<'a> {
    interpolation_factor: f32,
    #[cfg(feature = "graphics")]
    target: Target<'a>,
    #[cfg(not(feature = "graphics"))]
    target: PhantomData<&'a mut ()>,
}

impl<'a> Frame<'a> {
    #[cfg_attr(not(feature = "graphics"), expect(unused_variables))]
    pub(crate) fn new(interpolation_factor: f32, modules: &'a mut Modules) -> Self {
        Self {
            interpolation_factor,
            #[cfg(feature = "graphics")]
            target: modules.graphics.target(),
            #[cfg(not(feature = "graphics"))]
            target: PhantomData,
        }
    }

    /// Shows the frame and returns whether it reached the window.
    #[cfg(feature = "graphics")]
    pub(crate) fn present(self) -> bool {
        self.target.present()
    }

    /// Shows the frame and returns whether it reached the window, which it never does here.
    #[cfg(not(feature = "graphics"))]
    #[expect(clippy::unused_self, reason = "matches the graphics version")]
    pub(crate) fn present(self) -> bool {
        false
    }

    /// How far the time since the last update has advanced into the next fixed step, from 0 to 1.
    #[must_use]
    pub fn interpolation_factor(&self) -> f32 {
        self.interpolation_factor
    }

    /// Sets the colour the frame clears to. A frame with no call clears to black.
    #[cfg(feature = "graphics")]
    pub fn clear(&mut self, color: Color) {
        self.target.clear(color);
    }

    /// Starts drawing through `camera`. Only a [`View`] draws, and views draw in creation order.
    #[cfg(feature = "graphics")]
    pub fn view(&mut self, camera: Camera) -> View<'_> {
        self.target.view(camera)
    }
}
