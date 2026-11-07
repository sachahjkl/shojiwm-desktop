# Handoff

`wip-aero-polish` carries the first pass, done on a server with no display.
`claude/terminer-passation-wywvdz` continues it: it holds `wip-aero-polish`
unchanged plus the second pass below. `main` is untouched and still holds the
state before both.

Everything below was checked with a nested compositor inside an X server with
no screen, Mesa in software, at 1280x800 and scale 1. Nothing was ever run on
a real GPU.

## Validated by the owner — do not change

- **The top-right corner of a window and the three caption buttons.** The
  owner looked at the zoomed capture and called it right. The geometry, the
  colours and the placement of `assets/caption/*.svg`, of the group in
  `src/decoration.tsx`, and of the numbers in `src/theme.ts` that drive them
  must stay as they are. Change them only if a measurement proves a bug, and
  say why.
- **The title text.** The old version drew ten offset copies of the label as
  a fake glow, which put a hard white outline around every letter. It is now
  crisp dark text over one soft shape (`assets/title-glow.svg`). The owner
  asked for that and has seen the result.

## Done, not reviewed

1. The whole frame is painted by `assets/frame.frag`: 1 px dark outer line,
   1 px light line, the glass, 1 px light line, 1 px dark line, then the
   client. Top corners 6 px, bottom corners 5 px, square when maximised.
2. Maximised windows use the measured metrics: 22 px caption, 2 px border,
   the button group flush with the top and 1 px from the right.
3. The shell draws its text with grey antialiasing
   (`QSG_DISTANCEFIELD_ANTIALIASING=gray` in `shell/shell.qml`). Before that,
   every glyph of the bar and the menus had red and blue fringes.
4. The taskbar's top edge is a dark hairline with a lit one under it, and the
   body follows the Windows gradient shape.
5. The start orb is 36 px, 8 px from the left edge.
6. The wallpaper is brighter, so the glass reads like Aero rather than like
   smoked glass.

## Second pass: written, not reviewed

1. **Pinned taskbar programs.** `taskbarPinnedIds` in `shell/Apps.qml`. A
   pinned program with no window is an icon-only button (`pinnedButtonWidth`,
   `pinnedIconSize` in `shell/theme.js`); a click starts it, and its windows
   then gather behind the same button. With `combineWindows` off, the first
   window takes the pinned button's place and the others follow it. A middle
   click on any button starts another instance.
2. **A bug in the Start menu's pinned list.** Quickshell keys `.desktop`
   entries by the name without the extension, so
   `DesktopEntries.byId("kitty.desktop")` was null and the list was always
   empty. `Apps.entryById` strips the extension for both lists.
3. **Recent files in the jump list.** `Apps.recentFiles` reads
   `$XDG_DATA_HOME/recently-used.xbel` through a `FileView` that watches it.
   A file belongs to a program when one of its `bookmark:application` entries
   names the program (its id, name, `StartupWMClass`, the first word of its
   command, or the last part of a reverse-DNS id). The file is opened with the
   program's own command plus the path. Eight at most (`recentLimit`).
4. **Tray submenus.** `TrayMenu.qml` loads itself through a `LazyLoader` for
   an entry with children, beside the line, after `submenuDelay` or on a
   click. Check boxes and radio buttons show a check mark.
5. **Aero Peek.** `src/peek.tsx` serves a socket
   (`$XDG_RUNTIME_DIR/shojiwm-<display>-win7.sock`, handed to the shell in
   `SHOJIWM_WIN7_SOCKET`) with ShojiWM's own IPC. `peek {on}` swaps the output
   composition for the background layers, one glass outline per window, then
   the top layers. `show-desktop` calls the window manager's toggle, so the
   strip's second click brings the windows back, as `Super`+`D` does. The
   shell side is `shell/CompositorIpc.qml`; without the socket the strip falls
   back to minimising everything.
6. `tools/screenshots.sh` pins `foot` and `footclient` in its copy of the
   configuration and writes a `recently-used.xbel` naming `foot`, since the
   test machine has neither the default programs nor anything that records
   recent files. It also takes `peek.png`. `SHOJIWM_FLAKE` points it at a
   local ShojiWM clone.

Run in the headless session (`tools/screenshots.sh`, ShojiWM `9e778dc` built
from a local clone, Quickshell 0.3.2): the pinned idle button, the jump
list's "Recent" section, and Aero Peek all show in the captures, and the shell
stayed up through every step. Not exercised: a tray submenu (CopyQ has none),
and clicks on a recent file, a pinned button or the show-desktop strip.

