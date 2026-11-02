//! Overlays over the base animation, read back from what the character draws, on a `Graphics` with
//! no GPU.

use std::f32::consts::TAU;
use std::io::Cursor;

use image::{ImageFormat, Rgba, RgbaImage};
use olivia_animation::{Animation, Character, Error, Play};
use olivia_filesystem::Filesystem;
use olivia_graphics::{Camera, Config, Graphics, Mesh};
use olivia_math::Vec2;
use tempfile::TempDir;

/// Shows the game's coordinates as they are.
const WORLD: Camera = Camera {
    position: Vec2::ZERO,
    zoom: 1.0,
    rotation: 0.0,
};

/// An arm hanging from a torso, and a forearm hanging from the arm.
const SKELETON: &str = r#"{
    "version": 1,
    "joints": [
        { "name": "elbow", "parent": "arm", "parent_point": [5, 30], "part": "forearm", "point": [5, 0] },
        { "name": "root", "part": "torso", "point": [10, 20] },
        { "name": "shoulder", "parent": "torso", "parent_point": [16, 5], "part": "arm", "point": [5, 0] }
    ]
}"#;

const FOREARM: usize = 0;
const TORSO: usize = 1;
const ARM: usize = 2;

fn graphics() -> Graphics {
    Graphics::new(Config {
        vsync: true,
        msaa: 4,
    })
}

fn png(width: u32, height: u32) -> Vec<u8> {
    let image = RgbaImage::from_pixel(width, height, Rgba([255, 0, 0, 255]));
    let mut bytes = Cursor::new(Vec::new());
    image.write_to(&mut bytes, ImageFormat::Png).unwrap();
    bytes.into_inner()
}

fn character() -> Character {
    let dir = TempDir::new().unwrap();
    let mut filesystem = Filesystem::with_areas(dir.path().join("source"), dir.path().join("save"));
    filesystem
        .write("hero/skeleton.json", SKELETON.as_bytes())
        .unwrap();
    for (part, width, height) in [("forearm", 10, 20), ("torso", 20, 40), ("arm", 10, 30)] {
        filesystem
            .write(&format!("hero/{part}.png"), &png(width, height))
            .unwrap();
    }
    Character::load(&filesystem, &mut graphics(), "hero").unwrap()
}

