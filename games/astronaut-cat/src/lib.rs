//! A platformer drawn, voiced and acted out by a class, on every module of Olivia.

mod game;
pub mod level;

pub use game::{CEnemy, CHero, Phase, Platformer, CShot, CStar, CStatus};
