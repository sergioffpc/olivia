use std::collections::HashSet;

use olivia_math::Vec2;
use olivia_window::raw::{
    ElementState, Event, MouseButton, MouseScrollDelta, PhysicalKey, PhysicalPosition,
};

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

    /// What was typed since the last update, as the platform resolves it. Dead keys are included,
    /// and so are the control characters of keys like Backspace, which repeat while held.
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
    pub fn on_event(&mut self, event: &Event, scale_factor: f64) {
        match event {
            Event::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => {
                // A key held as the window gains focus arrives as a press nobody typed.
                let text = event.text.as_deref().filter(|_| !is_synthetic);
                let already_held = event.repeat || *is_synthetic;
                self.on_key(event.physical_key, event.state, already_held, text);
            }
            Event::MouseInput { button, state, .. } => self.on_mouse_button(*button, *state),
            Event::CursorMoved { position, .. } => self.on_cursor(*position, scale_factor),
            Event::MouseWheel { delta, .. } => self.on_scroll(*delta, scale_factor),
            Event::Focused(focused) => self.on_focus(*focused),
            _ => {}
        }
    }

    /// Receives a key from the window. `already_held` says it went down before this event.
    #[doc(hidden)]
    pub fn on_key(
        &mut self,
        key: PhysicalKey,
        state: ElementState,
        already_held: bool,
        text: Option<&str>,
    ) {
        if let PhysicalKey::Code(code) = key
            && let Some(key) = Key::from_code(code)
        {
            let button = Button::Key(key);
            if already_held && state == ElementState::Pressed {
                self.down.insert(button);
            } else {
                self.set(button, state);
            }
        }
        if let (ElementState::Pressed, Some(text)) = (state, text) {
            self.text.push_str(text);
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
    let position = position.to_logical::<f32>(scale_factor);
    Vec2::new(position.x, position.y)
}
