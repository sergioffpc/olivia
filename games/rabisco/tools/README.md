# Tools

`make_cards.py` and `make_sheets.py` write the class's cards and drawing sheets into `../docs`; see each script.

`index.html` opens the pages that make the game's files from them and check them, with tabs to move between them: **Movimentos**, **Sons**, **Desenhos** and **Pré-visualização**. A page keeps what was done on it while another is shown. The pages speak Portuguese, the language of the class; this file names their buttons and settings as they show them.

## Run it

Serve the folder from localhost, where the browser allows the camera and the microphone:

```sh
python3 -m http.server 8000 --directory games/rabisco/tools
```

Open <http://localhost:8000/> in Chrome or Edge, not in VS Code's Simple Browser, which cannot show the folder picker. In the devcontainer, VS Code forwards the port to the host. Each page also opens on its own address, such as <http://localhost:8000/#sound>.

## Assets folder

First choose `games/rabisco/assets` at the top: every page uses it. The page checks it holds `hero/` and `princess/`, with their `skeleton.json`, and keeps it until it is reloaded. Each page saves its files under the name and folder the card or sheet prints at its bottom or top right, and makes the folder, such as `sfx/`, if it is not there yet. Recording, taking a photo and saving wait for it. The preview only reads it, and watches it: whatever the other pages save there shows straight away.

Browsers without folder access, such as Firefox and Safari, skip it for the capture pages, which download the files instead. The preview then takes the folder from a file picker, and needs it chosen again to see a change.

## Motion capture

**Movimentos**, `mocap/capture.html`, records a movement card from the webcam and saves it as an animation file for the game. It fetches MediaPipe and its pose model from the network as it opens; the other pages need no network.

### Record

1. Pick the movement card, M1 to M9, from `docs/cartoes_sons_e_movimentos.pdf`. The card sets the file and whether it loops.
2. Stand side-on to the camera, facing either way, far enough back that the whole body, feet included, is in view. The side nearer the camera becomes the parts named right, the far side the parts named left.
3. Press **Gravar**, wait for the 3 s countdown, do the movement and press **Parar**. A recording stops on its own after 20 s.

### Review

The left view replays the camera with the detected pose. The right one is a stick figure posed only by the saved rows, so it shows what the game will play.

| Setting | What it does |
|---|---|
| Início, Fim | Cuts the recording |
| Suavizar | Removes jitter; too much softens fast moves |
| Velocidade | Plays faster or slower, still at 30 fps |
| Repete | Fades the end into the start, over the seam blend, so the loop has no jump |
| Trocar esquerda e direita | Fixes a recording where the wrong side was taken as the near one |
| Articulações guardadas | Leave some out to make an overlay, such as arms only for firing |

### Save

**Guardar** writes the card's file inside the assets folder, such as `hero/anim_walk.json`. A running game picks it up through hot reload. Browsers without folder access, such as Firefox and Safari, download the file instead; move it to the same place by hand.

## Sound capture

**Sons**, `sound/capture.html`, records a sound card from the microphone and saves it as a sound file for the game: mono, 16-bit, 22 050 Hz, like the other files of `assets/sfx/`.

### Record

1. Pick the sound card, S1 to S10, from `docs/cartoes_sons_e_movimentos.pdf`. The card sets the file and whether it loops.
2. Pick the microphone and make the sound once to check the level: the bar turns red when it is too loud and would distort.
3. Press **Gravar**, stay quiet for the 3 s countdown, make the sound and press **Parar**. The countdown is how the page learns the room's noise. A recording stops on its own after 30 s.

### Review

The top view is the recording, with the cut parts shaded; the bottom one is the result the game gets. **Tocar** and **Tocar original** compare them. A looping sound plays round, so the seam can be heard.

| Setting | What it does |
|---|---|
| Tirar o ruído de fundo | Turns down hiss, fans and hum heard in the countdown; too much makes the voice sound watery |
| Tirar o ronco | Cuts the deep sounds below 80 Hz: traffic, desk knocks, breath on the microphone |
| Início, Fim | Cuts the recording; set on their own around the sound, **Cortar o silêncio nas pontas** sets them again |
| Tom | Higher is smaller and faster, lower is bigger and slower, in semitones |
| Volume | How loud the loudest moment is; 100% is the most without distorting |
| Entrada suave, Saída suave | A short fade stops clicks at the ends |
| Repete | Fades the end into the start, over the seam blend, so the repeat has no jump |

### Save

