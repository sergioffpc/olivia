//! Draws plain and wrapped text, coloured, turning and behind a sprite, and quits when asked to
//! close.
//!
//! Run it with `cargo run --example text`.

use olivia::graphics::{Blend, Camera, Draw, Font, Texture};
use olivia::math::{Color, Vec2};
use olivia::window::WindowEvent;
use olivia::{Config, Frame, Game, Result, Setup, Update};

const WRAPPED: &str = "Wrapped text breaks between words to stay inside its width, and shaping \
                       joins سلام and orders it right to left.";
const WIDTH: f32 = 280.0;

struct Demo {
    font: Font,
    white: Texture,
    viewport: Vec2,
    time: f32,
}

impl Game for Demo {
    fn load(s: Setup<'_>) -> Result<Self> {
        let bytes = s.filesystem.read("DejaVuSansMono.ttf")?;
        Ok(Self {
            font: s.graphics.font(&bytes)?,
            white: s.graphics.texture_rgba(1, 1, &[255; 4])?,
            viewport: Vec2::ZERO,
            time: 0.0,
        })
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        for event in u.events {
            match event {
                WindowEvent::Resized(size) => self.viewport = *size,
                WindowEvent::CloseRequested => u.quit(),
                _ => {}
            }
        }
        self.time += u.timer.dt();
        Ok(())
    }

    fn draw(&self, f: &mut Frame<'_>) {
        f.clear(Color::rgb(0.1, 0.1, 0.12));
        let mut view = f.view(Camera::screen(self.viewport));

        view.text(
            &self.font,
            32.0,
            "Olivia",
            Vec2::new(24.0, 24.0),
            Draw::new(),
        );

        // A box from `measure` behind the wrapped text shows they agree.
        let pos = Vec2::new(24.0, 90.0);
        let size = self.font.measure_wrapped(WRAPPED, 18.0, WIDTH);
        let panel = Color::rgb(0.2, 0.2, 0.3);
        let behind = Draw::new().scale(size).color(panel).depth(-1.0);
        view.sprite(&self.white, pos, behind);
        view.text_wrapped(&self.font, 18.0, WIDTH, WRAPPED, pos, Draw::new());

        let centre = Vec2::new(460.0, 120.0);
        let turning = Draw::new()
            .origin(Vec2::splat(0.5))
            .rotation(self.time)
            .color(Color::rgb(1.0, 0.8, 0.2));
        view.text(&self.font, 24.0, "turning", centre, turning);

        let glow = Draw::new()
            .color(Color::new(0.2, 0.5, 1.0, 0.8))
            .blend(Blend::Add);
        view.text(&self.font, 40.0, "Add", Vec2::new(380.0, 250.0), glow);
        let offset = Vec2::new(380.0, 256.0);
        view.text(&self.font, 40.0, "Add", offset, glow);
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    olivia::run::<Demo>(Config::new("text").title("Olivia").size(640.0, 360.0))
}
