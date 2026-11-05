use olivia::ecs::{Entity, World};

/// The most lives the hero has.
pub(crate) const LIVES: u32 = 3;

/// How the game ends, or that it has not yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// Still running.
    Playing,
    /// The princess saved.
    #[expect(dead_code, reason = "reached once the level has the princess")]
    Won,
    /// No lives left.
    #[expect(dead_code, reason = "reached once the hero can lose lives")]
    Lost,
}

/// What a game has come to, on an entity of its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CStatus {
    /// Lives left.
    pub(crate) lives: u32,
    /// Coins picked up.
    pub(crate) score: u32,
    /// How it stands.
    pub(crate) outcome: Outcome,
    /// Whether the princess has called for help.
    pub(crate) has_called: bool,
}

impl CStatus {
    /// How a new game starts.
    pub(crate) const fn new() -> Self {
        Self {
            lives: LIVES,
            score: 0,
            outcome: Outcome::Playing,
            has_called: false,
        }
    }
}

/// The entity holding the status of a new game.
pub(crate) fn spawn(world: &mut World) -> Entity {
    world.spawn((CStatus::new(),))
}
