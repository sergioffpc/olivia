//! Loading a character and placing its parts, on a `Graphics` with no GPU.

use std::f32::consts::FRAC_PI_2;
use std::io::Cursor;

use image::{ImageFormat, Rgba, RgbaImage};
use olivia_animation::{Character, Error};
use olivia_filesystem::Filesystem;
use olivia_graphics::{Blend, Camera, Config, Draw, Filter, Graphics, Mesh};
use olivia_math::{Color, Vec2};
use tempfile::TempDir;

/// Shows the game's coordinates as they are.
const WORLD: Camera = Camera {
    position: Vec2::ZERO,
    zoom: 1.0,
    rotation: 0.0,
};

/// A torso 20 by 40 with its root joint in the centre, an arm 10 by 30 hanging from its top right and a
/// forearm 10 by 20 hanging from the bottom of the arm. The forearm comes first, so it is at the
/// back, before the joints it hangs from.
const SKELETON: &str = r#"{
    "version": 1,
    "joints": [
        { "name": "elbow", "parent": "arm", "parent_point": [5, 30], "part": "forearm", "point": [5, 0] },
        { "name": "root", "part": "torso", "point": [10, 20] },
        { "name": "shoulder", "parent": "torso", "parent_point": [16, 5], "part": "arm", "point": [5, 0] }
    ]
}"#;

const IMAGES: [(&str, u32, u32); 3] = [("forearm", 10, 20), ("torso", 20, 40), ("arm", 10, 30)];

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

/// A folder `hero` holding `skeleton` and `images`, and the directory that keeps it alive.
fn folder(skeleton: &str, images: &[(&str, u32, u32)]) -> (Filesystem, TempDir) {
    let dir = TempDir::new().unwrap();
    let mut filesystem = Filesystem::with_areas(dir.path().join("source"), dir.path().join("save"));
    filesystem
        .write("hero/skeleton.json", skeleton.as_bytes())
        .unwrap();
    for (part, width, height) in images {
        filesystem
            .write(&format!("hero/{part}.png"), &png(*width, *height))
            .unwrap();
    }
    (filesystem, dir)
}

fn load(skeleton: &str, images: &[(&str, u32, u32)]) -> Result<Character, Error> {
    let (filesystem, _dir) = folder(skeleton, images);
    Character::load(&filesystem, &mut graphics(), "hero")
}

fn character() -> Character {
    load(SKELETON, &IMAGES).unwrap()
}

/// What `character` draws at `position`, turned by `rotation`.
fn mesh(character: &Character, position: Vec2, rotation: f32) -> Mesh {
    let mut graphics = graphics();
    let mut target = graphics.target();
    character.render(&mut target.view(WORLD), position, rotation, 0.0);
    target.mesh()
}

/// The top-left corner of the part drawn at `index`.
fn top_left(mesh: &Mesh, index: usize) -> Vec2 {
    mesh.vertices[index * 6].position
}

/// The bottom-right corner of the part drawn at `index`.
fn bottom_right(mesh: &Mesh, index: usize) -> Vec2 {
    mesh.vertices[index * 6 + 5].position
}

fn assert_close(actual: Vec2, x: f32, y: f32) {
    assert!(
        (actual - Vec2::new(x, y)).length() < 1e-3,
        "{actual} != ({x}, {y})"
    );
}

#[test]
fn parts_hang_from_their_joints_in_the_order_of_the_file() {
    let mesh = mesh(&character(), Vec2::new(100.0, 50.0), 0.0);
    assert_eq!(mesh.vertices.len(), 18);
    assert_close(top_left(&mesh, 0), 101.0, 65.0); // the forearm
    assert_close(bottom_right(&mesh, 0), 111.0, 85.0);
    assert_close(top_left(&mesh, 1), 90.0, 30.0); // the torso
    assert_close(bottom_right(&mesh, 1), 110.0, 70.0);
    assert_close(top_left(&mesh, 2), 101.0, 35.0); // the arm
    assert_close(bottom_right(&mesh, 2), 111.0, 65.0);
}

#[test]
fn scale_converts_pixels_to_game_units() {
    let hero = character().draw(Draw::new().scale(Vec2::splat(0.5)));
    let mesh = mesh(&hero, Vec2::new(100.0, 50.0), 0.0);
    assert_close(top_left(&mesh, 1), 95.0, 40.0);
    assert_close(top_left(&mesh, 2), 100.5, 42.5);
    assert_close(bottom_right(&mesh, 2), 105.5, 57.5);
}

