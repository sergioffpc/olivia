//! Left and Right walk, Space or Up jumps and jumps again in the air, X shoots, Enter starts over
//! once the game has ended and Tab shows the colliders. Run it with `cargo run --release`.

use astronaut_cat::Platformer;
use olivia::Result;

fn main() -> Result<()> {
    olivia::run::<Platformer>(Platformer::config())
}
