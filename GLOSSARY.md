# Olivia

A Rust 2D game library inspired by LÖVE, built from one crate per module.

## Language

**Game**:
The user's program: the state it owns and the behaviour Olivia drives through the game loop.
_Avoid_: App, application, scene

**Phase**:
One of the three moments in which Olivia calls the game: load, update or draw.
_Avoid_: Callback, hook, stage

**Handle**:
A reference to one Olivia module that a phase hands to the game for the duration of that call.
_Avoid_: Context, service

**Fixed step**:
The constant interval at which update runs, independent of the display's refresh rate.
_Avoid_: Tick, frame (a frame is one draw)

**Interpolation factor**:
How far the time since the last update has advanced into the next fixed step, from 0 to 1. Draw uses it to render between two states.
_Avoid_: Alpha

**World**:
The physics simulation the game owns: the bodies and colliders in it, stepped once per update.
_Avoid_: Scene, space

**Frame**:
One draw: the image Olivia presents for it, and the handle the draw phase receives to produce it.
_Avoid_: Tick (a fixed step is not a frame)

**Camera**:
A plain value the game owns that says which part of the game's coordinates is seen: the point at the centre of the screen, plus zoom and rotation. Nothing is drawn without one.
_Avoid_: Viewport, transform

**View**:
What a frame gives for one camera, and the only thing that draws. A frame with no view shows just its clear colour.
_Avoid_: Layer, render pass

**Sound**:
Audio loaded and ready to play. It holds no playing state, so one sound can play many times at once.
_Avoid_: Source, clip

**Playback**:
One playing of a sound, and the thing the game controls while it lasts. Letting go of a playback does not stop it.
_Avoid_: Voice, instance

**Emitter**:
A position in the game's space that sounds play from. Every sound playing from an emitter moves with it.
_Avoid_: Source, speaker

**Listener**:
The one position in the game's space that emitters are heard from.
_Avoid_: Ear, audio camera