#[test]
fn a_negative_x_scale_mirrors_around_the_root() {
    let hero = character().draw(Draw::new().scale(Vec2::new(-1.0, 1.0)));
    let mesh = mesh(&hero, Vec2::new(100.0, 50.0), 0.0);
    assert_close(top_left(&mesh, 1), 110.0, 30.0);
    assert_close(top_left(&mesh, 2), 99.0, 35.0);
    assert_close(bottom_right(&mesh, 2), 89.0, 65.0);
}

#[test]
fn the_rotation_turns_the_whole_character_clockwise_around_the_root() {
    let mesh = mesh(&character(), Vec2::new(100.0, 50.0), FRAC_PI_2);
    assert_close(top_left(&mesh, 2), 115.0, 51.0);
    assert_close(bottom_right(&mesh, 2), 85.0, 61.0);
}

#[test]
fn the_rotation_and_origin_of_draw_are_ignored() {
    let hero = character().draw(Draw::new().rotation(1.0).origin(Vec2::ONE));
    let mesh = mesh(&hero, Vec2::new(100.0, 50.0), 0.0);
    assert_close(top_left(&mesh, 1), 90.0, 30.0);
    assert_close(top_left(&mesh, 2), 101.0, 35.0);
}

#[test]
fn color_filter_and_blend_apply_to_every_part() {
    let draw = Draw::new()
        .color(Color::RED)
        .filter(Filter::Nearest)
        .blend(Blend::Add);
    let mesh = mesh(&character().draw(draw), Vec2::ZERO, 0.0);
    for vertex in &mesh.vertices {
        assert_eq!(vertex.color, [1.0, 0.0, 0.0, 1.0]);
    }
    assert_eq!(mesh.batches.len(), 3);
    for batch in &mesh.batches {
        assert_eq!(batch.filter, Filter::Nearest);
        assert_eq!(batch.blend, Blend::Add);
    }
}

#[test]
fn depth_places_the_character_as_one_and_keeps_the_order_of_its_parts() {
    let mut graphics = graphics();
    let above = graphics.texture_rgba(1, 1, &[255; 4]).unwrap();
    let hero = character().draw(Draw::new().depth(1.0));
    let mut target = graphics.target();
    let mut view = target.view(WORLD);
    view.sprite(&above, Vec2::ZERO, Draw::new().depth(2.0));
    hero.render(&mut view, Vec2::new(100.0, 50.0), 0.0, 0.0);
    view.sprite(&above, Vec2::ZERO, Draw::new());

    let mesh = target.mesh();
    assert_close(top_left(&mesh, 0), 0.0, 0.0);
    assert_close(top_left(&mesh, 1), 101.0, 65.0);
    assert_close(top_left(&mesh, 2), 90.0, 30.0);
    assert_close(top_left(&mesh, 3), 101.0, 35.0);
    assert_close(top_left(&mesh, 4), 0.0, 0.0);
}

#[test]
fn a_character_is_a_cheap_clone_that_any_thread_can_hold() {
    fn component<T: Clone + Send + Sync + 'static>(_: &T) {}
    component(&character());
}

#[test]
fn a_missing_image_fails() {
    let error = load(SKELETON, &IMAGES[..2]).unwrap_err();
    assert!(matches!(
        error,
        Error::Filesystem(olivia_filesystem::Error::NotFound(path)) if path == "hero/arm.png"
    ));
}

#[test]
fn a_missing_skeleton_fails() {
    let dir = TempDir::new().unwrap();
    let filesystem = Filesystem::with_areas(dir.path().join("source"), dir.path().join("save"));
    let error = Character::load(&filesystem, &mut graphics(), "hero").unwrap_err();
    assert!(matches!(error, Error::Filesystem(_)));
}

#[test]
fn malformed_json_fails() {
    assert!(matches!(load("{", &[]).unwrap_err(), Error::Json(_)));
}

#[test]
fn an_image_that_is_no_image_fails() {
    let (mut filesystem, _dir) = folder(SKELETON, &IMAGES[..2]);
    filesystem.write("hero/arm.png", b"not a png").unwrap();
    let error = Character::load(&filesystem, &mut graphics(), "hero").unwrap_err();
    assert!(matches!(error, Error::Graphics(_)));
}

