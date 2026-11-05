//! Draws plain and wrapped text, coloured, turning and behind a sprite, and quits when asked to
//! close.
//!
//! Run it with `cargo run --example text`.

use olivia::ecs::{CClear, CHud, CSprite, CText, CTransform};
use olivia::graphics::{Blend, Draw};
use olivia::math::{Color, Vec2};
use olivia::window::WindowEvent;
use olivia::{Config, Game, Result, Setup, Update};

const WRAPPED: &str = "Wrapped text breaks between words to stay inside its width, and shaping \
                       joins سلام and orders it right to left.";
const WIDTH: f32 = 280.0;

/// Text that turns with the clock.
struct CTurning;

struct Demo;

impl Game for Demo {
    fn load(s: Setup<'_>) -> Result<Self> {
        let bytes = s.filesystem.read("DejaVuSansMono.ttf")?;
        let font = s.graphics.font(&bytes)?;
        let white = s.graphics.texture_rgba(1, 1, &[255; 4])?;
        let world = s.world;
        world.spawn((CClear(Color::rgb(0.1, 0.1, 0.12)),));

        let title = CText::new(font.clone(), 32.0, "Olivia");
        world.spawn((CHud, CTransform::at(24.0, 24.0), title));

        // A box from `measure` behind the wrapped text shows they agree.
        let pos = CTransform::at(24.0, 90.0);
        let size = font.measure_wrapped(WRAPPED, 18.0, WIDTH);
        let panel = Color::rgb(0.2, 0.2, 0.3);
        let behind = Draw::new().scale(size).color(panel).depth(-1.0);
        world.spawn((CHud, pos, CSprite::new(white).draw(behind)));
        let wrapped = CText::new(font.clone(), 18.0, WRAPPED).wrapped(WIDTH);
        world.spawn((CHud, pos, wrapped));

        let centred = Draw::new()
            .origin(Vec2::splat(0.5))
            .color(Color::rgb(1.0, 0.8, 0.2));
        let turning = CText::new(font.clone(), 24.0, "turning").draw(centred);
        world.spawn((CHud, CTurning, CTransform::at(460.0, 120.0), turning));

        let glow = Draw::new()
            .color(Color::new(0.2, 0.5, 1.0, 0.8))
            .blend(Blend::Add);
        let add = CText::new(font, 40.0, "Add").draw(glow);
        world.spawn((CHud, CTransform::at(380.0, 250.0), add.clone()));
        world.spawn((CHud, CTransform::at(380.0, 256.0), add));
        Ok(Self)
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        if u.events.contains(&WindowEvent::CloseRequested) {
            u.quit();
        }
        let dt = u.timer.dt();
        for (_, transform) in u.world.query_mut::<(&CTurning, &mut CTransform)>() {
            transform.rotation += dt;
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    olivia::run::<Demo>(Config::new("text").title("Olivia").size(640.0, 360.0))
}
