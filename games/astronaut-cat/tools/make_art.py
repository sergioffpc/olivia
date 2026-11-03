#!/usr/bin/env python3
"""Draws every sheet of folhas_para_desenhar.pdf the way a child would, into ../assets.

Needs Pillow. The same seed gives the same drawings.

Character parts follow the humanoid convention of the motion capture spec (#82): 8 pixels per
millimetre of sheet, arms and legs cut at the sheet's tick with a round seam. Single drawings are
written smaller, because nothing measures them.
"""

import json
import math
import random
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter

ASSETS = Path(__file__).resolve().parent.parent / "assets"
RANDOM = random.Random(20261102)

# Boxes measured on the sheets, in pixels at 8 pixels per millimetre.
HEAD = (511, 456)
HEAD_DOT = (256, 411)
TORSO = (484, 654)
ARM = (325, 654)
ARM_DOT = (162, 46)
ARM_TICK = 347
LEG = (383, 570)
LEG_DOT = (192, 46)
LEG_TICK = 307
WEAPON = (1333, 740)
WEAPON_DOT = (113, 370)
# The weapon box is twice as long as a leg, so at the shared scale the launcher would reach the floor.
WEAPON_SCALE = 0.45

INK = (35, 30, 40)
WHITE = (250, 250, 245)
GREY = (150, 155, 170)
ORANGE = (240, 140, 40)
PINK = (245, 150, 180)
GREEN = (90, 190, 80)
DARK_GREEN = (40, 120, 60)
BLUE = (70, 120, 225)
SKY = (120, 195, 245)
RED = (225, 60, 55)
YELLOW = (250, 210, 60)
PURPLE = (150, 85, 195)
BROWN = (170, 110, 60)


def wobble(points, amount=6.0, closed=False):
    """Resamples a line into many points and shakes them, as an unsteady hand does."""
    if closed:
        points = [*points, points[0]]
    shaken = []
    drift = [0.0, 0.0]
    for (ax, ay), (bx, by) in zip(points, points[1:]):
        steps = max(2, int(math.hypot(bx - ax, by - ay) / 14))
        for i in range(steps):
            t = i / steps
            drift[0] = drift[0] * 0.8 + RANDOM.uniform(-1, 1) * amount * 0.35
            drift[1] = drift[1] * 0.8 + RANDOM.uniform(-1, 1) * amount * 0.35
            shaken.append((ax + (bx - ax) * t + drift[0], ay + (by - ay) * t + drift[1]))
    shaken.append(points[-1])
    return shaken


def stroke(image, points, color=INK, width=14, closed=False):
    """Crayon: the same line twice, never quite on top of itself, with round ends."""
    draw = ImageDraw.Draw(image)
    for _ in range(2):
        shaken = wobble(points, closed=closed)
        draw.line(shaken, fill=(*color, 255), width=width, joint="curve")
        for x, y in (shaken[0], shaken[-1]):
            draw.ellipse((x - width / 2, y - width / 2, x + width / 2, y + width / 2), fill=(*color, 255))


def ellipse(cx, cy, rx, ry, steps=40, turn=0.0):
    """The points around an ellipse, turned clockwise by `turn`."""
    points = []
    for i in range(steps):
        a = 2 * math.pi * i / steps
        x, y = rx * math.cos(a), ry * math.sin(a)
        points.append((cx + x * math.cos(turn) - y * math.sin(turn), cy + x * math.sin(turn) + y * math.cos(turn)))
    return points


