#!/usr/bin/env python3
"""Draw the caption buttons from the measurements.

Run with `python3 -I tools/make-caption-svgs.py`. It writes
`assets/caption/*.svg`, one file per button and state. Every number here comes
from `docs/aero-measurements.md`; change a number there and in the tables
below, then run this again.

The buttons are one joined group of 103 x 18 px:

    dark 1 | minimize 27 | dark 1 | maximize 26 | dark 1 | close 46 | dark 1

so the three files are 28, 27 and 48 px wide, each carrying the dark line on
its left, the two 1 px light edges, the fill, the glyph, and the dark line
along its bottom. The close button also carries the dark line on its right.
"""

import os

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(os.path.dirname(HERE), "assets", "caption")

H = 18  # group height, the last row being the dark border
FILL_H = 17  # rows 0..16

# Measured: rows 0..16 of the fill, as (row, colour, alpha). The step between
# row 7 and row 8 is hard.
GREY_REST = [
    (0, "#ffffff", 0.53),
    (4, "#fffcf9", 0.47),
    (7, "#dbd7d5", 0.43),
    (8, "#5d6670", 0.57),
    (10, "#545f6c", 0.59),
    (13, "#6e748b", 0.48),
    (15, "#b4b1c3", 0.30),
    (16, "#ffffff", 0.35),
]
RED_REST = [
    (0, "#f3a99c", 0.85),
    (4, "#e58c7e", 0.88),
    (7, "#d57361", 0.91),
    (8, "#b63825", 0.99),
    (10, "#b43622", 1.0),
    (13, "#ba5642", 0.97),
    (15, "#cb735e", 0.93),
    (16, "#eeb4a4", 0.84),
]

# Not measured: no screenshot catches a caption button under the pointer.
# These keep the rest gradient's shape and move it the way Windows does, by
# lifting the top half and deepening the bottom half.
GREY_HOVER = [
    (0, "#ffffff", 0.80),
    (4, "#eaf6ff", 0.74),
    (7, "#cfe6f7", 0.70),
    (8, "#3f7ca8", 0.78),
    (10, "#3c7aa8", 0.80),
    (13, "#5a93ba", 0.70),
    (15, "#a8cfe6", 0.48),
    (16, "#ffffff", 0.55),
]
GREY_PRESS = [
    (0, "#cfe3f2", 0.85),
    (4, "#b9d6ea", 0.85),
    (7, "#a4c8e0", 0.85),
    (8, "#255a84", 0.90),
    (10, "#24597f", 0.92),
    (13, "#36708f", 0.85),
    (15, "#6fa6c6", 0.60),
    (16, "#cfe3f2", 0.60),
]
RED_HOVER = [
    (0, "#ffc0b2", 0.95),
    (4, "#fba390", 0.97),
    (7, "#f58573", 0.98),
    (8, "#d8301a", 1.0),
    (10, "#d62d16", 1.0),
    (13, "#de4a32", 1.0),
    (15, "#e9735c", 0.97),
    (16, "#ffc9bb", 0.90),
]
RED_PRESS = [
    (0, "#d08b7e", 1.0),
    (4, "#c07466", 1.0),
    (7, "#b05d4f", 1.0),
    (8, "#8d2110", 1.0),
    (10, "#8b1f0e", 1.0),
    (13, "#93301c", 1.0),
    (15, "#a24534", 1.0),
    (16, "#c08a7c", 1.0),
]
# Not measured either: the inactive window. Windows drains the colour out of
# the buttons and leaves the shape.
GREY_OFF = [
    (0, "#ffffff", 0.34),
    (4, "#fbfbfb", 0.30),
    (7, "#e4e4e4", 0.27),
    (8, "#6b6f76", 0.34),
    (10, "#696d74", 0.35),
    (13, "#7b7f88", 0.30),
    (15, "#b9b9c0", 0.20),
    (16, "#ffffff", 0.22),
]
RED_OFF = [
    (0, "#e8cdc8", 0.42),
    (4, "#dfc0ba", 0.44),
    (7, "#d6b3ac", 0.46),
    (8, "#9b665e", 0.50),
    (10, "#99645c", 0.50),
    (13, "#a3726a", 0.48),
    (15, "#b98d85", 0.45),
    (16, "#e6cdc8", 0.42),
]