## Third pass: the Start menu

Asked for by the owner: the name ran into the right edge, and the power
button did one thing only.

1. The name is the full name from `getent passwd` (else the login name), cut
   short with an ellipsis and kept `menuPadding` off the right edge.
2. The power button is split: the button runs `powerButtonAction`, the arrow
   opens `PowerMenu.qml` with the rest of `Apps.powerActions` (log off, lock,
   restart, sleep, hibernate). Lock is `loginctl lock-session`, which does
   nothing unless a locker listens for it.
3. The left pane: pinned programs in bold, then the programs started most
   (`Apps.frequent`, counted in `Quickshell.statePath("launch-counts.json")`),
   then **All Programs**, which swaps in the programs grouped into folders by
   their first freedesktop main category.
4. Search: arrow keys move through the results, Return starts the one picked,
   Escape clears the box before it closes the menu, and with no result Return
   runs the text through `sh -c`.
5. The places come from `user-dirs.dirs`, with Downloads and Videos added.

Captured in the headless session (`start-menu*.png`); none of the power
actions or a typed command was actually run.

## Fourth pass: launcher and polish

1. **Vicinae on `Super`+`D`.** `src/index.tsx` starts `vicinae server` once
   per session with `~/.config/shojiwm/vicinae.json` (written if missing; it
   imports `assets/vicinae/aero.json`, so Vicinae's own settings window writes
   there and the theme stays underneath). The Aero theme is
   `assets/vicinae/vicinae/themes/aero.toml`, found through `XDG_DATA_DIRS`.
   Vicinae's welcome window is marked as seen the first time: it would set a
   theme, a hotkey and a start at login, which this configuration does.
