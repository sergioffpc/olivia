# Olivia owns the ECS world

Olivia owns one `hecs` world, reached as `olivia::ecs`, and the game plays by spawning and changing entities in it. A game has two phases, `load` and `update`; there is no `Game::draw`. After each `update` Olivia steps physics, and each frame it draws every entity with a `Sprite`, `Text` or `Shape` through every `Camera` entity, then the entities marked `Hud` in window coordinates. The physics `World` belongs to Olivia too: spawning a `RigidBodyBuilder` or `ColliderBuilder` makes a body, physics then writes the entity's `Transform`, and despawning removes the body. The module crates stay standalone leaves with no ECS dependency, so the components and systems live in the umbrella crate.

This reverses the ECS-agnostic position of the first map and spec, and the game-owned physics `World`. The cost is that a game can no longer choose another ECS or draw immediately, and Olivia now retains a scene. In exchange games lose the code every one of them repeated: mapping bodies to entities, copying poses, cleaning up, and writing the same drawing loops, and there is a single model for all drawing.

## Considered Options

- ECS-agnostic, with `hecs` only in examples: the previous decision, with no engine dependency on an ECS, but every game repeats the same plumbing.
- Re-export `hecs` and leave ownership with the game: a small change that removes none of that plumbing.
- A framework with a scheduler, in the style of Bevy: the deepest integration, but it replaces `Game` and the per-phase handles, and a fixed list of phases is enough for this library.
- Keeping an optional `Game::draw` for the HUD: no new components, but two ways to draw. A `Hud` marker and a phase of the renderer cover it.
- Layers on cameras to separate the HUD: more general, but more concepts than one marker.
