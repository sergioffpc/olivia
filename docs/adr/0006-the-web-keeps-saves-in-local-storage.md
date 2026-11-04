# The web keeps the save area in localStorage

`Filesystem::write` and `remove` are synchronous and report their failure at the call, as a full disk does on the desktop. On the web the save area is therefore `localStorage`, the only synchronous storage on the main thread: each file is one key, `olivia/<identity>/<path>`, holding its bytes in base64, and directories exist only as key prefixes. A full quota makes `write` return `Error::Io` with `ErrorKind::StorageFull`, and a `localStorage` that cannot be reached is a start failure. The cost is room: about 5 MiB per origin, shared by every game published there and shrunk by a third by base64, which suits saves the size of a platformer's progress. The browser may still clear it, as Safari does after seven days without a visit; Olivia does not ask for persistent storage, because Firefox would show the player a permission prompt.

## Considered Options

- IndexedDB, preloaded while the game starts like the source: far more room, but `write` would return before anything is stored, so a failure could only be logged, never returned.
- The origin private file system: synchronous access exists only in a worker, and the game runs on the main thread.
