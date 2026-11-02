"""Turns one side-on video into one animation file. Run it through the `mocap` script."""

import argparse
import json
import math
import sys
from pathlib import Path

import numpy as np
from scipy.ndimage import gaussian_filter1d, median_filter

MODEL = Path(__file__).parent / "models" / "pose_landmarker_heavy.task"
LANDMARKS = 33
FPS = 30
MEDIAN_FRAMES = 5
GAUSSIAN_SECONDS = 0.035

JOINTS = [
    "shoulder_left",
    "elbow_left",
    "hip_left",
    "knee_left",
    "hip_right",
    "knee_right",
    "root",
    "shoulder_right",
    "elbow_right",
    "neck",
]

NOSE, EAR_LEFT, EAR_RIGHT = 0, 7, 8
LEFT = {"shoulder": 11, "elbow": 13, "wrist": 15, "hip": 23, "knee": 25, "ankle": 27}
RIGHT = {"shoulder": 12, "elbow": 14, "wrist": 16, "hip": 24, "knee": 26, "ankle": 28}

# The sheet's pose as raw joint angles: upright with every part hanging straight down, so upper
# arms and legs sit half a turn from the torso. See the stand-in's skeleton.json; the template is
# not in the repo.
REST = {
    "root": -np.pi / 2,
    "neck": 0.0,
    "shoulder": np.pi,
    "elbow": 0.0,
    "hip": np.pi,
    "knee": 0.0,
}


def wrap(angles):
    """Wraps radians to ±180°."""
    return (angles + np.pi) % (2 * np.pi) - np.pi


def segment_angle(start, end):
    """The world angle of each frame's segment: clockwise from +x, in pixels with y down."""
    delta = end - start
    return np.arctan2(delta[:, 1], delta[:, 0])


def face_right(points):
    """Mirrors the poses if the nose is left of the ears."""
    ears = points[:, [EAR_LEFT, EAR_RIGHT], 0].mean(axis=1)
    facing = np.median(points[:, NOSE, 0] - ears)
    return points if facing >= 0 else points * (-1.0, 1.0)


def near_side(visibility):
    """The body side closer to the camera. Visibility only chooses a side, never rejects points."""
    left = visibility[:, list(LEFT.values())].mean()
    right = visibility[:, list(RIGHT.values())].mean()
    return LEFT if left > right else RIGHT


def joint_angles(points, near):
    """Each joint as child minus parent, one column per entry of `JOINTS`. The near side is `right`."""
    far = LEFT if near is RIGHT else RIGHT
    hips = points[:, [LEFT["hip"], RIGHT["hip"]]].mean(axis=1)
    shoulders = points[:, [LEFT["shoulder"], RIGHT["shoulder"]]].mean(axis=1)
    ears = points[:, [EAR_LEFT, EAR_RIGHT]].mean(axis=1)
    torso = segment_angle(hips, shoulders)

    angles = {"root": torso, "neck": wrap(segment_angle(shoulders, ears) - torso)}
    for name, side in (("right", near), ("left", far)):
        upper_arm = segment_angle(points[:, side["shoulder"]], points[:, side["elbow"]])
        lower_arm = segment_angle(points[:, side["elbow"]], points[:, side["wrist"]])
        upper_leg = segment_angle(points[:, side["hip"]], points[:, side["knee"]])
        lower_leg = segment_angle(points[:, side["knee"]], points[:, side["ankle"]])
        angles[f"shoulder_{name}"] = wrap(upper_arm - torso)
        angles[f"elbow_{name}"] = wrap(lower_arm - upper_arm)
        angles[f"hip_{name}"] = wrap(upper_leg - torso)
        angles[f"knee_{name}"] = wrap(lower_leg - upper_leg)
    return np.column_stack([angles[joint] for joint in JOINTS])


def smooth(angles, times):
    """A median to remove spikes, then a zero-phase Gaussian, over continuous angles."""
    if len(times) < 2:
        return angles
    # Upper arms and legs hang half a turn from the torso, where wrapped angles flip sign on every
    # bit of jitter.
    continuous = np.unwrap(angles, axis=0)
    medians = median_filter(continuous, size=(MEDIAN_FRAMES, 1), mode="nearest")
    sigma = GAUSSIAN_SECONDS / np.median(np.diff(times))
    return gaussian_filter1d(medians, sigma, axis=0, mode="nearest")


def within_half_a_turn(column, centre):
    """Moves continuous angles by whole turns, to bring their median near `centre`."""
    return column - 2 * np.pi * np.round((np.median(column) - centre) / (2 * np.pi))


def bend_one_way(angles):
    """Facing right, a forearm only folds forwards (anticlockwise) and a shin only backwards.

    Unwrapping leaves a column on whichever turn its first frame was, so each one first moves to
    the turn around its joint's bend, or a video starting fully folded would flatten.
    """
    bent = angles.copy()
    for index, joint in enumerate(JOINTS):
        if joint.startswith("elbow"):
            bent[:, index] = np.minimum(within_half_a_turn(bent[:, index], -np.pi / 2), 0.0)
        elif joint.startswith("knee"):
            bent[:, index] = np.maximum(within_half_a_turn(bent[:, index], np.pi / 2), 0.0)
    return bent


def resample(angles, times):
    """Continuous angles at `FPS`, from the first pose up to the last whole frame."""
    count = int((times[-1] - times[0]) * FPS + 1e-9) + 1
    grid = times[0] + np.arange(count) / FPS
    return np.column_stack([np.interp(grid, times, column) for column in angles.T])


