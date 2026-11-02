//! Characters built from parts on a skeleton.

mod character;
mod skeleton;

pub use character::Character;

/// A failure of this module.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A file cannot be read.
    #[error("failed to read a file")]
    Filesystem(#[from] olivia_filesystem::Error),
    /// A file is not the JSON it should be.
    #[error("failed to parse the JSON")]
    Json(#[from] serde_json::Error),
    /// An image cannot be decoded.
    #[error("failed to load an image")]
    Graphics(#[from] olivia_graphics::Error),
    /// The file has a version this reader does not know.
    #[error("unknown version {0}")]
    UnknownVersion(u64),
    /// The joints do not form a skeleton.
    #[error("invalid skeleton: {0}")]
    InvalidSkeleton(String),
}