GREY_LINE = "#242c35"  # dark border under and beside the grey buttons
RED_LINE = "#52281e"  # the same around the close button
OFF_LINE = "#4a4f56"  # inactive
# The 1 px light edge inside each cell, on its two sides only: measured at 0.45
# on the sides, raised after the ShojiWM author's own frame, where every
# button reads as a dark outline with a light line just inside it.
EDGE = ("#ffffff", 0.7)
EDGE_OFF = ("#ffffff", 0.4)

GLYPH_LINE = "#535666"  # glyph outline
GLYPH_FILL = "#ffffff"
GLYPH_LINE_OFF = "#7e8088"
GLYPH_FILL_OFF = "#e8e8e8"


def gradient(stops, gid):
    out = [f'<linearGradient id="{gid}" x1="0" y1="0" x2="0" y2="1">']
    previous = None
    # The measured last row is a light line; the bottom of the group is the
    # dark line alone, so the row above it carries on to the end.
    stops = [stop for stop in stops if stop[0] != 16]
    for row, colour, alpha in stops:
        offset = row / FILL_H
        if previous is not None and row == 8:
            # Hard step between the two halves.
            out.append(
                f'<stop offset="{offset:.4f}" stop-color="{previous[0]}"'
                f' stop-opacity="{previous[1]}"/>'
            )
        out.append(
            f'<stop offset="{offset:.4f}" stop-color="{colour}"'
            f' stop-opacity="{alpha}"/>'
        )
        if row == 15:
            out.append(
                f'<stop offset="{16 / FILL_H:.4f}" stop-color="{colour}"'
                f' stop-opacity="{alpha}"/>'
            )
        previous = (colour, alpha)
    out.append(
        f'<stop offset="1" stop-color="{stops[-1][1]}"'
        f' stop-opacity="{stops[-1][2]}"/>'
    )
    out.append("</linearGradient>")
    return "".join(out)


def rect_outline(x, y, w, h, line, fill):
    """A 1 px outlined rectangle with a filled interior, on the pixel grid."""
    parts = [f'<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{line}"/>']
    if fill:
        parts.append(
            f'<rect x="{x + 1}" y="{y + 1}" width="{w - 2}" height="{h - 2}"'
            f' fill="{fill}"/>'
        )
    return "".join(parts)


def rect_ring(x, y, w, h, line):
    """A 1 px rectangular ring whose middle stays transparent."""
    outer = f"M{x},{y} h{w} v{h} h-{w} Z"
    inner = f"M{x + 1},{y + 1} h{w - 2} v{h - 2} h-{w - 2} Z"
    return f'<path d="{outer} {inner}" fill="{line}" fill-rule="evenodd"/>'


def rect_hollow(x, y, w, h, fill, hole):
    """A filled rectangle with a rectangular hole cut out of it."""
    hx, hy, hw, hh = hole
    outer = f"M{x},{y} h{w} v{h} h-{w} Z"
    inner = f"M{hx},{hy} h{hw} v{hh} h-{hw} Z"
    return f'<path d="{outer} {inner}" fill="{fill}" fill-rule="evenodd"/>'


def runs_to_rects(pixels, colour):
    """Merge a set of (x, y) pixels into one rectangle per horizontal run."""
    out = []
    for y in sorted({p[1] for p in pixels}):
        row = sorted(x for x, py in pixels if py == y)
        start = previous = None
        for x in row + [None]:
            if start is None:
                start = previous = x
                continue
            if x is not None and x == previous + 1:
                previous = x
                continue
            out.append(
                f'<rect x="{start}" y="{y}" width="{previous - start + 1}"'
                f' height="1" fill="{colour}"/>'
            )
            start = previous = x
    return "".join(out)


# The two outer bottom corners of the group are rounded, as on Windows 7: the
# bottom left of the minimize button and the bottom right of the close button.
CORNER = 4


