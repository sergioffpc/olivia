use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::Error;
use crate::filesystem::is_valid_segment;

/// Writes `assets.txt` in `dir`, the manifest of the source `dir/assets/`, which the web reads to
/// know the source's files: one path per line, sorted, `/`-separated, leaving out any name a path
/// cannot reach. With no source, the manifest is empty.
///
/// # Errors
///
/// Returns [`Error::Io`] if the source cannot be walked or the manifest cannot be written.
#[doc(hidden)]
pub fn write_manifest(dir: &Path) -> Result<(), Error> {
    let mut paths = Vec::new();
    collect(&dir.join("assets"), "", &mut paths)?;
    paths.sort();
    let manifest: String = paths.iter().flat_map(|path| [path, "\n"]).collect();
    let file = dir.join("assets.txt");
    fs::write(&file, manifest).map_err(|source| Error::Io { path: file, source })
}

/// Adds the files under `dir`, whose path is `prefix`, to `paths`.
fn collect(dir: &Path, prefix: &str, paths: &mut Vec<String>) -> Result<(), Error> {
    let io_error = |path: &Path| {
        let path = path.to_owned();
        move |source| Error::Io { path, source }
    };
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(source) if source.kind() == ErrorKind::NotFound && prefix.is_empty() => return Ok(()),
        Err(source) => return Err(io_error(dir)(source)),
    };
    for entry in entries {
        let entry = entry.map_err(io_error(dir))?;
        let name = entry.file_name();
        let Some(name) = name.to_str().filter(|name| is_valid_segment(name)) else {
            continue;
        };
        let file = entry.path();
        let metadata = fs::metadata(&file).map_err(io_error(&file))?;
        if metadata.is_dir() {
            collect(&file, &format!("{prefix}{name}/"), paths)?;
        } else if metadata.is_file() {
            paths.push(format!("{prefix}{name}"));
        }
    }
    Ok(())
}
