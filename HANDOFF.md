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

## Fifth pass: lock screen, Control Panel, light and dark

1. **Lock screen.** `WlSessionLock` in `shell/shell.qml`, one
   `components/LockScreen.qml` per screen, the PAM check once in
   `LockState.qml` (service `login`). Locks on: the Start menu's Lock,
   `Super`+`L` (IPC `lock lock`), logind's `Lock` signal for this session
   (`gdbus monitor` on the session's object path, so `loginctl lock-session`
   works), `PrepareForSleep` when the setting says so, and `IdleMonitor`
   after the chosen delay. Tested: a wrong password is refused by PAM and
   the field empties. Not tested: a right one (the test session has no
   password), a real idle timeout, a real suspend.
2. **Control Panel.** `ControlPanel.qml`, a `FloatingWindow`, so it is a
   real window with the Aero frame and its own taskbar button. Opened from
   the Start menu's Control Panel place, or IPC `controlpanel open` /
   `openPage <page>`.
3. **Settings.** `Settings.qml`, a `JsonAdapter` on
   `~/.config/shojiwm/settings.json` (outside what `install` replaces, and
   watched). `combineWindows` left `theme.js` for it.
4. **Light and dark.** Not in that file: it is
   `org.gnome.desktop.interface color-scheme` in dconf, read at start and
   watched, written with `gtk-theme` (Adwaita / Adwaita-dark for GTK 3).
   xdg-desktop-portal-gtk passes it to programs. The shell's colours that
   change are in `Scheme.qml` (not `Palette`: QtQuick has a `Palette` type,
   which hides a singleton of that name). Vicinae has `aero-dark.toml`.
   The built-in wallpaper has a night version, `wallpaper-dark.svg`, used
   for the desktop and the lock screen while the system prefers dark.
5. **`shell/tools.js`** holds `dconf` and `gdbus`; the Nix package writes
   their store paths there.

## Sixth pass: the lock screen from references

The owner found the first lock screen too far from Windows 7 and the wrong
password hard to see. Redrawn from two Windows 7 Ultimate log-on
screenshots (Seven Forums tutorials 5382 and 182279, read in a temporary
directory, not committed): the account frame (`UserTile.qml`, 116 px round
98 at 800 x 600, drawn 136 round 120), the 175 x 20 field, the pale round
arrow with a blue arrow, the red power button in one piece with its arrow
(`PowerSplitButton.qml`, 38 + 14 x 18) and a menu of restart, sleep,
hibernate and shut down above it. A wrong password now swaps the field for
"The user name or password is incorrect." with a red mark and an OK button
(`LogonButton.qml`, the dark glass of the log-on buttons), as Windows 7
does; Return, Escape or Space is OK.

A second look, zoomed against the reference: the frame is one stack of
rounded rectangles (outer line, highlight, glass band, light line, dark
line) with a single corner, and the picture fills its square. The double
white corners came from `assets/user.svg`, which drew its own rounded
frame; it is now full bleed. The arrow is `assets/arrow-bead.svg`: a dark
ring, pale glass above and blue below, and a broad solid white arrow. The
Start menu frames the account picture with the same `UserTile`, small.

The owner then supplied two better references (a 1192 x 670 log-on
screen and a 1280 x 720 capture of three of them). Measured on those, the
arrow is a bead of glass: translucent, a white veil over the top half with
a bright crescent, a band darker than the background under the middle, a
saturated cyan glow from the bottom, a dark ring, and a white arrow about
65 % of its width. `arrow-bead.svg` and `arrow-bead-hot.svg` (under the
pointer) cut every layer to the disc with its own path: Qt's SVG renderer
ignores clip paths, which had left square corners. The frame's band is
nearly white at the top.

The lock screen follows light and dark like the rest: smoked glass for
the frame (`UserTile.dark`), a dark field, `arrow-bead-dark*.svg`, a
stronger shade. The references are inspiration, not a spec (the owner's
words). IPC `appearance setDark <bool>` / `toggle` sets the preference.

Light and dark no longer forces Adwaita on GTK 3: `shell/gtk-switch.sh`
moves the theme in use to its installed light or dark variant, or leaves
it.

## Seventh pass: the modern style

A second look beside Aero, chosen in the Control Panel (`Settings.style`,
"aero" or "modern"; IPC `appearance setStyle`).

A first attempt flattened everything (flat fills, round corners, flat
caption buttons). The owner rejected it outright: modern means Aero
**squared**, keeping its identity, not a flat redesign. That attempt was
removed. What is left:

- **Frames.** The Aero frame itself, with `square: true` (`FrameOptions`):
  the shader's corner radii are 0, every line and the title glow stay.
- **How the compositor knows.** It cannot read files. The shell sends
  `appearance {style, dark}` over the `peek.tsx` socket when it connects
  and on every change (`CompositorIpc.qml`); `src/style.ts` keeps them.
- **Shell.** `Scheme.qml` keeps every Aero colour and gradient; only the
  radii (`popupRadius`, `menuRadius`, `itemRadius`, `fieldRadius`,
  `buttonRadius`) drop to 0. `UserTile.square`, `LogonButton.square`,
  `PowerSplitButton.square`, and `arrow-square*.svg`, the bead's glass in a
  square. The orb stays round: it is the identity.

## Eighth pass: softer glass, the title, the notification area

- **Glass light.** `frame.frag` lifts the glass towards white along the top
  of the caption and in broad slanted bands (`glass_light`, `sheen`
  uniform: 1 active, 0.55 inactive), the streaks that make Aero read as
  glass. The four lines are softer than measured (`src/theme.ts`), after
  the owner compared with official screenshots and found ours harsh.
