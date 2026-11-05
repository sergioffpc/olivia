//! Restarting a running game when its source changes, on real file events.
#![cfg(feature = "filesystem")]

use std::cell::{Cell, RefCell};
use std::fs;
use std::thread;
use std::time::{Duration, Instant};

use olivia::math::Vec2;
use olivia::window::{Window, WindowEvent};
use olivia::{Config, Game, GameLoop, Modules, Result, Setup, Update};
use tempfile::TempDir;

/// Longer than any change takes to arrive and settle.
const WAIT: Duration = Duration::from_secs(5);
/// Long enough for a change that was going to count to have counted.
const SETTLE: Duration = Duration::from_millis(500);

thread_local! {
    static LOADS: Cell<u32> = const { Cell::new(0) };
    static RESIZES: RefCell<Vec<Vec2>> = const { RefCell::new(Vec::new()) };
}

/// A marker on the entity each load spawns.
struct CLoaded;

/// Holds the text of `one.txt`, and refuses to load if it says `broken`.
struct Text {
    text: String,
}

impl Game for Text {
    fn load(s: Setup<'_>) -> Result<Self> {
        LOADS.set(LOADS.get() + 1);
        let text = String::from_utf8(s.filesystem.read("one.txt")?)?;
        if text == "broken" {
            return Err("broken".into());
        }
        s.world.spawn((CLoaded,));
        Ok(Self { text })
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        for event in u.events {
            if let WindowEvent::Resized(size) = event {
                RESIZES.with_borrow_mut(|resizes| resizes.push(*size));
            }
        }
        Ok(())
    }
}

struct Running {
    game_loop: GameLoop<Text>,
    window: Window,
    assets: TempDir,
}

impl Running {
    fn new(hot_reload: bool) -> Self {
        let assets = TempDir::new().unwrap();
        fs::write(assets.path().join("one.txt"), b"first").unwrap();
        let config = Config::new("olivia-test")
            .assets(assets.path())
            .hot_reload(hot_reload);
        let mut window = Window::headless();
        let modules = Modules::new(&config).unwrap();
        let game_loop = GameLoop::new(&mut window, Vec2::new(4.0, 3.0), modules).unwrap();
        let mut running = Self {
            game_loop,
            window,
            assets,
        };
        // Some platforms report what happened just before the watch began.
        running.run_for(SETTLE);
        LOADS.set(0);
        running
    }

    fn write(&self, text: &str) {
        fs::write(self.assets.path().join("one.txt"), text).unwrap();
    }

    fn run_for(&mut self, time: Duration) {
        let start = Instant::now();
        while start.elapsed() < time {
            self.frame();
        }
    }

    fn frame(&mut self) {
        self.game_loop
            .run_frame(Duration::from_millis(10), &mut self.window)
            .unwrap();
        thread::sleep(Duration::from_millis(10));
    }

    /// Runs until the game has been loaded `count` times since it started.
    fn run_until_loaded(&mut self, count: u32) {
        let start = Instant::now();
        while LOADS.get() < count {
            assert!(start.elapsed() < WAIT, "the game did not reload");
            self.frame();
        }
    }

    fn loaded(&mut self) -> usize {
        self.game_loop.world().query::<&CLoaded>().iter().count()
    }

    fn text(&self) -> &str {
        &self.game_loop.game().text
    }
}

#[test]
fn a_changed_file_replaces_the_game_and_empties_the_world() {
    let mut running = Running::new(true);
    running.write("second");
    running.run_until_loaded(1);
    assert_eq!(running.text(), "second");
    assert_eq!(running.loaded(), 1);
}

#[test]
fn a_reload_reading_the_source_does_not_cause_another() {
    let mut running = Running::new(true);
    running.write("second");
    running.run_until_loaded(1);
    running.run_for(SETTLE);
    assert_eq!(LOADS.get(), 1);
}

#[test]
fn a_failed_load_keeps_the_game_until_the_next_change() {
    let mut running = Running::new(true);
    running.write("broken");
    running.run_until_loaded(1);
    assert_eq!(running.text(), "first");
    assert_eq!(running.loaded(), 1);

    running.write("second");
    running.run_until_loaded(2);
    assert_eq!(running.text(), "second");
    assert_eq!(running.loaded(), 1);
}

#[test]
fn a_reloaded_game_hears_the_size_of_the_window() {
    let mut running = Running::new(true);
    running
        .game_loop
        .push(WindowEvent::Resized(Vec2::new(8.0, 6.0)));
    running.frame();
    RESIZES.take();

    running.write("second");
    running.run_until_loaded(1);
    running.frame();
    assert_eq!(RESIZES.take().last(), Some(&Vec2::new(8.0, 6.0)));
}

#[test]
fn with_hot_reload_off_the_game_ignores_the_source() {
    let mut running = Running::new(false);
    running.write("second");
    running.run_for(SETTLE);
    assert_eq!(running.text(), "first");
}
