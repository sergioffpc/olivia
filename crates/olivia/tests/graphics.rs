//! Graphics through the phases, in a headless game with no GPU.
#![cfg(feature = "graphics")]

use std::time::Duration;

use olivia::ecs::{Clear, Hud, Sprite, Transform};
use olivia::graphics::Graphics;
use olivia::math::{Color, Vec2};
use olivia::window::Window;
use olivia::{Config, Game, GameLoop, Modules, Result, Setup, Update};

/// Takes the graphics handle in `load` and `update`, and spawns a clear colour and a HUD sprite.
struct Clears;

fn handle(_: &mut Graphics) {}

impl Game for Clears {
    fn load(s: Setup<'_>) -> Result<Self> {
        handle(s.graphics);
        let texture = s.graphics.texture_rgba(1, 1, &[255; 4])?;
        s.world.spawn((Clear(Color::RED),));
        s.world
            .spawn((Hud, Transform::at(1.0, 2.0), Sprite::new(texture)));
        Ok(Self)
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        handle(u.graphics);
        Ok(())
    }
}

#[test]
fn a_game_draws_a_frame_with_the_graphics_handle_in_its_phases() {
    let mut window = Window::headless();
    let modules = Modules::new(&Config::new("test")).unwrap();
    let mut game_loop = GameLoop::<Clears>::new(&mut window, Vec2::ZERO, modules).unwrap();
    assert!(game_loop.run_frame(Duration::ZERO, &mut window).unwrap());
    assert_eq!(game_loop.mesh().vertices.len(), 6);
}
