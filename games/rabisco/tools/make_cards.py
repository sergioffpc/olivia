# /// script
# requires-python = ">=3.10"
# dependencies = ["fpdf2>=2.7"]
# ///
"""Writes the class's cards into ../docs, four to an A4 page:

- cartoes_historia.pdf: one card per choice of the story, where a child writes an idea.
- cartoes_sons_e_movimentos.pdf: one card per sound and per movement the class makes.

The cards are in Portuguese, the language of the class. The small label at the bottom of each card is
where its files go under `assets/`. Run it with `uv run tools/make_cards.py`.
"""

from pathlib import Path

from fpdf import FPDF

DOCS = Path(__file__).resolve().parent.parent / "docs"

HERO = "o herói"
SAVED = "quem é salvo"

# (question, hint, label)
STORY = [
    ("Quem é o herói?", "Tem 2 braços e 2 pernas.", "textures/hero_*"),
    ("Quem tem de ser salvo?", "Tem 2 braços e 2 pernas.", "textures/princess_*"),
    ("Que mau anda a rastejar?", "É comprido e baixinho.", "textures/enemy_crawl"),
    ("Que mau anda a rebolar?", "É redondo, porque vai girar.", "textures/enemy_roll"),
    ("Que mau anda pelo ar?", "Flutua sem tocar no chão.", "textures/enemy_float"),
    ("Onde acontece a história?", "O sítio onde o herói anda e salta.", "textures/background"),
    ("Onde está preso quem é salvo?", "O sítio onde está fechado.", "textures/goal"),
    ("O que apanha o herói pelo caminho?", "Há muitos espalhados pelo jogo.", "textures/coin"),
    ("Com que dispara o herói?", "Segura-o na mão direita.", "textures/hero_weapon"),
    ("O que sai quando dispara?", "Voa até acertar num mau.", "textures/projectile"),
]

# (title, task, label)
SOUNDS = [
    ("O herói salta", "Faz o som do herói a dar um salto.", "sounds/hero_jump"),
    ("O salto extra", "Faz o som do salto extra, lá no ar. É o salto do pum!", "sounds/hero_fart"),
    ("O herói dispara", "Faz o som do herói a disparar.", "sounds/hero_shoot"),
    ("Acertou num mau", "Faz o som do tiro a acertar num mau.", "sounds/hit"),
    ("Apanhou uma coisa", "Faz o som do herói a apanhar uma coisa pelo caminho.", "sounds/pickup"),
    ("O herói magoa-se", "Faz o som do herói quando um mau lhe toca e ele perde uma vida.", "sounds/hero_hurt"),
    ("Fim do jogo", "Faz o som de quando o herói perde a última vida.", "sounds/gameover"),
    ("Socorro!", "Faz o som de quem está preso a pedir ajuda ao herói.", "sounds/princess_help"),
    ("Vitória", "Faz o som do herói a salvar quem estava preso.", "sounds/hero_win"),
    ("A música", "Inventa a música que toca durante o jogo todo.", "musics/music"),
]

# (title, who, task, label)
MOVES = [
    ("Parado", HERO, "Fica parado, à espera.", "animations/hero_idle"),
    ("A andar", HERO, "Anda sem sair do sítio.", "animations/hero_walk"),
    ("Saltar", HERO, "Dá um salto.", "animations/hero_jump"),
    ("Salto extra", HERO, "Dá outro salto, como se já estivesses no ar.", "animations/hero_double_jump"),
    ("Disparar", HERO, "Dispara com o lançador na mão direita.", "animations/hero_shoot"),
    ("Magoar-se", HERO, "Faz de conta que um mau te tocou.", "animations/hero_hurt"),
    ("Vitória", HERO, "Festeja: salvaste quem estava preso!", "animations/hero_win"),
    ("Socorro!", SAVED, "Pede ajuda enquanto esperas que te venham salvar.", "animations/princess_calling"),
    ("Livre!", SAVED, "Festeja: já estás livre!", "animations/princess_freed"),
]

STORY_COLOUR = (60, 160, 80)
SOUND_COLOUR = (240, 130, 30)
MOVE_COLOUR = (40, 120, 210)
INK = (30, 30, 30)
GREY = (120, 120, 120)

