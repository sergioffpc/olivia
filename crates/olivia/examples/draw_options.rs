//! Shows `depth`, `filter` and `blend`, and quits when asked to close.
//!
//! Left to right: squares drawn back to front whatever their call order, a small checker scaled up
//! with each filter, and overlapping squares with each blend.
//!
//! Run it with `cargo run --example draw_options`.

use olivia::graphics::{Blend, Camera, Draw, Filter, Texture, View};
use olivia::math::{Color, Vec2};
use olivia::window::WindowEvent;
use olivia::{Config, Frame, Game, Result, Setup, Update};

const RED: Color = Color::new(1.0, 0.2, 0.2, 0.8);
const GREEN: Color = Color::new(0.2, 1.0, 0.2, 0.8);
const BLUE: Color = Color::new(0.2, 0.2, 1.0, 0.8);

struct Demo {
    white: Texture,
    checker: Texture,
    viewport: Vec2,
}

impl Demo {
    /// Three overlapping squares of `draw`, red, green then blue from `pos`.
    fn squares(&self, view: &mut View<'_>, pos: Vec2, draw: Draw) {
        let square = draw.scale(Vec2::splat(80.0));
        view.sprite(&self.white, pos, square.color(RED));
        let green = pos + Vec2::new(40.0, 20.0);
        view.sprite(&self.white, green, square.color(GREEN));
        let blue = pos + Vec2::new(20.0, 50.0);
        view.sprite(&self.white, blue, square.color(BLUE));
    }
}

impl Game for Demo {
    fn load(s: Setup<'_>) -> Result<Self> {
        let white = s.graphics.texture_rgba(1, 1, &[255; 4])?;
        #[rustfmt::skip]
        let checker = s.graphics.texture_rgba(2, 2, &[
            255, 255, 255, 255,   30, 30, 30, 255,
            30, 30, 30, 255,      255, 255, 255, 255,
        ])?;
        Ok(Self {
            white,
            checker,
            viewport: Vec2::ZERO,
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
        Ok(())
    }

    fn draw(&self, f: &mut Frame<'_>) {
        f.clear(Color::rgb(0.1, 0.1, 0.12));
        let mut view = f.view(Camera::screen(self.viewport));

        // Called red first, yet red ends on top.
        let square = Draw::new().scale(Vec2::splat(80.0));
        let on_top = square.color(RED).depth(2.0);
        view.sprite(&self.white, Vec2::new(40.0, 40.0), on_top);
        let middle = square.color(GREEN).depth(1.0);
        view.sprite(&self.white, Vec2::new(70.0, 60.0), middle);
        let below = square.color(BLUE).depth(-1.0);
        view.sprite(&self.white, Vec2::new(50.0, 80.0), below);

        let scaled = Draw::new().scale(Vec2::splat(50.0));
        view.sprite(&self.checker, Vec2::new(220.0, 40.0), scaled);
        let nearest = scaled.filter(Filter::Nearest);
        view.sprite(&self.checker, Vec2::new(220.0, 170.0), nearest);

        self.squares(&mut view, Vec2::new(370.0, 40.0), Draw::new());
        let add = Draw::new().blend(Blend::Add);
        self.squares(&mut view, Vec2::new(370.0, 180.0), add);
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    olivia::run::<Demo>(
        Config::new("draw_options")
            .title("Olivia")
            .size(640.0, 360.0),
    )
}
