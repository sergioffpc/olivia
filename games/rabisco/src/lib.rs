// A game, not a library: its code needs no documentation.
#![allow(missing_docs, clippy::missing_errors_doc, clippy::missing_panics_doc)]

mod assets;
mod camera;
mod depth;
mod game;
mod hud;
mod level;
mod random;
mod sprite;
mod status;

pub use game::Rabisco;