def rounded(w, left, right):
    """The outline of a w x H button, its bottom corners rounded as asked."""
    r = CORNER
    out = [f"M0,0 H{w}"]
    if right:
        out.append(f"V{H - r} A{r},{r} 0 0 1 {w - r},{H}")
    else:
        out.append(f"V{H}")
    if left:
        out.append(f"H{r} A{r},{r} 0 0 1 0,{H - r}")
    else:
        out.append("H0")
    out.append("Z")
    return " ".join(out)


def corner_line(w, left, right, colour):
    """The dark line round the rounded corners, 1 px inside the clip."""
    r = CORNER
    out = []
    if left:
        out.append(f"M0,{H - r - 1} V{H - r} A{r},{r} 0 0 0 {r},{H} H{r + 1}")
    if right:
        out.append(
            f"M{w},{H - r - 1} V{H - r} A{r},{r} 0 0 1 {w - r},{H} H{w - r - 1}"
        )
    if not out:
        return ""
    return (
        f'<path d="{" ".join(out)}" fill="none" stroke="{colour}"'
        ' stroke-width="2" shape-rendering="auto" clip-path="url(#c)"/>'
    )


ORIGIN = 1  # x=0 is the dark line, so the cell starts one pixel in


def glyph_minimize(cell_w, line, fill):
    """12 x 5 outlined rectangle, columns 8..19, rows 8..12 of the cell."""
    x = ORIGIN + round((cell_w - 12) / 2)
    return rect_outline(x, 8, 12, 5, line, fill)


def glyph_maximize(cell_w, line, fill):
    """12 x 10 rectangle, a 6 x 4 hole at columns 10..15, rows 6..9."""
    x = ORIGIN + round((cell_w - 12) / 2)
    out = [
        rect_ring(x, 3, 12, 10, line),
        # The hole keeps the fill of the button, so the white is cut away
        # and nothing opaque is left under it.
        rect_hollow(x + 1, 4, 10, 8, fill, (x + 3, 6, 6, 4)),
        rect_ring(x + 3, 6, 6, 4, line),
    ]
    return "".join(out)


def glyph_restore(cell_w, line, fill):
    """The front window, and the corner of one behind it."""
    x = ORIGIN + round((cell_w - 12) / 2)
    out = [
        # the one behind, drawn first and mostly covered
        rect_outline(x + 2, 2, 10, 10, line, fill),
        # the front one, with its hole cut out
        rect_ring(x, 4, 10, 10, line),
        rect_hollow(x + 1, 5, 8, 8, fill, (x + 3, 7, 4, 4)),
        rect_ring(x + 3, 7, 4, 4, line),
    ]
    return "".join(out)


def glyph_close(cell_w, line, fill):
    """An 11 x 10 cross: two 3 px strokes with a 1 px outline.

    The strokes move 6 columns over 7 rows, which is the measured slope, so
    each row is a run of three pixels at `1 + (r - 1) * 6 / 7`. Drawing them
    as runs keeps every edge on the pixel grid; a stroked diagonal would be
    soft and would bulge where the arms cross.
    """
    x = ORIGIN + round((cell_w - 11) / 2)
    white = set()
    for r in range(1, 9):
        a = round(1 + (r - 1) * 6 / 7)
        b = 8 - a
        for k in range(3):
            white.add((x + a + k, 3 + r))
            white.add((x + b + k, 3 + r))
    # The outline is the 4-neighbour ring around the strokes.
    dark = set()
    for px, py in white:
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            if (px + dx, py + dy) not in white:
                dark.add((px + dx, py + dy))
    return runs_to_rects(dark, line) + runs_to_rects(white, fill)


GLYPHS = {
    "min": glyph_minimize,
    "max": glyph_maximize,
    "restore": glyph_restore,
    "close": glyph_close,
}


