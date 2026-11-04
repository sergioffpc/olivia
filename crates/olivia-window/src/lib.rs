//! The window a game runs in.

mod driver;
mod window;

#[doc(hidden)]
pub use driver::{Attributes, Driver, Next, run};
pub use window::{Window, WindowEvent};

/// What `olivia-input` reads from the platform. Unstable: it changes with `winit`.
#[doc(hidden)]
pub mod raw {
    pub use winit::dpi::PhysicalPosition;
    pub use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent as Event};
    pub use winit::keyboard::{KeyCode, NativeKeyCode, PhysicalKey};
}
#[doc(hidden)]
pub use winit::window::Window as RawWindow;

/// A failure of this module.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The window could not be created.
    #[error("failed to create the window")]
    Create(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    fn create(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        Self::Create(error.into())
    }
}