2. **Vicinae is a dependency.** `flake.nix` has a `nixpkgs-unstable` input
   (FlakeHub's rolling unstable), and the package writes that Vicinae's path
   into `src/index.tsx`. `packages.vicinae` exposes it; the capture script
   uses it too. Showing the desktop moved to `Super`+`M`.
3. **Colour icons** from Icons8's Flat Color Icons (MIT) through Iconify,
   `tools/fetch-icons.sh`: the user's tile shows the place under the pointer,
   the All Programs folders, the Run line, and the recent files of a jump list
   by kind.
4. **A calendar** on a click of the clock. The clock and the calendar follow
   the locale.
5. Captured: `launcher*.png`, `calendar.png`, `start-menu-place.png`. The date
   under the clock reads "7 11 2026" there: ShojiWM starts processes with
   `sh -lc`, which takes the test machine's `/etc/default/locale`
   (`C.UTF-8`).

## What remains, in priority order

1. **Look at all of it on a real GPU.** Colours, blur, animation speed and
   input have never been seen outside llvmpipe.
2. **The inactive window.** `COLOR.glassInactive` in `src/theme.ts:74` is very
   pale. Windows keeps more colour in it. No 1:1 reference of an inactive
   window could be downloaded, so it is a guess.
3. **The hover and pressed caption buttons.** `tools/make-caption-svgs.py:52`
   and `:72` hold them. They are invented: no screenshot catches a caption
   button under the pointer. Check them against a running Windows 7 if you
   have one.
4. **The Start menu, Alt+Tab and the menus.** They are in the Aero spirit but
   were never measured: Wikimedia rate-limited the downloads before a 1:1
   Start menu or Alt+Tab could be fetched.
5. **The taskbar glass alpha.** Only one wallpaper was available behind a
   reference taskbar, so colour and alpha could not be separated. The
   gradient is right over a dark wallpaper and too dark over a light one.
6. **Live taskbar thumbnails.** `shell/theme.js` (`liveThumbnails`) turns
   them on. See the defect below.
7. **Peek at one window** from its thumbnail. The shell knows a
   `zwlr_foreign_toplevel` handle, the compositor a `WaylandWindow`; nothing
   maps one to the other over the socket yet (title and app id would be a
   start).
8. **Unpinning from the jump list.** Pins are configuration, not state; a
   "Pin / Unpin this program" line would need somewhere to write them.

## Known defects

| Defect                                                                                                                                 | Where                                                      | Why                                                                                                                                                                                                       |
| -------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| The glow behind the title is a soft box, not the shape of the letters. It lights the space between two words as much as the words.     | `src/decoration.tsx:165`, `assets/title-glow.svg`          | `SSDStyle` has no shadow and no blur for a `<Label/>`. There is no way to blur a text shape in the config API.                                                                                            |
| The Alt+Tab panel has no window title and square corners.                                                                              | `src/alt-tab.tsx:309`                                      | An output composition has no label node and `<Solid>` has no radius.                                                                                                                                      |
| Live window thumbnails end the shell's Wayland connection.                                                                             | `shell/theme.js:45`, `shell/components/ThumbnailPopup.qml` | With Quickshell 0.3.0 and 0.3.2 against ShojiWM `9e778dc`, asking for a capture kills the client: "fatal error: Invalid argument". Try again on the real session; it may be a software-rendering problem. |
| The caption button group is 103 px wide. The reference also has a 1 px light bevel outside its left and right dark borders, so 105 px. | `tools/make-caption-svgs.py`                               | Measured late, after the owner validated the corner. Left alone on purpose.                                                                                                                               |
| A menu closes 900 ms after the pointer leaves it.                                                                                      | `shell/components/Taskbar.qml`                             | A layer-shell popup is told nothing about a press outside itself.                                                                                                                                         |
| `tools/details.py` needs the frame sizes.                                                                                              | `tools/details.py:119`                                     | It finds a window by its close button, then uses the size the capture asked for. If `tools/screenshots.sh` changes the `foot` window size, change it there too.                                           |

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
nix run github:sachahjkl/shojiwm-win7/wip-aero-polish#install

# the compositor itself, if it is not installed yet
nix run github:bea4dev/ShojiWM#init-config     # links node_modules/shoji_wm
```

Then log out and pick **ShojiWM** in the login manager, or run `shoji_wm
--tty` from a console. `Super`+`Shift`+`R` reloads the configuration without
restarting the session; `Super`+`Shift`+`Q` quits it.

Logs are in `~/.local/state/shoji_wm/logs/latest.log`. A shader that fails to
compile is reported there and the component falls back to its built-in look,
so grep for `shader` after a change to `assets/*.frag`.

The shell needs `quickshell` in `PATH`, a font (Selawik, else Open Sans) and
an icon theme. `src/index.tsx` starts it.

## Taking captures there, and comparing

On a real session, `grim` or the portal screenshot is enough. To keep using
the scripts:

```sh
# the whole headless session again, which also works on a GPU machine
tools/screenshots.sh

# the zoomed crops of one capture
python3 -I tools/details.py assets/site/desktop.png assets/site/details \
  front=770x569 back=616x398

# a reference next to one of our captures, with the error per region
python3 -I tools/compare.py REFERENCE.png 12,12 CAPTURE.png 355,215
```

`tools/compare.py` writes its side-by-side to `/tmp`: it contains reference
pixels and must never be committed. Open a window of exactly 770 x 569 to
compare with the Notepad++ reference; `tools/screenshots.sh` already does.

## Not tested on a real GPU

- Every colour and the blur: llvmpipe only.
- Fractional scale (125%) and a second monitor.
- The animations, and anything that depends on frame timing.
- kitty, Thunar and Chromium: the captures use `foot`.
- Real input devices; everything was driven by xdotool.
- The caption-button tooltips, the double click on the caption bar, and the
  minimise animation.

## Reference images used

Downloaded to a temporary directory for measuring, never committed.

- <https://commons.wikimedia.org/wiki/File:Clementine_0.6_in_Windows.png>
- <https://commons.wikimedia.org/wiki/File:Notepad%2B%2B_screenshot2.png>
- <https://commons.wikimedia.org/wiki/File:Aoo_calc_3.4.png>
- <https://commons.wikimedia.org/wiki/File:PaintDotNET_Screenshot_zh-hans.png>
- <https://commons.wikimedia.org/wiki/File:Qt_Creator_3.1.1_editing_a_sample_UI_file_from_Qt_5.3_using_Designer.png>
- <https://commons.wikimedia.org/wiki/File:Clase_de_computaci%C3%B3n,_Carrera_Fisioterapia.png>
- <https://commons.wikimedia.org/wiki/File:7-Zip_9.20_DE.png>

## Checks

`nix flake check` passes on this branch: the TypeScript against the pinned
ShojiWM package, the QML against the Quickshell types, and the formatters.
