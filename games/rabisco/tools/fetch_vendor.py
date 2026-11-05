# /// script
# requires-python = ">=3.10"
# ///
"""Downloads what the motion capture page needs into tools/mocap/vendor/, so the tools work offline.

MediaPipe's vision tasks, as an ES module with its WebAssembly, and the full pose landmarker model.
They are about 28 MB, so git leaves them out: run this once, with a network, after cloning, with
`uv run tools/fetch_vendor.py`. Files already there are kept.
"""

import urllib.request
from pathlib import Path

VENDOR = Path(__file__).resolve().parent / "mocap" / "vendor"
TASKS = "https://cdn.jsdelivr.net/npm/@mediapipe/tasks-vision@0.10.14"
FILES = {
    "vision_bundle.mjs": f"{TASKS}/vision_bundle.mjs",
    **{f"wasm/{name}": f"{TASKS}/wasm/{name}" for name in (
        "vision_wasm_internal.js", "vision_wasm_internal.wasm",
        "vision_wasm_nosimd_internal.js", "vision_wasm_nosimd_internal.wasm",
    )},
    "pose_landmarker_full.task": "https://storage.googleapis.com/mediapipe-models/pose_landmarker/"
                                 "pose_landmarker_full/float16/1/pose_landmarker_full.task",
}


def main():
    for name, url in FILES.items():
        path = VENDOR / name
        if path.exists():
            print(f"have {path}")
            continue
        path.parent.mkdir(parents=True, exist_ok=True)
        part = path.with_name(path.name + ".part")
        with urllib.request.urlopen(url) as response, part.open("wb") as out:
            while chunk := response.read(1 << 20):
                out.write(chunk)
        part.rename(path)
        print(f"wrote {path}")


if __name__ == "__main__":
    main()
