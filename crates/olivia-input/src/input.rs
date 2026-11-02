use std::collections::HashSet;

use olivia_math::Vec2;
use olivia_window::winit::dpi::{LogicalPosition, PhysicalPosition};
use olivia_window::winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use olivia_window::winit::keyboard::PhysicalKey;

use crate::control::Button;
use crate::{Control, Key, Mouse};

/// The logical pixels of touchpad scrolling that make one line.
const PIXELS_PER_LINE: f32 = 20.0;

/// The keyboard and mouse as of this update.
///
/// What happened since the last update accumulates, so a tap shorter than one fixed step is never
/// lost. When a frame runs several steps, the first step sees all of it.
#[derive(Debug, Default)]
pub struct Input {
    down: HashSet<Button>,
    pressed: HashSet<Button>,
    released: HashSet<Button>,
    text: String,
    cursor: Vec2,
    scroll: Vec2,
}

impl Input {
    /// Whether `control` is held now.
    #[must_use]
    pub fn down(&self, control: impl Control) -> bool {
        self.down.contains(&control.button())
    }

    /// Whether `control` went down since the last update. Key repeat does not count.
    #[must_use]
    pub fn pressed(&self, control: impl Control) -> bool {
        self.pressed.contains(&control.button())
    }

    /// Whether `control` went up since the last update.
    #[must_use]
    pub fn released(&self, control: impl Control) -> bool {
        self.released.contains(&control.button())
    }

    /// What was typed since the last update, dead keys included and control characters left out.
    ///
    /// IME composition is not supported, which limits Chinese, Japanese and Korean input.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The cursor in logical pixels from the window's top-left corner. Outside the window it stays
    /// at the last known position.
    #[must_use]
    pub fn cursor(&self) -> Vec2 {
        self.cursor
    }

    /// The lines scrolled since the last update. Touchpad pixels are converted to lines.
    #[must_use]
    pub fn scroll(&self) -> Vec2 {
        self.scroll
    }

    /// Receives an event from the window, at its `scale_factor`.
    #[doc(hidden)]
    pub fn on_event(&mut self, event: &WindowEvent, scale_factor: f64) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                self.on_key(event.physical_key, event.state, event.text.as_deref());
            }
            WindowEvent::MouseInput { button, state, .. } => self.on_mouse_button(*button, *state),
            WindowEvent::CursorMoved { position, .. } => self.on_cursor(*position, scale_factor),
            WindowEvent::MouseWheel { delta, .. } => self.on_scroll(*delta, scale_factor),
            WindowEvent::Focused(focused) => self.on_focus(*focused),
            _ => {}
        }
    }

    /// Receives a key from the window.
    #[doc(hidden)]
    pub fn on_key(&mut self, key: PhysicalKey, state: ElementState, text: Option<&str>) {
        if let PhysicalKey::Code(code) = key
            && let Some(key) = Key::from_code(code)
        {
            self.set(Button::Key(key), state);
        }
        if let (ElementState::Pressed, Some(text)) = (state, text) {
            self.text.extend(text.chars().filter(|c| !c.is_control()));
        }
    }

    /// Receives a mouse button from the window.
    #[doc(hidden)]
    pub fn on_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        if let Some(button) = Mouse::from_button(button) {
            self.set(Button::Mouse(button), state);
        }
    }

    /// Receives a cursor position from the window, at its `scale_factor`.
    #[doc(hidden)]
    pub fn on_cursor(&mut self, position: PhysicalPosition<f64>, scale_factor: f64) {
        self.cursor = logical(position, scale_factor);
    }

    /// Receives a scroll from the window, at its `scale_factor`.
    #[doc(hidden)]
    pub fn on_scroll(&mut self, delta: MouseScrollDelta, scale_factor: f64) {
        self.scroll += match delta {
            MouseScrollDelta::LineDelta(x, y) => Vec2::new(x, y),
            MouseScrollDelta::PixelDelta(pixels) => logical(pixels, scale_factor) / PIXELS_PER_LINE,
        };
    }

    /// Receives a focus change from the window. The release of what is held would go to whichever
    /// window has the focus, so it happens here.
    #[doc(hidden)]
    pub fn on_focus(&mut self, focused: bool) {
        if !focused {
            self.released.extend(self.down.drain());
        }
    }

    /// Forgets what the update that just ran has seen.
    #[doc(hidden)]
    pub fn on_step(&mut self) {
        self.pressed.clear();
        self.released.clear();
        self.text.clear();
        self.scroll = Vec2::ZERO;
    }

    fn set(&mut self, button: Button, state: ElementState) {
        match state {
            ElementState::Pressed if self.down.insert(button) => {
                self.pressed.insert(button);
            }
            ElementState::Released if self.down.remove(&button) => {
                self.released.insert(button);
            }
            _ => {}
        }
    }
}

fn logical(position: PhysicalPosition<f64>, scale_factor: f64) -> Vec2 {
    let position: LogicalPosition<f32> = position.to_logical(scale_factor);
    Vec2::new(position.x, position.y)
}
