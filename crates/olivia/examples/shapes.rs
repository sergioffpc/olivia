//! Draws every shape, sorted with a sprite, and quits when asked to close.
//!
//! The top row is filled and the bottom row outlined: a rectangle turning around its centre, a
//! circle, and a concave polygon turning around `pos`. Lines turn around their shared end on the
//! right, and at the far left a sprite sits between two rectangles by `depth`.
//!
//! Run it with `cargo run --example shapes`.

use olivia::graphics::{Camera, Draw, Texture};
use olivia::math::{Color, Rect, Vec2};
use olivia::window::WindowEvent;
use olivia::{Config, Frame, Game, Result, Setup, Update};

const RED: Color = Color::rgb(1.0, 0.3, 0.3);
const GREEN: Color = Color::rgb(0.3, 1.0, 0.4);
const BLUE: Color = Color::rgb(0.4, 0.5, 1.0);
const YELLOW: Color = Color::rgb(1.0, 0.9, 0.3);

/// An arrow head with a notch in its back, which only a concave fill draws right.
const ARROW: [Vec2; 4] = [
    Vec2::new(0.0, -40.0),
    Vec2::new(35.0, 40.0),
    Vec2::new(0.0, 15.0),
    Vec2::new(-35.0, 40.0),
];

struct Demo {
    pixel: Texture,
    turn: f32,
    viewport: Vec2,
}

impl Game for Demo {
    fn load(s: Setup<'_>) -> Result<Self> {
        Ok(Self {
            pixel: s.graphics.texture_rgba(1, 1, &[255; 4])?,
            turn: 0.0,
            viewport: Vec2::ZERO,
        })
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        self.turn += 0.01;
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
        let turning = Draw::new().rotation(self.turn);
        let centred = turning.origin(Vec2::splat(0.5));

        // Called top first, yet depth puts the sprite between the rectangles.
        let top = Draw::new().color(RED).depth(1.0);
        view.rect(Rect::new(50.0, 110.0, 60.0, 60.0), top);
        let sprite = Draw::new().color(YELLOW).scale(Vec2::splat(60.0));
        view.sprite(&self.pixel, Vec2::new(30.0, 90.0), sprite);
        let bottom = Draw::new().color(BLUE).depth(-1.0);
        view.rect(Rect::new(10.0, 70.0, 60.0, 60.0), bottom);

        let filled = Rect::new(210.0, 90.0, 100.0, 60.0);
        view.rect(filled, centred.color(RED));
        view.circle(Vec2::new(350.0, 90.0), 45.0, Draw::new().color(GREEN));
        view.polygon(Vec2::new(470.0, 90.0), &ARROW, turning.color(BLUE));

        let outlined = Rect::new(210.0, 270.0, 100.0, 60.0);
        view.rect_outline(outlined, 4.0, centred.color(RED));
        let thin = Draw::new().color(GREEN);
        view.circle_outline(Vec2::new(350.0, 270.0), 45.0, 2.0, thin);
        let arrow = turning.color(BLUE);
        view.polygon_outline(Vec2::new(470.0, 270.0), &ARROW, 3.0, arrow);

        let hub = Vec2::new(570.0, 180.0);
        for (width, spread) in [(1.0, -100.0), (2.0, -33.0), (4.0, 33.0), (8.0, 100.0)] {
            let end = hub + Vec2::new(50.0, spread);
            view.line(hub, end, width, turning.color(YELLOW));
        }
    }
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    olivia::run::<Demo>(Config::new("shapes").title("Olivia").size(640.0, 360.0))
}
