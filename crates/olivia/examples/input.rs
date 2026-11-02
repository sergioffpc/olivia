//! Logs the keyboard and mouse: WASD, Space, the mouse buttons, text and scroll. Move the cursor and
//! press a mouse button to log where it is.
//!
//! Run it with `RUST_LOG=info cargo run --example input`.

use olivia::input::{Control, Input, Key, Mouse};
use olivia::math::Vec2;
use olivia::window::WindowEvent;
use olivia::{Config, Frame, Game, Result, Setup, Update};

struct Demo;

fn report(input: &Input, name: &str, control: impl Control) {
    if input.pressed(control) {
        tracing::info!(name, "pressed");
    }
    if input.released(control) {
        tracing::info!(name, "released");
    }
}

impl Game for Demo {
    fn load(_s: Setup<'_>) -> Result<Self> {
        Ok(Self)
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        for event in u.events {
            if let WindowEvent::CloseRequested = event {
                u.quit();
            }
        }

        let input = u.input;
        report(input, "W", Key::W);
        report(input, "A", Key::A);
        report(input, "S", Key::S);
        report(input, "D", Key::D);
        report(input, "Space", Key::Space);
        report(input, "left button", Mouse::Left);
        report(input, "right button", Mouse::Right);
        report(input, "middle button", Mouse::Middle);

        if input.pressed(Mouse::Left) {
            tracing::info!(cursor = ?input.cursor(), "cursor");
        }
        if !input.text().is_empty() {
            tracing::info!(text = input.text(), "typed");
        }
        if input.scroll() != Vec2::ZERO {
            tracing::info!(scroll = ?input.scroll(), "scrolled");
        }
        Ok(())
    }

    fn draw(&self, _f: &mut Frame<'_>) {}
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    olivia::run::<Demo>(
        Config::new("input")
            .title("Olivia input")
            .size(640.0, 360.0),
    )
}
