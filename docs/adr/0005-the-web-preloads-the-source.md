# The web preloads the whole source

`Filesystem::read`, `list` and `exists` are synchronous, and so is `load`, but a browser reaches files only through asynchronous `fetch`. On the web Olivia therefore fetches every file of the source into memory while the game starts, alongside the GPU and on the same executor, and runs `load` only once both are done. A manifest written by the web build lists the source's files, one path per line, so `list` and `exists` answer without the network. The interfaces stay the same on every platform. The cost is that the whole source sits in memory and must download before the first frame, which suits games the size of `astronaut-cat` (about 1 MB).

## Considered Options

- An asynchronous `read`: loads lazily and scales to large sources, but it breaks the identical interfaces and every game's `load`.
- Embedding the source in the wasm binary: no manifest and no requests, but the game's code or build has to name its assets, and the binary grows with every asset.