def rotations(times, points, visibility):
    """Turns poses into rows of radians at `FPS`, one column per entry of `JOINTS`.

    Zero is the pose on the sheet. Angles stay continuous until the end, so that filtering and
    resampling never cross a wrap.
    """
    angles = joint_angles(face_right(points), near_side(visibility))
    angles = bend_one_way(smooth(angles, times))
    rest = np.array([REST[joint.split("_")[0]] for joint in JOINTS])
    return wrap(resample(angles - rest, times))


def largest_steps(rows):
    """The largest frame-to-frame step per joint, along the shortest path."""
    if len(rows) < 2:
        return np.zeros(rows.shape[1])
    return np.abs(wrap(np.diff(rows, axis=0))).max(axis=0)


def trim(times, points, visibility):
    """Drops the frames with no pose at each end. A stretch with none in the middle is an error."""
    posed = ~np.isnan(points).any(axis=(1, 2))
    if not posed.any():
        raise SystemExit("no pose in the video")
    first, last = np.flatnonzero(posed)[[0, -1]]
    kept = slice(first, last + 1)

    gaps = np.flatnonzero(~posed[kept]) + first
    if len(gaps):
        stretches = np.split(gaps, np.flatnonzero(np.diff(gaps) > 1) + 1)
        ranges = [f"from {times[s[0]]:.2f} s to {times[s[-1]]:.2f} s" for s in stretches]
        raise SystemExit("no pose " + ", ".join(ranges))
    return times[kept], points[kept], visibility[kept]


def animation_file(rows, joints):
    """The animation file holding `joints` as its columns."""
    columns = [JOINTS.index(joint) for joint in joints]
    return json.dumps(
        {
            "version": 1,
            "fps": FPS,
            "joints": joints,
            "rows": np.round(rows[:, columns], 5).tolist(),
        },
        allow_nan=False,
    )


def read_poses(video, start, end):
    """Poses between `start` and `end`: seconds, pixels and visibility, NaN where none was found."""
    import cv2
    import mediapipe
    from mediapipe.tasks.python import BaseOptions
    from mediapipe.tasks.python.vision import (
        PoseLandmarker,
        PoseLandmarkerOptions,
        RunningMode,
    )

    capture = cv2.VideoCapture(video)
    if not capture.isOpened():
        raise SystemExit(f"cannot read {video}")
    options = PoseLandmarkerOptions(
        base_options=BaseOptions(model_asset_path=str(MODEL)),
        running_mode=RunningMode.VIDEO,
        num_poses=1,
    )
    times, points, visibility = [], [], []
    stamp = -1
    with PoseLandmarker.create_from_options(options) as landmarker:
        while True:
            read, frame = capture.read()
            seconds = capture.get(cv2.CAP_PROP_POS_MSEC) / 1000.0
            if not read or seconds > end:
                break
            if seconds < start:
                continue
            # MediaPipe needs timestamps that always increase, in whole milliseconds.
            stamp = max(int(seconds * 1000.0), stamp + 1)
            rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
            image = mediapipe.Image(image_format=mediapipe.ImageFormat.SRGB, data=rgb)
            poses = landmarker.detect_for_video(image, stamp).pose_landmarks

            height, width = frame.shape[:2]
            times.append(seconds)
            if poses:
                points.append([(mark.x * width, mark.y * height) for mark in poses[0]])
                visibility.append([mark.visibility for mark in poses[0]])
            else:
                points.append(np.full((LANDMARKS, 2), np.nan))
                visibility.append(np.full(LANDMARKS, np.nan))
    capture.release()
    if not times:
        raise SystemExit(f"no frames in {video} between {start} s and {end} s")
    return np.array(times), np.array(points), np.array(visibility)


def require_no_network(interfaces=Path("/proc/net/dev")):
    """Stops unless loopback is the only interface: MediaPipe contacts Google without being asked."""
    names = [line.split(":")[0].strip() for line in interfaces.read_text().splitlines()[2:]]
    if set(names) - {"lo"}:
        raise SystemExit("run this through tools/mocap/mocap, which cuts the network")


def joint_list(text):
    joints = text.split(",")
    unknown = sorted(set(joints) - set(JOINTS))
    if unknown or len(set(joints)) < len(joints):
        raise argparse.ArgumentTypeError(f"pick each joint once, from {', '.join(JOINTS)}")
    return joints


def parse_args(arguments):
    parser = argparse.ArgumentParser(prog="tools/mocap/mocap", description=__doc__)
    parser.add_argument("video")
    parser.add_argument("output")
    parser.add_argument("--joints", type=joint_list, default=JOINTS, help="columns, as a,b")
    parser.add_argument("--start", type=float, default=0.0, help="cut, in seconds")
    parser.add_argument("--end", type=float, default=math.inf, help="cut, in seconds")
    args = parser.parse_args(arguments)
    if args.start >= args.end:
        parser.error("--start must come before --end")
    return args


def main():
    args = parse_args(sys.argv[1:])
    require_no_network()

    times, points, visibility = read_poses(args.video, args.start, args.end)
    frames = len(times)
    times, points, visibility = trim(times, points, visibility)
    rows = rotations(times, points, visibility)
    Path(args.output).write_text(animation_file(rows, args.joints))

    print(f"duration: {times[-1] - times[0]:.2f} s, {len(rows)} rows at {FPS} fps")
    print(f"frames with no pose: {frames - len(times)} of {frames}")
    print("largest step per joint:")
    steps = dict(zip(JOINTS, np.degrees(largest_steps(rows))))
    for joint in args.joints:
        print(f"  {joint}: {steps[joint]:.1f}°")


if __name__ == "__main__":
    main()