PAGE_W, PAGE_H = 210, 297
MARGIN = 10
COLS, ROWS = 2, 2
CARD_W = (PAGE_W - 2 * MARGIN) / COLS
CARD_H = (PAGE_H - 2 * MARGIN) / ROWS
PAD = 6


def speaker(pdf, cx, cy, s):
    """A loudspeaker with three sound waves, centred on (cx, cy), about 2s wide."""
    pdf.set_line_width(1.2)
    pdf.polygon(
        [(cx - s, cy - s * 0.3), (cx - s * 0.6, cy - s * 0.3), (cx - s * 0.1, cy - s * 0.8),
         (cx - s * 0.1, cy + s * 0.8), (cx - s * 0.6, cy + s * 0.3), (cx - s, cy + s * 0.3)],
        style="DF",
    )
    for r in (0.35, 0.65, 0.95):
        pdf.arc(cx - s * 0.1 - s * r, cy - s * r, 2 * s * r, -45, 45)


def stick_figure(pdf, cx, cy, s):
    """A child seen side-on, walking to the right, centred on (cx, cy), about 2s tall."""
    pdf.set_line_width(1.6)
    head = s * 0.28
    pdf.circle(cx, cy - s + head, head, style="DF")
    pdf.line(cx + head * 0.9, cy - s + head * 0.9, cx + head * 1.3, cy - s + head * 1.1)
    neck, hip = cy - s + 2 * head, cy + s * 0.25
    shoulder = neck + s * 0.15
    pdf.line(cx, neck, cx, hip)
    pdf.polyline([(cx, shoulder), (cx + s * 0.3, shoulder + s * 0.3), (cx + s * 0.55, shoulder + s * 0.15)])
    pdf.polyline([(cx, shoulder), (cx - s * 0.25, shoulder + s * 0.3), (cx - s * 0.3, shoulder + s * 0.55)])
    pdf.polyline([(cx, hip), (cx + s * 0.25, hip + s * 0.35), (cx + s * 0.3, cy + s)])
    pdf.polyline([(cx, hip), (cx - s * 0.2, hip + s * 0.4), (cx - s * 0.45, cy + s * 0.9)])


def frame(pdf, x, y, colour, kind, number):
    """The card's border and its top band, with what kind of card it is and its number."""
    pdf.set_draw_color(*colour)
    pdf.set_line_width(1.2)
    pdf.rect(x + 2, y + 2, CARD_W - 4, CARD_H - 4, round_corners=True, corner_radius=6)
    pdf.set_fill_color(*colour)
    pdf.rect(x + 2, y + 2, CARD_W - 4, 16, style="F", round_corners=("TOP_LEFT", "TOP_RIGHT"), corner_radius=6)
    pdf.set_text_color(255, 255, 255)
    pdf.set_font("Helvetica", "B", 16)
    pdf.set_xy(x + PAD, y + 4)
    pdf.cell(CARD_W - 2 * PAD, 12, kind)
    pdf.set_xy(x + PAD, y + 4)
    pdf.cell(CARD_W - 2 * PAD, 12, number, align="R")


def footer(pdf, x, y, label, done):
    """The line for the author's name, an optional done box, and where the file goes."""
    pdf.set_text_color(*INK)
    pdf.set_draw_color(*INK)
    pdf.set_line_width(0.5)
    pdf.set_font("Helvetica", "B", 13)
    pdf.set_xy(x + PAD + 2, y + CARD_H - 32)
    pdf.cell(16, 8, "Nome:")
    pdf.line(x + PAD + 19, y + CARD_H - 26, x + CARD_W - PAD - 2, y + CARD_H - 26)
    if done:
        pdf.set_font("Helvetica", "", 13)
        pdf.rect(x + PAD + 2, y + CARD_H - 18, 7, 7)
        pdf.set_xy(x + PAD + 11, y + CARD_H - 18.5)
        pdf.cell(20, 8, "Feito")
    pdf.set_text_color(*GREY)
    pdf.set_font("Courier", "", 9)
    pdf.set_xy(x + PAD, y + CARD_H - 18.5)
    pdf.cell(CARD_W - 2 * PAD - 2, 8, label, align="R")


