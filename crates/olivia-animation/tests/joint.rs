//! Reading the pose of a joint, on a `Graphics` with no GPU.

use std::f32::consts::FRAC_PI_2;
use std::io::Cursor;

use image::{ImageFormat, Rgba, RgbaImage};
use olivia_animation::{Animation, Character, JointPose, Play};
use olivia_filesystem::Filesystem;
use olivia_graphics::{Config, Draw, Graphics};
use olivia_math::Vec2;
use tempfile::TempDir;

/// An arm hanging from a torso, and a forearm hanging from the arm. As drawn, the shoulder is at
/// (6, -15) from the root and the elbow at (6, 15).
const SKELETON: &str = r#"{
    "version": 1,
    "joints": [
        { "name": "elbow", "parent": "arm", "parent_point": [5, 30], "part": "forearm", "point": [5, 0] },
        { "name": "root", "part": "torso", "point": [10, 20] },
        { "name": "shoulder", "parent": "torso", "parent_point": [16, 5], "part": "arm", "point": [5, 0] }
    ]
}"#;

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
    let mut graphics = Graphics::new(Config {
        vsync: true,
        msaa: 4,
    });
    Character::load(&filesystem, &mut graphics, "hero").unwrap()
}

/// An animation of `joint` alone, one row per second.
fn animation<const R: usize>(joint: &str, rows: [f32; R]) -> Animation {
    let rows = rows.map(|angle| [angle]);
    let json = format!(r#"{{ "version": 1, "fps": 1, "joints": ["{joint}"], "rows": {rows:?} }}"#);
    Animation::parse(json.as_bytes()).unwrap()
}

#[track_caller]
fn assert_pose(pose: JointPose, x: f32, y: f32, rotation: f32) {
    assert!(
        (pose.offset - Vec2::new(x, y)).length() < 1e-3,
        "{} != ({x}, {y})",
        pose.offset
    );
    assert!(
        (pose.rotation - rotation).abs() < 1e-3,
        "{} != {rotation}",
        pose.rotation
    );
}

#[test]
fn a_joint_of_a_character_with_nothing_playing_is_as_drawn() {
    let hero = character();
    assert_pose(hero.joint("root").unwrap(), 0.0, 0.0, 0.0);
    assert_pose(hero.joint("elbow").unwrap(), 6.0, 15.0, 0.0);
}

#[test]
fn an_unknown_joint_is_none() {
    assert!(character().joint("wing").is_none());
}

#[test]
fn a_joint_no_animation_covers_follows_the_part_it_hangs_from() {
    let mut hero = character();
    hero.play(&animation("shoulder", [FRAC_PI_2]), Play::new())
        .unwrap();
    assert_pose(hero.joint("elbow").unwrap(), -24.0, -15.0, FRAC_PI_2);
}

#[test]
fn the_offset_has_the_scale_applied() {
    let hero = character().draw(Draw::new().scale(Vec2::splat(0.5)));
    assert_pose(hero.joint("elbow").unwrap(), 3.0, 7.5, 0.0);
}

#[test]
fn a_mirrored_character_gives_a_mirrored_offset_and_the_negative_angle() {
    let mut hero = character().draw(Draw::new().scale(Vec2::new(-0.5, 0.5)));
    hero.play(&animation("shoulder", [FRAC_PI_2]), Play::new())
        .unwrap();
    assert_pose(hero.joint("elbow").unwrap(), 12.0, -7.5, -FRAC_PI_2);
}

#[test]
fn a_bent_joint_is_where_a_scale_stretching_one_axis_draws_it() {
    let mut hero = character().draw(Draw::new().scale(Vec2::new(2.0, 1.0)));
    hero.play(&animation("shoulder", [FRAC_PI_2]), Play::new())
        .unwrap();
    assert_pose(hero.joint("elbow").unwrap(), -18.0, -15.0, FRAC_PI_2);

    hero.draw = Draw::new().scale(Vec2::new(-2.0, 1.0));
    assert_pose(hero.joint("elbow").unwrap(), 18.0, -15.0, -FRAC_PI_2);
}

#[test]
fn the_pose_counts_overlays_and_is_at_the_fixed_step() {
    let mut hero = character();
    hero.overlay(&animation("shoulder", [0.0, 1.0]), Play::new())
        .unwrap();
    hero.advance(0.5);
    assert_pose(hero.joint("shoulder").unwrap(), 6.0, -15.0, 0.5);
}
