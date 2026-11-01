# Third-party types are reached only through `olivia_*` paths

Every module crate re-exports, in a curated list, the third-party types its public API uses, and game code and other modules import them only from `olivia_*` paths (`olivia_math::Vec2`, `olivia_physics::RigidBody`), never from `glam` or `rapier2d` directly. `olivia-math` re-exports `glam` at the version `rapier2d` uses (0.33, through `glamx`, for `rapier2d` 0.36), because `rapier2d`'s public API has been built on `glam` since 0.35 and one shared set of math types avoids a conversion layer. The cost is maintaining the re-export lists and keeping `glam` pinned to `rapier2d`'s version, in exchange for Olivia owning when a dependency is swapped or upgraded.

## Considered Options

- `nalgebra` for math, with conversions to `glam` at the `olivia-physics` boundary: the first choice, made on the false premise that `rapier2d` 0.36 is built on `nalgebra`. It would add a conversion layer and stop the `rapier2d` builders being re-exported as they are.
- Olivia's own math types: full isolation, but conversions at the physics boundary and more code to own.
- Direct third-party imports in game code: no re-export upkeep, but dependency versions leak into every game.
