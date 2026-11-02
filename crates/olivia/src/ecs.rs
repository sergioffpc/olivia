//! The world of entities Olivia owns, and the components it acts on.
//!
//! The game spawns entities in [`Setup::world`](crate::Setup::world) and changes them in
//! [`Update::world`](crate::Update::world). Olivia then simulates what has a body and draws what
//! has something to show. A component of the game's own is any `Send + Sync + 'static` type.
//!
//! Spawn a [`RigidBodyBuilder`] or a [`ColliderBuilder`], or both, to give an entity a body. On the
//! next step Olivia consumes them and adds a [`Body`]; a collider alone gets a fixed body. From then
//! on physics owns the entity's [`Transform`], and despawning the entity removes its body.

pub use hecs::{Bundle, CommandBuffer, Entity, Query, With, Without, World};
#[cfg(feature = "graphics")]
use olivia_graphics::{Draw, Font, Texture};
#[cfg(feature = "graphics")]
use olivia_math::Color;
use olivia_math::Vec2;
#[cfg(feature = "physics")]
use olivia_physics::{self as physics, ColliderHandle, RigidBodyHandle};
#[cfg(feature = "physics")]
pub use olivia_physics::{ColliderBuilder, RigidBodyBuilder};

#[cfg(feature = "graphics")]
pub use olivia_graphics::Camera;

/// Where an entity is, in the game's coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Transform {
    /// The point the entity's pivot sits on.
    pub position: Vec2,
    /// How far the entity is turned clockwise, in radians.
    pub rotation: f32,
}

impl Transform {
    /// An entity at `(x, y)`, not turned.
    #[must_use]
    pub const fn at(x: f32, y: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            rotation: 0.0,
        }
    }
}

/// An entity's body in the physics world, added by Olivia.
#[cfg(feature = "physics")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Body(pub RigidBodyHandle);

/// The entity whose body holds `collider`, if it has one.
///
/// # Panics
///
/// Panics if `collider` is stale.
#[cfg(feature = "physics")]
#[must_use]
pub fn entity_of(physics: &physics::World, collider: ColliderHandle) -> Option<Entity> {
    Entity::from_bits(u64::try_from(physics.collider(collider).user_data).ok()?)
}

/// Shows a texture at an entity's [`Transform`], which overrides `draw`'s rotation.
#[cfg(feature = "graphics")]
#[derive(Clone, Debug)]
pub struct Sprite {
    /// The image to show.
    pub texture: Texture,
    /// How to show it.
    pub draw: Draw,
}

#[cfg(feature = "graphics")]
impl Sprite {
    /// Shows `texture` with the default [`Draw`].
    #[must_use]
    pub fn new(texture: Texture) -> Self {
        Self {
            texture,
            draw: Draw::new(),
        }
    }

    /// Sets how to show the texture.
    #[must_use]
    pub fn draw(mut self, draw: Draw) -> Self {
        self.draw = draw;
        self
    }
}

/// Shows text at an entity's [`Transform`], which overrides `draw`'s rotation.
#[cfg(feature = "graphics")]
#[derive(Clone, Debug)]
pub struct Text {
    /// The font to set it in.
    pub font: Font,
    /// The font size. Must be positive and finite.
    pub size: f32,
    /// What it says.
    pub text: String,
    /// The width to wrap it to, or none to break only at line breaks.
    pub width: Option<f32>,
    /// How to show it.
    pub draw: Draw,
}

#[cfg(feature = "graphics")]
impl Text {
    /// Shows `text` in `font` at `size`, unwrapped, with the default [`Draw`].
    #[must_use]
    pub fn new(font: Font, size: f32, text: impl Into<String>) -> Self {
        Self {
            font,
            size,
            text: text.into(),
            width: None,
            draw: Draw::new(),
        }
    }

    /// Wraps the text to lines no wider than `width`.
    #[must_use]
    pub fn wrapped(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets how to show the text.
    #[must_use]
    pub fn draw(mut self, draw: Draw) -> Self {
        self.draw = draw;
        self
    }
}

/// The outline of a [`Shape`], placed at an entity's [`Transform`].
#[cfg(feature = "graphics")]
#[derive(Clone, Debug, PartialEq)]
pub enum Geometry {
    /// A filled rectangle of this size, hanging from its top-left corner.
    Rect(Vec2),
    /// An outlined rectangle of this size, with a line this wide.
    RectOutline(Vec2, f32),
    /// A filled circle of this radius.
    Circle(f32),
    /// An outlined circle of this radius, with a line this wide.
    CircleOutline(f32, f32),
    /// A filled polygon through these points, relative to the entity.
    Polygon(Vec<Vec2>),
    /// An outlined polygon through these points, with a line this wide.
    PolygonOutline(Vec<Vec2>, f32),
    /// A line to this point, relative to the entity, this wide.
    Line(Vec2, f32),
}

/// Shows a geometry at an entity's [`Transform`], which overrides `draw`'s rotation.
#[cfg(feature = "graphics")]
#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    /// What to show.
    pub geometry: Geometry,
    /// How to show it.
    pub draw: Draw,
}

#[cfg(feature = "graphics")]
impl Shape {
    /// Shows `geometry` with the default [`Draw`].
    #[must_use]
    pub fn new(geometry: Geometry) -> Self {
        Self {
            geometry,
            draw: Draw::new(),
        }
    }

    /// Sets how to show the geometry.
    #[must_use]
    pub fn draw(mut self, draw: Draw) -> Self {
        self.draw = draw;
        self
    }
}

/// Marks an entity as part of the HUD: it is drawn after everything seen through a [`Camera`], with
/// its [`Transform`] in logical pixels from the top-left corner of the window.
#[cfg(feature = "graphics")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Hud;

/// The colour a frame clears to, on any entity. A frame with none clears to black.
#[cfg(feature = "graphics")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clear(pub Color);

/// Draws every collider and its bounding box through each [`Camera`], in this colour, on any entity.
#[cfg(all(feature = "graphics", feature = "physics"))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColliderLines(pub Color);
