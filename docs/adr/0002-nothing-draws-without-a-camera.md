# Nothing is drawn without a camera

`Frame` has no drawing methods. `f.view(camera)` returns a `View`, and only a `View` draws, so forgetting the camera does not compile and a frame with no view shows just its clear colour. Screen-space drawing such as a HUD is not a separate mode: it uses `Camera::screen(viewport)`, where `(0, 0)` is the top-left corner and one unit is one logical pixel. The cost is one extra type and a camera line in even the smallest game, in exchange for a single coordinate model and no hidden camera state on `Frame`.

## Considered Options

- Screen coordinates by default, with an optional `set_camera`/`clear_camera` on `Frame`, as in LÖVE: shorter for simple games, but two coordinate modes and state that carries between draw calls.
- `set_camera` on `Frame` with draws ignored until it is called: no new type, but a forgotten camera is a blank screen with no error.
- A transform stack or a bare `Mat3`: covers hierarchies or needs no new type, but game code then composes matrices for the common scroll and zoom case.
