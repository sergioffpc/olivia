//! What counts as a change to the source, on real file events.

use std::fs;
use std::thread;
use std::time::{Duration, Instant};

use olivia_filesystem::{Filesystem, Watcher};
use tempfile::TempDir;

/// Longer than any change takes to arrive and settle.
const WAIT: Duration = Duration::from_secs(5);
/// Long enough for a change that was going to count to have counted.
const SETTLE: Duration = Duration::from_millis(500);

/// A filesystem over a source holding `art/hero.txt`, with a watcher on it.
fn folder() -> (Filesystem, Watcher, TempDir) {
    let dir = TempDir::new().unwrap();
    let source = dir.path().join("source");
    fs::create_dir_all(source.join("art")).unwrap();
    fs::write(source.join("art/hero.txt"), b"one").unwrap();
    let filesystem = Filesystem::with_areas(source, dir.path().join("save"));
    let mut watcher = filesystem.watch().unwrap();
    // Some platforms report what happened just before the watch began.
    changes_within(&mut watcher, SETTLE);
    (filesystem, watcher, dir)
}

fn changes_within(watcher: &mut Watcher, time: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < time {
        if watcher.changed() {
            return true;
        }
        thread::sleep(Duration::from_millis(10));
    }
    false
}

#[test]
fn rewriting_a_nested_file_is_a_change_reported_once() {
    let (_filesystem, mut watcher, dir) = folder();
    fs::write(dir.path().join("source/art/hero.txt"), b"two").unwrap();
    assert!(changes_within(&mut watcher, WAIT));
    assert!(!changes_within(&mut watcher, SETTLE));
}

#[test]
fn adding_and_removing_a_file_are_changes() {
    let (_filesystem, mut watcher, dir) = folder();
    let added = dir.path().join("source/new.txt");
    fs::write(&added, b"").unwrap();
    assert!(changes_within(&mut watcher, WAIT));
    fs::remove_file(&added).unwrap();
    assert!(changes_within(&mut watcher, WAIT));
}

#[test]
fn a_file_in_a_new_directory_is_a_change() {
    let (_filesystem, mut watcher, dir) = folder();
    fs::create_dir(dir.path().join("source/fresh")).unwrap();
    assert!(changes_within(&mut watcher, WAIT));
    fs::write(dir.path().join("source/fresh/new.txt"), b"x").unwrap();
    assert!(changes_within(&mut watcher, WAIT));
}

#[test]
fn reading_is_no_change() {
    let (filesystem, mut watcher, _dir) = folder();
    filesystem.read("art/hero.txt").unwrap();
    filesystem.list("art").unwrap();
    assert!(!changes_within(&mut watcher, SETTLE));
}

#[test]
fn hidden_files_and_the_save_area_are_no_change() {
    let (mut filesystem, mut watcher, dir) = folder();
    fs::write(dir.path().join("source/art/.hero.txt.swp"), b"x").unwrap();
    filesystem.write("save.txt", b"x").unwrap();
    assert!(!changes_within(&mut watcher, SETTLE));
}

#[test]
fn a_missing_source_cannot_be_watched() {
    let dir = TempDir::new().unwrap();
    let filesystem = Filesystem::with_areas(dir.path().join("none"), dir.path().join("save"));
    assert!(filesystem.watch().is_err());
}
