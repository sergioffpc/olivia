//! The whole game in a headless loop with no GPU: real assets, real physics, made-up keys.

use std::path::Path;
use std::time::Duration;

use astronaut_cat::level::{CHECKPOINTS, GROUND, PLATFORMS, ROOT_HEIGHT};
use astronaut_cat::{CEnemy, CHero, Phase, Platformer, CStar, CStatus};
use olivia::ecs::CTransform;
use olivia::math::Vec2;
use olivia::window::Window;
use olivia::window::raw::ElementState::{Pressed, Released};
use olivia::window::raw::{KeyCode, PhysicalKey};
use olivia::{GameLoop, Modules};

const STEP: Duration = Duration::from_nanos(16_666_667);

struct Play {
    game: GameLoop<Platformer>,
    window: Window,
}

impl Play {
    fn new() -> Self {
        let mut window = Window::headless();
        let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        let modules = Modules::new(&Platformer::config().assets(assets)).unwrap();
        let game = GameLoop::new(&mut window, Vec2::new(1280.0, 720.0), modules).unwrap();
        let mut play = Self { game, window };
        play.run(2);
        play
    }

    fn run(&mut self, steps: u32) {
        for _ in 0..steps {
            self.game.run_frame(STEP, &mut self.window).unwrap();
        }
    }

    fn key(&mut self, code: KeyCode, is_down: bool) {
        let state = if is_down { Pressed } else { Released };
        let key = PhysicalKey::Code(code);
        self.game.input().on_key(key, state, false, None);
    }

    fn tap(&mut self, code: KeyCode) {
        self.key(code, true);
        self.run(1);
        self.key(code, false);
    }

    fn status(&mut self) -> CStatus {
        *self
            .game
            .world()
            .query_mut::<&CStatus>()
            .into_iter()
            .next()
            .unwrap()
    }

    fn hero(&mut self) -> (CHero, Vec2) {
        let world = self.game.world();
        let (hero, transform) = world
            .query_mut::<(&CHero, &CTransform)>()
            .into_iter()
            .next()
            .unwrap();
        (*hero, transform.position + Vec2::new(0.0, ROOT_HEIGHT))
    }

    fn feet(&mut self) -> Vec2 {
        self.hero().1
    }

    fn enemies(&mut self) -> usize {
        self.game.world().query_mut::<&CEnemy>().into_iter().count()
    }
}

#[test]
fn shooting_stars_fall_now_and_then_and_go() {
    let mut play = Play::new();
    let mut heights = Vec::new();
    let mut falls = 0;
    // Twenty seconds.
    for _ in 0..1200 {
        play.run(1);
        let star = play.game.world().query_mut::<&CTransform>().with::<&CStar>();
        let height = star
            .into_iter()
            .next()
            .map(|transform| transform.position.y);
        if let Some(height) = height {
            if heights.last().is_none_or(|last| height < *last) {
                falls += 1;
            }
            heights.push(height);
        } else {
            heights.clear();
        }
    }
    assert!(falls >= 2, "only {falls} shooting stars in twenty seconds");
}

#[test]
fn the_hero_stands_on_the_ground() {
    let mut play = Play::new();
    play.run(60);
    let feet = play.feet();
    assert!((feet.y - GROUND).abs() < 0.03, "feet at {feet}");
    assert!(play.hero().0.is_grounded);
}

#[test]
fn a_jump_leaves_the_ground_and_comes_back() {
    let mut play = Play::new();
    play.run(30);
    play.tap(KeyCode::Space);
    play.run(20);
    assert!(play.feet().y < GROUND - 1.0, "feet at {}", play.feet());
    play.run(90);
    assert!(play.hero().0.is_grounded);
}

#[test]
fn the_extra_jump_goes_higher_and_only_once() {
    let mut play = Play::new();
    play.run(30);
    play.tap(KeyCode::Space);
    play.run(25);
    play.tap(KeyCode::Space);
    let mut highest = GROUND;
    for _ in 0..25 {
        play.run(1);
        highest = highest.min(play.feet().y);
    }
    assert!(highest < GROUND - 2.2, "only reached {highest}");

    play.tap(KeyCode::Space);
    play.run(1);
    let falling = play.feet().y;
    play.run(5);
    assert!(play.feet().y >= falling, "a third jump lifted the hero");
}

