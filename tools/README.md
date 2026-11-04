# Tools

`index.html` opens the pages that make the game's files and check them, with tabs to move between them: **Mocap**, **Sound**, **Picture** and **Preview**. A page keeps what was done on it while another is shown.

## Run it

Serve the folder from localhost, where the browser allows the camera and the microphone:

```sh
python3 -m http.server 8000 --directory tools
```

Open <http://localhost:8000/> in Chrome or Edge, not in VS Code's Simple Browser, which cannot show the folder picker. In the devcontainer, VS Code forwards the port to the host. Each page also opens on its own address, such as <http://localhost:8000/#sound>.

## Assets folder

First choose the game's `assets` folder, such as `games/astronaut-cat/assets`, at the top: every page uses it. The page checks it holds the folders the pages write into, such as `hero/` and `sfx/`, and keeps it until it is reloaded. Recording, taking a photo and saving wait for it. The preview only reads it, and watches it: whatever the other pages save there shows straight away.

Browsers without folder access, such as Firefox and Safari, skip it for the capture pages, which download the files instead. The preview then takes the folder from a file picker, and needs it chosen again to see a change.

## Motion capture

**Mocap**, `mocap/capture.html`, records a movement card from the webcam and saves it as an animation file for the game. It fetches MediaPipe and its pose model from the network as it opens; the other pages need no network.

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

**Save** writes the card's file inside the assets folder, such as `hero/anim_walk.json`. A running game picks it up through hot reload. Browsers without folder access, such as Firefox and Safari, download the file instead; move it to the same place by hand.

## Sound capture

**Sound**, `sound/capture.html`, records a sound card from the microphone and saves it as a sound file for the game: mono, 16-bit, 22 050 Hz, like the other files of `assets/sfx/`.

### Record

1. Pick the sound from the cards of `games/astronaut-cat/docs/cards.md`. The card sets the file and whether it loops.
2. Pick the microphone and make the sound once to check the level: the bar turns red when it is too loud and would distort.
3. Press **Record**, stay quiet for the 3 s countdown, make the sound and press **Stop**. The countdown is how the page learns the room's noise. A recording stops on its own after 30 s.

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

**Save** writes the card's file inside the assets folder, such as `sfx/jump.wav`, over the sound that was there. Browsers without folder access download the file instead.

## Picture capture

**Picture**, `picture/capture.html`, photographs a box of a drawing sheet, cuts the drawing out of the paper and saves it as the game's picture, at the size of the file it replaces.

### Photograph

1. Pick the drawing from the sheets of `games/astronaut-cat/docs/sheets.md`. An arm or a leg is one drawing that the page cuts into the four parts, left and right.
2. Hold the sheet flat in front of the camera with only that drawing's box in view, and a little paper around it. The page draws the box in green once it finds its printed border.
3. Press **Take photo**. A photo taken with a phone works too: **Open a photo**, or drop it on the page.

### Review

The left view is the photo: drag the corners onto the box's corners when the page missed them, and **Find the box again** to start over. The right one is the picture the game gets, on a checkerboard where it is see-through. On an arm or a leg, a dashed line marks where it bends.

The looks set the colour and style settings in one go: **As drawn**, **Colourful**, **Cartoon**, **Pixel art** and **Sticker**.

| Setting | What it does |
|---|---|
| Even out the light | Makes the paper white all over, from the paper seen just outside the box, even under a shadow or a yellow lamp |
| Remove the paper | Makes the paper see-through; off for the scenery and the ground, which fill their box |
| How much counts as paper | Higher also removes pale colours and pencil marks |
| Keep the white inside the drawing | White inside the outline, such as the eyes, stays white; off for the numbers and the cage, whose holes stay see-through |
| Remove specks | Drops dust and stray dots around the drawing |
| Colourful, Brightness, Contrast | Brings out crayon colours that came out pale |
| Change the colours | Turns every colour round the rainbow |
| Smooth the crayon | Evens out the scribbles inside each colour |
| Colours | Fewer colours give flat patches, like a cartoon |
| Pixel size | Big pixels, like an old console game |
| Outline | A black or white line round the whole drawing, so it stands out on any scenery |

The page saves each drawing at its file's size, so a box should have the same shape as the file; the character boxes are the sizes measured in `tools/make_art.py`.

### Save

**Save** writes the drawing's files inside the assets folder, over the ones there. An arm or a leg writes its four parts and moves the elbows or knees in the character's `skeleton.json` to where the drawing was cut; the launcher is shrunk and turned to lie along the forearm, as `tools/make_art.py` does. A running game picks them up through hot reload. Browsers without folder access download the pictures instead, and cannot update `skeleton.json`.

## Preview

**Preview**, `preview/preview.html`, plays a character's pictures with a movement and a sound, as the game shows them, to check that what the other pages saved fits together.

### Choose

1. Pick the movement. It brings its character's pictures, the sound its card plays as it starts, and whether it loops; each can be changed, so the princess's pictures can try the hero's walk.
2. **Over it** is a movement played once on top, on its own key, as the game fires while walking or jumping. It plays only on the joints it has, with its own sound.

### Play

The stage stands the pictures on the ground, and the timeline below shows the movement, one tick a row, and the sound under it. Drag the sound along the timeline to make it start earlier or later; click or drag anywhere else to move through the movement.

| Key | What it does |
|---|---|
| Space | Plays or pauses |
| R | Plays from the start, with the sound |
| F | Plays the movement chosen to go over it |
| ← → | Steps one row back or forward |

| Setting | What it does |
|---|---|
| Speed | Slow motion or faster; the sound follows, so its pitch changes too |
| Loops | Plays round; off, it stops on the last row and, if chosen, plays again after a second |
| Change from the last movement | How long a newly picked movement takes to take over, as the game blends one into the next |
| Sound starts | Before or after the movement starts |
| Sound volume, Music behind | The music plays as in the game, at 40% |
| Sound on every loop | Plays the sound again each time a looping movement starts over |
| Size, Facing left, Behind | How the character is shown: the game turns it round to face left |
| Show the joints | Marks each joint; the faded ones are left out of the movement and stay as drawn |
| Show the rows before and after | Faint copies a few rows behind and ahead, to see how far each part moves |

Nothing here is saved: the settings are for looking, and the files are changed on the other pages.
