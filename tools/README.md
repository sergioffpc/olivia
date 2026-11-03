# Tools

## Motion capture

`mocap/capture.html` records a movement card from the webcam and saves it as an animation file for the game.

### Run it

Serve the folder from localhost, where the browser allows the camera:

```sh
python3 -m http.server 8000 --directory tools/mocap
```

Open <http://localhost:8000/capture.html> in Chrome or Edge. In the devcontainer, VS Code forwards the port to the host. The page fetches MediaPipe and its pose model from the network on load.

### Record

1. Pick the movement from the cards of `games/astronaut-cat/docs/cards.md`. The card sets the file and whether it loops.
2. Stand side-on to the camera, facing either way, far enough back that the whole body, feet included, is in view. The side nearer the camera becomes the parts named right, the far side the parts named left.
3. Press **Record**, wait for the 3 s countdown, do the movement and press **Stop**. A recording stops on its own after 20 s.

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

**Save** asks once for the game's `assets` folder and writes the card's file inside it, such as `hero/anim_walk.json`. A running game picks it up through hot reload. Browsers without folder access, such as Firefox and Safari, download the file instead; move it to the same place by hand.
