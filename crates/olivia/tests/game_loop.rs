//! The game loop, driven frame by frame on made-up frame times.

use std::cell::{Cell, RefCell};
use std::iter;
use std::sync::{Arc, Mutex};
use std::task::{self, Poll, Waker};
use std::time::{Duration, Instant};

use olivia::math::Vec2;
use olivia::window::{Driver, Next, Window};
use olivia::{Config, Game, GameLoop, Modules, Result, Setup, Update};
use tracing::span::{Attributes, Id};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::{Context, SubscriberExt};

const STEP: Duration = Duration::from_nanos(16_666_666);

thread_local! {
    static ELAPSED: RefCell<Vec<f64>> = const { RefCell::new(Vec::new()) };
    static LAST_DT: Cell<f32> = const { Cell::new(0.0) };
    static LAST_FRAME_RATE: Cell<f32> = const { Cell::new(0.0) };
}

/// Quits on update number `QUIT_AT`.
struct Traced<const QUIT_AT: u32> {
    updates: u32,
}

impl<const QUIT_AT: u32> Game for Traced<QUIT_AT> {
    fn load(_s: Setup<'_>) -> Result<Self> {
        Ok(Self { updates: 0 })
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        self.updates += 1;
        if self.updates == QUIT_AT {
            u.quit();
        }
        Ok(())
    }
}

/// Runs `G` on frames that take `deltas`, repeating the last one until it stops.
fn run_frames<G: Game>(deltas: &[Duration]) -> Result<()> {
    let mut window = Window::headless();
    let mut game_loop =
        GameLoop::<G>::new(&mut window, Vec2::ZERO, Modules::new(&Config::new("test"))?)?;
    let last = deltas.last().copied().unwrap_or_default();
    let mut frame_times = deltas.iter().copied().chain(iter::repeat(last));
    while game_loop.run_frame(frame_times.next().unwrap_or_default(), &mut window)? {}
    Ok(())
}

