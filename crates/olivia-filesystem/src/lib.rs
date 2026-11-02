//! Reading a game's assets and writing its saves, through virtual relative paths.
//!
//! Paths are UTF-8 with a `/` separator on every platform. Exact case matters: it is neither
//! checked nor normalised, and Linux tells `Hero.png` from `hero.png` where macOS and Windows
//! usually do not.

mod filesystem;

use std::io;
use std::path::PathBuf;

pub use filesystem::Filesystem;

/// A failure of this module.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The path is absolute, has a `.`, `..` or empty segment, or contains a `\` or a `:`.
    #[error("invalid path {0:?}")]
    InvalidPath(String),
    /// The identity cannot name a directory.
    #[error("invalid identity {0:?}")]
    InvalidIdentity(String),
    /// The file is in neither area, or for a removal, not in the save area.
    #[error("file not found: {0}")]
    NotFound(String),
    /// The file is already in the save area.
    #[error("file already exists: {0}")]
    AlreadyExists(String),
    /// The operating system failed.
    #[error("I/O failure on {path}")]
    Io {
        /// Where it failed.
        path: PathBuf,
        /// What failed.
        #[source]
        source: io::Error,
    },
}
