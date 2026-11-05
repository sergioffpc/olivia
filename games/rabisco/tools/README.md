# Ferramentas

`make_cards.py` e `make_sheets.py` escrevem em `../docs` os cartões e as folhas de desenho da turma; vê cada script.

`index.html` abre as páginas que fazem os ficheiros do jogo a partir deles e os verificam, com separadores para passar de uma para outra: **Movimentos**, **Sons**, **Desenhos** e **Pré-visualização**. Uma página guarda o que lá foi feito enquanto outra está à vista.

## Abrir

As páginas funcionam sem rede. Só a primeira vez, depois de clonar, é preciso descarregar o MediaPipe e o modelo de pose da captura de movimentos, uns 28 MB que o git deixa de fora:

```sh
uv run games/rabisco/tools/fetch_vendor.py
```

Serve a pasta a partir de localhost, onde o browser deixa usar a câmara e o microfone:

```sh
python3 -m http.server 8000 --directory games/rabisco/tools
```

Abre <http://localhost:8000/> no Chrome ou no Edge, e não no Simple Browser do VS Code, que não consegue mostrar a escolha de pasta. No devcontainer, o VS Code reencaminha a porta para o anfitrião. Cada página também abre no seu próprio endereço, como <http://localhost:8000/#sound>.

## Pasta assets

Primeiro escolhe `games/rabisco/assets` lá em cima: todas as páginas a usam. A página verifica que tem as pastas do jogo, `animations/`, `musics/`, `skeletons/`, `sounds/` e `textures/`, e guarda-a até ser recarregada. Cada página guarda os ficheiros com o nome e a pasta que o cartão ou a folha imprime em baixo ou no canto superior direito:

| Pasta | O que tem |
|---|---|
| `animations/` | Os movimentos, como `animations/hero_walk.json` |
| `sounds/` | Os sons, com um número no fim, como `sounds/hero_jump_1.wav` e `sounds/hero_jump_2.wav` |
| `musics/` | A música, `musics/music.wav` |
| `skeletons/` | O esqueleto de cada personagem, como `skeletons/hero.json`: onde as partes se ligam |
| `textures/` | Os desenhos, como `textures/coin.png`, e as personagens, com o nome da personagem à frente: `textures/hero_head.png` |

Gravar, tirar uma foto e guardar esperam pela pasta. A pré-visualização só a lê, e acompanha-a: o que as outras páginas lá guardam aparece logo.

Os browsers sem acesso a pastas, como o Firefox e o Safari, passam sem ela nas páginas de captura, que descarregam os ficheiros. A pré-visualização recebe então a pasta através de uma escolha de ficheiros, e é preciso escolhê-la outra vez para ver uma mudança.

## Captura de movimentos

**Movimentos**, `mocap/capture.html`, grava um cartão de movimento a partir da webcam e guarda-o como um ficheiro de animação para o jogo. Usa o MediaPipe e o seu modelo de pose, que ficam em `mocap/vendor/`.

### Gravar

1. Escolhe o cartão de movimento, de M1 a M9, de `docs/cartoes_sons_e_movimentos.pdf`. O cartão define o ficheiro e se o movimento se repete.
2. Põe-te de lado para a câmara, virado para qualquer um dos lados, longe o suficiente para que o corpo todo, pés incluídos, apareça. O lado mais perto da câmara passa a ser as partes chamadas direitas, o lado mais longe as partes chamadas esquerdas.
3. Carrega em **Gravar**, espera pela contagem de 3 s, faz o movimento e carrega em **Parar**. Uma gravação para sozinha ao fim de 20 s.

### Rever

A vista da esquerda repete a câmara com a pose detetada. A da direita é um boneco de palitos posto só pelas linhas guardadas, por isso mostra o que o jogo vai tocar.

| Definição | O que faz |
|---|---|
| Início, Fim | Corta a gravação |
| Suavizar | Tira os tremores; demasiado amolece os movimentos rápidos |
| Velocidade | Toca mais depressa ou mais devagar, sempre a 30 fps |
| Repete | Funde o fim no início, ao longo da mistura da junção, para a repetição não dar um salto |
| Trocar esquerda e direita | Corrige uma gravação em que o lado errado foi tomado como o mais perto |
| Articulações guardadas | Deixa algumas de fora para fazer um movimento por cima, como só os braços para disparar |

### Guardar

**Guardar** escreve o ficheiro do cartão dentro da pasta assets, como `animations/hero_walk.json`. Um jogo a correr apanha-o através do hot reload. Os browsers sem acesso a pastas, como o Firefox e o Safari, descarregam o ficheiro; move-o à mão para o mesmo sítio.

## Captura de sons

