# `run` never returns a phase failure

`olivia::run` has the same contract on every platform: it returns `Err` only for what fails before the game starts, an invalid identity or an event loop that cannot be created. A failure of `load`, `update` or the GPU never comes back through `run`. Olivia logs it and passes it to one failure report, and on desktop the process then exits with code 1 once the window is closed. The game's `main` stays `fn main() -> Result<()> { olivia::run::<G>(config) }`.

The web forced this. There winit's `spawn_app` returns before the game starts, so `run` has nothing to wait for and no way to hand a later error back. Olivia's interfaces are the same on every platform, so desktop follows the web instead of keeping a contract the web cannot honour. The cost on desktop is that a phase error no longer reaches `main` as a value: a caller cannot inspect it, wrap it, or recover from it there, and the exit code comes from Olivia, not from the caller's `Result`.

## Considered Options

- Return phase failures on desktop and only report them on the web: keeps today's desktop behaviour, but the same call means two different things depending on the platform.
- Return `Ok(())` on desktop after a phase failure: simpler, but a crashed game looks like one that quit normally.
- Make `run` return `()` and report everything internally, the invalid identity included: one channel for all errors, but every game's `main` has to change.
