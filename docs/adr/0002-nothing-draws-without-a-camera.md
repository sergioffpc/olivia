# Nothing is drawn without a camera

Since ADR 0003 the camera is a component: an entity is drawn through every `Camera` entity, so a game with no camera shows just its clear colour. The only exception is the HUD, a phase of the renderer: an entity marked `Hud` is drawn after the world through `Camera::screen(viewport)`, where `(0, 0)` is the top-left corner and one unit is one logical pixel. Inside the graphics crate only a `View` draws, and a `View` needs a `Camera`. The cost is a camera entity in every game that shows a world, in exchange for a single coordinate model and no hidden camera state.

## Considered Options

- Screen coordinates by default, with an optional `set_camera`/`clear_camera` on `Frame`, as in LÖVE: shorter for simple games, but two coordinate modes and state that carries between draw calls.
- `set_camera` on `Frame` with draws ignored until it is called: no new type, but a forgotten camera is a blank screen with no error.
- A transform stack or a bare `Mat3`: covers hierarchies or needs no new type, but game code then composes matrices for the common scroll and zoom case.
