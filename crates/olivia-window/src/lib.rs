//! The window a game runs in.

mod driver;
mod window;

#[doc(hidden)]
pub use driver::{Attributes, Driver, run};
pub use window::{Window, WindowEvent};

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