def button(name, cell_w, stops, line_colour, glyph, edge, glyph_colours):
    """One caption button: the dark lines, the fill, the edges, the glyph."""
    right_line = 1 if glyph == "close" else 0
    left = glyph == "min"
    right = glyph == "close"
    total_w = 1 + cell_w + right_line
    edge_colour, edge_alpha = edge
    glyph_line, glyph_fill = glyph_colours
    body = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{total_w}"'
        f' height="{H}" viewBox="0 0 {total_w} {H}"'
        f' shape-rendering="crispEdges">',
        "<!-- Drawn by tools/make-caption-svgs.py from docs/aero-measurements.md. -->",
        f"<defs>{gradient(stops, 'g')}"
        f'<clipPath id="c"><path d="{rounded(total_w, left, right)}"'
        ' shape-rendering="auto"/></clipPath></defs>',
        '<g clip-path="url(#c)">',
        # the fill
        f'<rect x="1" y="0" width="{cell_w}" height="{FILL_H}" fill="url(#g)"/>',
        # the 1 px light edge on both sides of the cell
        f'<rect x="1" y="0" width="1" height="{FILL_H}" fill="{edge_colour}"'
        f' fill-opacity="{edge_alpha}"/>',
        f'<rect x="{cell_w}" y="0" width="1" height="{FILL_H}"'
        f' fill="{edge_colour}" fill-opacity="{edge_alpha}"/>',
        # No light line along the top or the bottom: the group hangs straight
        # from the window's dark outline, and its bottom is the one dark line.
        # the dark line on the left, and under the whole button
        f'<rect x="0" y="0" width="1" height="{H}" fill="{line_colour}"/>',
        f'<rect x="0" y="{H - 1}" width="{total_w}" height="1"'
        f' fill="{line_colour}"/>',
    ]
    if right_line:
        body.append(
            f'<rect x="{total_w - 1}" y="0" width="1" height="{H}"'
            f' fill="{line_colour}"/>'
        )
    body.append(
        '<g shape-rendering="auto">'
        + GLYPHS[glyph](cell_w, glyph_line, glyph_fill)
        + "</g>"
    )
    body.append("</g>")
    body.append(corner_line(total_w, left, right, line_colour))
    body.append("</svg>")
    return "\n".join(body)


def main():
    os.makedirs(OUT, exist_ok=True)
    on = (GLYPH_LINE, GLYPH_FILL)
    off = (GLYPH_LINE_OFF, GLYPH_FILL_OFF)
    files = {
        "min": ("min", 27, GREY_REST, GREY_LINE, EDGE, on),
        "min-hover": ("min", 27, GREY_HOVER, GREY_LINE, EDGE, on),
        "min-press": ("min", 27, GREY_PRESS, GREY_LINE, EDGE, on),
        "min-inactive": ("min", 27, GREY_OFF, OFF_LINE, EDGE_OFF, off),
        "max": ("max", 26, GREY_REST, GREY_LINE, EDGE, on),
        "max-hover": ("max", 26, GREY_HOVER, GREY_LINE, EDGE, on),
        "max-press": ("max", 26, GREY_PRESS, GREY_LINE, EDGE, on),
        "max-inactive": ("max", 26, GREY_OFF, OFF_LINE, EDGE_OFF, off),
        "restore": ("restore", 26, GREY_REST, GREY_LINE, EDGE, on),
        "restore-hover": ("restore", 26, GREY_HOVER, GREY_LINE, EDGE, on),
        "restore-press": ("restore", 26, GREY_PRESS, GREY_LINE, EDGE, on),
        "restore-inactive": ("restore", 26, GREY_OFF, OFF_LINE, EDGE_OFF, off),
        "close": ("close", 46, RED_REST, RED_LINE, EDGE, on),
        "close-hover": ("close", 46, RED_HOVER, RED_LINE, EDGE, on),
        "close-press": ("close", 46, RED_PRESS, RED_LINE, EDGE, on),
        "close-inactive": ("close", 46, RED_OFF, OFF_LINE, EDGE_OFF, off),
    }
    for name, (glyph, width, stops, line, edge, colours) in files.items():
        svg = button(name, width, stops, line, glyph, edge, colours)
        path = os.path.join(OUT, name + ".svg")
        with open(path, "w") as handle:
            handle.write(svg + "\n")
        print("wrote", os.path.relpath(path, os.path.dirname(HERE)))


if __name__ == "__main__":
    main()
