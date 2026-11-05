// A game, not a library: its code needs no documentation.
#![allow(missing_docs, clippy::missing_errors_doc, clippy::missing_panics_doc)]

fn main() -> olivia::Result<()> {
    olivia::run::<rabisco::Rabisco>(rabisco::Rabisco::config())
}
