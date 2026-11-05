# /// script
# requires-python = ">=3.10"
# dependencies = ["fpdf2>=2.7"]
# ///
"""Writes ../docs/folhas_para_desenhar.pdf: one A4 sheet per thing the class draws and colours in.

The boxes of the hero and of who is saved follow the humanoid convention of the motion capture
spec (#82), with the sizes measured on the original template: one box per part, a black dot where
the part joins the one above it, and ticks outside the arm and leg boxes at the elbow and knee.
The sheets are in Portuguese, the language of the class; the label at the top right of each sheet
is where its drawings go under `assets/`. The drawing page, `tools/picture/capture.html`, finds the
boxes on a photo of a whole sheet from where they lie here, so it keeps the same layout: change both
together. Run it with `uv run tools/make_sheets.py`.
"""

from pathlib import Path

from fpdf import FPDF

OUT = Path(__file__).resolve().parent.parent / "docs" / "folhas_para_desenhar.pdf"

# Boxes in millimetres: (width, height), with dots as (x, y) inside the box and ticks as a height.
HEAD, HEAD_DOT = (63.9, 57.0), (32.0, 51.4)
TORSO = (60.5, 81.75)
ARM, ARM_DOT, ARM_TICK = (40.6, 81.75), (20.25, 5.75), 43.4
LEG, LEG_DOT, LEG_TICK = (47.9, 71.25), (24.0, 5.75), 38.4
WEAPON, WEAPON_DOT = (166.6, 92.5), (14.1, 46.25)

BODY_TASK = ("Desenha cada parte dentro da sua caixa, de frente, com os braços e as pernas esticados "
             "para baixo. Liga cada parte ao ponto preto. Depois pinta tudo!")

FILL = "Enche a caixa toda e pinta!"

# (title, task, label, (width, height), dot, orientation) for the sheets with one drawing. Boxes keep
# the proportions of the astronaut-cat's images.
SINGLES = [
    ("O lançador do herói",
     "Desenha com que o herói dispara, deitado como na caixa. A mão segura-o no ponto preto. Pinta-o!",
     "textures/hero_weapon", WEAPON, WEAPON_DOT, "P"),
    ("O que o herói dispara", f"Desenha a coisa que o herói dispara. {FILL}",
     "textures/projectile", (130.0, 130.0), None, "P"),
    ("A nuvem do pum", f"Desenha a nuvem que sai quando o herói dá o salto extra. {FILL}",
     "textures/fart", (150.0, 150.0), None, "P"),
    ("Um mau que rasteja", f"Desenha um mau comprido e baixinho, que anda a rastejar. {FILL}",
     "textures/enemy_crawl", (190.0, 110.0), None, "P"),
    ("Um mau que rebola", f"Desenha um mau redondo, que vai girar enquanto rebola. {FILL}",
     "textures/enemy_roll", (170.0, 170.0), None, "P"),
    ("Um mau que flutua", f"Desenha um mau que anda pelo ar. {FILL}",
     "textures/enemy_float", (150.0, 205.0), None, "P"),
    ("O cenário", "Desenha o que se vê ao longe, atrás do herói. Não desenhes o chão. Pinta tudo!",
     "textures/background", (277.0, 161.0), None, "L"),
    ("O chão", "Desenha o chão onde o herói anda e salta. Pinta a tira toda, de uma ponta à outra!",
     "textures/platform", (277.0, 62.0), None, "L"),
    ("O que o herói apanha", f"Desenha a coisa que o herói apanha pelo caminho. {FILL}",
     "textures/coin", (130.0, 130.0), None, "P"),
    ("Onde está preso",
     "Desenha o sítio onde está preso quem o herói vai salvar. Só o sítio, sem a personagem. Pinta-o!",
     "textures/goal", (150.0, 205.0), None, "P"),
]

INK = (30, 30, 30)
LINE = (150, 150, 150)
MARGIN = 10
DOT_RADIUS = 2.5
TICK = 3.0


