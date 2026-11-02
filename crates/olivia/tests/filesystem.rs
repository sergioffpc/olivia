//! How the filesystem reaches the game, driven frame by frame.
#![cfg(feature = "filesystem")]

use std::cell::RefCell;
use std::fs;
use std::time::Duration;

use olivia::filesystem::Error;
use olivia::math::Vec2;
use olivia::window::Window;
use olivia::{Config, Frame, Game, GameLoop, Modules, Result, Setup, Update, run};
use tempfile::TempDir;

thread_local! {
    static READ: RefCell<Vec<Vec<u8>>> = const { RefCell::new(Vec::new()) };
}

/// Reads `one.txt` in each phase that can.
struct Reader;

impl Game for Reader {
    fn load(s: Setup<'_>) -> Result<Self> {
        READ.with_borrow_mut(|read| read.push(s.filesystem.read("one.txt").unwrap()));
        Ok(Self)
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        READ.with_borrow_mut(|read| read.push(u.filesystem.read("one.txt").unwrap()));
        Ok(())
    }

    fn draw(&self, _f: &mut Frame<'_>) {}
}

#[test]
fn load_and_update_read_from_the_configured_assets() {
    let assets = TempDir::new().unwrap();
    fs::write(assets.path().join("one.txt"), b"asset").unwrap();
    let config = Config::new("olivia-test").assets(assets.path());

    let mut window = Window::headless();
    let modules = Modules::new(&config).unwrap();
    let mut game_loop = GameLoop::<Reader>::new(&mut window, Vec2::ZERO, modules).unwrap();
    game_loop.run_frame(Duration::ZERO, &mut window).unwrap();

    assert_eq!(READ.take(), [b"asset", b"asset"]);
}

/// Reads the asset the crate ships, from the default source.
struct Greeter;

impl Game for Greeter {
    fn load(s: Setup<'_>) -> Result<Self> {
        READ.with_borrow_mut(|read| read.push(s.filesystem.read("greeting.txt").unwrap()));
        Ok(Self)
    }

    fn update(&mut self, _u: Update<'_>) -> Result<()> {
        Ok(())
    }

    fn draw(&self, _f: &mut Frame<'_>) {}
}

#[test]
fn under_cargo_the_source_defaults_to_assets_at_the_project_root() {
    let mut window = Window::headless();
    let modules = Modules::new(&Config::new("olivia-test")).unwrap();
    GameLoop::<Greeter>::new(&mut window, Vec2::ZERO, modules).unwrap();
    assert_eq!(READ.take(), [b"Hello from the source.\n"]);
}

#[test]
fn an_invalid_identity_comes_out_of_run_before_any_window_opens() {
    let error = run::<Reader>(Config::new("not/valid")).unwrap_err();
    assert!(matches!(
        error.downcast_ref::<Error>(),
        Some(Error::InvalidIdentity(_))
    ));
}
