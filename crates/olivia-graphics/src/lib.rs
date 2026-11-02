//! Drawing to the game's window.

mod camera;
pub mod depth;
mod draw;
mod gpu;
mod graphics;
mod pipeline;
mod texture;
mod view;

pub use camera::Camera;
pub use draw::{Blend, Draw, Filter};
#[doc(hidden)]
pub use gpu::sample_count;
pub use graphics::Graphics;
#[doc(hidden)]
pub use graphics::{Config, Target};
pub use texture::Texture;
pub use view::View;
#[doc(hidden)]
pub use view::{Batch, Mesh, Vertex};

/// A failure of this module.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// No GPU adapter can draw to the window.
    #[error("found no GPU adapter")]
    NoAdapter(#[source] Option<Box<dyn std::error::Error + Send + Sync>>),
    /// The GPU adapter gave no device.
    #[error("failed to get a GPU device")]
    NoDevice(#[source] Box<dyn std::error::Error + Send + Sync>),
    /// The bytes are not a PNG or JPEG image.
    #[error("failed to decode the image")]
    UndecodableImage(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    fn no_adapter(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        Self::NoAdapter(Some(error.into()))
    }
}
