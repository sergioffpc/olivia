"""The angle maths on synthetic poses, with no MediaPipe and no video."""

import json
import math
import subprocess
import sys
from pathlib import Path

import numpy as np
import pytest

import mocap

# Landmark indices in MediaPipe's pose model.
NOSE, EAR_LEFT, EAR_RIGHT = 0, 7, 8
LEFT = {"shoulder": 11, "elbow": 13, "wrist": 15, "hip": 23, "knee": 25, "ankle": 27}
RIGHT = {"shoulder": 12, "elbow": 14, "wrist": 16, "hip": 24, "knee": 26, "ankle": 28}


def toward(start, degrees, length=100.0):
    """The point `length` pixels from `start`, clockwise from +x with y down."""
    radians = np.radians(degrees)
    return start + length * np.array([np.cos(radians), np.sin(radians)])


def pose(
    torso=-90.0,
    head=-90.0,
    near_arm=(90.0, 90.0),
    near_leg=(90.0, 90.0),
    far_arm=(90.0, 90.0),
    far_leg=(90.0, 90.0),
):
    """A person facing right, near side on MediaPipe's right, from segment angles in degrees."""
    points = np.zeros((33, 2))
    hips = np.array([500.0, 500.0])
    shoulders = toward(hips, torso, 200.0)
    for side, arm, leg in ((RIGHT, near_arm, near_leg), (LEFT, far_arm, far_leg)):
        points[side["hip"]] = hips
        points[side["shoulder"]] = shoulders
        points[side["elbow"]] = toward(shoulders, arm[0])
        points[side["wrist"]] = toward(points[side["elbow"]], arm[1])
        points[side["knee"]] = toward(hips, leg[0])
        points[side["ankle"]] = toward(points[side["knee"]], leg[1])
    ears = toward(shoulders, head, 50.0)
    points[EAR_LEFT] = points[EAR_RIGHT] = ears
    points[NOSE] = ears + (20.0, 0.0)
    return points


def clip(*poses, fps=30.0, near=RIGHT):
    """Times, points and visibility for one clip, with `near` the more visible side."""
    points = np.array(poses)
    visibility = np.full((len(poses), 33), 0.2)
    visibility[:, list(near.values())] = 0.9
    return np.arange(len(poses)) / fps, points, visibility


def column(rows, joint):
    return rows[:, mocap.JOINTS.index(joint)]


def test_the_pose_on_the_sheet_is_all_zeros():
    rows = mocap.rotations(*clip(*[pose()] * 10))

    assert rows.shape == (10, 10)
    assert np.allclose(rows, 0.0)


def assert_degrees(rows, joint, expected):
    assert np.allclose(np.degrees(column(rows, joint)), expected)


def test_leaning_forward_turns_the_root_clockwise_and_keeps_hanging_arms_in_the_world():
    rows = mocap.rotations(*clip(*[pose(torso=-80.0)] * 10))

    assert_degrees(rows, "root", 10.0)
    assert_degrees(rows, "shoulder_right", -10.0)
    assert_degrees(rows, "hip_right", -10.0)
    assert_degrees(rows, "neck", -10.0)


def test_an_arm_raised_in_front_turns_the_shoulder_anticlockwise():
    rows = mocap.rotations(*clip(*[pose(near_arm=(0.0, 0.0))] * 10))

    assert_degrees(rows, "shoulder_right", -90.0)
    assert_degrees(rows, "elbow_right", 0.0)
    assert_degrees(rows, "shoulder_left", 0.0)


def test_the_more_visible_side_is_written_as_right():
    poses = [pose(near_arm=(0.0, 0.0))] * 10

    rows = mocap.rotations(*clip(*poses, near=LEFT))

    assert_degrees(rows, "shoulder_right", 0.0)
    assert_degrees(rows, "shoulder_left", -90.0)


def test_a_video_filmed_facing_left_comes_out_facing_right():
    facing_right = pose(torso=-80.0, near_arm=(0.0, 45.0), near_leg=(60.0, 120.0))
    facing_left = facing_right * (-1.0, 1.0) + (1000.0, 0.0)

    rows = mocap.rotations(*clip(*[facing_left] * 10))

    assert np.allclose(rows, mocap.rotations(*clip(*[facing_right] * 10)))
    assert_degrees(rows, "root", 10.0)


def test_elbows_and_knees_bend_one_way_only():
    bent = mocap.rotations(*clip(*[pose(near_arm=(90.0, 0.0), near_leg=(90.0, 180.0))] * 10))
    backwards = mocap.rotations(*clip(*[pose(near_arm=(90.0, 100.0), near_leg=(90.0, 80.0))] * 10))

    assert_degrees(bent, "elbow_right", -90.0)
    assert_degrees(bent, "knee_right", 90.0)
    assert_degrees(backwards, "elbow_right", 0.0)
    assert_degrees(backwards, "knee_right", 0.0)


def test_a_one_frame_spike_is_removed():
    poses = [pose()] * 20
    poses[10] = pose(near_arm=(0.0, 0.0))

    rows = mocap.rotations(*clip(*poses))

    assert np.abs(np.degrees(column(rows, "shoulder_right"))).max() < 0.01


def test_jitter_across_half_a_turn_stays_small():
    # A hanging arm sits half a turn from the torso, where raw joint angles flip between ±180°.
    poses = [pose(near_arm=(90.0 + 2.0 * (-1) ** frame, 90.0)) for frame in range(20)]

    rows = mocap.rotations(*clip(*poses))

    assert np.degrees(np.abs(column(rows, "shoulder_right"))).max() <= 2.0


