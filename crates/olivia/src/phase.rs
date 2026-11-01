use std::cell::Cell;
use std::marker::PhantomData;

use crate::Timer;

/// The handles available to [`Game::load`](crate::Game::load).
#[non_exhaustive]
#[derive(Debug)]
pub struct Setup<'a> {
    handles: PhantomData<&'a mut ()>,
}

impl Setup<'_> {
    pub(crate) fn new() -> Self {
        Self {
            handles: PhantomData,
        }
    }
}

/// The handles available to [`Game::update`](crate::Game::update).
#[non_exhaustive]
#[derive(Debug)]
pub struct Update<'a> {
    /// The fixed step, elapsed time and frame rate.
    pub timer: &'a Timer,
    quit: &'a Cell<bool>,
}

impl<'a> Update<'a> {
    pub(crate) fn new(timer: &'a Timer, quit: &'a Cell<bool>) -> Self {
        Self { timer, quit }
    }

    /// Asks the loop to stop once this update returns.
    pub fn quit(&self) {
        self.quit.set(true);
    }
}

/// The argument of [`Game::draw`](crate::Game::draw).
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
