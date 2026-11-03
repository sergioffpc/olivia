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
    /// and `origin` is ignored, because each part turns around its own joint.
    pub draw: Draw,
    /// The parts in drawing order, from back to front.
    parts: Arc<[Part]>,
    /// The index of each part, with every part after the one it hangs from.
    walk: Arc<[usize]>,
    /// The animation playing, once one has been asked for.
    base: Option<Track>,
    /// The overlays over it, the last added on top.
    overlays: Vec<Track>,
    /// The length of the last fixed step, in seconds.
    step: f32,
}

/// Where a joint is and how its part is turned, read with [`Character::joint`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JointPose {
    /// Where the joint is from the root joint, in game units, with the character's scale and
    /// mirror applied. It is relative to the entity: add its `Transform`, and account for the
    /// entity's rotation if it has any.
    pub offset: Vec2,
    /// The angle the part adds up to, in radians, where zero is as drawn. It is the angle shown on
    /// screen, so a mirrored character gives its negative.
    pub rotation: f32,
}

/// One image, and how it hangs.
#[derive(Debug)]
struct Part {
    /// The name of its joint.
    name: String,
    texture: Texture,
    /// Its joint, normalised to the image.
    origin: Vec2,
    /// The index of the part it hangs from.
    parent: Option<usize>,
    /// Where its joint is from the joint of the part it hangs from as drawn, in pixels.
    offset: Vec2,
}

/// An animation in play.
#[derive(Clone, Debug)]
struct Track {
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

impl Track {
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
        if base.is_some_and(|track| track.animation.same_as(animation)) {
            return Ok(());
        }
        self.base = Some(self.track(animation, play, self.base_angles(0.0))?);
        Ok(())
    }

    /// Plays `animation` over the base, on only the joints it covers.
    ///
    /// # Errors
    ///
    /// Returns [`Error::MissingJoint`] if the animation names a joint the skeleton lacks.
    pub fn overlay(&mut self, animation: &Animation, play: Play) -> Result<(), Error> {
        if self.active(animation).is_none() {
            let track = self.track(animation, play, self.angles(0.0))?;
            self.overlays.push(track);
        }
        Ok(())
    }

    /// The index of `animation` among the overlays, if it is active.
    fn active(&self, animation: &Animation) -> Option<usize> {
        let same = |track: &Track| !track.leaving && track.animation.same_as(animation);
        self.overlays.iter().position(same)
    }

    /// Takes `animation` off, moving its joints to what is below it over `crossfade` seconds.
    /// Removing one that isn't active does nothing.
    pub fn remove_overlay(&mut self, animation: &Animation, crossfade: f32) {
        if let Some(index) = self.active(animation) {
            let from = self.angles(0.0);
            let track = &mut self.overlays[index];
            track.from = from;
            track.crossfade = crossfade;
            track.age = 0.0;
            track.leaving = true;
        }
    }

    /// A track starting now, from the angles `from`.
    fn track(&self, animation: &Animation, play: Play, from: Vec<f32>) -> Result<Track, Error> {
        let joints = animation.joints();
        let lacking = |joint: &&String| !self.parts.iter().any(|part| part.name == **joint);
        if let Some(joint) = joints.iter().find(lacking) {
            return Err(Error::MissingJoint(joint.clone()));
        }
        let column = |part: &Part| joints.iter().position(|joint| *joint == part.name);
        Ok(Track {
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
        let done = |track: &Track| {
            track.animation.same_as(animation)
                && !track.leaving
                && !track.looping
                && animation.ended(track.age)
        };
        let mut tracks = self.base.iter().chain(&self.overlays);
        tracks.any(done)
    }

    /// Moves what is playing on by a fixed step of `dt` seconds.
    #[doc(hidden)]
    pub fn advance(&mut self, dt: f32) {
        self.step = dt;
        for track in self.base.iter_mut().chain(&mut self.overlays) {
            track.age += f64::from(dt);
        }
        // Kept until the whole of the last step is past its crossfade, for what `render` looks back at.
        let dt = f64::from(dt);
        self.overlays
            .retain(|track| !track.leaving || track.age - dt < f64::from(track.crossfade));
    }

    /// The angle the base gives each part `behind` seconds before the last fixed step.
    fn base_angles(&self, behind: f32) -> Vec<f32> {
        match &self.base {
            Some(track) => track.angles(behind),
            None => vec![0.0; self.parts.len()],
        }
    }

    /// The angle of each part `behind` seconds before the last fixed step.
    fn angles(&self, behind: f32) -> Vec<f32> {
        let mut angles = self.base_angles(behind);
        for track in &self.overlays {
            track.over(&mut angles, behind);
        }
        angles
    }

    /// How the scale shows angles: -1 if it mirrors the character, or else 1.
    fn mirror(&self) -> f32 {
        let scale = self.draw.scale_value();
        if scale.x * scale.y < 0.0 { -1.0 } else { 1.0 }
    }

    /// The pose `angles` give each joint. Offsets scale in the frame of the part they lie on, then
    /// turn with it, as a sprite does, so that joints stay attached under any scale.
    fn poses(&self, angles: &[f32]) -> Vec<JointPose> {
        let scale = self.draw.scale_value();
        let mirror = self.mirror();
        let root = JointPose {
            offset: Vec2::ZERO,
            rotation: 0.0,
        };
        let mut poses = vec![root; self.parts.len()];
        for &index in self.walk.iter() {
            let part = &self.parts[index];
            let parent = part.parent.map_or(root, |parent| poses[parent]);
            let turn = Vec2::from_angle(parent.rotation);
            poses[index] = JointPose {
                offset: parent.offset + turn.rotate(part.offset * scale),
                rotation: parent.rotation + mirror * angles[index],
            };
        }
        poses
    }

    /// The pose of the joint called `name` at the last fixed step, if there is one.
    #[must_use]
    pub fn joint(&self, name: &str) -> Option<JointPose> {
        let index = self.parts.iter().position(|part| part.name == name)?;
        Some(self.poses(&self.angles(0.0))[index])
    }

    /// Draws the character with its root joint at `position`, turned by `rotation`, and its pose
    /// `factor` of the way between the last two fixed steps, as `interpolated_position` has it.
    #[doc(hidden)]
    pub fn render(&self, view: &mut View<'_>, position: Vec2, rotation: f32, factor: f32) {
        let turn = Vec2::from_angle(rotation);
        let poses = self.poses(&self.angles((1.0 - factor) * self.step));
        for (part, pose) in self.parts.iter().zip(poses) {
            let draw = self.draw.rotation(rotation + pose.rotation);
            let joint = position + turn.rotate(pose.offset);
            view.sprite(&part.texture, joint, draw.origin(part.origin));
        }
    }
}
