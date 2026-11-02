//! How window events reach the game, driven frame by frame.

use std::cell::RefCell;
use std::time::Duration;

use olivia::math::Vec2;
use olivia::window::{Window, WindowEvent};
use olivia::{Frame, Game, GameLoop, Result, Setup, Update};

const STEP: Duration = Duration::from_nanos(16_666_666);
const SIZE: Vec2 = Vec2::new(800.0, 600.0);

thread_local! {
    static STEPS: RefCell<Vec<Vec<WindowEvent>>> = const { RefCell::new(Vec::new()) };
    static TRACE: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Records the events of each step, traces phases as `U` and `D`, and quits on `CloseRequested`
/// if `QUITS`.
struct Recorder<const QUITS: bool>;

impl<const QUITS: bool> Game for Recorder<QUITS> {
    fn load(s: Setup<'_>) -> Result<Self> {
        s.window.set_title("recorder");
        Ok(Self)
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        u.window.set_size(SIZE);
        TRACE.with_borrow_mut(|trace| trace.push('U'));
        STEPS.with_borrow_mut(|steps| steps.push(u.events.to_vec()));
        if QUITS && u.events.contains(&WindowEvent::CloseRequested) {
            u.quit();
        }
        Ok(())
    }

    fn draw(&self, _f: &mut Frame<'_>) {
        TRACE.with_borrow_mut(|trace| trace.push('D'));
    }
}

/// A loaded game with its window and nothing recorded yet.
fn start<G: Game>() -> (GameLoop<G>, Window) {
    STEPS.take();
    TRACE.take();
    let mut window = Window::headless();
    let game_loop = GameLoop::new(&mut window, SIZE).unwrap();
    (game_loop, window)
}

#[test]
fn the_first_step_receives_the_initial_size() {
    let (mut game_loop, mut window) = start::<Recorder<false>>();
    game_loop.run_frame(Duration::ZERO, &mut window).unwrap();
    assert_eq!(STEPS.take(), [[WindowEvent::Resized(SIZE)]]);
}

#[test]
fn events_accumulate_in_order_until_the_next_step() {
    let (mut game_loop, mut window) = start::<Recorder<false>>();
    game_loop.run_frame(Duration::ZERO, &mut window).unwrap();
    STEPS.take();

    game_loop.push(WindowEvent::Focus(false));
    game_loop.run_frame(STEP / 4, &mut window).unwrap();
    assert_eq!(STEPS.take().len(), 0);

    game_loop.push(WindowEvent::CloseRequested);
    game_loop.run_frame(STEP, &mut window).unwrap();
    assert_eq!(
        STEPS.take(),
        [[WindowEvent::Focus(false), WindowEvent::CloseRequested]]
    );
}

#[test]
fn the_first_step_of_a_frame_consumes_the_events() {
    let (mut game_loop, mut window) = start::<Recorder<false>>();
    game_loop.run_frame(Duration::ZERO, &mut window).unwrap();
    STEPS.take();

    game_loop.push(WindowEvent::Focus(true));
    game_loop.run_frame(STEP * 3, &mut window).unwrap();
    assert_eq!(
        STEPS.take(),
        [vec![WindowEvent::Focus(true)], vec![], vec![]]
    );
}

#[test]
fn close_requested_does_not_quit_by_itself() {
    let (mut game_loop, mut window) = start::<Recorder<false>>();
    game_loop.push(WindowEvent::CloseRequested);
    assert!(game_loop.run_frame(STEP, &mut window).unwrap());
}

#[test]
fn the_game_quits_by_calling_quit_on_close_requested() {
    let (mut game_loop, mut window) = start::<Recorder<true>>();
    game_loop.push(WindowEvent::CloseRequested);
    assert!(!game_loop.run_frame(STEP, &mut window).unwrap());
}

#[test]
fn while_minimised_draw_is_skipped_and_update_keeps_running() {
    let (mut game_loop, mut window) = start::<Recorder<false>>();
    game_loop.run_frame(Duration::ZERO, &mut window).unwrap();
    TRACE.take();

    game_loop.push(WindowEvent::Minimized(true));
    game_loop.run_frame(STEP, &mut window).unwrap();
    game_loop.run_frame(STEP, &mut window).unwrap();
    assert_eq!(TRACE.take(), "UU");

    game_loop.push(WindowEvent::Minimized(false));
    game_loop.run_frame(STEP, &mut window).unwrap();
    assert_eq!(TRACE.take(), "UD");
}
