use std::f32::consts::{PI, TAU};
use std::sync::Arc;

use olivia_filesystem::Filesystem;
use olivia_graphics::{Draw, Graphics, Texture, View};
use olivia_math::Vec2;

use crate::{Animation, Error, Play, skeleton};

/// A set of parts on a skeleton, with what is playing on it. Cheap to clone, and placed in the
/// world with a `Transform`.
#[derive(Clone, Debug)]
pub struct Character {
    /// How to show the character.
    ///
    /// `scale` converts image pixels to game units, and a negative x mirrors the character around
    /// its root joint. `color`, `filter` and `blend` apply to every part. Parts share `depth`, and
    /// draw in the order of the skeleton's joints. The entity's `Transform` overrides `rotation`,
    /// and `origin` is ignored, because each part has its own pivot.
    pub draw: Draw,
    /// The parts in drawing order, from back to front.
    parts: Arc<[Part]>,
    /// The index of each part, with every part after the one it hangs from.
    walk: Arc<[usize]>,
    /// The animation playing, once one has been asked for.
    base: Option<Layer>,
    /// The overlays over it, the last added on top.
    overlays: Vec<Layer>,
    /// The length of the last fixed step, in seconds.
    step: f32,
}

/// One image, and how it hangs.
#[derive(Debug)]
struct Part {
    /// The name of its joint.
    name: String,
    texture: Texture,
    /// Its pivot, normalised to the image.
    origin: Vec2,
    /// The index of the part it hangs from.
    parent: Option<usize>,
    /// Where its pivot is from the pivot of the part it hangs from as drawn, in pixels.
    offset: Vec2,
}

/// An animation in play.
#[derive(Clone, Debug)]
struct Layer {
    animation: Animation,
    looping: bool,
    /// For each part, the animation's column for its joint, if it covers it.
    columns: Vec<Option<usize>>,
    /// The angle of each part when it started, frozen.
    from: Vec<f32>,
    /// Seconds it takes to get from `from` to the animation.
    crossfade: f32,
    /// Seconds since it started, at the last fixed step.
    age: f64,
    /// Whether this overlay is on its way out, moving from `from` to what is below it.
    leaving: bool,
}

impl Layer {
    /// The angle of each part `behind` seconds before the last fixed step.
    fn angles(&self, behind: f32) -> Vec<f32> {
        let age = self.age - f64::from(behind);
        let fade = self.fade(age);
        let (row, next, between) = self.animation.rows(age, self.looping);
        let angle = |(column, from): (&Option<usize>, &f32)| {
            let to = column.map_or(0.0, |column| shortest(row[column], next[column], between));
            shortest(*from, to, fade)
        };
        self.columns.iter().zip(&self.from).map(angle).collect()
    }

    /// Puts this overlay over `angles`, on the joints it covers.
    fn over(&self, angles: &mut [f32], behind: f32) {
        let own = self.angles(behind);
        let fade = self.fade(self.age - f64::from(behind));
        for (index, column) in self.columns.iter().enumerate() {
            if column.is_some() {
                angles[index] = if self.leaving {
                    shortest(self.from[index], angles[index], fade)
                } else {
                    own[index]
                };
            }
        }
    }

    /// How far the crossfade has gone at `age`, from 0 to 1. Before it began it has not moved.
    #[expect(clippy::cast_possible_truncation, reason = "a fraction")]
    fn fade(&self, age: f64) -> f32 {
        let crossfade = f64::from(self.crossfade);
        if age < 0.0 {
            0.0
        } else if age >= crossfade {
            1.0
        } else {
            (age / crossfade) as f32
        }
    }
}

/// The angle `amount` of the way from `from` to `to`, along the shortest path.
fn shortest(from: f32, to: f32, amount: f32) -> f32 {
    from + ((to - from + PI).rem_euclid(TAU) - PI) * amount
}

impl Character {
    /// Loads the character in `folder`: its `skeleton.json`, and a PNG image for each part.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Filesystem`] if a file is missing or cannot be read, [`Error::Json`] if
    /// `skeleton.json` is malformed, [`Error::UnknownVersion`] if it has a version this reader does
    /// not know, [`Error::InvalidSkeleton`] if its joints do not form a tree with one root and one
    /// joint per name and part, or [`Error::Graphics`] if an image cannot be decoded.
    ///
    /// # Panics
    ///
    /// Panics if an image is larger than the GPU supports.
    pub fn load(
        filesystem: &Filesystem,
        graphics: &mut Graphics,
        folder: &str,
    ) -> Result<Self, Error> {
        let skeleton = skeleton::parse(&filesystem.read(&format!("{folder}/skeleton.json"))?)?;
        let part = |joint: skeleton::Joint| {
            let image = filesystem.read(&format!("{folder}/{}.png", joint.part))?;
            let texture = graphics.texture(&image)?;
            Ok(Part {
                name: joint.name,
                origin: joint.point / texture.size(),
                parent: joint.parent,
                offset: joint.offset,
                texture,
            })
        };
        let parts = skeleton.joints.into_iter().map(part);
        Ok(Self {
            draw: Draw::new(),
            parts: parts.collect::<Result<_, Error>>()?,
            walk: skeleton.walk.into(),
            base: None,
            overlays: Vec::new(),
            step: 0.0,
        })
    }

    /// Sets how to show the character.
    #[must_use]
    pub fn draw(mut self, draw: Draw) -> Self {
        self.draw = draw;
        self
    }

