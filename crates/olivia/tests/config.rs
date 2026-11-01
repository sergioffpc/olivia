//! The chained setters of `Config`.

use olivia::Config;

#[test]
fn the_defaults_match_their_setters() {
    let explicit = Config::new("demo")
        .title("demo")
        .size(1280.0, 720.0)
        .resizable(true)
        .vsync(true)
        .fullscreen(false)
        .msaa(4);
    assert_eq!(explicit, Config::new("demo"));
}

#[test]
fn each_setter_changes_the_config() {
    let base = Config::new("demo");
    assert_ne!(base.clone().title("Demo"), base);
    assert_ne!(base.clone().size(640.0, 480.0), base);
    assert_ne!(base.clone().resizable(false), base);
    assert_ne!(base.clone().vsync(false), base);
    assert_ne!(base.clone().fullscreen(true), base);
    assert_ne!(base.clone().msaa(1), base);
    assert_ne!(base.clone().assets("path/to/assets"), base);
}

#[test]
fn identity_tells_configs_apart() {
    assert_ne!(Config::new("one"), Config::new("two"));
}