def test_it_resamples_to_30_fps_by_time():
    arms = np.linspace(90.0, 30.0, 61)
    poses = [pose(near_arm=(arm, arm)) for arm in arms]

    rows = mocap.rotations(*clip(*poses, fps=60.0))

    assert len(rows) == 31
    # The filter only flattens the ends of the ramp.
    shoulder = np.degrees(column(rows, "shoulder_right"))
    assert np.allclose(shoulder[3:-3], np.linspace(0.0, -60.0, 31)[3:-3], atol=0.5)


def test_a_clip_with_uneven_frame_times_keeps_its_duration():
    times, points, visibility = clip(*[pose()] * 4)
    times = np.array([0.0, 0.1, 0.5, 1.0])

    assert len(mocap.rotations(times, points, visibility)) == 31


def test_a_motion_past_half_a_turn_resamples_along_the_short_way():
    # The arm swings up past overhead; between frames it must not spin back through the body.
    arms = [-75.0, -85.0, -95.0, -105.0, -115.0, -125.0, -135.0, -145.0, -155.0, -165.0]
    poses = [pose(near_arm=(arm, arm)) for arm in arms]

    rows = mocap.rotations(*clip(*poses, fps=15.0))

    steps = np.degrees(mocap.largest_steps(rows))
    assert steps[mocap.JOINTS.index("shoulder_right")] < 8.0


def test_a_single_pose_gives_a_single_row():
    assert mocap.rotations(*clip(pose())).shape == (1, 10)


def missing(times, points, visibility, frames):
    """The clip with no pose in `frames`, as `read_poses` leaves those frames."""
    points, visibility = points.copy(), visibility.copy()
    points[frames] = np.nan
    visibility[frames] = np.nan
    return times, points, visibility


def test_frames_with_no_pose_are_trimmed_at_each_end():
    times, points, visibility = missing(*clip(*[pose()] * 10), [0, 1, 9])

    trimmed_times, trimmed_points, trimmed_visibility = mocap.trim(times, points, visibility)

    assert np.allclose(trimmed_times, times[2:9])
    assert not np.isnan(trimmed_points).any()
    assert not np.isnan(trimmed_visibility).any()


def test_a_stretch_with_no_pose_in_the_middle_is_an_error_naming_its_time_range():
    data = missing(*clip(*[pose()] * 30), [9, 10, 11, 12])

    with pytest.raises(SystemExit, match=r"no pose from 0\.30 s to 0\.40 s"):
        mocap.trim(*data)


def test_a_clip_with_no_pose_at_all_is_an_error():
    data = missing(*clip(*[pose()] * 3), [0, 1, 2])

    with pytest.raises(SystemExit, match="no pose"):
        mocap.trim(*data)


def test_low_visibility_rejects_nothing():
    poses = [pose(far_arm=(0.0, 0.0))] * 10
    times, points, visibility = clip(*poses)
    visibility[:, list(LEFT.values())] = 0.0

    rows = mocap.rotations(times, points, visibility)

    assert_degrees(rows, "shoulder_left", -90.0)


def test_the_largest_step_goes_the_short_way_across_half_a_turn():
    rows = np.radians([[179.0], [-179.0], [-178.0]])

    assert np.allclose(np.degrees(mocap.largest_steps(rows)), [2.0])


def test_the_file_holds_the_chosen_columns_at_30_fps():
    rows = np.zeros((2, 10))
    rows[:, mocap.JOINTS.index("elbow_right")] = [0.1, 0.2]
    rows[:, mocap.JOINTS.index("shoulder_right")] = [0.3, 0.4]

    file = json.loads(mocap.animation_file(rows, ["shoulder_right", "elbow_right"]))

    assert file == {
        "version": 1,
        "fps": 30,
        "joints": ["shoulder_right", "elbow_right"],
        "rows": [[0.3, 0.1], [0.4, 0.2]],
    }


def test_unknown_joints_are_rejected():
    with pytest.raises(SystemExit):
        mocap.parse_args(["in.mp4", "out.json", "--joints", "neck,wrist"])


def test_without_options_it_takes_every_joint_and_the_whole_video():
    args = mocap.parse_args(["in.mp4", "out.json"])

    assert args.joints == mocap.JOINTS
    assert (args.start, args.end) == (0.0, math.inf)


def test_a_cut_that_ends_before_it_starts_is_rejected():
    with pytest.raises(SystemExit):
        mocap.parse_args(["in.mp4", "out.json", "--start", "2", "--end", "1"])


def test_importing_the_tool_does_not_load_mediapipe():
    code = "import sys, mocap; sys.exit('mediapipe' in sys.modules or 'cv2' in sys.modules)"
    folder = Path(mocap.__file__).parent

    assert subprocess.run([sys.executable, "-c", code], cwd=folder).returncode == 0


def test_it_refuses_to_read_a_video_with_a_network(tmp_path):
    interfaces = tmp_path / "dev"
    interfaces.write_text("Inter-| Receive\n face |bytes\n    lo: 0\n  eth0: 0\n")

    with pytest.raises(SystemExit, match="tools/mocap/mocap"):
        mocap.require_no_network(interfaces)


def test_loopback_alone_is_no_network(tmp_path):
    interfaces = tmp_path / "dev"
    interfaces.write_text("Inter-| Receive\n face |bytes\n    lo: 0\n")

    mocap.require_no_network(interfaces)
