# Every platform gets WebGL2's limits

The web target falls back to WebGL2 where a browser has no WebGPU, as Firefox on Linux and Intel Macs, most of Chromium on Linux and Safari on macOS 15 do. So that a game behaves the same wherever it runs, the GPU device asks for WebGL2's minimum limits on every platform, the desktop included, instead of everything the adapter offers. The cost is the desktop's headroom, most visibly textures: `texture` and `texture_rgba` panic above 2048 pixels on a side, and Olivia checks this itself before the upload, so a headless game and its tests catch an oversized image too. In exchange a game that works on the author's machine does not fail in someone else's browser, and the WebGL2 path, which nobody exercises on a machine with WebGPU, has little left that can differ. The desktop still has no GLES backend; WebGL2 exists only in the wasm build, and nothing lets a game or a URL force it.

## Considered Options

- No WebGL2, with `NoAdapter` in a browser without WebGPU: one backend to test, but the game shows an error on Linux and on Macs still on macOS 15.
- Each platform asks for the adapter's limits, as before: the desktop keeps its headroom, but an image that loads at home can fail only in some browsers.
