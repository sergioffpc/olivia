use std::collections::BTreeMap;
use std::io::{self, ErrorKind};
use std::path::PathBuf;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use js_sys::Uint8Array;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{DomException, RequestCache, RequestInit, Response, Storage};

use crate::Error;

/// The source at the URL path `assets`, `assets/` by default, and the save area of `identity`,
/// which needs `localStorage`.
pub(crate) fn areas(identity: &str, assets: Option<PathBuf>) -> Result<(Source, Save), Error> {
    let save = Save::new(PathBuf::from(identity));
    save.storage()?;
    let source = Source::new(assets.unwrap_or_else(|| PathBuf::from("assets")));
    Ok((source, save))
}

/// The source, fetched into memory by [`preload`](Self::preload).
#[derive(Debug)]
pub(crate) struct Source {
    /// The URL path of the source, relative to the page's base URL, with no trailing `/`.
    root: String,
    files: BTreeMap<String, Vec<u8>>,
}

impl Source {
    #[expect(clippy::needless_pass_by_value, reason = "matches the desktop")]
    pub(crate) fn new(root: PathBuf) -> Self {
        Self {
            root: root.to_string_lossy().trim_end_matches('/').to_owned(),
            files: BTreeMap::new(),
        }
    }

    /// Fetches the manifest beside the source, and then every file it lists, all at once,
    /// reporting to `progress` the files fetched so far out of those listed.
    pub(crate) async fn preload(
        &mut self,
        mut progress: impl FnMut(usize, usize),
    ) -> Result<(), Error> {
        let path = format!("{}.txt", self.root);
        let manifest = fetch(&path).await?;
        let manifest = String::from_utf8(manifest).map_err(|error| Error::Io {
            path: PathBuf::from(&path),
            source: io::Error::new(ErrorKind::InvalidData, error),
        })?;
        let requests: Vec<_> = manifest
            .lines()
            .filter(|line| !line.is_empty())
            .map(|file| (file, fetch(&format!("{}/{file}", self.root))))
            .collect();
        let total = requests.len();
        progress(0, total);
        let mut files = BTreeMap::new();
        for (file, request) in requests {
            files.insert(file.to_owned(), request.await?);
            progress(files.len(), total);
        }
        self.files = files;
        Ok(())
    }

    #[expect(clippy::unnecessary_wraps, reason = "matches the desktop")]
    pub(crate) fn is_file(&self, path: &str) -> Result<bool, Error> {
        Ok(self.files.contains_key(path))
    }

    #[expect(clippy::unnecessary_wraps, reason = "matches the desktop")]
    pub(crate) fn read(&self, path: &str) -> Result<Option<Vec<u8>>, Error> {
        Ok(self.files.get(path).cloned())
    }

    #[expect(clippy::unnecessary_wraps, reason = "matches the desktop")]
    pub(crate) fn list(&self, path: &str) -> Result<Option<Vec<String>>, Error> {
        Ok(names_in(self.files.keys().map(String::as_str), path))
    }

    pub(crate) fn watch(&self) -> Result<Watcher, Error> {
        Err(Error::Io {
            path: PathBuf::from(&self.root),
            source: io::Error::new(
                ErrorKind::Unsupported,
                "the source never changes on the web",
            ),
        })
    }
}

/// Starts fetching `path`, revalidating any cached copy, and returns its bytes once they arrive.
fn fetch(path: &str) -> impl Future<Output = Result<Vec<u8>, Error>> + use<> {
    let init = RequestInit::new();
    init.set_cache(RequestCache::NoCache);
    let response =
        window().map(|window| JsFuture::from(window.fetch_with_str_and_init(path, &init)));
    let path = PathBuf::from(path);
    async move {
        let io_error = |source| Error::Io {
            path: path.clone(),
            source,
        };
        let js_error = |error| io_error(js_error(&error));
        let response: Response = response
            .map_err(io_error)?
            .await
            .map_err(js_error)?
            .unchecked_into();
        if !response.ok() {
            let status = response.status();
            let kind = if status == 404 {
                ErrorKind::NotFound
            } else {
                ErrorKind::Other
            };
            return Err(io_error(io::Error::new(
                kind,
                format!("HTTP status {status}"),
            )));
        }
        let body = JsFuture::from(response.array_buffer().map_err(js_error)?)
            .await
            .map_err(js_error)?;
        Ok(Uint8Array::new(&body).to_vec())
    }
}

/// The save area, the `localStorage` keys under `olivia/<name>/`, one per file, holding its bytes
/// in base64.
#[derive(Debug)]
pub(crate) struct Save {
    prefix: String,
}

