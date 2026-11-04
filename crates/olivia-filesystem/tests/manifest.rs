//! What `olivia-manifest` writes beside a source.

use std::fs;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn olivia_manifest(dir: &Path) {
    let status = Command::new(env!("CARGO_BIN_EXE_olivia-manifest"))
        .arg(dir)
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn the_manifest_of_the_fixture_is_the_committed_one() {
    let dir = TempDir::new().unwrap();
    copy_dir(
        &Path::new(FIXTURES).join("assets"),
        &dir.path().join("assets"),
    );
    olivia_manifest(dir.path());
    assert_eq!(
        fs::read_to_string(dir.path().join("assets.txt")).unwrap(),
        fs::read_to_string(Path::new(FIXTURES).join("assets.txt")).unwrap()
    );
}

#[test]
fn a_game_with_no_source_gets_an_empty_manifest() {
    let dir = TempDir::new().unwrap();
    olivia_manifest(dir.path());
    assert_eq!(
        fs::read_to_string(dir.path().join("assets.txt")).unwrap(),
        ""
    );
}

fn put(dir: &Path, paths: &[&str]) {
    for path in paths {
        let file = dir.join("assets").join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, b"source").unwrap();
    }
}

#[test]
fn the_manifest_lists_every_file_sorted_and_no_directory() {
    let dir = TempDir::new().unwrap();
    put(dir.path(), &["b.txt", "a/z.txt", "a/b/c.txt"]);
    fs::create_dir_all(dir.path().join("assets/empty")).unwrap();
    olivia_manifest(dir.path());
    assert_eq!(
        fs::read_to_string(dir.path().join("assets.txt")).unwrap(),
        "a/b/c.txt\na/z.txt\nb.txt\n"
    );
}

#[cfg(unix)]
#[test]
fn the_manifest_leaves_out_names_a_path_cannot_reach() {
    let dir = TempDir::new().unwrap();
    put(
        dir.path(),
        &["one.txt", "a:b.txt", "a\\b.txt", "c:d/two.txt"],
    );
    olivia_manifest(dir.path());
    assert_eq!(
        fs::read_to_string(dir.path().join("assets.txt")).unwrap(),
        "one.txt\n"
    );
}
