# Olivia

A Rust 2D game library inspired by LÖVE, built from one crate per module.

## Language

**Game**:
The user's program: the behaviour Olivia drives through the game loop, acting on the entities in its world.
_Avoid_: App, application, scene

**Platform**:
Where a game runs: the desktop (Windows, macOS, Linux) or the web (a desktop browser). A game sees the same interfaces on every platform.
_Avoid_: Target, native, backend

**Phase**:
One of the two moments in which Olivia calls the game: load or update.
_Avoid_: Callback, hook, stage

**Game loop**:
What drives a game through its phases, from load until it quits or a phase fails.
_Avoid_: Main loop, runner

**Config**:
What a game fixes before the game loop starts: its identity and how its window begins.
_Avoid_: Settings, options

**Setup**:
The handles the load phase receives.
_Avoid_: Init, context

**Handle**:
A reference to one Olivia module that a phase hands to the game for the duration of that call.
_Avoid_: Context, service

**Source**:
The read-only area a game's assets come from.
_Avoid_: Assets folder, resources

**Hot reload**:
Restarting a running game on the changed files of the source, by running load again from an empty world. A load that fails leaves the running game as it was.
_Avoid_: Live reload, asset watching

**Save area**:
The writable area of a game's files, kept for the user on their own machine and shared by every game with the same identity. A file here shadows the source file with the same path.
_Avoid_: Save folder, user data

**Fixed step**:
The constant interval at which update runs, independent of the display's refresh rate.
_Avoid_: Tick, frame (a frame is one draw)

**Interpolation factor**:
How far the time since the last update has advanced into the next fixed step, from 0 to 1. Olivia uses it to draw between two states.
_Avoid_: Alpha

**Timer**:
The read-only view of the game's time: the fixed step, the time it has advanced through and the frame rate.
_Avoid_: Clock, stopwatch

**World**:
The entities Olivia owns for a game, in `olivia::ecs`. The physics world is the simulation Olivia keeps in step with them: the bodies and colliders in it, stepped once after each update.
_Avoid_: Scene, space

**Entity**:
One thing in the world, defined only by its components.
_Avoid_: Object, actor

**Component**:
A piece of data on an entity. Olivia acts on its own, such as `Transform`, `Sprite` and `Body`; a game can add any others.
_Avoid_: Property, attribute

**Frame**:
One draw of the world: the image Olivia presents for it.
_Avoid_: Tick (a fixed step is not a frame)

**Camera**:
A component that says which part of the game's coordinates is seen: the point at the centre of the screen, plus zoom and rotation. Nothing is drawn without one.
_Avoid_: Viewport, transform

**View**:
What a frame gives for one camera, and the only thing that draws.
_Avoid_: Layer, render pass

**HUD**:
The entities marked `Hud`, drawn in a phase of their own after the world, in logical pixels from the window's top-left corner, with no camera entity.
_Avoid_: UI, overlay (an overlay is an animation)

**Sound**:
Audio loaded and ready to play. It holds no playing state, so one sound can play many times at once.
_Avoid_: Source, clip

**Cue**:
One playing of a sound, and the thing the game controls while it lasts. Letting go of a cue does not stop it.
_Avoid_: Playback, voice, instance

**Emitter**:
A position in the game's space that sounds play from. Every sound playing from an emitter moves with it.
_Avoid_: Source, speaker

**Listener**:
The one position in the game's space that emitters are heard from.
_Avoid_: Ear, audio camera

**Character**:
A component: a set of parts on a skeleton, with what is playing on it. Olivia advances and draws it at its entity's pose.
_Avoid_: Actor, puppet, entity

**Skeleton**:
The tree of joints an animated character is built on, measured per character. Characters whose skeletons have the same joint names and structure share a skeleton, and so can share their animations.
_Avoid_: Rig, armature

**Joint**:
A point of a skeleton around which one part rotates relative to the part it hangs from. The root joint hangs from nothing: it turns the first part relative to the character.
_Avoid_: Bone, pivot

**Part**:
One rigid image of a character attached to a skeleton, such as a head or a forearm.
_Avoid_: Piece, limb, sprite

**Animation**:
Timed poses for the joints of a skeleton, loaded and ready to play on any character built on it.
_Avoid_: Clip, motion

**Overlay**:
An animation that covers only some joints and plays over another, which keeps the rest.
_Avoid_: Layer, blend

**Crossfade**:
The move of each joint from its current pose to a new animation's pose over a chosen time. A time of zero is a cut.
_Avoid_: Blend, transition
