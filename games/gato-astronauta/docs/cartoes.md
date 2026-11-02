# Cartões de sons e movimentos

*Text of `cartoes_sons_e_movimentos.pdf`.*

## Sons

| Cartão | Quando | Ficheiro |
|---|---|---|
| O herói salta | o herói dá um salto normal | `sfx/jump.wav` |
| O salto extra | o herói dá o salto extra (o do pum) | `sfx/fart.wav` |
| O herói dispara | o herói dispara com o lançador | `sfx/shoot.wav` |
| Acertou num mau | o que o herói disparou acerta num mau | `sfx/hit.wav` |
| Apanhou uma coisa | o herói apanha uma coisa pelo caminho | `sfx/pickup.wav` |
| O herói magoa-se | um mau toca no herói e ele perde uma vida | `sfx/hurt.wav` |
| Fim do jogo | o herói perde a última vida | `sfx/gameover.wav` |
| Pedir ajuda | o herói chega perto de quem está preso | `sfx/help.wav` |
| Vitória | o herói salva quem estava preso | `sfx/win.wav` |
| A música | toca durante o jogo todo, sempre a repetir | `sfx/music.wav` |

## Movimentos

| Cartão | Quem | Faz | Repete | Ficheiro |
|---|---|---|---|---|
| Parado | o herói | está parado à espera | sim, sempre | `hero/anim_idle.json` |
| Andar | o herói | anda no sítio, virado para a câmara | sim, sempre | `hero/anim_walk.json` |
| Saltar | o herói | dá um salto | não, uma vez | `hero/anim_jump.json` |
| Salto extra | o herói | dá um segundo salto no ar | não, uma vez | `hero/anim_double_jump.json` |
| Disparar | o herói | dispara com o lançador na mão | não, uma vez | `hero/anim_shoot.json` |
| Magoar-se | o herói | é tocado por um mau | não, uma vez | `hero/anim_hurt.json` |
| Vitória | o herói | festeja porque salvou | não, uma vez | `hero/anim_win.json` |
| Pedir ajuda | quem está preso | pede ajuda enquanto espera | sim, sempre | `princess/anim_help.json` |
| Ficar livre | quem estava preso | festeja porque foi salvo | não, uma vez | `princess/anim_win.json` |
