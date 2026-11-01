//! The game loop, driven by a fake clock.

use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use olivia::{Clock, Config, Frame, Game, Result, Setup, Update, run, run_with};
use tracing::span::{Attributes, Id};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::{Context, SubscriberExt};

const STEP: Duration = Duration::from_nanos(16_666_666);

thread_local! {
    static TRACE: RefCell<String> = const { RefCell::new(String::new()) };
    static FACTORS: RefCell<Vec<f32>> = const { RefCell::new(Vec::new()) };
    static ELAPSED: RefCell<Vec<f64>> = const { RefCell::new(Vec::new()) };
    static LAST_DT: Cell<f32> = const { Cell::new(0.0) };
    static LAST_FPS: Cell<f32> = const { Cell::new(0.0) };
}

fn trace(phase: char) {
    TRACE.with_borrow_mut(|t| t.push(phase));
}

/// Replays `deltas` as frame times, then repeats the last one.
struct FakeClock<'a> {
    deltas: &'a [Duration],
}

impl Clock for FakeClock<'_> {
    fn next_frame(&mut self) -> Duration {
        match self.deltas {
            [delta] => *delta,
            [delta, rest @ ..] => {
                self.deltas = rest;
                *delta
            }
            [] => Duration::ZERO,
        }
    }
}

/// Traces each phase as `L`, `U` or `D`, and quits on update number `QUIT_AT`.
struct Traced<const QUIT_AT: u32> {
    updates: u32,
}

impl<const QUIT_AT: u32> Game for Traced<QUIT_AT> {
    fn load(_s: Setup<'_>) -> Result<Self> {
        trace('L');
        Ok(Self { updates: 0 })
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        trace('U');
        self.updates += 1;
        if self.updates == QUIT_AT {
            u.quit();
        }
        Ok(())
    }

    fn draw(&self, f: &mut Frame<'_>) {
        trace('D');
        FACTORS.with_borrow_mut(|factors| factors.push(f.interpolation_factor()));
    }
}

/// Runs `G` under the fake clock and returns the phases it went through.
fn drive<G: Game>(deltas: &[Duration]) -> (Result<()>, String) {
    TRACE.take();
    let result = run_with::<G, _>(Config::new("test"), FakeClock { deltas });
    (result, TRACE.take())
}

#[test]
fn an_update_runs_before_the_first_draw_even_if_no_time_passed() {
    let (result, phases) = drive::<Traced<2>>(&[Duration::ZERO, STEP]);
    assert!(result.is_ok());
    assert_eq!(phases, "LUDU");
}

#[test]
fn updates_run_once_per_fixed_step_of_accumulated_time() {
    let half = STEP / 2;
    // Seeded with one step, then 2, 0, 1 and 1 step of accumulated time.
    let deltas = [Duration::ZERO, STEP * 2, half, half, STEP];
    let (_, phases) = drive::<Traced<5>>(&deltas);
    assert_eq!(phases, "LUDUUDDUDU");
}

#[test]
fn a_frame_with_no_step_due_draws_without_updating() {
    let (_, phases) = drive::<Traced<2>>(&[Duration::ZERO, STEP / 4, STEP / 4, STEP]);
    assert_eq!(phases, "LUDDDU");
}

#[test]
fn the_interpolation_factor_is_the_fraction_of_a_step_left_over() {
    FACTORS.take();
    let deltas = [Duration::ZERO, STEP / 4, STEP / 2, STEP / 2, STEP];
    let _ = drive::<Traced<3>>(&deltas);
    let expected = [0.0, 0.25, 0.75, 0.25];
    let factors = FACTORS.take();
    assert_eq!(factors.len(), expected.len());
    for (factor, expected) in factors.iter().zip(expected) {
        assert!((factor - expected).abs() < 1e-3, "{factors:?}");
    }
}

#[test]
fn a_frame_too_far_behind_drops_steps_and_warns() {
    let deltas = [Duration::ZERO, STEP * 100, STEP];
    let (records, (_, phases)) = recorded(|| drive::<Traced<10>>(&deltas));
    assert_eq!(phases, "LUDUUUUUUUUDU");
    assert!(records.contains(&(Level::WARN, "event")));
}

#[test]
fn a_frame_that_keeps_up_does_not_warn() {
    let (records, _) = recorded(|| drive::<Traced<9>>(&[Duration::ZERO, STEP * 8]));
    assert!(!records.iter().any(|(level, _)| *level == Level::WARN));
}

#[test]
fn load_is_an_info_span_and_update_and_draw_are_trace_spans() {
    let (records, _) = recorded(|| drive::<Traced<2>>(&[Duration::ZERO, STEP]));
    assert_eq!(
        records,
        [
            (Level::INFO, "load"),
            (Level::TRACE, "update"),
            (Level::TRACE, "draw"),
            (Level::TRACE, "update"),
        ]
    );
}

