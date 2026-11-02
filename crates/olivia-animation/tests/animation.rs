//! Parsing animation files and the options of `Play`.

use olivia_animation::{Animation, Error, Play};

fn parse(json: &str) -> Result<Animation, Error> {
    Animation::parse(json.as_bytes())
}

#[test]
fn a_valid_file_parses() {
    let json =
        r#"{ "version": 1, "fps": 30, "joints": ["a", "b"], "rows": [[0.1, 0.4], [0.2, 0.5]] }"#;
    assert!(parse(json).is_ok());
}

#[test]
fn an_animation_is_a_cheap_clone_that_any_thread_can_hold() {
    fn shared<T: Clone + Send + Sync + 'static>(_: &T) {}
    shared(&parse(r#"{ "version": 1, "fps": 1, "joints": [], "rows": [[]] }"#).unwrap());
}

#[test]
fn malformed_json_fails() {
    assert!(matches!(parse("{").unwrap_err(), Error::Json(_)));
}

#[test]
fn an_unknown_version_fails_whatever_else_the_file_holds() {
    let error = parse(r#"{ "version": 2, "curves": {} }"#).unwrap_err();
    assert!(matches!(error, Error::UnknownVersion(2)));
}

/// The reason `json` is refused, which must say `reason`.
#[track_caller]
fn assert_invalid(json: &str, reason: &str) {
    match parse(json) {
        Err(Error::InvalidAnimation(error)) => assert!(error.contains(reason), "{error}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn an_fps_of_zero_fails() {
    assert_invalid(
        r#"{ "version": 1, "fps": 0, "joints": ["a"], "rows": [[0.1]] }"#,
        "fps",
    );
}

#[test]
fn no_rows_fail() {
    assert_invalid(
        r#"{ "version": 1, "fps": 30, "joints": ["a"], "rows": [] }"#,
        "no rows",
    );
}

#[test]
fn a_row_shorter_than_the_joints_fails() {
    assert_invalid(
        r#"{ "version": 1, "fps": 30, "joints": ["a", "b"], "rows": [[0.1, 0.2], [0.3]] }"#,
        "row 1",
    );
}

#[test]
fn a_row_longer_than_the_joints_fails() {
    assert_invalid(
        r#"{ "version": 1, "fps": 30, "joints": ["a"], "rows": [[0.1, 0.2]] }"#,
        "row 0",
    );
}

#[test]
fn play_defaults_to_a_cut_that_does_not_loop() {
    assert_eq!(Play::new(), Play::default());
    assert_ne!(Play::new(), Play::new().looping(true));
    assert_ne!(Play::new(), Play::new().crossfade(0.5));
    assert_eq!(Play::new(), Play::new().crossfade(0.0).looping(false));
}
