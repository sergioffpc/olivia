#!/usr/bin/env python3
"""Acts out every card of cartoes_sons_e_movimentos.pdf into ../assets. Needs only the standard library.

Movement cards become animation files by hand, with no camera: rows of radians at 30 fps, clockwise,
relative to the parent, where zero is the pose drawn on the sheet. Sound cards are synthesised at
22050 Hz, standing in for a child's voice.
"""

import json
import math
import random
import struct
import wave
from pathlib import Path

ASSETS = Path(__file__).resolve().parent.parent / "assets"
FPS = 30
RATE = 22050
RANDOM = random.Random(7)
TAU = math.tau

BODY = ["root", "neck", "shoulder_left", "elbow_left", "shoulder_right", "elbow_right",
        "hip_left", "knee_left", "hip_right", "knee_right"]
# A hanging arm turns outward: clockwise on the viewer's left, anticlockwise on the right.
ARMS_UP = 2.5


def ease(t):
    t = min(max(t, 0.0), 1.0)
    return t * t * (3 - 2 * t)


def write_animation(path, seconds, pose, joints=BODY):
    """Samples `pose(t)` with t from 0 towards 1. A loop leaves t = 1 to the closing interval."""
    frames = round(seconds * FPS)
    rows = []
    for frame in range(frames):
        values = pose(frame / frames)
        rows.append([round(values.get(joint, 0.0), 3) for joint in joints])
    document = {"version": 1, "fps": FPS, "joints": joints, "rows": rows}
    target = ASSETS / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(document) + "\n")


def idle(t):
    breath = math.sin(TAU * t)
    return {"neck": 0.05 * math.sin(TAU * t + 1), "shoulder_left": 0.05 + 0.04 * breath,
            "shoulder_right": -0.05 - 0.04 * breath, "elbow_left": 0.08, "elbow_right": -0.08}


def walk(t):
    swing = math.sin(TAU * t)
    lift_left = max(0.0, math.sin(TAU * t + 1.2))
    lift_right = max(0.0, math.sin(TAU * t + 1.2 + math.pi))
    return {"root": 0.05, "neck": -0.05 + 0.03 * math.sin(2 * TAU * t),
            "hip_left": -0.55 * swing, "knee_left": 0.15 + 0.9 * lift_left,
            "hip_right": 0.55 * swing, "knee_right": 0.15 + 0.9 * lift_right,
            "shoulder_left": 0.45 * swing, "elbow_left": -0.3,
            "shoulder_right": -0.45 * swing, "elbow_right": -0.3}


def jump(t):
    crouch = 1 - ease(t / 0.25)
    up = ease((t - 0.2) / 0.4)
    return {"neck": -0.15 * up, "hip_left": 0.3 * up + 0.35 * crouch, "knee_left": 0.7 * up + 0.8 * crouch,
            "hip_right": -0.55 * up - 0.35 * crouch, "knee_right": 1.0 * up + 0.8 * crouch,
            "shoulder_left": ARMS_UP * up, "shoulder_right": -ARMS_UP * up}


def double_jump(t):
    tuck = math.sin(math.pi * t)
    return {"root": TAU * ease(t), "hip_left": 0.9 * tuck, "knee_left": 1.5 * tuck, "hip_right": -0.9 * tuck,
            "knee_right": 1.5 * tuck, "shoulder_left": 1.2, "shoulder_right": -1.2, "elbow_left": 0.5 * tuck,
            "elbow_right": -0.5 * tuck}


def shoot(t):
    aim = ease(t / 0.3)
    kick = max(0.0, 1 - abs(t - 0.45) / 0.15)
    lower = ease((t - 0.75) / 0.25)
    # The launcher's down, a quarter turn, is where the shot leaves.
    return {"shoulder_right": (-math.pi / 2 - 0.15 * kick) * aim * (1 - lower), "elbow_right": 0.2 * kick}


def hurt(t):
    hit = math.sin(math.pi * min(t / 0.35, 1.0))
    settle = 1 - ease((t - 0.35) / 0.65)
    shake = math.sin(4 * TAU * t) * settle
    return {"root": -0.45 * hit, "neck": 0.3 * shake, "shoulder_left": 1.7 * settle, "shoulder_right": -1.7 * settle,
            "elbow_left": 0.7 * shake, "elbow_right": -0.7 * shake, "hip_left": 0.4 * settle, "hip_right": -0.4 * settle,
            "knee_left": 0.5 * settle, "knee_right": 0.5 * settle}


def celebrate(hops):
    def pose(t):
        hop = abs(math.sin(hops * math.pi * t))
        wave_arm = 0.35 * math.sin(2 * hops * TAU * t)
        return {"neck": 0.15 * math.sin(hops * TAU * t), "root": 0.08 * math.sin(hops * TAU * t),
                "shoulder_left": ARMS_UP + wave_arm, "shoulder_right": -ARMS_UP + wave_arm,
                "elbow_left": 0.3 * hop, "elbow_right": -0.3 * hop,
                "hip_left": 0.4 * hop, "knee_left": 0.8 * hop, "hip_right": -0.4 * hop, "knee_right": 0.8 * hop}
    return pose