def header(pdf, title, task, label):
    """Title, task, label and the line for the author's name. Returns where the boxes may start."""
    pdf.set_text_color(*INK)
    pdf.set_font("Helvetica", "B", 22)
    pdf.set_xy(MARGIN, MARGIN)
    pdf.cell(pdf.w - 2 * MARGIN, 10, title)
    pdf.set_text_color(*LINE)
    pdf.set_font("Courier", "", 10)
    pdf.set_xy(MARGIN, MARGIN)
    pdf.cell(pdf.w - 2 * MARGIN, 10, label, align="R")

    pdf.set_text_color(*INK)
    pdf.set_font("Helvetica", "", 12)
    pdf.set_xy(MARGIN, MARGIN + 11)
    pdf.multi_cell(pdf.w - 2 * MARGIN, 6, task)
    top = pdf.get_y() + 3
    pdf.set_font("Helvetica", "B", 12)
    pdf.set_xy(MARGIN, top)
    pdf.cell(16, 7, "Nome:")
    pdf.set_draw_color(*INK)
    pdf.set_line_width(0.4)
    pdf.line(MARGIN + 16, top + 6, MARGIN + 100, top + 6)
    return top + 14


def box(pdf, x, y, size, name, dot=None, tick=None, name_size=9):
    """One box with its name above it, an optional black dot and optional ticks on both sides."""
    w, h = size
    pdf.set_text_color(*LINE)
    pdf.set_font("Helvetica", "B" if name_size > 9 else "", name_size)
    pdf.set_xy(x, y - name_size * 0.5)
    pdf.cell(w, name_size * 0.5, name)
    pdf.set_draw_color(*LINE)
    pdf.set_line_width(0.5)
    pdf.rect(x, y, w, h)
    if dot:
        pdf.set_fill_color(*INK)
        pdf.circle(x + dot[0], y + dot[1], DOT_RADIUS, style="F")
    if tick:
        pdf.set_draw_color(*INK)
        pdf.set_line_width(0.6)
        pdf.line(x - TICK, y + tick, x - 0.8, y + tick)
        pdf.line(x + w + 0.8, y + tick, x + w + TICK, y + tick)


def body_sheet(pdf, title, label):
    pdf.add_page()
    top = header(pdf, title, BODY_TASK, label)
    middle = pdf.w / 2

    head_y = top + 3
    box(pdf, middle - HEAD[0] / 2, head_y, HEAD, "cabeça", HEAD_DOT)

    row = head_y + HEAD[1] + 8
    torso_x = middle - TORSO[0] / 2
    box(pdf, torso_x, row, TORSO, "tronco")
    box(pdf, torso_x - 8 - ARM[0], row, ARM, "braço", ARM_DOT, ARM_TICK)
    box(pdf, torso_x + TORSO[0] + 8, row, ARM, "braço", ARM_DOT, ARM_TICK)

    legs = row + TORSO[1] + 8
    box(pdf, middle - 4 - LEG[0], legs, LEG, "perna", LEG_DOT, LEG_TICK)
    box(pdf, middle + 4, legs, LEG, "perna", LEG_DOT, LEG_TICK)


def single_sheet(pdf, title, task, label, size, dot, orientation):
    pdf.add_page(orientation=orientation)
    top = header(pdf, title, task, label)
    box(pdf, (pdf.w - size[0]) / 2, top + 3, size, "", dot)


def grid_sheet(pdf, title, task, label, names, size, cols, name_size=9):
    """Boxes of one size in rows of `cols`, each named."""
    pdf.add_page()
    top = header(pdf, title, task, label)
    gap = 4 + name_size * 0.5
    left = (pdf.w - cols * size[0] - (cols - 1) * gap) / 2
    for i, name in enumerate(names):
        col, row = i % cols, i // cols
        box(pdf, left + col * (size[0] + gap), top + name_size * 0.5 - 2 + row * (size[1] + gap), size, name, name_size=name_size)


def main():
    pdf = FPDF(format="A4")
    pdf.set_title("Folhas para desenhar")
    pdf.set_auto_page_break(False)
    body_sheet(pdf, "O herói", "textures/hero_*")
    body_sheet(pdf, "Quem é salvo", "textures/princess_*")
    for sheet in SINGLES:
        single_sheet(pdf, *sheet)
    grid_sheet(pdf, "Os números", "Escreve o número de cada caixa, bem grande, e enfeita-o. Depois pinta-o!",
               "textures/hud_digit_0..9", [str(n) for n in range(10)], (52.0, 52.0), 3, 16)
    grid_sheet(pdf, "As vidas", "Desenha o símbolo das vidas do herói: uma vida que ainda tem e uma que já "
               "perdeu. Pinta-as!", "textures/hud_life, textures/hud_life_lost", ["uma vida", "uma vida perdida"], (80.0, 80.0), 2)
    pdf.output(OUT)
    print(f"wrote {OUT}")


if __name__ == "__main__":
    main()