/// Records each span and event as a level and a name, `event` for events.
struct Recorder(Arc<Mutex<Vec<(Level, &'static str)>>>);

impl<S: Subscriber> Layer<S> for Recorder {
    fn on_new_span(&self, attrs: &Attributes<'_>, _id: &Id, _ctx: Context<'_, S>) {
        let meta = attrs.metadata();
        self.0.lock().unwrap().push((*meta.level(), meta.name()));
    }

    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let level = *event.metadata().level();
        self.0.lock().unwrap().push((level, "event"));
    }
}

fn recorded<T>(body: impl FnOnce() -> T) -> (Vec<(Level, &'static str)>, T) {
    let records = Arc::default();
    let subscriber = tracing_subscriber::registry().with(Recorder(Arc::clone(&records)));
    let output = tracing::subscriber::with_default(subscriber, body);
    (records.lock().unwrap().clone(), output)
}

struct FailsToLoad;

impl Game for FailsToLoad {
    fn load(_s: Setup<'_>) -> Result<Self> {
        Err("no load".into())
    }

    fn update(&mut self, _u: Update<'_>) -> Result<()> {
        trace('U');
        Ok(())
    }

    fn draw(&self, _f: &mut Frame<'_>) {
        trace('D');
    }
}

/// Fails on its second update.
struct FailsToUpdate {
    updates: u32,
}

impl Game for FailsToUpdate {
    fn load(_s: Setup<'_>) -> Result<Self> {
        Ok(Self { updates: 0 })
    }

    fn update(&mut self, _u: Update<'_>) -> Result<()> {
        trace('U');
        self.updates += 1;
        if self.updates == 2 {
            return Err("no update".into());
        }
        Ok(())
    }

    fn draw(&self, _f: &mut Frame<'_>) {
        trace('D');
    }
}

#[test]
fn an_err_from_load_returns_before_any_other_phase() {
    let (result, phases) = drive::<FailsToLoad>(&[STEP]);
    assert_eq!(result.unwrap_err().to_string(), "no load");
    assert_eq!(phases, "");
}

#[test]
fn an_err_from_update_stops_the_loop_at_once() {
    let (result, phases) = drive::<FailsToUpdate>(&[Duration::ZERO, STEP * 3]);
    assert_eq!(result.unwrap_err().to_string(), "no update");
    assert_eq!(phases, "UDU");
}

#[test]
fn quitting_stops_the_steps_left_in_a_frame() {
    let (_, phases) = drive::<Traced<2>>(&[STEP * 3]);
    assert_eq!(phases, "LUU");
}

#[test]
fn run_drives_a_game_on_the_wall_clock_until_it_quits() {
    TRACE.take();
    assert!(run::<Traced<2>>(Config::new("test")).is_ok());
    assert_eq!(TRACE.take(), "LUDU");
}

/// Records the timer, and quits on update number `QUIT_AT` or once the frame rate is known.
struct Timed<const QUIT_AT: u32> {
    updates: u32,
}

impl<const QUIT_AT: u32> Game for Timed<QUIT_AT> {
    fn load(_s: Setup<'_>) -> Result<Self> {
        Ok(Self { updates: 0 })
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        LAST_DT.set(u.timer.dt());
        LAST_FPS.set(u.timer.fps());
        ELAPSED.with_borrow_mut(|elapsed| elapsed.push(u.timer.elapsed()));
        self.updates += 1;
        if self.updates == QUIT_AT || u.timer.fps() > 0.0 {
            u.quit();
        }
        Ok(())
    }

    fn draw(&self, _f: &mut Frame<'_>) {}
}

#[test]
fn dt_is_the_fixed_step_in_seconds() {
    let _ = drive::<Timed<1>>(&[Duration::ZERO]);
    assert!((LAST_DT.get() - 1.0 / 60.0).abs() < 1e-6);
}

#[test]
fn elapsed_advances_one_fixed_step_per_update_and_skips_dropped_steps() {
    ELAPSED.take();
    let _ = drive::<Timed<12>>(&[Duration::ZERO, STEP * 20, STEP]);
    let elapsed = ELAPSED.take();
    assert_eq!(elapsed.len(), 12);
    for (update, steps) in [(0, 0.0), (1, 1.0), (8, 8.0), (11, 11.0)] {
        let seconds = steps / 60.0;
        assert!((elapsed[update] - seconds).abs() < 1e-6, "{elapsed:?}");
    }
}

#[test]
fn fps_counts_frames_over_the_last_second() {
    let _ = drive::<Timed<1000>>(&[STEP * 2]);
    assert!((LAST_FPS.get() - 30.0).abs() < 0.1, "{}", LAST_FPS.get());
}
