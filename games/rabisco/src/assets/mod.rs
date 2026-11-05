//! What the game loads from its folders, each with a default standing in for it while it is
//! missing.

mod animations;
mod characters;
mod fallback;
mod musics;
mod sounds;
mod textures;

pub(crate) use animations::Animations;
pub(crate) use characters::{Characters, ROOT_HEIGHT};
pub(crate) use musics::Musics;
pub(crate) use sounds::Sounds;
pub(crate) use textures::Textures;

use olivia::{
    Result,
    filesystem::{self, Filesystem},
};

/// Reads an asset, or warns and gives `None` if it is not there yet, so that a default can stand
/// in for it. An asset that is there but cannot be read is still an error.
fn read(filesystem: &Filesystem, path: &str) -> Result<Option<Vec<u8>>> {
    match filesystem.read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(filesystem::Error::NotFound(_)) => {
            tracing::warn!(path, "missing asset, using a default");
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}

/// The files in the folder `path`, or none if the folder is not there yet. The caller warns when
/// it finds nothing it can use.
fn list(filesystem: &Filesystem, path: &str) -> Result<Vec<String>> {
    match filesystem.list(path) {
        Ok(files) => Ok(files),
        Err(filesystem::Error::NotFound(_)) => Ok(Vec::new()),
        Err(error) => Err(error.into()),
    }
}

/// Whether `extension` names a kind of audio file the game can play, sound or music.
fn is_audio(extension: &str) -> bool {
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "wav" | "ogg" | "mp3" | "flac"
    )
}