**Sons**, `sound/capture.html`, grava um cartão de som a partir do microfone e guarda-o como um ficheiro de som para o jogo: mono, 16 bits, 22 050 Hz, como os outros ficheiros de `assets/sounds/`. Um som pode ser gravado muitas vezes, por quem quiser: cada gravação fica com o seu número, e o jogo toca uma delas ao acaso sempre que o som toca.

### Gravar

1. Escolhe o cartão de som, de S1 a S10, de `docs/cartoes_sons_e_movimentos.pdf`. O cartão define o ficheiro e se o som se repete.
2. Escolhe o microfone e faz o som uma vez para verificar o nível: a barra fica vermelha quando está alto demais e iria distorcer.
3. Carrega em **Gravar**, fica em silêncio durante a contagem de 3 s, faz o som e carrega em **Parar**. É pela contagem que a página conhece o ruído da sala. Uma gravação para sozinha ao fim de 30 s.

### Rever

A vista de cima é a gravação, com as partes cortadas sombreadas; a de baixo é o resultado que o jogo recebe. **Tocar** e **Tocar original** comparam-nos. Um som que se repete toca em volta, para se ouvir a junção.

| Definição | O que faz |
|---|---|
| Tirar o ruído de fundo | Baixa o chiado, as ventoinhas e o zumbido ouvidos na contagem; demasiado faz a voz soar aguada |
| Tirar o ronco | Corta os sons graves abaixo de 80 Hz: trânsito, pancadas na mesa, o sopro no microfone |
| Início, Fim | Cortam a gravação; ficam sozinhos à volta do som, e **Cortar o silêncio nas pontas** volta a pô-los |
| Tom | Mais agudo é mais pequeno e mais rápido, mais grave é maior e mais lento, em meios-tons |
| Volume | O volume do momento mais alto; 100% é o máximo sem distorcer |
| Entrada suave, Saída suave | Um desvanecimento curto evita estalidos nas pontas |
| Repete | Funde o fim no início, ao longo da mistura da junção, para a repetição não dar um salto |

### Guardar

**Guardar** escreve uma gravação nova dentro da pasta assets, com o número a seguir ao da última, como `sounds/hero_jump_3.wav` depois de `sounds/hero_jump_1.wav` e `sounds/hero_jump_2.wav`. A escolha ao lado de **Guardar** também a pode pôr por cima de uma gravação que já lá está; para tirar uma, apaga o seu ficheiro. A música é uma só, `musics/music.wav`, e é substituída. Os browsers sem acesso a pastas descarregam o ficheiro, que fica com o número 1: muda-o para o número a seguir ao da última gravação.

## Captura de desenhos

**Desenhos**, `picture/capture.html`, fotografa uma folha de desenho inteira, encontra cada uma das suas caixas, recorta os desenhos do papel e guarda-os como as imagens do jogo, ao tamanho dos ficheiros que substituem.

### Fotografar

1. Escolhe a folha de `docs/folhas_para_desenhar.pdf`, pelo seu título. A página conhece onde ficam as caixas de cada folha: a cabeça, o tronco, os braços e as pernas de uma personagem, os dez números ou as duas vidas.
2. Segura a folha inteira direita à frente da câmara, com todas as caixas à vista. Cada caixa fica contornada a verde, com o seu nome, quando a página encontra o seu contorno impresso.
3. Com **Tirar sozinha**, ligada à partida, a página tira a foto assim que estão todas verdes e a folha fica quieta por quase um segundo; uma barra verde em baixo mostra quanto falta. Depois de uma foto, só volta a tirar outra sozinha quando a folha sair da vista. Sem ela, ou a qualquer momento, carrega em **Tirar foto**.

Uma foto tirada com um telemóvel também serve, e tem mais pormenor do que uma webcam, que numa folha inteira dá poucos píxeis a cada caixa: **Abrir uma foto**, ou larga-a na página.

Uma caixa cujo contorno não se vê, como quando um desenho sai por cima dele, é posta pelas outras, no sítio onde a folha a imprime.

### Rever

A vista da esquerda é a foto, com cada caixa: a verde as encontradas, a laranja as postas pelas outras. Arrasta os cantos de uma caixa para os cantos impressos quando a página não os acertou, e **Procurar as caixas outra vez** para recomeçar. À direita estão as imagens que o jogo recebe, sobre um xadrez onde são transparentes; desmarca as que não são para guardar. Num braço ou numa perna, uma linha tracejada marca onde dobra.

Os estilos e as definições valem para todos os desenhos da folha, e definem as cores e o aspeto de uma só vez: **Como desenhado**, **Colorido**, **Desenho animado**, **Pixel art** e **Autocolante**.