def story_card(pdf, x, y, number, question, hint, label):
    frame(pdf, x, y, STORY_COLOUR, "HISTÓRIA", number)
    pdf.set_text_color(*INK)
    pdf.set_font("Helvetica", "B", 20)
    pdf.set_xy(x + PAD, y + 22)
    pdf.multi_cell(CARD_W - 2 * PAD, 9, question, align="C")
    pdf.set_font("Helvetica", "", 14)
    pdf.set_xy(x + PAD, y + 42)
    pdf.cell(CARD_W - 2 * PAD, 7, hint, align="C")

    # Lines to write the idea on.
    pdf.set_text_color(*GREY)
    pdf.set_font("Helvetica", "I", 11)
    pdf.set_xy(x + PAD, y + 52)
    pdf.cell(CARD_W - 2 * PAD, 6, "Escreve a tua ideia:", align="C")
    pdf.set_draw_color(*GREY)
    pdf.set_line_width(0.4)
    for line_y in range(70, int(CARD_H) - 38, 12):
        pdf.line(x + PAD + 2, y + line_y, x + CARD_W - PAD - 2, y + line_y)
    footer(pdf, x, y, label, done=False)


def task_card(pdf, x, y, colour, kind, number, title, who, task, label):
    frame(pdf, x, y, colour, kind, number)

    # The picture.
    pdf.set_draw_color(*INK)
    pdf.set_fill_color(255, 255, 255)
    (speaker if kind == "SOM" else stick_figure)(pdf, x + CARD_W / 2, y + 36, 14)

    # Title and task.
    pdf.set_text_color(*INK)
    pdf.set_font("Helvetica", "B", 24)
    pdf.set_xy(x + PAD, y + 52)
    pdf.cell(CARD_W - 2 * PAD, 12, title, align="C")
    pdf.set_font("Helvetica", "", 15)
    pdf.set_xy(x + PAD + 2, y + 66)
    pdf.multi_cell(CARD_W - 2 * PAD - 4, 7.5, task, align="C")

    # Who acts it out, and how to stand.
    if who:
        pdf.set_font("Helvetica", "B", 13)
        pdf.set_xy(x + PAD, y + CARD_H - 49)
        pdf.cell(CARD_W - 2 * PAD, 7, f"Quem: {who}", align="C")
        pdf.set_xy(x + PAD, y + CARD_H - 42)
        pdf.cell(CARD_W - 2 * PAD, 7, "De lado, ombro direito para a câmara", align="C")
    footer(pdf, x, y, label, done=True)


def cut_lines(pdf):
    pdf.set_draw_color(*GREY)
    pdf.set_line_width(0.2)
    with pdf.local_context():
        pdf.set_dash_pattern(dash=2, gap=2)
        pdf.line(MARGIN + CARD_W, MARGIN - 4, MARGIN + CARD_W, PAGE_H - MARGIN + 4)
        pdf.line(MARGIN - 4, MARGIN + CARD_H, PAGE_W - MARGIN + 4, MARGIN + CARD_H)


def write(name, title, cards, draw):
    """Lays `cards` out four to a page, drawing each with `draw(pdf, x, y, *card)`."""
    pdf = FPDF(format="A4")
    pdf.set_title(title)
    pdf.set_auto_page_break(False)
    per_page = COLS * ROWS
    for i, args in enumerate(cards):
        if i % per_page == 0:
            pdf.add_page()
            cut_lines(pdf)
        col, row = i % COLS, (i % per_page) // COLS
        draw(pdf, MARGIN + col * CARD_W, MARGIN + row * CARD_H, *args)
    pdf.output(DOCS / name)
    print(f"wrote {DOCS / name}")


def main():
    story = [(f"H{i}", question, hint, label) for i, (question, hint, label) in enumerate(STORY, 1)]
    write("cartoes_historia.pdf", "Cartões da história", story, story_card)

    # A sound may be recorded many times, as numbered takes the game picks one of at random.
    tasks = [(SOUND_COLOUR, "SOM", f"S{i}", t, None, task, label + ("_N.wav" if label.startswith("sounds/") else ".wav"))
             for i, (t, task, label) in enumerate(SOUNDS, 1)]
    tasks += [(MOVE_COLOUR, "MOVIMENTO", f"M{i}", t, who, task, label + ".json")
              for i, (t, who, task, label) in enumerate(MOVES, 1)]
    write("cartoes_sons_e_movimentos.pdf", "Cartões de sons e movimentos", tasks, task_card)


if __name__ == "__main__":
    main()
