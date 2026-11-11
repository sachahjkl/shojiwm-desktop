# Handoff

The shell is `shojiwm-shell` (`crates/shell`, Rust on AntiWebUI). It replaced
the Quickshell shell (`shell/*.qml`), which is gone; `shell/` keeps only the
pictures and `gtk-switch.sh`. The passes that built the QML shell (pinned
programs, jump lists, the Start menu, the lock screen, the Control Panel,
light and dark, the desktop icons, the orb) are in the git history before
commit `7cda0c8`; their behaviour carried over to the Rust shell and the
README describes it.

Everything below was checked with a nested compositor inside an X server with
no screen, Mesa in software, at 1280x800 and scale 1. Nothing was ever run on
a real GPU by these passes.

## Validated by the owner — do not change

- **The top-right corner of a window and the three caption buttons (Aero).**
  The owner looked at the zoomed capture and called it right. The geometry,
  the colours and the placement of `assets/caption/*.svg`, of the group in
  `src/themes/aero/frame.tsx`, and of the numbers in
  `src/themes/aero/metrics.ts` that drive them must stay as they are. Change
  them only if a measurement proves a bug, and say why.
- **The title text.** Plain dark text on light glass, no glow: the owner
  compared with the ShojiWM author's own Windows 7 setup.
- **Neutral lines.** Every frame and popup line is translucent black or
  white. The navy and cyan in the author's screenshot are those alphas seen
  over a blue wallpaper; never copy them as opaque colours.
- **No "modern" flat style.** The owner rejected it twice. A square variant
  of Aero exists (`squareCorners`); Kernel is a separate theme with its own
  identity, not a flattened Aero.

## What remains, in priority order

1. **Look at all of it on a real GPU.** Colours, blur, animation speed and
   input have never been seen outside llvmpipe, in either theme.
2. **The Kernel screenshots.** `THEME=kernel OUT=assets/site/kernel
tools/screenshots.sh`; the site and the README use them.
3. **Live taskbar thumbnails.** The shell asks for a capture of each window
   (`crates/shell/src/taskbar/thumbnails.rs`). In the headless session no
   picture arrives and the card keeps the icon. Check on a real session.
4. **The inactive window.** `COLOR.glassInactive` in
   `src/themes/aero/metrics.ts` is very pale. No 1:1 reference of an inactive
   window could be downloaded, so it is a guess.
5. **The hover and pressed caption buttons.** `tools/make-caption-svgs.py`
   holds them. They are invented: no screenshot catches a caption button
   under the pointer.
6. **The Start menu, Alt+Tab and the menus.** They are in the Aero spirit but
   were never measured.
7. **The taskbar glass alpha.** Only one wallpaper was available behind a
   reference taskbar, so colour and alpha could not be separated.
8. **Peek at one window** from its thumbnail. The shell knows a
   foreign-toplevel handle, the compositor a `WaylandWindow`; nothing maps one
   to the other over the socket yet (title and app id would be a start).

## Known defects

| Defect                                                                                                                                 | Where                        | Why                                                                                                                                                             |
| -------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| The Alt+Tab panel has square corners.                                                                                                  | `src/alt-tab.tsx`            | `<Solid>` has no radius. The titles are drawn by the shell (`crates/shell/src/dialogs/alt_tab.rs`), since a composition has no label node.                      |
| The caption button group is 103 px wide. The reference also has a 1 px light bevel outside its left and right dark borders, so 105 px. | `tools/make-caption-svgs.py` | Measured late, after the owner validated the corner. Left alone on purpose.                                                                                     |
| `tools/details.py` needs the frame sizes.                                                                                              | `tools/details.py`           | It finds a window by its close button, then uses the size the capture asked for. If `tools/screenshots.sh` changes the `foot` window size, change it there too. |

## The measurements

`docs/aero-measurements.md` holds every number and the image it came from:
frame layers and radii, the caption button group and its gradients row by
row, the four glyphs pixel by pixel, the title metrics, the maximised
metrics, and the taskbar. Change a number there and in
`tools/make-caption-svgs.py` together, then run:

```sh
python3 -I tools/make-caption-svgs.py
```

The reference images are **not** in the repository and must not be committed.

## Running it on a real session

```sh
# once, to put the configuration in ~/.config/shojiwm
nix run github:sachahjkl/shojiwm-desktop#install
nix profile install github:sachahjkl/shojiwm-desktop#shell

# on this checkout, nested in the current session
tools/dev.sh
```

`Super`+`Shift`+`R` reloads the configuration without restarting the
session. Logs are in `~/.local/state/shoji_wm/logs/latest.log`. A shader that
fails to compile is reported there and the component falls back to its
built-in look, so grep for `shader` after a change to `assets/*.frag`.

## Taking captures, and comparing

```sh
# the whole headless session, Aero then Kernel
tools/screenshots.sh
THEME=kernel OUT=assets/site/kernel tools/screenshots.sh

# the zoomed crops of one capture
python3 -I tools/details.py assets/site/desktop.png assets/site/details \
  front=770x569 back=616x398

# a reference next to one of our captures, with the error per region
python3 -I tools/compare.py REFERENCE.png 12,12 CAPTURE.png 355,215
```

`tools/compare.py` writes its side-by-side to `/tmp`: it contains reference
pixels and must never be committed. Open a window of exactly 770 x 569 to
compare with the Notepad++ reference; `tools/screenshots.sh` already does.

## Reference images used

Downloaded to a temporary directory for measuring, never committed.

- <https://commons.wikimedia.org/wiki/File:Clementine_0.6_in_Windows.png>
- <https://commons.wikimedia.org/wiki/File:Notepad%2B%2B_screenshot2.png>
- <https://commons.wikimedia.org/wiki/File:Aoo_calc_3.4.png>
- <https://commons.wikimedia.org/wiki/File:PaintDotNET_Screenshot_zh-hans.png>
- <https://commons.wikimedia.org/wiki/File:Qt_Creator_3.1.1_editing_a_sample_UI_file_from_Qt_5.3_using_Designer.png>
- <https://commons.wikimedia.org/wiki/File:Clase_de_computaci%C3%B3n,_Carrera_Fisioterapia.png>
- <https://commons.wikimedia.org/wiki/File:7-Zip_9.20_DE.png>