def help_me(t):
    wave_arm = 0.5 * math.sin(2 * TAU * t)
    return {"neck": 0.12 * math.sin(TAU * t), "root": 0.04 * math.sin(TAU * t),
            "shoulder_right": -ARMS_UP + wave_arm, "elbow_right": -0.6 * math.sin(2 * TAU * t + 0.8),
            "shoulder_left": ARMS_UP - wave_arm, "elbow_left": 0.6 * math.sin(2 * TAU * t + 0.8),
            "hip_left": 0.08 * math.sin(TAU * t), "hip_right": 0.08 * math.sin(TAU * t)}


def animations():
    write_animation("hero/anim_idle.json", 2.0, idle)
    write_animation("hero/anim_walk.json", 0.8, walk)
    write_animation("hero/anim_jump.json", 0.5, jump)
    write_animation("hero/anim_double_jump.json", 0.6, double_jump)
    # An overlay, like the spec's shoot: the rest of the body keeps doing what it was doing.
    write_animation("hero/anim_shoot.json", 0.4, shoot, ["shoulder_right", "elbow_right"])
    write_animation("hero/anim_hurt.json", 0.7, hurt)
    write_animation("hero/anim_win.json", 2.0, celebrate(4))
    write_animation("princess/anim_help.json", 1.2, help_me)
    write_animation("princess/anim_win.json", 2.0, celebrate(3))


def sine(phase):
    return math.sin(TAU * phase)


def square(phase):
    return 1.0 if phase % 1.0 < 0.5 else -1.0


def tone(seconds, frequency, shape=sine, volume=0.6, attack=0.005, release=0.04):
    """A note whose `frequency` may be a function of t from 0 to 1."""
    samples = []
    phase = 0.0
    total = int(seconds * RATE)
    for i in range(total):
        t = i / total
        phase += (frequency(t) if callable(frequency) else frequency) / RATE
        at = i / RATE
        envelope = min(1.0, at / attack) * min(1.0, (seconds - at) / release)
        samples.append(shape(phase) * envelope * volume)
    return samples


def noise(seconds, volume=0.5, smooth=0.0):
    samples = []
    held = 0.0
    total = int(seconds * RATE)
    for i in range(total):
        held = held * smooth + RANDOM.uniform(-1, 1) * (1 - smooth)
        samples.append(held * volume * (1 - i / total))
    return samples


def mix(*tracks):
    out = [0.0] * max(len(track) for track in tracks)
    for track in tracks:
        for i, value in enumerate(track):
            out[i] += value
    return out


def write_sound(name, samples):
    target = ASSETS / "sfx" / f"{name}.wav"
    target.parent.mkdir(parents=True, exist_ok=True)
    frames = b"".join(struct.pack("<h", int(max(-1.0, min(1.0, s)) * 32000)) for s in samples)
    with wave.open(str(target), "wb") as file:
        file.setnchannels(1)
        file.setsampwidth(2)
        file.setframerate(RATE)
        file.writeframes(frames)


def melody(notes, seconds, shape=square, volume=0.25):
    return [s for note in notes for s in tone(seconds, note, shape, volume)]


def sounds():
    write_sound("jump", tone(0.22, lambda t: 300 + 600 * t, square, 0.3))
    fart = tone(0.65, lambda t: 95 - 40 * t + 18 * sine(22 * t), lambda p: 2 * (p % 1.0) - 1, 0.55)
    write_sound("fart", mix(fart, noise(0.65, 0.25, 0.8)))
    write_sound("shoot", tone(0.2, lambda t: 1300 - 1050 * t, square, 0.3))
    write_sound("hit", mix(noise(0.2, 0.7, 0.3), tone(0.18, lambda t: 240 - 150 * t, square, 0.35)))
    write_sound("pickup", melody([988, 1319], 0.09, square, 0.3))
    write_sound("hurt", tone(0.4, lambda t: 560 - 330 * t + 40 * sine(14 * t), square, 0.35))
    write_sound("gameover", melody([440, 415, 392, 330], 0.32, square, 0.3))
    # Three high cries, "so-co-rro!".
    cry = lambda base: tone(0.26, lambda t: base + 70 * sine(9 * t), sine, 0.5) + [0.0] * 1100
    write_sound("help", cry(740) + cry(830) + cry(990))
    write_sound("win", melody([523, 659, 784, 1047, 784, 1047, 1319], 0.16, square, 0.3))

    # Eight bars of a xylophone over a plodding bass, on a pentatonic scale so nothing can clash.
    tune = [523, 587, 659, 784, 659, 587, 523, 440, 523, 659, 784, 880, 784, 659, 587, 659,
            880, 784, 659, 784, 659, 587, 523, 587, 659, 587, 523, 440, 392, 440, 523, 523]
    roots = [131, 196, 110, 165, 131, 196, 147, 196]
    beat = 0.25
    lead = [s for note in tune for s in tone(beat, note, sine, 0.28, release=0.18)]
    bass = [s for root in roots for s in tone(beat * 2, root, sine, 0.3, release=0.1) * 2]
    length = min(len(lead), len(bass))
    write_sound("music", mix(lead[:length], bass[:length]))


if __name__ == "__main__":
    animations()
    sounds()