#[test]
fn an_unknown_version_fails_whatever_else_the_file_holds() {
    let error = load(r#"{ "version": 2, "bones": {} }"#, &[]).unwrap_err();
    assert!(matches!(error, Error::UnknownVersion(2)));
}

/// The reason `joints` is refused, which must say `reason`.
#[track_caller]
fn assert_invalid(joints: &str, reason: &str) {
    let skeleton = format!(r#"{{ "version": 1, "joints": [{joints}] }}"#);
    let images = [("a", 1, 1), ("b", 1, 1), ("c", 1, 1)];
    match load(&skeleton, &images) {
        Err(Error::InvalidSkeleton(error)) => assert!(error.contains(reason), "{error}"),
        other => panic!("{other:?}"),
    }
}

const ROOT: &str = r#"{ "name": "r", "part": "a", "point": [0, 0] }"#;

/// A joint called `name` with the part `part`, hanging from `parent`.
fn hanging(name: &str, parent: &str, part: &str) -> String {
    format!(
        r#"{{ "name": "{name}", "parent": "{parent}", "parent_point": [0, 0], "part": "{part}", "point": [0, 0] }}"#
    )
}

#[test]
fn no_joints_at_all_have_no_root() {
    assert_invalid("", "no joint");
}

#[test]
fn only_a_cycle_has_no_root() {
    let joints = [hanging("x", "b", "a"), hanging("y", "a", "b")];
    assert_invalid(&joints.join(","), "no joint");
}

#[test]
fn two_roots_fail() {
    let other = r#"{ "name": "s", "part": "b", "point": [0, 0] }"#;
    assert_invalid(&format!("{ROOT},{other}"), "more than one");
}

#[test]
fn an_unknown_parent_fails() {
    let joint = hanging("x", "nothing", "b");
    assert_invalid(&format!("{ROOT},{joint}"), "unknown parent");
}

#[test]
fn a_cycle_beside_the_root_fails() {
    let (x, y) = (hanging("x", "c", "b"), hanging("y", "b", "c"));
    assert_invalid(&format!("{ROOT},{x},{y}"), "cycle");
}

#[test]
fn a_joint_hanging_from_itself_is_a_cycle() {
    let joint = hanging("x", "b", "b");
    assert_invalid(&format!("{ROOT},{joint}"), "cycle");
}

#[test]
fn two_joints_with_one_name_fail() {
    let joint = hanging("r", "a", "b");
    assert_invalid(&format!("{ROOT},{joint}"), "named");
}

#[test]
fn two_joints_with_one_part_fail() {
    let joint = hanging("x", "a", "a");
    assert_invalid(&format!("{ROOT},{joint}"), "part");
}

#[test]
fn a_joint_with_a_parent_and_no_parent_point_fails() {
    let joint = r#"{ "name": "x", "parent": "a", "part": "b", "point": [0, 0] }"#;
    assert_invalid(&format!("{ROOT},{joint}"), "parent_point");
}

#[test]
fn a_character_parses_with_textures_from_anywhere() {
    let mut graphics = graphics();
    let mut asked = Vec::new();
    let character = Character::parse(SKELETON.as_bytes(), |part| {
        asked.push(part.to_owned());
        let (_, width, height) = IMAGES.iter().find(|(name, ..)| *name == part).unwrap();
        Ok::<_, Error>(graphics.texture(&png(*width, *height))?)
    })
    .unwrap();
    assert_eq!(asked, ["forearm", "torso", "arm"]);
    let positions = |mesh: &Mesh| mesh.vertices.iter().map(|v| v.position).collect::<Vec<_>>();
    let parsed = mesh(&character, Vec2::ZERO, 0.0);
    let loaded = mesh(&self::character(), Vec2::ZERO, 0.0);
    assert_eq!(positions(&parsed), positions(&loaded));
}

#[test]
fn the_first_error_of_a_texture_stops_parsing() {
    let missing = |part: &str| olivia_filesystem::Error::NotFound(part.to_owned()).into();
    let error = Character::parse(SKELETON.as_bytes(), |part| Err(missing(part))).unwrap_err();
    assert!(matches!(
        error,
        Error::Filesystem(olivia_filesystem::Error::NotFound(part)) if part == "forearm"
    ));
}
