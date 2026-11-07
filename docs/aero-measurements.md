# Measurements of Windows 7 Aero

Every number below was read off a real Windows 7 screenshot with Pillow, one
pixel at a time. Nothing here is estimated by eye, except where it says so.

The reference images are **not** in this repository. The table lists where
each one comes from; download them yourself if you want to check the figures.

## The references

| Key    | File                                           | Size      | Source                                                                                                             |
| ------ | ---------------------------------------------- | --------- | ------------------------------------------------------------------------------------------------------------------ |
| `clem` | Clementine 0.6 in Windows.png                  | 1115×636  | <https://commons.wikimedia.org/wiki/File:Clementine_0.6_in_Windows.png>                                            |
| `npp`  | Notepad++ screenshot2.png                      | 800×600   | <https://commons.wikimedia.org/wiki/File:Notepad%2B%2B_screenshot2.png>                                            |
| `aoo`  | Aoo calc 3.4.png                               | 1278×1002 | <https://commons.wikimedia.org/wiki/File:Aoo_calc_3.4.png>                                                         |
| `pdn`  | PaintDotNET Screenshot zh-hans.png             | 966×738   | <https://commons.wikimedia.org/wiki/File:PaintDotNET_Screenshot_zh-hans.png>                                       |
| `qt`   | Qt Creator 3.1.1 editing a sample UI file…png  | 1212×734  | <https://commons.wikimedia.org/wiki/File:Qt_Creator_3.1.1_editing_a_sample_UI_file_from_Qt_5.3_using_Designer.png> |
| `desk` | Clase de computación, Carrera Fisioterapia.png | 1366×768  | <https://commons.wikimedia.org/wiki/File:Clase_de_computaci%C3%B3n,_Carrera_Fisioterapia.png>                      |

All six are PNG screenshots at 1:1, 96 DPI, no scaling. `clem`, `npp`, `aoo`
and `pdn` show an **active** window with the glass frame; `desk` shows the
taskbar with the start orb.

Three of them sit on very different backdrops — `npp` on black, `clem` on a
mid-blue wallpaper, `aoo` on a pale sky — which is what makes it possible to
separate a translucent colour into "colour" and "alpha": with
`C = a·F + (1-a)·B` measured over two backdrops `B1` and `B2`,
`a = 1 − (C1−C2)/(B1−B2)` per channel.

## What was not found

- **No hover or pressed caption button.** A screenshot almost never catches
  one. Those two states are drawn from the rest state and are the only part of
  the caption that is invented. They are marked below.
- **No inactive window at 1:1.** The inactive state follows Windows'
  documented behaviour (same geometry, paler glass, grey text, grey close
  button) rather than measured pixels.
- **No Alt+Tab or Start menu at 1:1** in the images that could be fetched;
  Wikimedia rate-limits bulk downloads of originals. Those two are built from
  the taskbar measurements and the published layout.

## The frame

Measured on `clem` (window at 0,0) and `npp` (window at 12,12). Both agree.

| Item                                     | Value                                                    | Where                                                                                                       |
| ---------------------------------------- | -------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| Caption height, window top to client top | **30 px**                                                | `npp` x=400: dark y=12, light y=13, glass y=14..39, light y=40, dark y=41, client y=42                      |
| Left, right and bottom border            | **8 px**                                                 | `npp` y=200: x=13 dark, 14 light, 15..18 glass, 19 light, 20 dark, 21 client                                |
| Layers, outside inwards                  | dark 1 px, light 1 px, glass 4 px, light 1 px, dark 1 px | same                                                                                                        |
| Top corner radius                        | **6 px**                                                 | `npp`: the dark line meets the top edge 6 px in (x=18 at y=12) and the left edge 6 rows down (x=13 at y=18) |
| Bottom corner radius                     | **5 px**                                                 | `npp` bottom-left, rows 575..580                                                                            |
| Caption glass gradient                   | **none**                                                 | `npp` x=400, y=14..39 is `#28445b` on every row                                                             |