- **Title.** A ShojiWM label ignores its colour's alpha and its opacity
  (both tried: every copy came out opaque white), so a glow cannot be made
  of text copies. The white cloud is three images (`title-glow-left`,
  `-middle`, `-right`), denser than before, with ends of fixed size.
- **Taskbar.** Translucent blue-grey instead of 80 % near-black
  (`shell/theme.js`), so the blur shows.
- **Notification area.** As on Windows 7: icons chosen to show, the
  others behind the arrow in `TrayOverflow.qml`, with "Customize...",
  which opens the Control Panel's "Notification Area Icons" page
  (`Settings.trayShown`, `trayShowAll`). New icons are hidden. The menu of
  a hidden icon opens on the icon, over the popover.
- **Popups kept closing.** Their "pointer inside" was a MouseArea under
  their rows, which lost the hover to the rows; it is now a HoverHandler in
  every popup.

## Ninth pass: clearer glass

The owner compared with the ShojiWM author's own Windows 7 setup (not
published): there the frames and the taskbar are clear glass, the
wallpaper plainly visible through them. The window tint went from about
48 % to 24 % (`glassActive`), the blur from 3 passes to 2, and the
taskbar's alphas down by about a third.

Tried and dropped: a `<ShaderEffect/>` blurring a white copy of the title
for a glow that follows the letters. `backdropSource()` in a decoration
does not see the decoration's own nodes, so the copy came out sharp.

## Tenth pass: the author's look, scroll bars, compact taskbar

From the ShojiWM author's own Windows 7 desktop (a screenshot the owner
sent): the title is plain dark text with no glow at all, readable because
the glass is light (a pale tint, `glassActive` about 26 %), not a dark
blue one. The glow images are gone.

- **Scroll bars.** `AeroScrollBar.qml`: the Windows 7 trough, arrow
  buttons (hold to repeat) and grip thumb, blue under the pointer, light
  and dark, square in the modern style. On the Start menu's lists and the
  Control Panel's page. Programs draw their own, from their GTK or Qt
  theme.
- **Compact taskbar.** `Settings.taskbarCompact` ("Compact: show the icons
  alone", Control Panel → Taskbar and Start Menu): every button is the
  icon alone, `Theme.compactButtonWidth` wide.
- `tools/screenshots.sh` puts two dozen generated pictures in ~/Pictures,
  so the background chooser has something to show and to scroll.

## Eleventh pass: the author's Start menu and frame lines; no modern style

- **Modern style removed.** The owner did not want it in any form. Gone:
  `Settings.style`, `Scheme.modern`, the square variants, `src/style.ts`,
  the `appearance` socket method, `arrow-square*.svg`, the Control Panel's
  style chooser and the modern captures. (The sixth and seventh passes
  above describe what it was.)
- **Start menu, after the ShojiWM author's.** The whole menu is a pane of
  steel-blue glass with the frame's lines; the programs sit in a white pane
  set into it; the account picture stands above the top edge
  (`Theme.menuTileOverhang`, the menu window is that much taller); the
  places are white text on the glass, no icons, with etched separators
  (`PlaceRow.qml`); the Shut down button is glass (`PowerPart.glass`).
  "Run..." focuses the search box, which runs what it does not find.
- **Frame lines.** From his screenshot: a near-opaque dark navy outline,
  a cyan-white line inside it, the same pair reversed round the client, and
  a centred soft shadow. Each caption button now has its light line on all
  four sides (`tools/make-caption-svgs.py`, `EDGE`). Popups got the same
  double line (`Scheme.popupInnerLine`).

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
| The glow behind the title is a soft box, not the shape of the letters. It lights the space between two words as much as the words.     | `src/decoration.tsx`, `assets/title-glow-*.svg`            | `SSDStyle` has no shadow and no blur for a `<Label/>`. There is no way to blur a text shape in the config API.                                                                                            |
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

### Twelfth pass: neutral lines, round corners, flush buttons

- Every frame and popup line is translucent black or white (`src/theme.ts`,
  `Scheme.qml`, `theme.js`, `PowerPart.qml`, `LockScreen.qml`). The navy and
  cyan in the author's screenshot are those alphas seen over a blue
  wallpaper; never copy them as opaque colours again.
- Both corner radii are 8 px: at 5 px the bottom corners read as square.
- The caption buttons start one pixel down, right under the dark outline, and
  have no light line along their top (`tools/make-caption-svgs.py`).

### Thirteenth pass: taskbar, desktop icons, the desktop's menu

- Caption buttons: their bottom is a single dark line, rounded 4 px at the
  two outer corners (a clip in each SVG, `tools/make-caption-svgs.py`).
- Taskbar: buttons are glass plates 36 px tall (dark outline, light line
  inside, lighter upper half), icons alone by default (`taskbarCompact`
  defaults to true, as Windows 7's "always combine, hide labels"); labels
  are white with a shadow. Combined windows stack at the right. The clock
  writes the date in full; the show-desktop button is a visible glass strip.
- `shell/components/Desktop.qml`: desktop icons over the wallpaper (Recycle
  Bin + XDG_DESKTOP_DIR), picking, a box, renaming (the wallpaper surface
  takes the keyboard only then), opening through `gio` (path substituted by
  the flake into `shell/tools.js`).
- `shell/components/DesktopMenu.qml`: the right-click menus. They are items,
  not popups, drawn in a full-screen Overlay surface while open
  (`shell.qml`): ShojiWM hit-tests windows before the popups of a
  background surface (`surface_under` in `state.rs`), so a popup of the
  wallpaper would be drawn over a window and give it its clicks. The
  overlay also closes the menu on a click anywhere else, and on Escape.
- Settings: `desktopIconSize`, `showDesktopIcons`, `desktopSort`.
