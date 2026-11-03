# Tools

## Motion capture

`mocap/capture.html` records a movement card from the webcam and saves it as an animation file for the game.

### Run it

Serve the folder from localhost, where the browser allows the camera:

```sh
python3 -m http.server 8000 --directory tools/mocap
```

Open <http://localhost:8000/capture.html> in Chrome or Edge, not in VS Code's Simple Browser, which cannot show the folder picker. In the devcontainer, VS Code forwards the port to the host. The page fetches MediaPipe and its pose model from the network on load.

### Record

1. Choose the game's `assets` folder, such as `games/astronaut-cat/assets`. The page checks it holds the folders the cards write into, such as `hero/`, and keeps it until the page is reloaded. **Record** waits for it. Browsers without folder access, such as Firefox and Safari, skip this step.
2. Pick the movement from the cards of `games/astronaut-cat/docs/cards.md`. The card sets the file and whether it loops.
3. Stand side-on to the camera, facing either way, far enough back that the whole body, feet included, is in view. The side nearer the camera becomes the parts named right, the far side the parts named left.
4. Press **Record**, wait for the 3 s countdown, do the movement and press **Stop**. A recording stops on its own after 20 s.

### Review

The left view replays the camera with the detected pose. The right one is a stick figure posed only by the saved rows, so it shows what the game will play.

| Setting | What it does |
|---|---|
| Start, End | Cuts the recording |
| Smoothing | Removes jitter; too much softens fast moves |
| Speed | Plays faster or slower, still at 30 fps |
| Loops | Fades the end into the start, over the seam blend, so the loop has no jump |
| Swap left and right | Fixes a recording where the wrong side was taken as the near one |
| Joints saved | Leave some out to make an overlay, such as arms only for firing |

### Save

**Save** writes the card's file inside the folder chosen in the first step, such as `hero/anim_walk.json`. A running game picks it up through hot reload. Browsers without folder access, such as Firefox and Safari, download the file instead; move it to the same place by hand.

## Sound capture

`sound/capture.html` records a sound card from the microphone and saves it as a sound file for the game: mono, 16-bit, 22 050 Hz, like the other files of `assets/sfx/`.

### Run it

```sh
python3 -m http.server 8001 --directory tools/sound
```

Open <http://localhost:8001/capture.html> in Chrome or Edge, not in VS Code's Simple Browser, which cannot show the folder picker. The page needs no network.

### Record

1. Choose the game's `assets` folder, as for motion capture. The page checks it holds `sfx/`.
2. Pick the sound from the cards of `games/astronaut-cat/docs/cards.md`. The card sets the file and whether it loops.
3. Pick the microphone and make the sound once to check the level: the bar turns red when it is too loud and would distort.
4. Press **Record**, stay quiet for the 3 s countdown, make the sound and press **Stop**. The countdown is how the page learns the room's noise. A recording stops on its own after 30 s.

### Review

The top view is the recording, with the cut parts shaded; the bottom one is the result the game gets. **Play** and **Play original** compare them. A looping sound plays round, so the seam can be heard.

| Setting | What it does |
|---|---|
| Remove background noise | Turns down hiss, fans and hum heard in the countdown; too much makes the voice sound watery |
| Remove rumble | Cuts the deep sounds below 80 Hz: traffic, desk knocks, breath on the microphone |
| Start, End | Cuts the recording; set on their own around the sound, **Cut the silence at the ends** sets them again |
| Pitch | Higher is smaller and faster, lower is bigger and slower, in semitones |
| Loudness | How loud the loudest moment is; 100% is the most without distorting |
| Fade in, Fade out | A short fade stops clicks at the ends |
| Loops | Fades the end into the start, over the seam blend, so the repeat has no jump |

### Save

**Save** writes the card's file inside the folder chosen in the first step, such as `sfx/jump.wav`, over the sound that was there. Browsers without folder access download the file instead.
