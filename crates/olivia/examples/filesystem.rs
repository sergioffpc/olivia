//! Reads an asset, keeps a count of its runs in a save, and quits.
//!
//! Run it with `cargo run --example filesystem`, twice to see the count grow.

use olivia::{Config, Game, Result, Setup, Update};

const RUNS: &str = "progress/runs.txt";

struct Demo;

impl Game for Demo {
    fn load(s: Setup<'_>) -> Result<Self> {
        let greeting = String::from_utf8(s.filesystem.read("greeting.txt")?)?;
        tracing::info!(greeting = greeting.trim(), "read an asset");

        let mut runs = 0;
        if s.filesystem.exists(RUNS)? {
            runs = String::from_utf8(s.filesystem.read(RUNS)?)?.parse::<u32>()?;
            // A write never overwrites.
            s.filesystem.remove(RUNS)?;
        }
        runs += 1;
        s.filesystem.write(RUNS, runs.to_string().as_bytes())?;
        tracing::info!(runs, "wrote a save");
        Ok(Self)
    }

    fn update(&mut self, u: Update<'_>) -> Result<()> {
        u.quit();
        Ok(())
    }
}

fn main() -> Result<()> {
    olivia::run::<Demo>(Config::new("olivia-filesystem-example"))
}
