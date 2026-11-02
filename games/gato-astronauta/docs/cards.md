# Sound and movement cards

*Translated from `cartoes_sons_e_movimentos.pdf`.*

## Sounds

| Card | When | File |
|---|---|---|
| The hero jumps | the hero does a normal jump | `sfx/jump.wav` |
| The extra jump | the hero does the extra jump (the fart one) | `sfx/fart.wav` |
| The hero fires | the hero fires the launcher | `sfx/shoot.wav` |
| Hit a baddie | what the hero fired hits a baddie | `sfx/hit.wav` |
| Picked something up | the hero picks something up along the way | `sfx/pickup.wav` |
| The hero gets hurt | a baddie touches the hero, who loses a life | `sfx/hurt.wav` |
| Game over | the hero loses the last life | `sfx/gameover.wav` |
| Calling for help | the hero gets close to who is held | `sfx/help.wav` |
| Victory | the hero saves who was held | `sfx/win.wav` |
| The music | plays through the whole game, always repeating | `sfx/music.wav` |

## Movements

| Card | Who | Does | Repeats | File |
|---|---|---|---|---|
| Standing still | the hero | stands waiting | yes, always | `hero/anim_idle.json` |
| Walking | the hero | walks on the spot, facing the camera | yes, always | `hero/anim_walk.json` |
| Jumping | the hero | does a jump | no, once | `hero/anim_jump.json` |
| Extra jump | the hero | does a second jump in the air | no, once | `hero/anim_double_jump.json` |
| Firing | the hero | fires with the launcher in hand | no, once | `hero/anim_shoot.json` |
| Getting hurt | the hero | is touched by a baddie | no, once | `hero/anim_hurt.json` |
| Victory | the hero | celebrates having saved them | no, once | `hero/anim_win.json` |
| Calling for help | who is held | calls for help while waiting | yes, always | `princess/anim_help.json` |
| Getting free | who was held | celebrates being saved | no, once | `princess/anim_win.json` |