impl Save {
    #[expect(clippy::needless_pass_by_value, reason = "matches the desktop")]
    pub(crate) fn new(name: PathBuf) -> Self {
        Self {
            prefix: format!("olivia/{}/", name.to_string_lossy()),
        }
    }

    fn storage(&self) -> Result<Storage, Error> {
        let storage = window().and_then(|window| {
            window
                .local_storage()
                .map_err(|error| js_error(&error))?
                .ok_or_else(|| io::Error::new(ErrorKind::Unsupported, "no localStorage"))
        });
        storage.map_err(|source| Error::Io {
            path: PathBuf::from(&self.prefix),
            source,
        })
    }

    fn key(&self, path: &str) -> String {
        format!("{}{path}", self.prefix)
    }

    fn get(&self, path: &str) -> Result<Option<String>, Error> {
        let key = self.key(path);
        self.storage()?
            .get_item(&key)
            .map_err(|error| io_error(key, &error))
    }

    pub(crate) fn is_file(&self, path: &str) -> Result<bool, Error> {
        Ok(self.get(path)?.is_some())
    }

    pub(crate) fn read(&self, path: &str) -> Result<Option<Vec<u8>>, Error> {
        let Some(value) = self.get(path)? else {
            return Ok(None);
        };
        STANDARD.decode(value).map(Some).map_err(|error| Error::Io {
            path: PathBuf::from(self.key(path)),
            source: io::Error::new(ErrorKind::InvalidData, error),
        })
    }

    /// Stores the file whole, or not at all.
    pub(crate) fn write(&self, path: &str, bytes: &[u8]) -> Result<(), Error> {
        let key = self.key(path);
        // A file in place of a parent, as a directory would be on disk.
        for (end, _) in path.match_indices('/') {
            if self.is_file(&path[..end])? {
                return Err(Error::Io {
                    path: PathBuf::from(key),
                    source: ErrorKind::NotADirectory.into(),
                });
            }
        }
        if self.is_file(path)? {
            return Err(Error::AlreadyExists(path.to_owned()));
        }
        self.storage()?
            .set_item(&key, &STANDARD.encode(bytes))
            .map_err(|error| io_error(key, &error))
    }

    pub(crate) fn remove(&self, path: &str) -> Result<(), Error> {
        if !self.is_file(path)? {
            return Err(Error::NotFound(path.to_owned()));
        }
        let key = self.key(path);
        self.storage()?
            .remove_item(&key)
            .map_err(|error| io_error(key, &error))
    }

    pub(crate) fn list(&self, path: &str) -> Result<Option<Vec<String>>, Error> {
        let storage = self.storage()?;
        let error = |error| io_error(self.prefix.clone(), &error);
        let mut keys = Vec::new();
        for index in 0..storage.length().map_err(error)? {
            keys.extend(storage.key(index).map_err(error)?);
        }
        let paths = keys.iter().filter_map(|key| key.strip_prefix(&self.prefix));
        Ok(names_in(paths, path))
    }
}

/// The source never changes on the web, so no watcher is ever made.
#[derive(Debug)]
pub(crate) enum Watcher {}

impl Watcher {
    pub(crate) fn changed(&mut self) -> bool {
        match *self {}
    }
}

/// The names of the files directly in the directory `dir`, among the files at `paths`, or `None`
/// if no path is inside it.
fn names_in<'a>(paths: impl Iterator<Item = &'a str>, dir: &str) -> Option<Vec<String>> {
    let prefix = format!("{dir}/");
    let mut found = false;
    let mut names = Vec::new();
    for rest in paths.filter_map(|path| path.strip_prefix(&prefix)) {
        found = true;
        if !rest.contains('/') {
            names.push(rest.to_owned());
        }
    }
    found.then_some(names)
}

fn window() -> Result<web_sys::Window, io::Error> {
    web_sys::window().ok_or_else(|| io::Error::new(ErrorKind::Unsupported, "no window"))
}

/// A failure at the `localStorage` key `key`, where a full quota is [`ErrorKind::StorageFull`].
fn io_error(key: String, error: &JsValue) -> Error {
    let is_full = error
        .dyn_ref::<DomException>()
        .is_some_and(|error| error.name() == "QuotaExceededError");
    let source = if is_full {
        io::Error::new(ErrorKind::StorageFull, "the save area is full")
    } else {
        js_error(error)
    };
    Error::Io {
        path: PathBuf::from(key),
        source,
    }
}

/// What a JavaScript exception says.
fn js_error(error: &JsValue) -> io::Error {
    let message = error.dyn_ref::<js_sys::Error>().map_or_else(
        || format!("{error:?}"),
        |error| String::from(error.message()),
    );
    io::Error::other(message)
}