#[test]
fn a_wall_of_ground_stops_nobody_walking_on_top_of_it() {
    let mut play = Play::new();
    play.run(30);
    play.key(KeyCode::ArrowRight, true);
    play.run(60);
    assert!(play.feet().x > 3.5, "stuck at {}", play.feet());
    assert!((play.feet().y - GROUND).abs() < 0.03);
}

#[test]
fn walking_into_the_enemies_ends_the_game() {
    let mut play = Play::new();
    play.key(KeyCode::ArrowRight, true);
    play.run(3000);
    let status = play.status();
    assert_eq!(status.phase, Phase::Lost);
    assert_eq!(status.lives, 0);
}

#[test]
fn enter_starts_a_lost_game_over() {
    let mut play = Play::new();
    play.key(KeyCode::ArrowRight, true);
    play.run(3000);
    play.key(KeyCode::ArrowRight, false);
    play.tap(KeyCode::Enter);
    play.run(3);
    let status = play.status();
    assert_eq!(
        (status.phase, status.lives, status.score),
        (Phase::Playing, 3, 0)
    );
    assert_eq!(play.enemies(), 7);
    assert!(play.feet().x < 2.0);
}

#[test]
fn a_shot_leaves_the_launcher_and_kills_what_it_meets() {
    let mut play = Play::new();
    play.run(30);
    let before = play.enemies();
    play.key(KeyCode::ArrowRight, true);
    for _ in 0..600 {
        play.tap(KeyCode::KeyX);
        play.run(5);
        if play.enemies() < before {
            return;
        }
    }
    panic!("no shot hit anything");
}

#[test]
fn a_fall_into_a_pit_costs_a_life_and_starts_at_a_checkpoint() {
    let mut play = Play::new();
    play.run(30);
    play.key(KeyCode::ArrowRight, true);
    let mut has_shot = false;
    for step in 0..360u32 {
        // Clears the way so that only the pit can cost the life.
        if step.is_multiple_of(30) {
            play.tap(KeyCode::KeyX);
            has_shot = true;
        } else {
            play.run(1);
        }
        if play.status().lives < 3 {
            break;
        }
    }
    assert!(has_shot);
    assert!(play.enemies() < 7, "the shots flew over the crawler");
    play.key(KeyCode::ArrowRight, false);
    play.run(30);
    assert_eq!(play.status().lives, 2);
    assert!(
        (play.feet().x - CHECKPOINTS[0]).abs() < 0.1,
        "at {}",
        play.feet()
    );
    assert!(play.hero().0.is_grounded);
}

/// Whether the ground goes on just ahead of `x`.
fn has_ground_ahead(x: f32) -> bool {
    let ahead = x + 0.9;
    PLATFORMS
        .iter()
        .any(|p| p.pos.y >= GROUND && p.pos.x <= ahead && ahead <= p.pos.x + p.size.x)
}

#[test]
fn the_level_can_be_finished() {
    let mut play = Play::new();
    play.run(30);
    play.key(KeyCode::ArrowRight, true);
    for step in 0..2400u32 {
        let (hero, feet) = play.hero();
        let is_at_edge = hero.is_grounded && !has_ground_ahead(feet.x);
        let is_dropping = !hero.is_grounded && feet.y > GROUND - 0.5;
        if is_at_edge || (is_dropping && !has_ground_ahead(feet.x - 0.9)) {
            play.tap(KeyCode::Space);
        } else if step.is_multiple_of(15) {
            play.tap(KeyCode::KeyX);
        } else {
            play.run(1);
        }
        if play.status().phase != Phase::Playing {
            break;
        }
    }
    let status = play.status();
    assert_eq!(status.phase, Phase::Won, "stopped at {}", play.feet());
    assert!(status.has_called);
    assert!(status.score > 0);
    assert!(play.enemies() < 7);
    assert!(status.lives > 0);
}
