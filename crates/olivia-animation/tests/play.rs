//! Playing animations on a character, read back from what it draws, on a `Graphics` with no GPU.

use std::f32::consts::{FRAC_PI_2, PI, TAU};
use std::io::Cursor;

use image::{ImageFormat, Rgba, RgbaImage};
use olivia_animation::{Animation, Character, Error, Play};
use olivia_filesystem::Filesystem;
use olivia_graphics::{Camera, Config, Draw, Graphics, Mesh};
use olivia_math::Vec2;
use tempfile::TempDir;

/// Shows the game's coordinates as they are.
const WORLD: Camera = Camera {
    position: Vec2::ZERO,
    zoom: 1.0,
    rotation: 0.0,
};

/// A torso 20 by 40 with its pivot in the centre, an arm 10 by 30 hanging from its top right and a
/// forearm 10 by 20 hanging from the bottom of the arm. The forearm comes first, before the joints
/// it hangs from.
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

/// An animation of `joints` at `fps`, with `rows` of angles.
fn animation<const N: usize>(fps: u32, joints: [&str; N], rows: &[[f32; N]]) -> Animation {
    let json =
        format!(r#"{{ "version": 1, "fps": {fps}, "joints": {joints:?}, "rows": {rows:?} }}"#);
    Animation::parse(json.as_bytes()).unwrap()
}

/// An animation of `shoulder` alone, one row per second.
fn shoulder<const R: usize>(rows: [f32; R]) -> Animation {
    animation(1, ["shoulder"], &rows.map(|angle| [angle]))
}

/// What `character` draws at the origin, `factor` of the way between its last two steps.
fn mesh(character: &Character, factor: f32) -> Mesh {
    let mut graphics = graphics();
    let mut target = graphics.target();
    character.render(&mut target.view(WORLD), Vec2::ZERO, 0.0, factor);
    target.mesh()
}

fn top_left(mesh: &Mesh, index: usize) -> Vec2 {
    mesh.vertices[index * 6].position
}

/// The angle the part drawn at `index` is turned by, measured along its top edge.
fn angle(mesh: &Mesh, index: usize) -> f32 {
    let top_right = mesh.vertices[index * 6 + 1].position;
    (top_right - top_left(mesh, index)).to_angle()
}

/// The angle of the arm of `character` at its last step.
fn arm(character: &Character) -> f32 {
    angle(&mesh(character, 1.0), ARM)
}

#[track_caller]
fn assert_close(actual: Vec2, x: f32, y: f32) {
    assert!(
        (actual - Vec2::new(x, y)).length() < 1e-3,
        "{actual} != ({x}, {y})"
    );
}

/// Whether `actual` and `expected` are one angle, however many turns apart.
#[track_caller]
fn assert_angle(actual: f32, expected: f32) {
    let apart = (actual - expected).rem_euclid(TAU);
    assert!(apart.min(TAU - apart) < 1e-3, "{actual} != {expected}");
}

#[test]
fn a_character_with_nothing_playing_stays_as_drawn() {
    let mut hero = character();
    hero.advance(1.0);
    let mesh = mesh(&hero, 1.0);
    assert_close(top_left(&mesh, ARM), 1.0, -15.0);
    assert_close(top_left(&mesh, FOREARM), 1.0, 15.0);
}

#[test]
fn a_joint_turns_its_part_clockwise_around_its_pivot() {
    let mut hero = character();
    hero.play(&shoulder([FRAC_PI_2]), Play::new()).unwrap();
    let mesh = mesh(&hero, 1.0);
    assert_angle(angle(&mesh, ARM), FRAC_PI_2);
    assert_close(top_left(&mesh, ARM), 6.0, -20.0);
}

#[test]
fn a_part_hangs_from_where_the_turned_parent_puts_it() {
    let mut hero = character();
    hero.play(&shoulder([FRAC_PI_2]), Play::new()).unwrap();
    let mesh = mesh(&hero, 1.0);
    assert_angle(angle(&mesh, FOREARM), FRAC_PI_2);
    assert_close(top_left(&mesh, FOREARM), -24.0, -20.0);
}

#[test]
fn angles_add_up_along_the_tree_from_the_root() {
    let quarter = animation(1, ["root", "elbow"], &[[FRAC_PI_2, FRAC_PI_2]]);
    let mut hero = character();
    hero.play(&quarter, Play::new()).unwrap();
    let mesh = mesh(&hero, 1.0);
    assert_angle(angle(&mesh, TORSO), FRAC_PI_2);
    assert_angle(angle(&mesh, ARM), FRAC_PI_2);
    assert_angle(angle(&mesh, FOREARM), PI);
    assert_close(top_left(&mesh, ARM), 15.0, 1.0);
    assert_close(top_left(&mesh, FOREARM), -10.0, 6.0);
}

#[test]
fn a_mirrored_character_shows_each_angle_negated_around_its_root() {
    let hero_draw = Draw::new().scale(Vec2::new(-1.0, 1.0));
    let mut hero = character().draw(hero_draw);
    hero.play(&shoulder([FRAC_PI_2]), Play::new()).unwrap();
    let mesh = mesh(&hero, 1.0);
    // As drawn this corner goes to (-24, -10), and the mirror takes it across the root.
    assert_close(mesh.vertices[ARM * 6 + 5].position, 24.0, -10.0);
}

#[test]
fn an_animation_with_a_joint_the_skeleton_lacks_fails() {
    let wing = animation(1, ["shoulder", "wing"], &[[0.0, 0.0]]);
    let error = character().play(&wing, Play::new()).unwrap_err();
    assert!(matches!(error, Error::MissingJoint(joint) if joint == "wing"));
}

#[test]
fn a_failed_play_leaves_the_animation_playing() {
    let raised = shoulder([1.0, 2.0]);
    let mut hero = character();
    hero.play(&raised, Play::new()).unwrap();
    hero.play(&animation(1, ["wing"], &[[0.0]]), Play::new())
        .unwrap_err();
    assert_angle(arm(&hero), 1.0);
    hero.advance(1.0);
    assert_angle(arm(&hero), 2.0);
}

#[test]
fn an_animation_needs_only_the_joints_it_covers() {
    let mut hero = character();
    hero.play(&shoulder([0.5]), Play::new()).unwrap();
    assert_angle(angle(&mesh(&hero, 1.0), FOREARM), 0.5);
}

#[test]
fn requesting_the_animation_already_playing_does_nothing() {
    let raise = shoulder([0.0, 1.0]);
    let mut hero = character();
    hero.play(&raise, Play::new()).unwrap();
    hero.advance(0.5);
    hero.play(&raise, Play::new()).unwrap();
    hero.play(&raise.clone(), Play::new()).unwrap();
    assert_angle(arm(&hero), 0.5);
}

#[test]
fn another_play_of_the_animation_already_playing_is_ignored() {
    let raise = shoulder([0.0, 1.0]);
    let mut hero = character();
    hero.play(&raise, Play::new()).unwrap();
    hero.play(&raise, Play::new().looping(true).crossfade(9.0))
        .unwrap();
    hero.advance(5.0);
    assert_angle(arm(&hero), 1.0);
}

#[test]
fn a_second_parse_of_the_same_bytes_is_another_animation() {
    let mut hero = character();
    hero.play(&shoulder([0.0, 1.0]), Play::new()).unwrap();
    hero.advance(1.0);
    hero.play(&shoulder([0.0, 1.0]), Play::new()).unwrap();
    assert_angle(arm(&hero), 0.0);
}

#[test]
fn a_pose_interpolates_between_rows() {
    let mut hero = character();
    hero.play(&shoulder([0.2, 0.4, 1.0]), Play::new()).unwrap();
    hero.advance(0.25);
    assert_angle(arm(&hero), 0.25);
    hero.advance(1.0);
    assert_angle(arm(&hero), 0.55);
}

#[test]
fn time_runs_at_the_fps_of_the_file() {
    let mut hero = character();
    let fast = animation(4, ["shoulder"], &[[0.0], [1.0], [2.0]]);
    hero.play(&fast, Play::new()).unwrap();
    hero.advance(0.125);
    assert_angle(arm(&hero), 0.5);
}

#[test]
fn rows_interpolate_along_the_shortest_path() {
    let mut hero = character();
    hero.play(&shoulder([3.0, -3.0]), Play::new()).unwrap();
    hero.advance(0.5);
    assert_angle(arm(&hero), PI);
}

#[test]
fn without_looping_the_last_row_holds() {
    let mut hero = character();
    hero.play(&shoulder([0.0, 1.0]), Play::new()).unwrap();
    hero.advance(1.75);
    assert_angle(arm(&hero), 1.0);
}

#[test]
fn a_looping_animation_closes_from_its_last_row_to_its_first() {
    let mut hero = character();
    hero.play(&shoulder([0.0, 1.0]), Play::new().looping(true))
        .unwrap();
    hero.advance(1.5);
    assert_angle(arm(&hero), 0.5);
}

#[test]
fn a_looping_animation_lasts_its_rows_over_its_fps() {
    let mut hero = character();
    hero.play(&shoulder([0.25, 1.0, 0.5]), Play::new().looping(true))
        .unwrap();
    hero.advance(3.0);
    assert_angle(arm(&hero), 0.25);
    hero.advance(7.0);
    assert_angle(arm(&hero), 1.0);
}

#[test]
fn a_crossfade_moves_from_the_old_pose_to_the_new_one() {
    let mut hero = character();
    hero.play(&shoulder([1.0]), Play::new()).unwrap();
    hero.play(&shoulder([0.0]), Play::new().crossfade(1.0))
        .unwrap();
    assert_angle(arm(&hero), 1.0);
    hero.advance(0.25);
    assert_angle(arm(&hero), 0.75);
    hero.advance(0.75);
    assert_angle(arm(&hero), 0.0);
}

#[test]
fn a_crossfade_starts_from_the_current_pose_and_follows_the_new_animation() {
    let mut hero = character();
    hero.play(&shoulder([0.0, 1.0]), Play::new()).unwrap();
    hero.advance(0.5);
    hero.play(&shoulder([0.0, 2.0]), Play::new().crossfade(1.0))
        .unwrap();
    hero.advance(0.5);
    assert_angle(arm(&hero), 0.75);
}

#[test]
fn a_crossfade_works_in_the_middle_of_another() {
    let mut hero = character();
    hero.play(&shoulder([0.0]), Play::new()).unwrap();
    hero.play(&shoulder([1.0]), Play::new().crossfade(1.0))
        .unwrap();
    hero.advance(0.5);
    hero.play(&shoulder([-1.0]), Play::new().crossfade(1.0))
        .unwrap();
    assert_angle(arm(&hero), 0.5);
    hero.advance(0.5);
    assert_angle(arm(&hero), -0.25);
}

#[test]
fn a_crossfade_takes_the_shortest_path() {
    let mut hero = character();
    hero.play(&shoulder([3.0]), Play::new()).unwrap();
    hero.play(&shoulder([-3.0]), Play::new().crossfade(1.0))
        .unwrap();
    hero.advance(0.5);
    assert_angle(arm(&hero), PI);
}

#[test]
fn a_crossfade_takes_joints_the_new_animation_does_not_cover_to_zero() {
    let mut hero = character();
    hero.play(&shoulder([0.8]), Play::new()).unwrap();
    let elbow = animation(1, ["elbow"], &[[0.0]]);
    hero.play(&elbow, Play::new().crossfade(1.0)).unwrap();
    hero.advance(0.5);
    assert_angle(arm(&hero), 0.4);
    hero.advance(0.5);
    assert_angle(arm(&hero), 0.0);
}

#[test]
fn a_crossfade_of_zero_is_a_cut() {
    let mut hero = character();
    hero.play(&shoulder([1.0]), Play::new()).unwrap();
    hero.play(&shoulder([0.25]), Play::new()).unwrap();
    assert_angle(arm(&hero), 0.25);
}

#[test]
fn an_animation_not_playing_has_not_finished() {
    assert!(!character().finished(&shoulder([0.0])));
}

#[test]
fn an_animation_finishes_at_its_last_row() {
    let raise = shoulder([0.0, 1.0, 2.0]);
    let mut hero = character();
    hero.play(&raise, Play::new()).unwrap();
    assert!(!hero.finished(&raise));
    hero.advance(1.75);
    assert!(!hero.finished(&raise));
    hero.advance(0.25);
    assert!(hero.finished(&raise));
    hero.advance(9.0);
    assert!(hero.finished(&raise));
}

#[test]
fn finished_knows_an_animation_by_its_identity() {
    let raise = shoulder([0.0]);
    let mut hero = character();
    hero.play(&raise, Play::new()).unwrap();
    assert!(hero.finished(&raise.clone()));
    assert!(!hero.finished(&shoulder([0.0])));
}

#[test]
fn a_looping_animation_never_finishes() {
    let raise = shoulder([0.0, 1.0]);
    let mut hero = character();
    hero.play(&raise, Play::new().looping(true)).unwrap();
    for _ in 0..8 {
        hero.advance(0.5);
        assert!(!hero.finished(&raise));
    }
}

#[test]
fn the_previous_animation_has_not_finished_during_a_crossfade() {
    let (raise, lower) = (shoulder([1.0]), shoulder([0.0, 1.0]));
    let mut hero = character();
    hero.play(&raise, Play::new()).unwrap();
    assert!(hero.finished(&raise));
    hero.play(&lower, Play::new().crossfade(5.0)).unwrap();
    assert!(!hero.finished(&raise));
}

#[test]
fn an_animation_can_finish_while_a_crossfade_to_it_runs() {
    let (raise, lower) = (shoulder([1.0]), shoulder([0.0, 1.0]));
    let mut hero = character();
    hero.play(&raise, Play::new()).unwrap();
    hero.play(&lower, Play::new().crossfade(5.0)).unwrap();
    hero.advance(1.0);
    assert!(hero.finished(&lower));
}

#[test]
fn the_factor_places_the_pose_between_the_last_two_steps() {
    let mut hero = character();
    hero.play(&shoulder([0.0, 1.0]), Play::new()).unwrap();
    hero.advance(0.5);
    assert_angle(angle(&mesh(&hero, 0.0), ARM), 0.0);
    assert_angle(angle(&mesh(&hero, 0.5), ARM), 0.25);
    assert_angle(angle(&mesh(&hero, 1.0), ARM), 0.5);
}

#[test]
fn only_the_last_step_counts_towards_the_factor() {
    let mut hero = character();
    hero.play(&shoulder([0.0, 1.0]), Play::new()).unwrap();
    hero.advance(0.5);
    hero.advance(0.25);
    assert_angle(angle(&mesh(&hero, 0.0), ARM), 0.5);
}

#[test]
fn a_crossfade_is_placed_by_the_factor_too() {
    let mut hero = character();
    hero.play(&shoulder([0.0]), Play::new()).unwrap();
    hero.play(&shoulder([1.0]), Play::new().crossfade(1.0))
        .unwrap();
    hero.advance(0.5);
    assert_angle(angle(&mesh(&hero, 0.5), ARM), 0.25);
}

#[test]
fn an_instant_before_a_crossfade_began_shows_its_start() {
    let mut hero = character();
    hero.play(&shoulder([0.0, 1.0]), Play::new()).unwrap();
    hero.advance(0.5);
    hero.play(&shoulder([2.0]), Play::new().crossfade(1.0))
        .unwrap();
    assert_angle(angle(&mesh(&hero, 0.25), ARM), 0.5);
}

#[test]
fn an_instant_before_a_cut_shows_the_pose_it_cut_from() {
    let mut hero = character();
    hero.play(&shoulder([0.0, 1.0]), Play::new()).unwrap();
    hero.advance(0.5);
    hero.play(&shoulder([2.0]), Play::new()).unwrap();
    assert_angle(angle(&mesh(&hero, 0.5), ARM), 0.5);
    assert_angle(angle(&mesh(&hero, 1.0), ARM), 2.0);
}