/// An animation of `joints`, one row per second.
fn animation<const N: usize>(joints: [&str; N], rows: &[[f32; N]]) -> Animation {
    let json = format!(r#"{{ "version": 1, "fps": 1, "joints": {joints:?}, "rows": {rows:?} }}"#);
    Animation::parse(json.as_bytes()).unwrap()
}

/// An animation of `shoulder` alone, one row per second.
fn shoulder<const R: usize>(rows: [f32; R]) -> Animation {
    animation(["shoulder"], &rows.map(|angle| [angle]))
}

/// What `character` draws at the origin, `factor` of the way between its last two steps.
fn mesh(character: &Character, factor: f32) -> Mesh {
    let mut graphics = graphics();
    let mut target = graphics.target();
    character.render(&mut target.view(WORLD), Vec2::ZERO, 0.0, factor);
    target.mesh()
}

/// The angle the part drawn at `index` is turned by, measured along its top edge.
fn angle(mesh: &Mesh, index: usize) -> f32 {
    let top_left = mesh.vertices[index * 6].position;
    let top_right = mesh.vertices[index * 6 + 1].position;
    (top_right - top_left).to_angle()
}

/// The angle of the arm of `character` at its last step.
fn arm(character: &Character) -> f32 {
    angle(&mesh(character, 1.0), ARM)
}

/// Whether `actual` and `expected` are one angle, however many turns apart.
#[track_caller]
fn assert_angle(actual: f32, expected: f32) {
    let apart = (actual - expected).rem_euclid(TAU);
    assert!(apart.min(TAU - apart) < 1e-3, "{actual} != {expected}");
}

#[test]
fn an_overlay_takes_only_the_joints_it_covers() {
    let run = animation(["root", "shoulder"], &[[0.25, 0.5]]);
    let mut hero = character();
    hero.play(&run, Play::new()).unwrap();
    hero.overlay(&shoulder([1.0]), Play::new()).unwrap();
    let mesh = mesh(&hero, 1.0);
    assert_angle(angle(&mesh, TORSO), 0.25);
    assert_angle(angle(&mesh, ARM), 1.25);
}

#[test]
fn an_overlay_and_the_base_advance_together() {
    let mut hero = character();
    hero.play(&animation(["root"], &[[0.0], [1.0]]), Play::new())
        .unwrap();
    hero.overlay(&shoulder([0.0, 1.0]), Play::new()).unwrap();
    hero.advance(0.5);
    let mesh = mesh(&hero, 1.0);
    assert_angle(angle(&mesh, TORSO), 0.5);
    assert_angle(angle(&mesh, ARM), 1.0);
}

#[test]
fn adding_an_active_overlay_does_nothing() {
    let raise = shoulder([0.0, 1.0]);
    let mut hero = character();
    hero.overlay(&raise, Play::new()).unwrap();
    hero.advance(0.5);
    hero.overlay(&raise, Play::new().crossfade(9.0)).unwrap();
    hero.overlay(&raise.clone(), Play::new().looping(true))
        .unwrap();
    assert_angle(arm(&hero), 0.5);
}

#[test]
fn with_several_overlays_the_last_added_wins_on_a_shared_joint() {
    let first = animation(["shoulder", "elbow"], &[[1.0, 0.5]]);
    let mut hero = character();
    hero.overlay(&first, Play::new()).unwrap();
    hero.overlay(&shoulder([2.0]), Play::new()).unwrap();
    hero.overlay(&first, Play::new()).unwrap();
    let mesh = mesh(&hero, 1.0);
    assert_angle(angle(&mesh, ARM), 2.0);
    assert_angle(angle(&mesh, FOREARM), 2.5);
}

#[test]
fn overlay_fails_if_the_animation_names_a_joint_the_skeleton_lacks() {
    let wing = animation(["shoulder", "wing"], &[[1.0, 0.0]]);
    let mut hero = character();
    let error = hero.overlay(&wing, Play::new()).unwrap_err();
    assert!(matches!(error, Error::MissingJoint(joint) if joint == "wing"));
    assert_angle(arm(&hero), 0.0);
}

#[test]
fn an_overlay_crossfades_in_on_its_joints_only() {
    let run = animation(["root", "shoulder"], &[[0.5, 0.5]]);
    let mut hero = character();
    hero.play(&run, Play::new()).unwrap();
    hero.overlay(&shoulder([1.5]), Play::new().crossfade(1.0))
        .unwrap();
    assert_angle(arm(&hero), 1.0);
    hero.advance(0.5);
    let mesh = mesh(&hero, 1.0);
    assert_angle(angle(&mesh, TORSO), 0.5);
    assert_angle(angle(&mesh, ARM), 1.5);
}

#[test]
fn a_non_looping_overlay_holds_its_last_row_and_finished_reports_it() {
    let raise = shoulder([0.0, 1.0]);
    let mut hero = character();
    hero.overlay(&raise, Play::new()).unwrap();
    assert!(!hero.finished(&raise));
    hero.advance(5.0);
    assert!(hero.finished(&raise));
    assert_angle(arm(&hero), 1.0);
}

#[test]
fn a_looping_overlay_never_finishes() {
    let raise = shoulder([0.0, 1.0]);
    let mut hero = character();
    hero.overlay(&raise, Play::new().looping(true)).unwrap();
    hero.advance(1.5);
    assert!(!hero.finished(&raise));
    assert_angle(arm(&hero), 0.5);
}

#[test]
fn on_the_way_out_the_frozen_pose_blends_towards_the_moving_pose_below() {
    let shot = shoulder([1.0]);
    let mut hero = character();
    hero.play(&shoulder([0.0, 0.5, 0.5]), Play::new()).unwrap();
    hero.overlay(&shot, Play::new()).unwrap();
    hero.remove_overlay(&shot, 2.0);
    assert_angle(arm(&hero), 1.0);
    hero.advance(1.0);
    assert_angle(arm(&hero), 0.75);
    hero.advance(1.0);
    assert_angle(arm(&hero), 0.5);
}

#[test]
fn a_crossfade_out_leaves_the_other_joints_alone() {
    let shot = shoulder([1.0]);
    let mut hero = character();
    hero.play(&animation(["root"], &[[0.25]]), Play::new())
        .unwrap();
    hero.overlay(&shot, Play::new()).unwrap();
    hero.remove_overlay(&shot, 1.0);
    hero.advance(0.5);
    let mesh = mesh(&hero, 1.0);
    assert_angle(angle(&mesh, TORSO), 0.25);
    assert_angle(angle(&mesh, ARM), 0.75);
}

#[test]
fn an_overlay_leaves_towards_the_overlays_below_it() {
    let shot = shoulder([1.0]);
    let mut hero = character();
    hero.overlay(&shoulder([0.5]), Play::new()).unwrap();
    hero.overlay(&shot, Play::new()).unwrap();
    hero.remove_overlay(&shot, 0.0);
    assert_angle(arm(&hero), 0.5);
}

#[test]
fn removing_an_inactive_overlay_does_nothing() {
    let raise = shoulder([0.0, 1.0]);
    let mut hero = character();
    hero.play(&raise, Play::new()).unwrap();
    hero.advance(0.5);
    hero.remove_overlay(&raise, 0.0);
    hero.remove_overlay(&shoulder([3.0]), 0.0);
    assert_angle(arm(&hero), 0.5);
}

#[test]
fn removing_a_leaving_overlay_again_changes_nothing() {
    let shot = shoulder([1.0]);
    let mut hero = character();
    hero.overlay(&shot, Play::new()).unwrap();
    hero.remove_overlay(&shot, 1.0);
    hero.advance(0.5);
    hero.remove_overlay(&shot, 0.0);
    assert_angle(arm(&hero), 0.5);
}

#[test]
fn a_leaving_overlay_is_not_active() {
    let shot = shoulder([1.0]);
    let mut hero = character();
    hero.overlay(&shot, Play::new()).unwrap();
    assert!(hero.finished(&shot));
    hero.remove_overlay(&shot, 5.0);
    assert!(!hero.finished(&shot));
}

#[test]
fn adding_a_leaving_overlay_again_starts_it_afresh_from_the_current_pose() {
    let shot = shoulder([1.0, 2.0]);
    let mut hero = character();
    hero.overlay(&shot, Play::new()).unwrap();
    hero.remove_overlay(&shot, 2.0);
    hero.advance(1.0);
    assert_angle(arm(&hero), 0.5);
    hero.overlay(&shot, Play::new().crossfade(1.0)).unwrap();
    assert_angle(arm(&hero), 0.5);
    hero.advance(0.5);
    assert_angle(arm(&hero), 1.0);
    hero.advance(0.5);
    assert_angle(arm(&hero), 2.0);
}

#[test]
fn a_base_crossfade_under_an_overlay_starts_from_the_base_pose() {
    let shot = shoulder([2.0]);
    let mut hero = character();
    hero.play(&shoulder([0.0]), Play::new()).unwrap();
    hero.overlay(&shot, Play::new()).unwrap();
    hero.play(&shoulder([1.0]), Play::new().crossfade(1.0))
        .unwrap();
    hero.advance(0.5);
    assert_angle(arm(&hero), 2.0);
    hero.remove_overlay(&shot, 0.0);
    assert_angle(arm(&hero), 0.5);
}

#[test]
fn the_factor_places_an_overlay_between_the_last_two_steps() {
    let mut hero = character();
    hero.overlay(&shoulder([0.0, 1.0]), Play::new()).unwrap();
    hero.advance(0.5);
    assert_angle(angle(&mesh(&hero, 0.5), ARM), 0.25);
}

#[test]
fn an_instant_before_a_removal_shows_the_pose_it_left_from() {
    let shot = shoulder([1.0]);
    let mut hero = character();
    hero.overlay(&shot, Play::new()).unwrap();
    hero.advance(0.5);
    hero.remove_overlay(&shot, 0.0);
    assert_angle(angle(&mesh(&hero, 0.5), ARM), 1.0);
    assert_angle(angle(&mesh(&hero, 1.0), ARM), 0.0);
}

#[test]
fn a_finished_crossfade_out_stays_until_the_factor_has_passed_it() {
    let shot = shoulder([1.0]);
    let mut hero = character();
    hero.overlay(&shot, Play::new()).unwrap();
    hero.remove_overlay(&shot, 1.0);
    hero.advance(0.75);
    hero.advance(0.5);
    assert_angle(angle(&mesh(&hero, 0.0), ARM), 0.25);
    assert_angle(angle(&mesh(&hero, 1.0), ARM), 0.0);
    hero.advance(0.5);
    assert_angle(angle(&mesh(&hero, 0.0), ARM), 0.0);
}

#[test]
fn an_overlay_of_the_base_animation_starts_on_its_own() {
    let raise = shoulder([0.0, 1.0]);
    let mut hero = character();
    hero.play(&raise, Play::new()).unwrap();
    hero.advance(0.5);
    hero.overlay(&raise, Play::new()).unwrap();
    assert_angle(arm(&hero), 0.0);
}