/// The phases in `records`, as `L`, `U` or `D`.
fn phases(records: &[(Level, &'static str)]) -> String {
    let phase = |(_, name): &(Level, &str)| match *name {
        "load" => Some('L'),
        "update" => Some('U'),
        "draw" => Some('D'),
        _ => None,
    };
    records.iter().filter_map(phase).collect()
}

/// Runs `G` and returns the phases it went through.
fn drive<G: Game>(deltas: &[Duration]) -> (Result<()>, String) {
    let (records, result) = recorded(|| run_frames::<G>(deltas));
    (result, phases(&records))
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
    let mut window = Window::headless();
    let modules = Modules::new(&Config::new("test")).unwrap();
    let mut game_loop = GameLoop::<Traced<3>>::new(&mut window, Vec2::ZERO, modules).unwrap();
    let deltas = [Duration::ZERO, STEP / 4, STEP / 2, STEP / 2];
    for (delta, expected) in deltas.into_iter().zip([0.0, 0.25, 0.75, 0.25]) {
        game_loop.run_frame(delta, &mut window).unwrap();
        let factor = game_loop.interpolation_factor();
        assert!((factor - expected).abs() < 1e-3, "{factor}");
    }
}

#[test]
fn a_frame_too_far_behind_drops_steps_and_warns() {
    let deltas = [Duration::ZERO, STEP * 100, STEP];
    let (records, _) = recorded(|| run_frames::<Traced<10>>(&deltas));
    assert_eq!(phases(&records), "LUDUUUUUUUUDU");
    assert!(records.contains(&(Level::WARN, "event")));
}

#[test]
fn a_frame_that_keeps_up_does_not_warn() {
    let (records, _) = recorded(|| run_frames::<Traced<9>>(&[Duration::ZERO, STEP * 8]));
    assert!(!records.iter().any(|(level, _)| *level == Level::WARN));
}

#[test]
fn load_is_an_info_span_and_update_and_draw_are_trace_spans() {
    let (records, _) = recorded(|| run_frames::<Traced<2>>(&[Duration::ZERO, STEP]));
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

/// A loaded `Traced<0>` on a headless window, whose frames are never presented.
fn start() -> (GameLoop<Traced<0>>, Window) {
    let mut window = Window::headless();
    let modules = Modules::new(&Config::new("test")).unwrap();
    let game_loop = GameLoop::new(&mut window, Vec2::ZERO, modules).unwrap();
    (game_loop, window)
}

#[test]
fn an_unpresented_frame_asks_for_the_next_one_a_fixed_step_after_it_began() {
    let (mut game_loop, mut window) = start();
    let now = Instant::now();
    let next = game_loop.run_frame_at(now, &mut window).unwrap();
    assert_eq!(next, Next::At(now + STEP));
}

#[test]
fn a_frame_that_comes_late_runs_the_steps_it_missed() {
    let (mut game_loop, mut window) = start();
    let now = Instant::now();
    game_loop.run_frame_at(now, &mut window).unwrap();
    let (records, _) = recorded(|| game_loop.run_frame_at(now + STEP * 3, &mut window));
    assert_eq!(phases(&records), "UUUD");
}

#[test]
fn a_frame_after_a_pause_restarts_the_clock_with_no_catch_up_steps() {
    let (mut game_loop, mut window) = start();
    let now = Instant::now();
    game_loop.run_frame_at(now, &mut window).unwrap();
    let after_pause = now + Duration::from_secs(5);
    let (records, _) = recorded(|| game_loop.run_frame_at(after_pause, &mut window));
    assert_eq!(records, [(Level::TRACE, "draw")]);
    let (records, _) = recorded(|| game_loop.run_frame_at(after_pause + STEP, &mut window));
    assert_eq!(phases(&records), "UD");
}

#[test]
fn a_frame_that_quits_asks_for_no_next_one() {
    let mut window = Window::headless();
    let modules = Modules::new(&Config::new("test")).unwrap();
    let mut game_loop = GameLoop::<Traced<1>>::new(&mut window, Vec2::ZERO, modules).unwrap();
    let next = game_loop.run_frame_at(Instant::now(), &mut window).unwrap();
    assert_eq!(next, Next::Stop);
}

/// Records each span and event of the loop as a level and a name, `event` for events. Leaves out
/// the `physics` span, which only exists with that feature, and `olivia_audio`, which warns at start
/// on a machine with no audio device.
struct Recorder(Arc<Mutex<Vec<(Level, &'static str)>>>);

impl<S: Subscriber> Layer<S> for Recorder {
    fn on_new_span(&self, attrs: &Attributes<'_>, _id: &Id, _ctx: Context<'_, S>) {
        let meta = attrs.metadata();
        if meta.name() == "physics" {
            return;
        }
        self.0.lock().unwrap().push((*meta.level(), meta.name()));
    }

    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let meta = event.metadata();
        if meta.target().starts_with("olivia_audio") {
            return;
        }
        self.0.lock().unwrap().push((*meta.level(), "event"));
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
        Ok(())
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
        self.updates += 1;
        if self.updates == 2 {
            return Err("no update".into());
        }
        Ok(())
    }
}

#[test]
fn an_err_from_load_returns_before_any_other_phase() {
    let (result, phases) = drive::<FailsToLoad>(&[STEP]);
    assert_eq!(result.unwrap_err().to_string(), "no load");
    assert_eq!(phases, "L");
}

#[test]
fn an_err_from_update_stops_the_loop_at_once() {
    let (result, phases) = drive::<FailsToUpdate>(&[Duration::ZERO, STEP * 3]);
    assert_eq!(result.unwrap_err().to_string(), "no update");
    assert_eq!(phases, "LUDU");
}

#[test]
fn quitting_stops_the_steps_left_in_a_frame() {
    let (_, phases) = drive::<Traced<2>>(&[STEP * 3]);
    assert_eq!(phases, "LUU");
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
        LAST_FRAME_RATE.set(u.timer.frame_rate());
        ELAPSED.with_borrow_mut(|elapsed| elapsed.push(u.timer.elapsed()));
        self.updates += 1;
        if self.updates == QUIT_AT || u.timer.frame_rate() > 0.0 {
            u.quit();
        }
        Ok(())
    }
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
fn frame_rate_counts_frames_over_the_last_second() {
    let _ = drive::<Timed<1000>>(&[STEP * 2]);
    assert!(
        (LAST_FRAME_RATE.get() - 30.0).abs() < 0.1,
        "{}",
        LAST_FRAME_RATE.get()
    );
}

#[test]
fn staying_behind_warns_once_until_the_loop_catches_up() {
    let far_behind = STEP * 20;
    let deltas = [
        Duration::ZERO,
        far_behind,
        far_behind,
        STEP,
        far_behind,
        STEP,
    ];
    let (records, _) = recorded(|| run_frames::<Traced<27>>(&deltas));
    let warnings = records.iter().filter(|(level, _)| *level == Level::WARN);
    assert_eq!(warnings.count(), 2);
}

#[test]
fn frame_rate_ignores_the_first_frame_when_no_time_has_passed() {
    let _ = drive::<Timed<1000>>(&[Duration::ZERO, STEP * 2]);
    assert!(
        (LAST_FRAME_RATE.get() - 30.0).abs() < 0.1,
        "{}",
        LAST_FRAME_RATE.get()
    );
}

#[test]
fn a_failure_is_logged_as_an_error() {
    let error = "no GPU".into();
    let (records, ()) = recorded(|| GameLoop::<Traced<0>>::fail(&error));
    assert_eq!(records, [(Level::ERROR, "event")]);
}

#[test]
fn a_loop_that_awaits_nothing_is_ready_at_once() {
    let (mut game_loop, _window) = start();
    let poll = game_loop.poll(&mut task::Context::from_waker(Waker::noop()));
    assert!(matches!(poll, Poll::Ready(Ok(()))));
}