    /// Changes the animation playing. Requesting the one already playing does nothing, even with a
    /// different `play`, so the call can repeat every update.
    ///
    /// # Errors
    ///
    /// Returns [`Error::MissingJoint`] if the animation names a joint the skeleton lacks.
    pub fn play(&mut self, animation: &Animation, play: Play) -> Result<(), Error> {
        let base = self.base.as_ref();
        if base.is_some_and(|layer| layer.animation.is(animation)) {
            return Ok(());
        }
        self.base = Some(self.layer(animation, play, self.base_angles(0.0))?);
        Ok(())
    }

    /// Plays `animation` over the base, on only the joints it covers.
    ///
    /// # Errors
    ///
    /// Returns [`Error::MissingJoint`] if the animation names a joint the skeleton lacks.
    pub fn overlay(&mut self, animation: &Animation, play: Play) -> Result<(), Error> {
        if self.active(animation).is_none() {
            let layer = self.layer(animation, play, self.angles(0.0))?;
            self.overlays.push(layer);
        }
        Ok(())
    }

    /// The index of `animation` among the overlays, if it is active.
    fn active(&self, animation: &Animation) -> Option<usize> {
        let same = |layer: &Layer| !layer.leaving && layer.animation.is(animation);
        self.overlays.iter().position(same)
    }

    /// Takes `animation` off, moving its joints to what is below it over `crossfade` seconds.
    /// Removing one that isn't active does nothing.
    pub fn remove_overlay(&mut self, animation: &Animation, crossfade: f32) {
        if let Some(index) = self.active(animation) {
            let from = self.angles(0.0);
            let layer = &mut self.overlays[index];
            layer.from = from;
            layer.crossfade = crossfade;
            layer.age = 0.0;
            layer.leaving = true;
        }
    }

    /// A layer starting now, from the angles `from`.
    fn layer(&self, animation: &Animation, play: Play, from: Vec<f32>) -> Result<Layer, Error> {
        let joints = animation.joints();
        let lacking = |joint: &&String| !self.parts.iter().any(|part| part.name == **joint);
        if let Some(joint) = joints.iter().find(lacking) {
            return Err(Error::MissingJoint(joint.clone()));
        }
        let column = |part: &Part| joints.iter().position(|joint| *joint == part.name);
        Ok(Layer {
            animation: animation.clone(),
            looping: play.looping,
            columns: self.parts.iter().map(column).collect(),
            from,
            crossfade: play.crossfade,
            age: 0.0,
            leaving: false,
        })
    }

    /// Whether `animation` is active, as base or overlay, doesn't loop and has reached its last
    /// row. It has not once another has taken over, even during the crossfade.
    #[must_use]
    pub fn finished(&self, animation: &Animation) -> bool {
        let done = |layer: &Layer| {
            layer.animation.is(animation)
                && !layer.leaving
                && !layer.looping
                && animation.ended(layer.age)
        };
        let mut layers = self.base.iter().chain(&self.overlays);
        layers.any(done)
    }

    /// Moves what is playing on by a fixed step of `dt` seconds.
    #[doc(hidden)]
    pub fn advance(&mut self, dt: f32) {
        self.step = dt;
        for layer in self.base.iter_mut().chain(&mut self.overlays) {
            layer.age += f64::from(dt);
        }
        // Kept until the whole of the last step is past its crossfade, for what `render` looks back at.
        let dt = f64::from(dt);
        self.overlays
            .retain(|layer| !layer.leaving || layer.age - dt < f64::from(layer.crossfade));
    }

    /// The angle the base gives each part `behind` seconds before the last fixed step.
    fn base_angles(&self, behind: f32) -> Vec<f32> {
        match &self.base {
            Some(layer) => layer.angles(behind),
            None => vec![0.0; self.parts.len()],
        }
    }

    /// The angle of each part `behind` seconds before the last fixed step.
    fn angles(&self, behind: f32) -> Vec<f32> {
        let mut angles = self.base_angles(behind);
        for layer in &self.overlays {
            layer.over(&mut angles, behind);
        }
        angles
    }

    /// Where `angles` put the pivot of each part from the root joint, in pixels, and the angle each
    /// part adds up to.
    fn placed(&self, angles: &[f32]) -> Vec<(Vec2, f32)> {
        let mut placed = vec![(Vec2::ZERO, 0.0); self.parts.len()];
        for &index in self.walk.iter() {
            let part = &self.parts[index];
            let (pivot, angle) = part
                .parent
                .map_or((Vec2::ZERO, 0.0), |parent| placed[parent]);
            let pivot = pivot + Vec2::from_angle(angle).rotate(part.offset);
            placed[index] = (pivot, angle + angles[index]);
        }
        placed
    }

    /// Draws the character with its root joint at `position`, turned by `rotation`, and its pose
    /// `factor` of the way between the last two fixed steps, as `interpolated_position` has it.
    #[doc(hidden)]
    pub fn render(&self, view: &mut View<'_>, position: Vec2, rotation: f32, factor: f32) {
        let scale = self.draw.scale_value();
        let mirror = if scale.x * scale.y < 0.0 { -1.0 } else { 1.0 };
        let turn = Vec2::from_angle(rotation);
        let placed = self.placed(&self.angles((1.0 - factor) * self.step));
        for (part, (pivot, angle)) in self.parts.iter().zip(placed) {
            let draw = self.draw.rotation(rotation + mirror * angle);
            let pivot = position + turn.rotate(pivot * scale);
            view.sprite(&part.texture, pivot, draw.origin(part.origin));
        }
    }
}