**Guardar** writes the card's file inside the assets folder, such as `sfx/jump.wav`, over the sound that was there. Browsers without folder access download the file instead.

## Picture capture

**Desenhos**, `picture/capture.html`, photographs a box of a drawing sheet, cuts the drawing out of the paper and saves it as the game's picture, at the size of the file it replaces.

### Photograph

1. Pick the drawing from the sheets of `docs/folhas_para_desenhar.pdf`, named as their titles. An arm or a leg is one drawing that the page cuts into the four parts, left and right.
2. Hold the sheet flat in front of the camera with only that drawing's box in view, and a little paper around it. The page draws the box in green once it finds its printed border.
3. Press **Tirar foto**. A photo taken with a phone works too: **Abrir uma foto**, or drop it on the page.

### Review

The left view is the photo: drag the corners onto the box's corners when the page missed them, and **Procurar a caixa outra vez** to start over. The right one is the picture the game gets, on a checkerboard where it is see-through. On an arm or a leg, a dashed line marks where it bends.

The looks set the colour and style settings in one go: **Como desenhado**, **Colorido**, **Desenho animado**, **Pixel art** and **Autocolante**.

| Setting | What it does |
|---|---|
| Igualar a luz | Makes the paper white all over, from the paper seen just outside the box, even under a shadow or a yellow lamp |
| Tirar o papel | Makes the paper see-through; off for the scenery and the ground, which fill their box |
| Quanto conta como papel | Higher also removes pale colours and pencil marks |
| Manter o branco dentro do desenho | White inside the outline, such as the eyes, stays white; off for the numbers and the cage, whose holes stay see-through |
| Tirar pintinhas | Drops dust and stray dots around the drawing |
| Colorido, Brilho, Contraste | Brings out crayon colours that came out pale |
| Mudar as cores | Turns every colour round the rainbow |
| Alisar o lápis de cera | Evens out the scribbles inside each colour |
| Cores | Fewer colours give flat patches, like a cartoon |
| Tamanho do pixel | Big pixels, like an old console game |
| Contorno | A black or white line round the whole drawing, so it stands out on any scenery |

The page saves each drawing at its file's size, so a box should have the same shape as the file; the character boxes are the sizes measured in `games/astronaut-cat/tools/make_art.py`, which `make_sheets.py` draws.

### Save

**Guardar** writes the drawing's files inside the assets folder, over the ones there. An arm or a leg writes its four parts and moves the elbows or knees in the character's `skeleton.json` to where the drawing was cut; the hero's weapon, `hero/weapon.png`, is shrunk and turned to lie along the forearm, as `games/astronaut-cat/tools/make_art.py` does the launcher. A running game picks them up through hot reload. Browsers without folder access download the pictures instead, and cannot update `skeleton.json`.

## Preview

**Pré-visualização**, `preview/preview.html`, plays a character's pictures with a movement and a sound, as the game shows them, to check that what the other pages saved fits together.

### Choose

1. Pick the movement. It brings its character's pictures, the sound its card plays as it starts, and whether it loops; each can be changed, so the princess's pictures can try the hero's walk.
2. **Por cima** is a movement played once on top, on its own key, as the game fires while walking or jumping. It plays only on the joints it has, with its own sound.

### Play

The stage stands the pictures on the ground, and the timeline below shows the movement, one tick a row, and the sound under it. Drag the sound along the timeline to make it start earlier or later; click or drag anywhere else to move through the movement.

| Key | What it does |
|---|---|
| Espaço | Plays or pauses |
| R | Plays from the start, with the sound |
| F | Plays the movement chosen to go over it |
| ← → | Steps one row back or forward |

| Setting | What it does |
|---|---|
| Velocidade | Slow motion or faster; the sound follows, so its pitch changes too |
| Repete | Plays round; off, it stops on the last row and, if chosen, plays again after a second |
| Passagem do movimento anterior | How long a newly picked movement takes to take over, as the game blends one into the next |
| O som começa | Before or after the movement starts |
| Volume do som, Música por trás | The music plays as in the game, at 40% |
| Som em cada volta | Plays the sound again each time a looping movement starts over |
| Tamanho, Virado para a esquerda, Atrás | How the character is shown: the game turns it round to face left |
| Mostrar as articulações | Marks each joint; the faded ones are left out of the movement and stay as drawn |
| Mostrar as linhas antes e depois | Faint copies a few rows behind and ahead, to see how far each part moves |

Nothing here is saved: the settings are for looking, and the files are changed on the other pages.