| Definição | O que faz |
|---|---|
| Igualar a luz | Torna o papel branco por todo o lado, a partir do papel visto mesmo fora da caixa, mesmo debaixo de uma sombra ou de uma lâmpada amarela |
| Tirar o papel | Torna o papel transparente; desligado para o cenário e o chão, que enchem a caixa |
| Quanto conta como papel | Mais alto também tira as cores claras e os traços de lápis |
| Manter o branco dentro do desenho | O branco dentro do contorno, como os olhos, fica branco; desligado para os números e a jaula, cujos buracos ficam transparentes |
| Tirar pintinhas | Tira o pó e os pontos soltos à volta do desenho |
| Colorido, Brilho, Contraste | Realçam as cores de lápis de cera que saíram pálidas |
| Mudar as cores | Roda todas as cores à volta do arco-íris |
| Alisar o lápis de cera | Iguala os rabiscos dentro de cada cor |
| Cores | Menos cores dão manchas lisas, como num desenho animado |
| Tamanho do pixel | Pixels grandes, como num jogo de consola antigo |
| Contorno | Uma linha preta ou branca à volta de todo o desenho, para se destacar em qualquer cenário |

A página guarda cada desenho ao tamanho do seu ficheiro, por isso uma caixa deve ter a mesma forma do ficheiro; as caixas das personagens têm os tamanhos medidos em `games/astronaut-cat/tools/make_art.py`, que o `make_sheets.py` desenha.

### Guardar

**Guardar** escreve os ficheiros dos desenhos marcados dentro da pasta assets, por cima dos que lá estão. As partes de uma personagem levam o seu nome à frente, como `textures/hero_head.png` ou `textures/princess_torso.png`. Cada braço e cada perna é cortado em dois no cotovelo ou no joelho: o da esquerda da folha dá as partes `_left`, tal como foi desenhado, e o da direita as partes `_right`. Os cotovelos e os joelhos mudam no esqueleto da personagem, `skeletons/hero.json` ou `skeletons/princess.json`, para onde o desenho foi cortado. O lançador do herói, `textures/hero_weapon.png`, é encolhido e rodado para ficar ao longo do antebraço, como o `games/astronaut-cat/tools/make_art.py` faz ao lançador. Um jogo a correr apanha-os através do hot reload. Os browsers sem acesso a pastas descarregam as imagens, e não conseguem atualizar o esqueleto.

## Pré-visualização

**Pré-visualização**, `preview/preview.html`, toca as imagens de uma personagem com um movimento e um som, como o jogo os mostra, para verificar que o que as outras páginas guardaram encaixa.

### Escolher

1. Escolhe o movimento. Traz as imagens da sua personagem, uma das gravações do som que o seu cartão toca quando começa, ao acaso como no jogo, e se se repete; cada um pode ser mudado, por isso as imagens da princesa podem experimentar o andar do herói.
2. **Por cima** é um movimento tocado uma vez por cima, na sua própria tecla, como o jogo dispara enquanto anda ou salta. Só toca nas articulações que tem, com o seu próprio som.

### Tocar

O palco põe as imagens de pé no chão, e a linha do tempo em baixo mostra o movimento, um traço por linha, e o som por baixo. Arrasta o som ao longo da linha do tempo para ele começar mais cedo ou mais tarde; clica ou arrasta em qualquer outro sítio para andar pelo movimento.

| Tecla | O que faz |
|---|---|
| Espaço | Toca ou pausa |
| R | Toca desde o início, com o som |
| F | Toca o movimento escolhido para ir por cima |
| ← → | Anda uma linha para trás ou para a frente |

| Definição | O que faz |
|---|---|
| Velocidade | Câmara lenta ou mais depressa; o som acompanha, por isso o tom também muda |
| Repete | Toca em volta; desligado, para na última linha e, se escolhido, volta a tocar ao fim de um segundo |
| Passagem do movimento anterior | Quanto tempo um movimento acabado de escolher demora a tomar o lugar do anterior, como o jogo funde um no seguinte |
| O som começa | Antes ou depois de o movimento começar |
| Volume do som, Música por trás | A música toca como no jogo, a 40% |
| Som em cada volta | Toca o som outra vez sempre que um movimento que se repete recomeça |
| Tamanho, Virado para a esquerda, Atrás | Como a personagem é mostrada: o jogo vira-a para olhar para a esquerda |
| Mostrar as articulações | Marca cada articulação; as esbatidas ficam de fora do movimento e ficam como desenhadas |
| Mostrar as linhas antes e depois | Cópias ténues algumas linhas atrás e à frente, para ver quanto se mexe cada parte |

Aqui nada é guardado: as definições são para ver, e os ficheiros mudam-se nas outras páginas.
