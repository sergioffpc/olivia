// A game, not a library: its code needs no documentation.
#![allow(missing_docs)]

fn main() -> olivia::Result<()> {
    olivia::run::<rabisco::Rabisco>(rabisco::Rabisco::config())
}
