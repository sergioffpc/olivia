//! How the source and the save area combine, on temporary directories.

use std::fs;
use std::path::Path;

use olivia_filesystem::{Error, Filesystem};
use tempfile::TempDir;

/// A filesystem over two temporary areas, and those areas.
struct Areas {
    fs: Filesystem,
    source: TempDir,
    save: TempDir,
}

fn areas() -> Areas {
    let source = TempDir::new().unwrap();
    let save = TempDir::new().unwrap();
    let fs = Filesystem::with_areas(source.path().to_owned(), save.path().to_owned());
    Areas { fs, source, save }
}

fn put(root: &Path, path: &str, bytes: &[u8]) {
    let file = root.join(path);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, bytes).unwrap();
}

#[test]
fn read_finds_a_file_in_the_source() {
    let a = areas();
    put(a.source.path(), "maps/one.txt", b"source");
    assert_eq!(a.fs.read("maps/one.txt").unwrap(), b"source");
}

#[test]
fn read_prefers_the_save_area() {
    let a = areas();
    put(a.source.path(), "one.txt", b"source");
    put(a.save.path(), "one.txt", b"save");
    assert_eq!(a.fs.read("one.txt").unwrap(), b"save");
}

#[test]
fn read_of_a_file_in_neither_area_is_not_found() {
    let a = areas();
    assert!(matches!(a.fs.read("one.txt"), Err(Error::NotFound(_))));
}

#[test]
fn write_goes_only_to_the_save_area_and_creates_parents() {
    let mut a = areas();
    a.fs.write("saves/slot/one.txt", b"save").unwrap();
    assert_eq!(
        fs::read(a.save.path().join("saves/slot/one.txt")).unwrap(),
        b"save"
    );
    assert!(!a.source.path().join("saves").exists());
}

#[test]
fn write_never_overwrites() {
    let mut a = areas();
    a.fs.write("one.txt", b"first").unwrap();
    let result = a.fs.write("one.txt", b"second");
    assert!(matches!(result, Err(Error::AlreadyExists(_))));
    assert_eq!(a.fs.read("one.txt").unwrap(), b"first");
}

#[test]
fn write_can_shadow_a_source_file() {
    let mut a = areas();
    put(a.source.path(), "one.txt", b"source");
    a.fs.write("one.txt", b"save").unwrap();
    assert_eq!(a.fs.read("one.txt").unwrap(), b"save");
    assert_eq!(
        fs::read(a.source.path().join("one.txt")).unwrap(),
        b"source"
    );
}

#[test]
fn write_creates_the_save_area_on_first_use() {
    let source = TempDir::new().unwrap();
    let root = TempDir::new().unwrap();
    let save = root.path().join("game");
    let mut fs = Filesystem::with_areas(source.path().to_owned(), save.clone());
    assert!(!save.exists());
    fs.write("one.txt", b"save").unwrap();
    assert!(save.join("one.txt").exists());
}

#[test]
fn a_failed_write_reports_the_os_error() {
    let mut a = areas();
    a.fs.write("one.txt", b"file").unwrap();
    let result = a.fs.write("one.txt/two.txt", b"nested");
    assert!(matches!(result, Err(Error::Io { .. })));
}

#[test]
fn exists_looks_in_both_areas() {
    let a = areas();
    put(a.source.path(), "one.txt", b"source");
    put(a.save.path(), "two.txt", b"save");
    assert!(a.fs.exists("one.txt").unwrap());
    assert!(a.fs.exists("two.txt").unwrap());
    assert!(!a.fs.exists("three.txt").unwrap());
}

#[test]
fn a_directory_is_not_a_file() {
    let a = areas();
    put(a.source.path(), "maps/one.txt", b"source");
    assert!(!a.fs.exists("maps").unwrap());
    assert!(matches!(a.fs.read("maps"), Err(Error::NotFound(_))));
}

#[test]
fn a_file_in_place_of_a_parent_hides_nothing_behind_it() {
    let a = areas();
    put(a.save.path(), "maps", b"save");
    put(a.source.path(), "maps/one.txt", b"source");
    assert_eq!(a.fs.read("maps/one.txt").unwrap(), b"source");
}

#[test]
fn remove_deletes_from_the_save_area_and_uncovers_the_source() {
    let mut a = areas();
    put(a.source.path(), "one.txt", b"source");
    a.fs.write("one.txt", b"save").unwrap();
    a.fs.remove("one.txt").unwrap();
    assert_eq!(a.fs.read("one.txt").unwrap(), b"source");
}

#[test]
fn remove_never_touches_the_source() {
    let mut a = areas();
    put(a.source.path(), "one.txt", b"source");
    assert!(matches!(a.fs.remove("one.txt"), Err(Error::NotFound(_))));
    assert!(a.source.path().join("one.txt").exists());
}

#[test]
fn every_operation_rejects_an_invalid_path() {
    let mut a = areas();
    let invalid = [
        "",
        "/etc/passwd",
        "../outside.txt",
        "maps/../../outside.txt",
        "..",
        "maps//one.txt",
        "maps/",
        "./one.txt",
        "C:/one.txt",
        "maps\\one.txt",
        "..\\outside.txt",
    ];
    for path in invalid {
        assert!(
            matches!(a.fs.read(path), Err(Error::InvalidPath(_))),
            "{path}"
        );
        assert!(
            matches!(a.fs.exists(path), Err(Error::InvalidPath(_))),
            "{path}"
        );
        assert!(
            matches!(a.fs.write(path, b"x"), Err(Error::InvalidPath(_))),
            "{path}"
        );
        assert!(
            matches!(a.fs.remove(path), Err(Error::InvalidPath(_))),
            "{path}"
        );
    }
}

#[test]
fn an_identity_that_cannot_name_a_directory_is_invalid() {
    for identity in ["", " ", " game", "game ", ".", "..", "a/b", "a\\b", "a:b"] {
        let result = Filesystem::new(identity, None);
        assert!(
            matches!(result, Err(Error::InvalidIdentity(_))),
            "{identity}"
        );
    }
}

#[test]
fn a_valid_identity_starts_with_an_empty_save_area() {
    let source = TempDir::new().unwrap();
    let fs = Filesystem::new("olivia-test-untouched", Some(source.path().to_owned())).unwrap();
    assert!(!fs.exists("one.txt").unwrap());
}

#[test]
fn the_assets_override_is_the_source() {
    let source = TempDir::new().unwrap();
    put(source.path(), "one.txt", b"source");
    let fs = Filesystem::new("olivia-test-override", Some(source.path().to_owned())).unwrap();
    assert_eq!(fs.read("one.txt").unwrap(), b"source");
}
