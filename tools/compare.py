#!/usr/bin/env python3
"""Put a Windows 7 screenshot and one of ours side by side, and measure.

    python3 -I tools/compare.py REFERENCE REF_X,REF_Y CAPTURE CAP_X,CAP_Y

`REF_X,REF_Y` and `CAP_X,CAP_Y` are the top-left corner of the window in each
picture. The two windows must be the same size; `tools/screenshots.sh` opens
one that matches the reference.

It prints the mean and worst absolute colour error for each part of the
frame, and writes the two crops side by side to /tmp.

The glass itself is a blur of whatever is behind the window, which is a
different desktop in the two pictures, so the error over the glass says
nothing. The lines, the corners and the buttons are what this measures.

The side-by-side carries reference pixels, so it stays in /tmp: it is never
committed.
"""

import os
import sys

try:
    from PIL import Image
except ImportError:  # pragma: no cover - told to the user, not handled
    sys.exit(
        "needs Pillow:\n"
        "  nix shell --impure --expr 'let p = builtins.getFlake \"nixpkgs\"; "
        "pkgs = p.legacyPackages.x86_64-linux; in "
        "pkgs.python3.withPackages (ps: [ ps.pillow ])' -c "
        "python3 -I tools/compare.py ..."
    )

OUT = "/tmp/shojiwm-desktop-compare"


def regions(w, h):
    """Named rectangles inside the window, left to right, top to bottom."""
    return [
        ("corner top left", 0, 0, 14, 14),
        ("corner top right", w - 14, 0, 14, 14),
        ("corner bottom left", 0, h - 14, 14, 14),
        ("corner bottom right", w - 14, h - 14, 14, 14),
        ("border left", 0, 80, 8, 120),
        ("border right", w - 8, 80, 8, 120),
        ("border bottom", 80, h - 8, 120, 8),
        ("caption top lines", 40, 0, w - 160, 2),
        ("caption buttons", w - 111, 0, 111, 20),
        ("client edge, top", 40, 28, w - 120, 2),
        ("client edge, left", 6, 60, 2, 120),
    ]


def crop(image, x, y, w, h):
    return image.crop((x, y, x + w, y + h))


def error(a, b):
    pa, pb = a.load(), b.load()
    total = 0.0
    worst = 0
    count = 0
    for y in range(a.size[1]):
        for x in range(a.size[0]):
            for ca, cb in zip(pa[x, y], pb[x, y]):
                d = abs(ca - cb)
                total += d
                worst = max(worst, d)
                count += 1
    return (total / count if count else 0.0), worst


def main():
    if len(sys.argv) != 5:
        sys.exit(__doc__)
    ref_path, ref_origin, cap_path, cap_origin = sys.argv[1:]
    rx, ry = (int(v) for v in ref_origin.split(","))
    cx, cy = (int(v) for v in cap_origin.split(","))

    reference = Image.open(ref_path).convert("RGB")
    capture = Image.open(cap_path).convert("RGB")

    w = min(reference.size[0] - rx, capture.size[0] - cx)
    h = min(reference.size[1] - ry, capture.size[1] - cy)
    print(f"window {w} x {h}")
    print(f"{'region':22s} {'mean':>7s} {'worst':>6s}")

    for name, x, y, rw, rh in regions(w, h):
        if x < 0 or y < 0 or x + rw > w or y + rh > h:
            continue
        a = crop(reference, rx + x, ry + y, rw, rh)
        b = crop(capture, cx + x, cy + y, rw, rh)
        mean, worst = error(a, b)
        print(f"{name:22s} {mean:7.1f} {worst:6d}")

    os.makedirs(OUT, exist_ok=True)
    left = crop(reference, rx, ry, w, h)
    right = crop(capture, cx, cy, w, h)
    both = Image.new("RGB", (w * 2 + 8, h), (20, 20, 20))
    both.paste(left, (0, 0))
    both.paste(right, (w + 8, 0))
    path = os.path.join(OUT, "side-by-side.png")
    both.save(path)
    print(f"\nwrote {path} (reference on the left; kept out of the repository)")


if __name__ == "__main__":
    main()
