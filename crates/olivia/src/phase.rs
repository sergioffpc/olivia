use std::cell::Cell;
use std::marker::PhantomData;

use olivia_window::{Window, WindowEvent};

use crate::Timer;

/// The handles available to [`Game::load`](crate::Game::load).
#[non_exhaustive]
#[derive(Debug)]
pub struct Setup<'a> {
    /// The game's window.
    pub window: &'a mut Window,
}

impl<'a> Setup<'a> {
    pub(crate) fn new(window: &'a mut Window) -> Self {
        Self { window }
    }
}

/// The handles available to [`Game::update`](crate::Game::update).
#[non_exhaustive]
#[derive(Debug)]
pub struct Update<'a> {
    /// The game's window.
    pub window: &'a mut Window,
    /// What happened to the window since the last update. Only the first step of a frame has any.
    pub events: &'a [WindowEvent],
    /// The fixed step, elapsed time and frame rate.
    pub timer: &'a Timer,
    quit: &'a Cell<bool>,
}

impl<'a> Update<'a> {
    pub(crate) fn new(
        window: &'a mut Window,
        events: &'a [WindowEvent],
        timer: &'a Timer,
        quit: &'a Cell<bool>,
    ) -> Self {
        Self {
            window,
            events,
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
    target: PhantomData<&'a mut ()>,
}

impl Frame<'_> {
    pub(crate) fn new(interpolation_factor: f32) -> Self {
        Self {
            interpolation_factor,
            target: PhantomData,
        }
    }

    /// How far the time since the last update has advanced into the next fixed step, from 0 to 1.
    #[must_use]
    pub fn interpolation_factor(&self) -> f32 {
        self.interpolation_factor
    }
}
