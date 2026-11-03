# The astronaut cat

A Super Mario style platformer built on Olivia from what a class makes: one story, drawings on the template sheets, and the sound and movement cards. The rules are fixed; the class changes only the art and the sounds.

Here the class is played by Claude, as a rehearsal of the whole path: [docs/story.md](docs/story.md) has the invented story, and every file in `assets/` comes from the scripts in `tools/`. Real drawings and recordings replace them file for file.

## Play

```sh
cd games/astronaut-cat
cargo run --release
```

Left and Right walk, Space or Up jumps and jumps again in the air, X shoots, Enter starts over after the game ends and Tab shows the colliders.

## Layout

```text
assets/
  hero/            the hero: 11 parts, skeleton.json, anim_*.json
  princess/        who is saved: 10 parts, skeleton.json, anim_*.json
  enemy_crawl.png  enemy_roll.png  enemy_float.png
  fx/              projectile.png, fart.png, star.png
  level/           background.png, platform.png, coin.png, goal.png
  hud/             digit_0..9.png, life.png, life_lost.png
  sfx/             one .wav per sound card
docs/              the class's documents, as text
src/level.rs       the fixed layout, in metres
src/game.rs        the rules, the bodies and everything on screen
tests/game.rs      the whole game, played headless on made-up keys
tools/             the scripts that made assets/
```

Paths are the labels printed on the sheets and cards, so a finished sheet tells you where its file goes. The exception is `hero/weapon`, which is written as `hero/launcher.png`, the part name in the motion capture spec.

## Modules

The game uses every Olivia module:

- `window`, `input` and `math`: the window, the keys and the positions.
- `filesystem`: every asset is read from the source.
- `graphics`: sprites, the HUD and the camera that follows the hero.
- `animation`: the hero and who is saved are `Character`s, with the shoot move as an overlay and shots leaving from `hand_right`.
- `audio`: one sound per card, and the music on a loop.
- `physics`: everything is a body, in metres. The hero is dynamic and finds the ground with a ray, enemies are kinematic, shots are dynamic and arc down under part of gravity, and coins, the goal and the pits are sensors found with an overlap query.

Platforms are solid, because Olivia exposes no way to make one-way platforms, so the hero bumps its head on them from below.

## Remaking the assets

```sh
python3 tools/make_art.py    # needs Pillow
python3 tools/make_cards.py  # needs only the standard library
```

Both use fixed seeds, so they rewrite the same files. Running them replaces anything a class has put in `assets/`.