def fill(image, polygon, color, outline=INK, width=14):
    """Colours a shape: a pale coat so paper never shows through, scribbles that spill a little."""
    size = image.size
    mask = Image.new("L", size, 0)
    ImageDraw.Draw(mask).polygon(polygon, fill=255)
    coat = Image.new("RGBA", size, (*tuple(min(255, c + (255 - c) // 3) for c in color), 255))
    image.paste(coat, (0, 0), mask)

    scribble = Image.new("L", size, 0)
    draw = ImageDraw.Draw(scribble)
    xs = [p[0] for p in polygon]
    ys = [p[1] for p in polygon]
    left, right, top, bottom = min(xs) - 20, max(xs) + 20, min(ys) - 20, max(ys) + 20
    slope = RANDOM.uniform(-0.5, 0.5)
    y = top - abs(slope) * (right - left)
    forward = True
    line = []
    while y < bottom + abs(slope) * (right - left):
        ends = (left, right) if forward else (right, left)
        line.append((ends[0], y + slope * (ends[0] - left) + RANDOM.uniform(-6, 6)))
        line.append((ends[1], y + 8 + slope * (ends[1] - left) + RANDOM.uniform(-6, 6)))
        y += RANDOM.uniform(16, 24)
        forward = not forward
    draw.line(line, fill=255, width=18)
    spill = mask.filter(ImageFilter.MaxFilter(11))
    scribble = ImageChops.multiply(scribble, spill)
    layer = Image.new("RGBA", size, (*color, 255))
    image.paste(layer, (0, 0), scribble)
    if outline:
        stroke(image, polygon, outline, width, closed=True)


def dot(image, x, y, radius, color=INK):
    fill(image, ellipse(x, y, radius, radius, 16), color, outline=None)


def canvas(size):
    return Image.new("RGBA", size, (0, 0, 0, 0))


def save(image, path):
    target = ASSETS / path
    target.parent.mkdir(parents=True, exist_ok=True)
    image.save(target)


def round_cut(limb, tick):
    """Cuts a limb at its tick: both halves keep a half disc of the drawing around the joint."""
    alpha = limb.getchannel("A")
    row = alpha.crop((0, tick, limb.width, tick + 1)).point(lambda a: 255 if a > 40 else 0)
    box = row.getbbox() or (0, 0, limb.width, 1)
    middle = (box[0] + box[2]) // 2
    radius = (box[2] - box[0]) // 2 + 2

    disc = Image.new("L", limb.size, 0)
    ImageDraw.Draw(disc).ellipse((middle - radius, tick - radius, middle + radius, tick + radius), fill=255)
    upper_mask = disc.copy()
    ImageDraw.Draw(upper_mask).rectangle((0, 0, limb.width, tick), fill=255)
    lower_mask = disc.copy()
    ImageDraw.Draw(lower_mask).rectangle((0, tick, limb.width, limb.height), fill=255)

    upper = limb.copy()
    upper.putalpha(ImageChops.multiply(alpha, upper_mask))
    lower = limb.copy()
    lower.putalpha(ImageChops.multiply(alpha, lower_mask))
    top = tick - radius
    upper = upper.crop((0, 0, limb.width, tick + radius))
    lower = lower.crop((0, top, limb.width, limb.height))
    return upper, lower, (middle, tick), (middle, tick - top)


def character(folder, head, torso, arm, leg, launcher=None):
    """Cuts the drawings into the parts of the convention and writes `skeleton.json`."""
    save(head, f"{folder}/head.png")
    save(torso, f"{folder}/torso.png")

    width, height = TORSO
    joints = {
        "root": {"part": "torso", "point": [width // 2, height // 2]},
        "neck": {"parent": "torso", "parent_point": [width // 2, 34], "part": "head", "point": list(HEAD_DOT)},
    }
    shoulders = {"left": [44, 96], "right": [width - 44, 96]}
    hips = {"left": [width // 2 - 95, height - 50], "right": [width // 2 + 95, height - 50]}
    hand = None
    for side, flip in (("left", False), ("right", True)):
        drawing = arm.transpose(Image.Transpose.FLIP_LEFT_RIGHT) if flip else arm
        upper, lower, elbow_up, elbow_low = round_cut(drawing, ARM_TICK)
        save(upper, f"{folder}/upper_arm_{side}.png")
        save(lower, f"{folder}/lower_arm_{side}.png")
        dot_x = ARM[0] - ARM_DOT[0] if flip else ARM_DOT[0]
        joints[f"shoulder_{side}"] = {
            "parent": "torso", "parent_point": shoulders[side],
            "part": f"upper_arm_{side}", "point": [dot_x, ARM_DOT[1]],
        }
        joints[f"elbow_{side}"] = {
            "parent": f"upper_arm_{side}", "parent_point": list(elbow_up),
            "part": f"lower_arm_{side}", "point": list(elbow_low),
        }
        hand = [elbow_low[0], lower.height - 40]

        drawing = leg.transpose(Image.Transpose.FLIP_LEFT_RIGHT) if flip else leg
        upper, lower, knee_up, knee_low = round_cut(drawing, LEG_TICK)
        save(upper, f"{folder}/upper_leg_{side}.png")
        save(lower, f"{folder}/lower_leg_{side}.png")
        dot_x = LEG[0] - LEG_DOT[0] if flip else LEG_DOT[0]
        joints[f"hip_{side}"] = {
            "parent": "torso", "parent_point": hips[side],
            "part": f"upper_leg_{side}", "point": [dot_x, LEG_DOT[1]],
        }
        joints[f"knee_{side}"] = {
            "parent": f"upper_leg_{side}", "parent_point": list(knee_up),
            "part": f"lower_leg_{side}", "point": list(knee_low),
        }

    order = ["shoulder_left", "elbow_left", "hip_left", "knee_left", "hip_right", "knee_right", "root",
             "shoulder_right", "elbow_right", "neck"]
    if launcher:
        # Turned a quarter clockwise to lie along the forearm: what fired right now fires down.
        small = launcher.resize((round(WEAPON[0] * WEAPON_SCALE), round(WEAPON[1] * WEAPON_SCALE)))
        turned = small.transpose(Image.Transpose.ROTATE_270)
        save(turned, f"{folder}/launcher.png")
        x, y = WEAPON_DOT[0] * WEAPON_SCALE, WEAPON_DOT[1] * WEAPON_SCALE
        joints["hand_right"] = {
            "parent": "lower_arm_right", "parent_point": hand,
            "part": "launcher", "point": [round(small.height - y), round(x)],
        }
        order.insert(order.index("elbow_right") + 1, "hand_right")

    entries = [{"name": name, **joints[name]} for name in order]
    (ASSETS / folder / "skeleton.json").write_text(json.dumps({"version": 1, "joints": entries}, indent=2) + "\n")


def hero():
    """The astronaut cat."""
    w, h = HEAD
    head = canvas(HEAD)
    stroke_ears = [[(120, 190), (90, 45), (215, 120)], [(300, 120), (420, 45), (392, 190)]]
    for ear in stroke_ears:
        fill(head, ear, ORANGE)
    fill(head, ellipse(w / 2, 235, 195, 170), ORANGE)
    for x in (178, 330):
        fill(head, ellipse(x, 195, 38, 46), WHITE)
        dot(head, x + 8, 200, 18)
    fill(head, [(238, 262), (276, 262), (257, 290)], PINK)
    stroke(head, [(195, 320), (230, 340), (257, 300), (285, 340), (320, 320)], width=10)
    for side in (-1, 1):
        for dy in (-22, 12):
            stroke(head, [(w / 2 + side * 110, 280), (w / 2 + side * 235, 280 + dy)], width=7)
    stroke(head, ellipse(w / 2, 225, 240, 212), SKY, 16, closed=True)
    stroke(head, [(120, 60), (155, 40)], WHITE, 12)

    w, h = TORSO
    torso = canvas(TORSO)
    suit = [(70, 40), (w - 70, 40), (w - 40, h - 30), (40, h - 30)]
    fill(torso, suit, WHITE)
    fill(torso, [(150, 170), (w - 150, 170), (w - 150, 360), (150, 360)], BLUE)
    dot(torso, 200, 225, 24, RED)
    dot(torso, 285, 225, 24, YELLOW)
    stroke(torso, [(190, 310), (300, 310)], WHITE, 12)
    stroke(torso, [(60, h - 150), (w - 60, h - 150)], ORANGE, 26)

    w, h = ARM
    arm = canvas(ARM)
    fill(arm, [(95, 26), (230, 26), (222, h - 95), (102, h - 95)], WHITE)
    stroke(arm, [(102, 330), (225, 335)], ORANGE, 22)
    fill(arm, ellipse(w / 2, h - 62, 72, 56), ORANGE)

    w, h = LEG
    leg = canvas(LEG)
    fill(leg, [(100, 26), (283, 26), (270, h - 110), (112, h - 110)], WHITE)
    stroke(leg, [(112, 300), (272, 305)], ORANGE, 22)
    fill(leg, [(82, h - 112), (292, h - 112), (332, h - 26), (72, h - 26)], BLUE)

    w, h = WEAPON
    launcher = canvas(WEAPON)
    # A fish cannon: a yellow barrel with a fish's mouth.
    fill(launcher, [(60, 300), (1180, 200), (1200, 540), (60, 450)], YELLOW, width=22)
    fill(launcher, ellipse(1180, 370, 90, 190), RED, width=22)
    dot(launcher, 1200, 370, 90, INK)
    for x in (360, 620, 880):
        stroke(launcher, [(x, 270), (x + 10, 490)], ORANGE, 30)
    fill(launcher, [(170, 440), (300, 440), (270, 680), (180, 680)], BROWN, width=22)
    character("hero", head, torso, arm, leg, launcher)


def princess():
    """The robot grandmother."""
    w, h = HEAD
    head = canvas(HEAD)
    fill(head, [(70, 110), (w - 70, 110), (w - 80, 400), (80, 400)], GREY)
    # A bun of silver hair, with an antenna through it.
    fill(head, ellipse(w / 2, 90, 130, 62), WHITE)
    stroke(head, [(w / 2, 40), (w / 2 + 12, 12)], RED, 14)
    for x in (165, 345):
        fill(head, ellipse(x, 215, 52, 52), YELLOW)
        dot(head, x, 218, 22)
    stroke(head, [(110, 215), (130, 215)], width=12)
    stroke(head, [(217, 215), (293, 215)], width=12)
    stroke(head, [(380, 215), (400, 215)], width=12)
    fill(head, [(175, 305), (335, 305), (335, 365), (175, 365)], WHITE)
    for x in (215, 255, 295):
        stroke(head, [(x, 305), (x, 365)], width=9)

    w, h = TORSO
    torso = canvas(TORSO)
    # An apron over a metal body.
    fill(torso, [(80, 40), (w - 80, 40), (w - 50, h - 30), (50, h - 30)], GREY)
    fill(torso, [(135, 200), (w - 135, 200), (w - 90, h - 70), (90, h - 70)], PINK)
    for x, y in ((180, 320), (300, 400), (210, 500), (330, 540)):
        fill(torso, ellipse(x, y, 34, 34, 7), YELLOW, width=9)
    for x in (165, 240, 315):
        dot(torso, x, 110, 22, RED)

    w, h = ARM
    arm = canvas(ARM)
    fill(arm, [(112, 26), (212, 26), (208, h - 110), (116, h - 110)], GREY)
    for y in (160, 240, 420, 500):
        stroke(arm, [(114, y), (210, y)], width=9)
    fill(arm, [(80, h - 120), (245, h - 120), (250, h - 30), (160, h - 60), (76, h - 30)], RED)

    w, h = LEG
    leg = canvas(LEG)
    fill(leg, [(128, 26), (255, 26), (250, h - 110), (134, h - 110)], GREY)
    for y in (150, 220, 390, 450):
        stroke(leg, [(132, y), (252, y)], width=9)
    fill(leg, [(95, h - 112), (288, h - 112), (305, h - 26), (82, h - 26)], PURPLE)
    character("princess", head, torso, arm, leg)


def fx():
    image = canvas((256, 256))
    # The fish the cannon shoots, swimming right.
    fill(image, ellipse(140, 128, 95, 62), SKY, width=10)
    fill(image, [(55, 128), (10, 70), (10, 186)], BLUE, width=10)
    dot(image, 190, 110, 12)
    stroke(image, [(120, 75), (150, 128), (120, 180)], WHITE, 8)
    save(image, "fx/projectile.png")

    image = canvas((384, 384))
    points = []
    for i in range(12):
        a = 2 * math.pi * i / 12
        radius = 165 if i % 2 else 125
        points.append((192 + radius * math.cos(a), 192 + radius * math.sin(a)))
    fill(image, points, (150, 200, 70), outline=DARK_GREEN)
    for x, y in ((120, 150), (240, 130), (195, 260)):
        dot(image, x, y, 20, DARK_GREEN)
    stroke(image, [(90, 250), (110, 230), (130, 250), (150, 230)], DARK_GREEN, 10)
    save(image, "fx/fart.png")


def enemies():
    # The crawling stinky sock, long and low.
    image = canvas((860, 500))
    fill(image, [(30, 300), (560, 280), (600, 90), (780, 90), (790, 330), (700, 460), (60, 460)], WHITE)
    for y in (140, 200):
        stroke(image, [(602, y), (782, y)], RED, 22)
    for x, y in ((130, 380), (280, 400), (430, 370)):
        fill(image, ellipse(x, y, 44, 34, 9), (190, 180, 110), width=9)
    for x in (620, 720):
        fill(image, ellipse(x, 280, 38, 44), YELLOW, width=9)
        dot(image, x - 8, 290, 14)
    stroke(image, [(610, 390), (650, 360), (690, 390), (730, 360)], width=12)
    for x in (120, 300, 480):
        stroke(image, [(x, 240), (x + 28, 190), (x - 12, 150), (x + 18, 100)], DARK_GREEN, 10)
    save(image, "enemy_crawl.png")

    # The round angry doughnut, seen from the front so that it can spin.
    image = canvas((512, 512))
    fill(image, ellipse(256, 256, 232, 232), PINK)
    for a in range(0, 360, 40):
        x = 256 + 182 * math.cos(math.radians(a))
        y = 256 + 182 * math.sin(math.radians(a))
        stroke(image, [(x - 12, y - 12), (x + 12, y + 12)], RANDOM.choice((BLUE, YELLOW, GREEN, RED)), 14)
    fill(image, ellipse(170, 215, 52, 62), WHITE)
    fill(image, ellipse(342, 215, 52, 62), WHITE)
    dot(image, 182, 228, 22)
    dot(image, 330, 228, 22)
    stroke(image, [(110, 130), (215, 175)], width=20)
    stroke(image, [(402, 130), (297, 175)], width=20)
    fill(image, [(170, 330), (342, 330), (256, 410)], INK, outline=None)
    for x in (205, 256, 307):
        fill(image, [(x - 20, 330), (x + 20, 330), (x, 368)], WHITE, outline=None)
    save(image, "enemy_roll.png")

    # The floating broccoli ghost.
    image = canvas((440, 600))
    for cx, cy, r in ((130, 170, 105), (310, 170, 105), (220, 100, 95), (220, 215, 110)):
        fill(image, ellipse(cx, cy, r, r), GREEN, DARK_GREEN)
    fill(image, [(130, 260), (310, 260), (340, 560), (280, 500), (220, 570), (160, 500), (100, 560)], (205, 225, 140),
         DARK_GREEN)
    for x in (178, 262):
        fill(image, ellipse(x, 340, 30, 40), WHITE, width=9)
        dot(image, x, 350, 13)
    fill(image, ellipse(220, 425, 24, 32), INK, outline=None)
    save(image, "enemy_float.png")


def level():
    # The ice cream planet: far away, no ground.
    image = Image.new("RGBA", (1920, 1114), (*PURPLE, 255))
    fill(image, [(-20, -20), (1940, -20), (1940, 1134), (-20, 1134)], (45, 35, 110), outline=None)
    for _ in range(70):
        x, y = RANDOM.uniform(0, 1920), RANDOM.uniform(0, 720)
        stroke(image, [(x - 10, y), (x + 10, y)], YELLOW, 6)
        stroke(image, [(x, y - 10), (x, y + 10)], YELLOW, 6)
    fill(image, ellipse(320, 230, 130, 130), YELLOW)
    fill(image, ellipse(380, 200, 118, 118), (45, 35, 110), outline=None)
    fill(image, ellipse(1500, 260, 180, 180), PINK)
    stroke(image, ellipse(1500, 260, 290, 62, turn=-0.25), SKY, 22, closed=True)
    for x, height, scoop in ((180, 420, PINK), (640, 520, GREEN), (1100, 400, YELLOW), (1680, 480, SKY)):
        top = 1114 - height
        fill(image, [(x - 120, top + 100), (x + 120, top + 100), (x, 1140)], BROWN)
        for i in (-60, 0, 60):
            stroke(image, [(x + i - 50, top + 130), (x + i + 40, top + 340)], (120, 75, 40), 8)
        fill(image, ellipse(x, top + 40, 130, 110), scoop)
    save(image.convert("RGB").convert("RGBA"), "level/background.png")

    # A wafer ground, with melted ice cream on the top line the hero steps on.
    image = canvas((1024, 230))
    fill(image, [(0, 0), (1024, 0), (1024, 230), (0, 230)], (225, 170, 90), outline=None)
    for x in range(0, 1024, 90):
        stroke(image, [(x, 40), (x + 70, 230)], BROWN, 9)
        stroke(image, [(x + 70, 40), (x, 230)], BROWN, 9)
    drips = [(0, 0), (1024, 0), (1024, 44)]
    x = 1024
    while x > 0:
        x -= RANDOM.uniform(50, 110)
        drips += [(x + 30, RANDOM.uniform(60, 110)), (x, 40)]
    fill(image, [*drips, (0, 40)], PINK, outline=None)
    save(image, "level/platform.png")

    # What is picked up: a cheese.
    image = canvas((256, 256))
    fill(image, [(20, 180), (225, 70), (240, 200), (30, 235)], YELLOW, width=10)
    for x, y in ((90, 195), (160, 170), (205, 150)):
        fill(image, ellipse(x, y, 16, 16), ORANGE, BROWN, width=6)
    save(image, "level/coin.png")

    # Where the grandmother is held: only the place, a cage with its bars.
    image = canvas((440, 600))
    stroke(image, ellipse(220, 300, 195, 275), RED, 26, closed=True)
    for x in range(70, 400, 75):
        half = 270 * math.sqrt(max(0.0, 1 - ((x - 220) / 195) ** 2))
        stroke(image, [(x, 300 - half), (x, 300 + half)], GREY, 22)
    stroke(image, [(40, 300), (400, 300)], GREY, 22)
    fill(image, ellipse(220, 22, 18, 18), YELLOW, width=8)
    fill(image, [(180, 270), (260, 270), (260, 350), (180, 350)], YELLOW, width=9)
    dot(image, 220, 305, 12)
    save(image, "level/goal.png")


def hud():
    # Each digit written big and decorated, like the sheet asks.
    strokes = {
        0: [ellipse(128, 128, 70, 100)],
        1: [[(90, 70), (135, 26), (135, 228)], [(85, 228), (185, 228)]],
        2: [[(65, 75), (100, 30), (165, 35), (185, 85), (70, 225), (195, 225)]],
        3: [[(65, 45), (185, 45), (120, 115), (190, 165), (135, 230), (60, 205)]],
        4: [[(160, 228), (160, 28), (50, 165), (205, 165)]],
        5: [[(185, 35), (75, 35), (70, 115), (150, 105), (190, 165), (135, 228), (60, 205)]],
        6: [[(175, 30), (90, 90), (68, 170), (120, 228), (185, 180), (140, 125), (72, 165)]],
        7: [[(60, 35), (195, 35), (100, 228)]],
        8: [ellipse(128, 75, 55, 50), ellipse(128, 175, 65, 55)],
        9: [[(180, 90), (130, 30), (75, 80), (125, 130), (180, 90), (165, 228)]],
    }
    colours = (RED, ORANGE, YELLOW, GREEN, SKY, BLUE, PURPLE, PINK, RED, GREEN)
    for digit, lines in strokes.items():
        image = canvas((256, 256))
        for line in lines:
            closed = line is lines[0] and digit in (0, 8) or digit == 8
            stroke(image, line, colours[digit], 44, closed=closed)
            stroke(image, line, WHITE, 8, closed=closed)
        for _ in range(4):
            dot(image, RANDOM.uniform(30, 226), RANDOM.uniform(20, 236), 7, YELLOW)
        save(image, f"hud/digit_{digit}.png")

    def heart(colour):
        points = [(128, 225), (30, 120), (35, 60), (85, 35), (128, 80), (170, 35), (222, 60), (226, 120)]
        image = canvas((256, 256))
        fill(image, points, colour)
        return image

    save(heart(RED), "hud/life.png")
    lost = heart(GREY)
    stroke(lost, [(128, 75), (100, 120), (150, 150), (120, 220)], INK, 12)
    save(lost, "hud/life_lost.png")


def shooting_star():
    image = canvas((512, 256))
    # Flying right, with its tail behind it.
    fill(image, [(12, 128), (340, 78), (340, 178)], ORANGE, outline=None)
    fill(image, [(90, 128), (340, 100), (340, 156)], YELLOW, outline=None)
    points = []
    for i in range(10):
        a = 2 * math.pi * i / 10
        radius = 50 if i % 2 else 115
        points.append((370 + radius * math.cos(a), 128 + radius * math.sin(a)))
    fill(image, points, YELLOW, width=10)
    save(image, "fx/star.png")


if __name__ == "__main__":
    hero()
    princess()
    fx()
    enemies()
    level()
    hud()
    # Last, so that it leaves the seed's other drawings as they were.
    shooting_star()
