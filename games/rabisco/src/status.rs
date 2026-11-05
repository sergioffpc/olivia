/// The most lives the hero has.
pub(crate) const LIVES: u32 = 3;

/// How the game stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    /// Still running.
    Playing,
    /// Saved.
    Won,
    /// No lives left.
    Lost,
}

/// What a game has come to, on an entity of its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CStatus {
    /// Lives left.
    pub(crate) lives: u32,
    /// Things picked up.
    pub(crate) score: u32,
    /// How it stands.
    pub(crate) phase: Phase,
    /// Whether who is held has called for help.
    pub(crate) has_called: bool,
}

impl CStatus {
    /// How a new game starts.
    pub(crate) const fn new() -> Self {
        Self {
            lives: LIVES,
            score: 0,
            phase: Phase::Playing,
            has_called: false,
        }
    }
}