Colours, solved over two backdrops:

| Item                    | Colour    | Alpha | Reading                                                                                                                     |
| ----------------------- | --------- | ----- | --------------------------------------------------------------------------------------------------------------------------- |
| Outer dark line         | `#000000` | 0.80  | `#0e1722` over a `#4a6fa0` desktop (`clem`), `#020304` over black (`npp`)                                                   |
| Inner light line        | `#ffffff` | 0.92  | `#e9eef4` over `#2c6297` glass, `#f7f8f9` over `#28445b` glass                                                              |
| Glass tint              | `#538ebe` | 0.48  | `a·F = (40,68,91)` straight off the black backdrop of `npp`; the same pair predicts `#7bacd8` for `aoo`, measured `#86b2e0` |
| Client edge, light line | `#ffffff` | 0.70  | `#dfe2e6` (`npp`), `#a2bad1` (`clem`)                                                                                       |
| Client edge, dark line  | `#0b1b2a` | 0.75  | `#1b1e22` (`npp`), `#1b344d` (`clem`)                                                                                       |

There is **no sheen or gloss over the caption**: the glass is the blurred
backdrop plus the flat tint, plus the 1 px light line at the very top.

## The caption buttons

The three buttons are **one joined group**, not three separate buttons.
Measured identically on `clem` (x 1005..1107), `npp` (x 672..774) and `aoo`
(x 1174..1276).

| Item                   | Value                                                                                                                                                                                                                                                                                                                       |
| ---------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Group size             | **103 × 18 px**                                                                                                                                                                                                                                                                                                             |
| Top of the group       | the **first glass row**, 2 px below the window's outer top edge. The group has **no top border of its own**: the frame's 1 px dark line and 1 px light line are its top edge, so the outline runs on without a seam. `clem` x=1016: y=0 `#060e18` (frame, dark), y=1 `#fcfcfc` (frame, light), y=2 `#b5c2d1` (button fill). |
| Right of the group     | its 1 px dark border sits **7 px** from the window's outer right edge, on the same column as the frame's inner dark line. `clem` W=1115: border at x=1107, outer edge at x=1114.                                                                                                                                            |
| Bottom of the group    | 1 px dark line on row 17, then glass again: the group hangs from the top, it is not centred in the caption.                                                                                                                                                                                                                 |
| Columns, left to right | dark 1 px · **minimize 27** · dark 1 px · **maximize 26** · dark 1 px · **close 46** · dark 1 px                                                                                                                                                                                                                            |
| Inside each cell       | 1 px light edge, the fill, 1 px light edge                                                                                                                                                                                                                                                                                  |
| Rows                   | fill rows 0..16, bright line on row 16, dark border on row 17                                                                                                                                                                                                                                                               |

The fill is a two-part gradient with a hard step at row 8, solved over the
`clem` and `npp` backdrops:

| Row         | Minimize / maximize | Close            |
| ----------- | ------------------- | ---------------- |
| 0           | `#ffffff` α 0.53    | `#f3a99c` α 0.85 |
| 4           | `#fffcf9` α 0.47    | `#e58c7e` α 0.88 |
| 7           | `#dbd7d5` α 0.43    | `#d57361` α 0.91 |
| 8           | `#5d6670` α 0.57    | `#b63825` α 0.99 |
| 10          | `#545f6c` α 0.59    | `#b43622` α 1.00 |
| 13          | `#6e748b` α 0.48    | `#ba5642` α 0.97 |
| 15          | `#b4b1c3` α 0.30    | `#cb735e` α 0.93 |
| 16          | `#ffffff` α 0.35    | `#eeb4a4` α 0.84 |
| 17 (border) | `#242c35` opaque    | `#52281e` opaque |

### The glyphs

Read pixel by pixel off `npp` and `aoo`. Outline `#535666`, fill white.

