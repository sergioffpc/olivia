//! Characters built from parts on a skeleton, and the animations they play.

mod animation;
mod character;
mod skeleton;

use serde::Deserialize;
use serde::de::DeserializeOwned;

pub use animation::{Animation, Play};
pub use character::{Character, JointPose};

const VERSION: u64 = 1;

/// A failure of this module.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A file cannot be read.
    #[error("failed to read a file")]
    Filesystem(#[from] olivia_filesystem::Error),
    /// A file is not the JSON it should be.
    #[error("failed to parse the JSON")]
    Json(#[source] Box<dyn std::error::Error + Send + Sync>),
    /// An image cannot be decoded.
    #[error("failed to load an image")]
    Graphics(#[from] olivia_graphics::Error),
    /// The file has a version this reader does not know.
    #[error("unknown version {0}")]
    UnknownVersion(u64),
    /// The joints do not form a skeleton.
    #[error("invalid skeleton: {0}")]
    InvalidSkeleton(String),
    /// The animation file breaks a rule of the format.
    #[error("invalid animation: {0}")]
    InvalidAnimation(String),
    /// The animation names a joint the skeleton lacks.
    #[error("the skeleton has no joint {0:?}")]
    MissingJoint(String),
}

impl Error {
    fn json(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        Self::Json(error.into())
    }
}

/// Read on its own, so that a newer file with another shape still reports its version.
#[derive(Deserialize)]
struct Header {
    version: u64,
}

/// Reads a JSON file of the version this reader knows.
fn read<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, Error> {
    let Header { version } = serde_json::from_slice(bytes).map_err(Error::json)?;
    if version != VERSION {
        return Err(Error::UnknownVersion(version));
    }
    serde_json::from_slice(bytes).map_err(Error::json)
}
