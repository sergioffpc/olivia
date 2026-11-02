//! What the game reads back from the input the window reports.

use olivia_input::{Input, Key, Mouse};
use olivia_math::Vec2;
use olivia_window::winit::dpi::PhysicalPosition;
use olivia_window::winit::event::ElementState::{Pressed, Released};
use olivia_window::winit::event::{MouseButton, MouseScrollDelta};
use olivia_window::winit::keyboard::{KeyCode, NativeKeyCode, PhysicalKey};

const W: PhysicalKey = PhysicalKey::Code(KeyCode::KeyW);

#[test]
fn a_tap_within_one_step_is_pressed_and_released_but_not_down() {
    let mut input = Input::default();
    input.on_key(W, Pressed, None);
    input.on_key(W, Released, None);
    assert!(input.pressed(Key::W));
    assert!(input.released(Key::W));
    assert!(!input.down(Key::W));
}

#[test]
fn a_held_key_stays_down_after_pressed_ends_with_the_step() {
    let mut input = Input::default();
    input.on_key(W, Pressed, None);
    assert!(input.pressed(Key::W) && input.down(Key::W));

    input.on_step();
    assert!(!input.pressed(Key::W));
    assert!(input.down(Key::W));
}

#[test]
fn key_repeat_does_not_press_again() {
    let mut input = Input::default();
    input.on_key(W, Pressed, None);
    input.on_step();
    input.on_key(W, Pressed, None);
    assert!(!input.pressed(Key::W));
}

#[test]
fn a_key_with_no_physical_code_is_ignored() {
    let mut input = Input::default();
    let unknown = PhysicalKey::Unidentified(NativeKeyCode::Unidentified);
    input.on_key(unknown, Pressed, None);
    assert!(!input.down(Key::W));
}

#[test]
fn mouse_buttons_answer_the_same_queries_as_keys() {
    let mut input = Input::default();
    input.on_mouse_button(MouseButton::Left, Pressed);
    assert!(input.pressed(Mouse::Left) && input.down(Mouse::Left));
    assert!(!input.down(Mouse::Right));

    input.on_mouse_button(MouseButton::Left, Released);
    assert!(input.released(Mouse::Left));
    assert!(!input.down(Mouse::Left));
}

#[test]
fn losing_focus_releases_what_is_held() {
    let mut input = Input::default();
    input.on_key(W, Pressed, None);
    input.on_mouse_button(MouseButton::Left, Pressed);
    input.on_step();

    input.on_focus(false);
    assert!(input.released(Key::W) && input.released(Mouse::Left));
    assert!(!input.down(Key::W) && !input.down(Mouse::Left));
}

#[test]
fn text_joins_what_was_typed_until_the_step() {
    let mut input = Input::default();
    input.on_key(W, Pressed, Some("w"));
    input.on_key(W, Released, Some("w"));
    input.on_key(W, Pressed, Some("é"));
    input.on_key(W, Pressed, Some("é"));
    assert_eq!(input.text(), "wéé");

    input.on_step();
    assert_eq!(input.text(), "");
}

#[test]
fn text_leaves_out_control_characters() {
    let mut input = Input::default();
    input.on_key(W, Pressed, Some("a\u{8}\r\u{1b}"));
    assert_eq!(input.text(), "a");
}

#[test]
fn the_cursor_is_in_logical_pixels_and_outlives_the_step() {
    let mut input = Input::default();
    assert_eq!(input.cursor(), Vec2::ZERO);

    input.on_cursor(PhysicalPosition::new(300.0, 100.0), 2.0);
    input.on_step();
    assert_eq!(input.cursor(), Vec2::new(150.0, 50.0));
}

#[test]
fn scroll_sums_lines_until_the_step() {
    let mut input = Input::default();
    input.on_scroll(MouseScrollDelta::LineDelta(1.0, -1.0), 1.0);
    input.on_scroll(MouseScrollDelta::LineDelta(0.5, -2.0), 1.0);
    assert_eq!(input.scroll(), Vec2::new(1.5, -3.0));

    input.on_step();
    assert_eq!(input.scroll(), Vec2::ZERO);
}

#[test]
fn touchpad_pixels_scroll_by_lines_of_twenty_logical_pixels() {
    let mut input = Input::default();
    let pixels = PhysicalPosition::new(40.0, -80.0);
    input.on_scroll(MouseScrollDelta::PixelDelta(pixels), 2.0);
    assert_eq!(input.scroll(), Vec2::new(1.0, -2.0));
}
