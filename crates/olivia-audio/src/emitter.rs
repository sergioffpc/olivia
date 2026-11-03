use std::sync::{Arc, Mutex};

use kira::track::{SpatialTrackBuilder, SpatialTrackHandle};
use olivia_math::Vec2;

use crate::audio::{Shared, retire, warn_about_refusal};
use crate::cue::TRANSITION;
use crate::{Cue, Play, Sound};

/// A position in the game's space that sounds play from, heard from the listener. Every sound
/// playing from it moves with it. Dropping it lets those sounds finish at the last position; a
/// looping one must be stopped through its [`Cue`]. An emitter that could not start behaves the same
/// and all it plays is inert.
#[derive(Debug)]
pub struct Emitter {
    live: Option<Live>,
}

#[derive(Debug)]
struct Live {
    track: SpatialTrackHandle,
    height: f32,
    shared: Arc<Mutex<Shared>>,
}

/// Where an emitter at `pos` sits: `height` off the game's plane, so pan moves gradually within that
/// radius, and on the side `kira`'s listener faces, so it is not heard from behind.
pub(crate) fn position(pos: Vec2, height: f32) -> [f32; 3] {
    [pos.x, pos.y, -height]
}

/// A track that is at full volume up to `near` and silent from `far`, along the game's plane. `kira`
/// measures straight to the emitter, which sits at height `near`.
pub(crate) fn track_builder(near: f32, far: f32) -> SpatialTrackBuilder {
    SpatialTrackBuilder::new().distances((near.hypot(near), far.hypot(near)))
}

impl Emitter {
    pub(crate) fn new(track: SpatialTrackHandle, height: f32, shared: Arc<Mutex<Shared>>) -> Self {
        Self {
            live: Some(Live {
                track,
                height,
                shared,
            }),
        }
    }

    /// An emitter with nothing behind it.
    pub(crate) fn inert() -> Self {
        Self { live: None }
    }

    /// Plays `sound` here with the default options.
    pub fn play(&mut self, sound: &Sound) -> Cue {
        self.play_with(sound, Play::new())
    }

    /// Plays `sound` here as `play` says, ignoring its pan. Its volume combines with the distance
    /// from the listener. Above `kira`'s limits the cue is inert.
    pub fn play_with(&mut self, sound: &Sound, play: Play) -> Cue {
        let Some(live) = &mut self.live else {
            return Cue::inert();
        };
        sound
            .start(&mut live.track, Play { pan: 0.0, ..play })
            .unwrap_or_else(|error| {
                warn_about_refusal(&live.shared, &error);
                Cue::inert()
            })
    }

    /// Moves the emitter, and every sound playing from it, to `pos`.
    pub fn set_position(&mut self, pos: Vec2) {
        if let Some(live) = &mut self.live {
            live.track
                .set_position(position(pos, live.height), TRANSITION);
        }
    }
}

impl Drop for Emitter {
    fn drop(&mut self) {
        if let Some(live) = self.live.take()
            && live.track.num_sounds() > 0
        {
            retire(&live.shared, live.track);
        }
    }
}
