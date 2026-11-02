//! How input reaches the game, driven frame by frame.
#![cfg(feature = "input")]

use std::cell::RefCell;
use std::time::Duration;

use olivia::input::Key;
use olivia::math::Vec2;
use olivia::window::Window;
use olivia::window::winit::event::ElementState::{Pressed, Released};
use olivia::window::winit::keyboard::{KeyCode, PhysicalKey};
use olivia::{Config, Frame, Game, GameLoop, Modules, Result, Setup, Update};

const STEP: Duration = Duration::from_nanos(16_666_666);
const W: PhysicalKey = PhysicalKey::Code(KeyCode::KeyW);

thread_local! {
    /// Whether W was pressed, released and down, with the typed text, at each step.
    static STEPS: RefCell<Vec<(bool, bool, bool, String)>> = const { RefCell::new(Vec::new()) };
}

struct Recorder;

impl Game for Recorder {
    fn load(_s: Setup<'_>) -> Result<Self> {
        Ok(Self)
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        let step = (
            u.input.pressed(Key::W),
            u.input.released(Key::W),
            u.input.down(Key::W),
            u.input.text().to_owned(),
        );
        STEPS.with_borrow_mut(|steps| steps.push(step));
        Ok(())
    }

    fn draw(&self, _f: &mut Frame<'_>) {}
}

/// A game past its first frame, with nothing recorded yet.
fn start() -> (GameLoop<Recorder>, Window) {
    let mut window = Window::headless();
    let modules = Modules::new(&Config::new("test")).unwrap();
    let mut game_loop = GameLoop::new(&mut window, Vec2::ZERO, modules).unwrap();
    game_loop.run_frame(Duration::ZERO, &mut window).unwrap();
    STEPS.take();
    (game_loop, window)
}

#[test]
fn a_tap_between_two_steps_reaches_the_next_one() {
    let (mut game_loop, mut window) = start();
    game_loop.input().on_key(W, Pressed, Some("w"));
    game_loop.input().on_key(W, Released, None);
    game_loop.run_frame(STEP, &mut window).unwrap();
    assert_eq!(STEPS.take(), [(true, true, false, "w".to_owned())]);
}

#[test]
fn a_frame_with_no_step_keeps_the_input_for_the_next_frame() {
    let (mut game_loop, mut window) = start();
    game_loop.input().on_key(W, Pressed, Some("w"));
    game_loop.run_frame(STEP / 4, &mut window).unwrap();
    assert_eq!(STEPS.take(), []);

    game_loop.run_frame(STEP, &mut window).unwrap();
    assert_eq!(STEPS.take(), [(true, false, true, "w".to_owned())]);
}

#[test]
fn the_first_step_of_a_frame_consumes_the_input() {
    let (mut game_loop, mut window) = start();
    game_loop.input().on_key(W, Pressed, Some("w"));
    game_loop.run_frame(STEP * 2, &mut window).unwrap();
    assert_eq!(
        STEPS.take(),
        [
            (true, false, true, "w".to_owned()),
            (false, false, true, String::new())
        ]
    );
}
