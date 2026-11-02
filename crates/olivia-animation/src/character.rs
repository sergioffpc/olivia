use std::sync::Arc;

use olivia_filesystem::Filesystem;
use olivia_graphics::{Draw, Graphics, Texture, View};
use olivia_math::Vec2;

use crate::{Error, skeleton};

/// A set of parts on a skeleton. Cheap to clone, and placed in the world with a `Transform`.
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
}

/// One image, and where it goes as drawn.
#[derive(Debug)]
struct Part {
    texture: Texture,
    /// Its pivot, normalised to the image.
    origin: Vec2,
    /// Where its pivot is from the root joint, in pixels.
    offset: Vec2,
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
        let joints = skeleton::parse(&filesystem.read(&format!("{folder}/skeleton.json"))?)?;
        let part = |joint: skeleton::Joint| {
            let image = filesystem.read(&format!("{folder}/{}.png", joint.part))?;
            let texture = graphics.texture(&image)?;
            Ok(Part {
                origin: joint.point / texture.size(),
                offset: joint.offset,
                texture,
            })
        };
        Ok(Self {
            draw: Draw::new(),
            parts: joints.into_iter().map(part).collect::<Result<_, Error>>()?,
        })
    }

    /// Sets how to show the character.
    #[must_use]
    pub fn draw(mut self, draw: Draw) -> Self {
        self.draw = draw;
        self
    }

    /// Draws the character with its root joint at `position`, turned by `rotation`, `factor` of the
    /// way into the next fixed step.
    #[doc(hidden)]
    #[expect(unused_variables, reason = "nothing moves between steps yet")]
    pub fn render(&self, view: &mut View<'_>, position: Vec2, rotation: f32, factor: f32) {
        let scale = self.draw.scale_value();
        let turn = Vec2::from_angle(rotation);
        let draw = self.draw.rotation(rotation);
        for part in self.parts.iter() {
            let pivot = position + turn.rotate(part.offset * scale);
            view.sprite(&part.texture, pivot, draw.origin(part.origin));
        }
    }
}