| Glyph    | Box                                                                 | Position in its cell             |
| -------- | ------------------------------------------------------------------- | -------------------------------- |
| Minimize | 12 × 5 outlined rectangle                                           | columns 8..19, rows 8..12        |
| Maximize | 12 × 10 outlined rectangle, 6 × 4 hole at columns 10..15, rows 6..9 | columns 7..18, rows 3..12        |
| Restore  | two 10 × 10 rectangles, the second offset 3 px right and 2 px up    | `aoo`, columns 6..17, rows 2..13 |
| Close    | 11 × 10 cross, 3 px white strokes with a 1 px outline               | columns 17..27, rows 3..12       |

The glyphs are centred horizontally in their cells. Minimize sits 1.5 px
below the vertical centre; maximize and close 1 px above it.

### Maximised

Measured on `aoo`, which is a maximised window filling a 1278 × 1002 frame.
Windows pushes the frame off the screen, so the border and the top of the
caption go with it.

| Item               | Normal   | Maximised                                       | Reading                                                        |
| ------------------ | -------- | ----------------------------------------------- | -------------------------------------------------------------- |
| Caption height     | 30 px    | **22 px**                                       | `aoo` x=600: glass y=0..19, light y=20, dark y=21, client y=22 |
| Border             | 8 px     | **2 px** (only the two lines around the client) | `aoo` y=400: x=0 `#a0a0a0`, x=1 `#696969`, x=2 client          |
| Top of the group   | 2 px     | **0**, flush with the top edge                  | `aoo` x=1240: red at y=0                                       |
| Right of the group | 7 px     | **1 px**                                        | `aoo` y=8: dark border at x=1276, light at x=1277, W=1278      |
| Group size         | 103 × 18 | **unchanged**                                   | `aoo` x=1174..1276                                             |
| Corners            | rounded  | square                                          | `aoo` row 0 runs to the edge                                   |

The close button is **not** wider when the window is maximised; that is a
later Windows. No reference at 125% could be found, so the scaling is left to
the compositor: lines stay one physical pixel, everything else scales.

## The title

| Item           | Value                                                                                      | Reading                                                                                                            |
| -------------- | ------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------ |
| Face           | Segoe UI, 9 pt = **12 px** at 96 DPI, normal weight                                        | cap height 9 px measured on `clem` (a stem runs y=10..18)                                                          |
| Colour, active | black                                                                                      | darkest text pixel `#062362` over `#2c6297` glass                                                                  |
| Icon           | 16 × 16, its left edge **11 px** from the window's outer left edge, centred in the caption | `npp`: the icon's pixels run x=11..26, y=7..22, window-relative                                                    |
| Text start     | **3 px** after the icon box, so 30 px from the window's left edge                          | `npp`: first text pixels at x=30                                                                                   |
| Glow           | white halo, peak α ≈ **0.55** at the text's own middle, gone 12 px away                    | `clem` x=100: `#a5b7ce` at the text middle against `#2c6297` glass far from it, decaying to `#4f73a2` 13 rows down |

The glow is the text shape blurred, so it fills the gaps between letters and
fades in every direction. It is **not** an outline.

## The taskbar

Measured on `desk` (1366×768, taskbar at y=728..767).

| Item                              | Value                                                                                           |
| --------------------------------- | ----------------------------------------------------------------------------------------------- |
| Height                            | **40 px**                                                                                       |
| Top edge                          | 1 px dark line `#313030`, then 1 px light line `#a6a7a4`                                        |
| Body, over a dark green wallpaper | `#484c47` at the top, down to `#131b13` at 45%, up to `#243121` at 75%, `#0e120e` at the bottom |
| Start orb                         | ≈ 38 px across, left margin ≈ 8 px, centred in the bar                                          |

The taskbar alpha could not be solved: only one wallpaper was available
behind it. Its gradient is therefore reproduced as measured over that
wallpaper, which makes it slightly too dark over a light one.
