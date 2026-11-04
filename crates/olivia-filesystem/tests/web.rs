//! How the source and the save area combine on the web, in a browser, over the fixture in
//! `tests/fixtures/` and the page's `localStorage`.

#![cfg(target_arch = "wasm32")]

use std::io::ErrorKind;

use olivia_filesystem::{Error, Filesystem};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(inline_js = "
    let original;
    export function block_local_storage() {
        original = Object.getOwnPropertyDescriptor(window, 'localStorage');
        Object.defineProperty(window, 'localStorage', {
            configurable: true,
            get() { throw new DOMException('site data is blocked', 'SecurityError'); },
        });
    }
    export function unblock_local_storage() {
        Object.defineProperty(window, 'localStorage', original);
    }
")]
extern "C" {
    fn block_local_storage();
    fn unblock_local_storage();
}

/// Removes every save of the game named `identity`, left by an earlier run.
fn clear(identity: &str) {
    let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    let prefix = format!("olivia/{identity}/");
    let keys: Vec<String> = (0..storage.length().unwrap())
        .filter_map(|index| storage.key(index).unwrap())
        .filter(|key| key.starts_with(&prefix))
        .collect();
    for key in keys {
        storage.remove_item(&key).unwrap();
    }
}

/// The filesystem of the game named `identity`, with no saves, over the source `assets`.
fn filesystem(identity: &str, assets: &str) -> Filesystem {
    clear(identity);
    Filesystem::new(identity, Some(assets.into())).unwrap()
}

/// The filesystem of the game named `identity`, with no saves, over the preloaded fixture.
async fn preloaded(identity: &str) -> Filesystem {
    let mut fs = filesystem(identity, "tests/fixtures/assets");
    fs.preload(|_, _| {}).await.unwrap();
    fs
}

#[wasm_bindgen_test]
async fn read_finds_a_file_in_the_source() {
    let fs = preloaded("read-source").await;
    assert_eq!(fs.read("maps/one.txt").unwrap(), b"one\n");
}

#[wasm_bindgen_test]
async fn read_prefers_the_save_area() {
    let mut fs = preloaded("read-save").await;
    fs.write("hero.txt", b"save").unwrap();
    assert_eq!(fs.read("hero.txt").unwrap(), b"save");
}

#[wasm_bindgen_test]
async fn read_of_a_file_in_neither_area_is_not_found() {
    let fs = preloaded("read-neither").await;
    assert!(matches!(fs.read("three.txt"), Err(Error::NotFound(_))));
}

#[wasm_bindgen_test]
fn write_goes_to_local_storage_and_reads_back() {
    let mut fs = filesystem("write", "nowhere");
    fs.write("saves/slot/one.txt", &[0, 159, 255]).unwrap();
    assert_eq!(fs.read("saves/slot/one.txt").unwrap(), [0, 159, 255]);
    let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    assert_eq!(
        storage.get_item("olivia/write/saves/slot/one.txt").unwrap(),
        Some("AJ//".to_owned())
    );
}

#[wasm_bindgen_test]
fn write_never_overwrites() {
    let mut fs = filesystem("write-twice", "nowhere");
    fs.write("one.txt", b"first").unwrap();
    let result = fs.write("one.txt", b"second");
    assert!(matches!(result, Err(Error::AlreadyExists(_))));
    assert_eq!(fs.read("one.txt").unwrap(), b"first");
}

#[wasm_bindgen_test]
fn a_failed_write_reports_the_error() {
    let mut fs = filesystem("write-under-file", "nowhere");
    fs.write("one.txt", b"file").unwrap();
    let result = fs.write("one.txt/two.txt", b"nested");
    assert!(matches!(result, Err(Error::Io { .. })));
}

#[wasm_bindgen_test]
fn a_full_save_area_is_storage_full_and_stores_nothing() {
    let mut fs = filesystem("full", "nowhere");
    let result = fs.write("big.bin", &vec![0; 16 << 20]);
    let Err(Error::Io { source, .. }) = result else {
        panic!("{result:?}");
    };
    assert_eq!(source.kind(), ErrorKind::StorageFull);
    assert!(!fs.exists("big.bin").unwrap());
}

#[wasm_bindgen_test]
fn no_local_storage_is_a_start_failure() {
    block_local_storage();
    let result = Filesystem::new("blocked", None);
    unblock_local_storage();
    assert!(matches!(result, Err(Error::Io { .. })), "{result:?}");
}

#[wasm_bindgen_test]
async fn exists_looks_in_both_areas() {
    let mut fs = preloaded("exists").await;
    fs.write("two.txt", b"save").unwrap();
    assert!(fs.exists("hero.txt").unwrap());
    assert!(fs.exists("two.txt").unwrap());
    assert!(!fs.exists("three.txt").unwrap());
    assert!(!fs.exists("maps").unwrap());
}

#[wasm_bindgen_test]
async fn remove_deletes_from_the_save_area_and_uncovers_the_source() {
    let mut fs = preloaded("remove").await;
    fs.write("hero.txt", b"save").unwrap();
    fs.remove("hero.txt").unwrap();
    assert_eq!(fs.read("hero.txt").unwrap(), b"hero\n");
}

#[wasm_bindgen_test]
async fn remove_never_touches_the_source() {
    let mut fs = preloaded("remove-source").await;
    assert!(matches!(fs.remove("hero.txt"), Err(Error::NotFound(_))));
    assert!(fs.exists("hero.txt").unwrap());
}

#[wasm_bindgen_test]
async fn list_merges_both_areas_in_order_with_a_shadowed_file_once() {
    let mut fs = preloaded("list").await;
    fs.write("maps/a.txt", b"save").unwrap();
    fs.write("maps/two.txt", b"save").unwrap();
    fs.write("maps/old/three.txt", b"save").unwrap();
    let files = fs.list("maps").unwrap();
    assert_eq!(files, ["maps/a.txt", "maps/one.txt", "maps/two.txt"]);
    assert_eq!(fs.read(&files[2]).unwrap(), b"save");
}

#[wasm_bindgen_test]
async fn list_of_a_directory_in_neither_area_is_not_found() {
    let fs = preloaded("list-neither").await;
    assert!(matches!(fs.list("sounds"), Err(Error::NotFound(_))));
    assert!(matches!(fs.list("hero.txt"), Err(Error::NotFound(_))));
}

#[wasm_bindgen_test]
async fn preload_reports_the_files_fetched_out_of_the_manifest() {
    let mut fs = filesystem("progress", "tests/fixtures/assets");
    let mut progress = Vec::new();
    fs.preload(|fetched, total| progress.push((fetched, total)))
        .await
        .unwrap();
    assert_eq!(progress, [(0, 3), (1, 3), (2, 3), (3, 3)]);
}

#[wasm_bindgen_test]
async fn a_listed_but_missing_file_is_a_start_failure() {
    let mut fs = filesystem("broken", "tests/fixtures/broken");
    let result = fs.preload(|_, _| {}).await;
    let Err(Error::Io { path, source }) = result else {
        panic!("{result:?}");
    };
    assert_eq!(path.to_str(), Some("tests/fixtures/broken/missing.txt"));
    assert_eq!(source.to_string(), "HTTP status 404");
}

#[wasm_bindgen_test]
async fn a_missing_manifest_is_a_start_failure() {
    let mut fs = filesystem("no-manifest", "tests/fixtures/nowhere");
    let result = fs.preload(|_, _| {}).await;
    let Err(Error::Io { path, .. }) = result else {
        panic!("{result:?}");
    };
    assert_eq!(path.to_str(), Some("tests/fixtures/nowhere.txt"));
}

#[wasm_bindgen_test]
async fn watch_is_unsupported() {
    let fs = preloaded("watch").await;
    let result = fs.watch();
    let Err(Error::Io { source, .. }) = result else {
        panic!("the web source cannot be watched");
    };
    assert_eq!(source.kind(), ErrorKind::Unsupported);
}
