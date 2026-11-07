#!/usr/bin/env python3
"""Cut the zoomed detail crops out of a capture.

    python3 -I tools/details.py CAPTURE OUTDIR [NAME=x,y,w,h,zoom ...]

With no extra arguments it finds the windows itself and cuts the parts of the
frame that are worth looking at closely: the four corners, the joins between
the caption and the borders, the caption buttons and the title.

A window is found by its close button, the only strong red in the picture.
The button group is 103 px wide and sits 7 px from the right edge of the
window and 2 px below its top, so one red rectangle gives the whole frame.
"""

import os
import sys

try:
    from PIL import Image
except ImportError:  # pragma: no cover
    sys.exit("needs Pillow; see tools/compare.py for the one-liner")

ZOOM = 8


def red_boxes(image):
    """The close buttons, as (x0, y0, x1, y1), left to right."""
    w, h = image.size
    px = image.load()
    seen = set()
    boxes = []
    for y in range(h):
        for x in range(w):
            if (x, y) in seen:
                continue
            r, g, b = px[x, y]
            if not (r > 110 and r - g > 45 and r - b > 45):
                continue
            # flood the run of red around it
            stack = [(x, y)]
            cells = []
            while stack:
                cx, cy = stack.pop()
                if (cx, cy) in seen or not (0 <= cx < w and 0 <= cy < h):
                    continue
                pr, pg, pb = px[cx, cy]
                if not (pr > 100 and pr - pg > 35 and pr - pb > 35):
                    continue
                seen.add((cx, cy))
                cells.append((cx, cy))
                stack += [(cx + 1, cy), (cx - 1, cy), (cx, cy + 1), (cx, cy - 1)]
            if len(cells) < 200:
                continue
            xs = [c[0] for c in cells]
            ys = [c[1] for c in cells]
            boxes.append((min(xs), min(ys), max(xs), max(ys)))
    boxes.sort()
    return boxes


def frames(image):
    """Every window in the picture, as (left, top, right, bottom)."""
    out = []
    for x0, y0, x1, y1 in red_boxes(image):
        if x1 - x0 < 30 or y1 - y0 < 10:
            continue
        # the red fill stops 2 px short of the group's right edge, which is
        # itself 7 px from the window's edge
        right = x1 + 9
        top = y0 - 2
        out.append((right, top))
    return out


def cut(image, out, name, x, y, w, h, zoom=ZOOM):
    iw, ih = image.size
    x, y = max(0, x), max(0, y)
    w, h = min(w, iw - x), min(h, ih - y)
    if w <= 0 or h <= 0:
        print("skipped", name)
        return
    crop = image.crop((x, y, x + w, y + h))
    crop = crop.resize((w * zoom, h * zoom), Image.NEAREST)
    crop.save(os.path.join(out, name + ".png"))
    print("wrote", os.path.join(os.path.basename(out), name + ".png"))


def window_box(right, top, size):
    """The whole frame. `size` is the frame size the capture asked for."""
    fw, fh = size
    return right - fw + 1, top, right, top + fh - 1


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    path, out = sys.argv[1], sys.argv[2]
    os.makedirs(out, exist_ok=True)
    image = Image.open(path).convert("RGB")

    extra = []
    for arg in sys.argv[3:]:
        name, numbers = arg.split("=", 1)
        if name in ("front", "back"):
            continue
        values = [int(v) for v in numbers.split(",")]
        extra.append((name, values))
    for name, values in extra:
        cut(image, out, name, *values)
    if extra:
        return

    found = frames(image)
    if not found:
        sys.exit("no window found in " + path)

    # The capture opens its windows at sizes it chooses, and passes them
    # here, so no edge has to be guessed.
    front_size = (770, 569)
    back_size = (616, 398)
    for arg in sys.argv[3:]:
        if arg.startswith("front="):
            front_size = tuple(int(v) for v in arg[6:].split("x"))
        elif arg.startswith("back="):
            back_size = tuple(int(v) for v in arg[5:].split("x"))

    # The front window is the one whose caption sits lowest: the capture
    # moves it down and to the right.
    front = window_box(*max(found, key=lambda f: f[1]), front_size)
    back = window_box(*min(found, key=lambda f: f[1]), back_size)
    print("front window", front)
    print("other window", back)

    for label, (left, top, right, bottom) in (
        ("", front),
        ("inactive-", back),
    ):
        cut(image, out, label + "corner-top-left", left, top, 24, 24)
        cut(image, out, label + "corner-top-right", right - 24, top, 24, 24)
        cut(image, out, label + "corner-bottom-left", left, bottom - 24, 24, 24)
        cut(
            image,
            out,
            label + "corner-bottom-right",
            right - 24,
            bottom - 24,
            24,
            24,
        )
        cut(image, out, label + "caption-buttons", right - 118, top - 1, 118, 24, 7)
        cut(image, out, label + "caption-title", left, top, 180, 32, 6)
        cut(image, out, label + "caption-to-border", left, top + 18, 24, 24)
        cut(image, out, label + "client-edge", left + 2, top + 24, 22, 18)


if __name__ == "__main__":
    main()
