# shojiwm-win7

A Windows 7 "Aero" desktop for [ShojiWM](https://github.com/bea4dev/ShojiWM),
the Wayland compositor you configure in TypeScript.

The compositor draws the window frames: a blurred glass caption bar, the
caption buttons at the top right, and a thin glossy border. A
[Quickshell](https://quickshell.org) shell draws the rest of the desktop: the
taskbar, the Start menu and the wallpaper.

All artwork is original. No file from Windows is copied, embedded or
required.

## What you get

|                         |                                                                                                                                                                                                                                                                             |
| ----------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Glass window frames** | The compositor blurs what is behind the caption bar and tints it blue-grey. A drawn sheen lies over the blur. The focused window has a stronger tint, a lighter outline and a deeper shadow. The frame is rounded at the top and square at the bottom, as Windows draws it. |
| **Caption buttons**     | Minimise, maximise or restore, and a wider close button. Each one lights up under the pointer; the close button glows red. Each shows a label after half a second.                                                                                                          |
| **Floating windows**    | Every window floats. New windows open in a cascade in the middle of the screen.                                                                                                                                                                                             |
| **Fast animations**     | Windows open in 140 ms and close in 110 ms. Minimise and restore take 130 ms.                                                                                                                                                                                               |
| **Alt+Tab**             | A panel of live thumbnails over the desktop. Tab, the arrow keys and the pointer pick a window.                                                                                                                                                                             |
| **Aero Snap**           | Drag a window against the top, left or right edge, or hold `Super` and press an arrow key. The rectangle the window would take is drawn while you drag.                                                                                                                     |
| **Double click**        | On the caption bar: maximise, and again to restore.                                                                                                                                                                                                                         |
| **Taskbar**             | Glass bar at the bottom: the orb, one button per program, the notification area, the clock, and the show-desktop strip at the far right.                                                                                                                                    |
| **Combined buttons**    | Every window of one program sits behind one button, drawn as a stack. One setting in `shell/theme.js` gives one button per window instead.                                                                                                                                  |
| **Window thumbnails**   | Rest on a taskbar button and its windows are listed with their titles. A click raises one, a middle click closes it. The live picture of each window is written but off; see "Real limits".                                                                                 |
| **Jump lists**          | Right click a taskbar button for the tasks of the program's `.desktop` file, a second instance, and close.                                                                                                                                                                  |
| **Notification area**   | A tray item's own menu, drawn in the same style.                                                                                                                                                                                                                            |
| **Start menu**          | Programs on the left with a search box, places on the right, and the shut-down button in the corner.                                                                                                                                                                        |

## Screenshots

Every picture here is a capture of this configuration running. None is drawn
or mocked up. `tools/screenshots.sh` makes them on a machine with no display:
ShojiWM's nested backend inside an X server with no screen, Mesa rendering in
software, and the keys and the pointer driven by xdotool.

![The desktop](assets/site/desktop.png)

The wallpaper, two windows, and the taskbar with one combined button, the
notification area and the clock.

![Two window frames](assets/site/frames.png)

The frames, cut out of the picture above: the focused window on top with a
lighter outline, the other behind it.

![The Start menu](assets/site/start-menu.png)

The Start menu: programs on the left, places on the right, the search box at
the bottom and the shut-down button in the corner.

![The thumbnails of a taskbar button](assets/site/taskbar-thumbnail.png)

Resting on a taskbar button lists the windows behind it, with their titles.
Each card carries the program's icon; see "Real limits" for why it is not a
live picture of the window.

![A jump list](assets/site/jump-list.png)

A right click on the same button. `foot` declares no tasks in its `.desktop`
file, so the list holds the program itself and "Close all windows".

![A notification-area menu](assets/site/tray-menu.png)

The menu of a tray item, drawn in the same style.

![Aero Snap](assets/site/snap.png)

Dragging a window against the left edge draws the rectangle it would take.

![Alt+Tab](assets/site/alt-tab.png)

Alt+Tab: a panel of live thumbnails, the selected window lit.

## Requirements

- **ShojiWM**, installed and registered as a session.
- **Quickshell 0.3.0 or later** — it draws the taskbar, the Start menu and the
  wallpaper.
- **A font.** The text is set in **Inter**, and falls back to Noto Sans and
  then DejaVu Sans.
- **An icon theme** with an `application-x-executable` icon, such as
  **Adwaita** or **Papirus**. Window and taskbar icons come from it.
- The programs the keys start: **kitty**, **Thunar** and **Chromium**. Change
  them in `src/index.tsx` and `shell/Apps.qml` if you use others.

Optional: `systemd` for the shut-down button, and any status-notifier
application for the notification area.

## Install

### With Nix

Install the configuration into `~/.config/shojiwm`:

```sh
nix run github:sachahjkl/shojiwm-win7#install
```

The command copies `src`, `assets`, `shell`, `package.json` and
`tsconfig.json`. It does not touch `node_modules`, which ShojiWM manages.

To use the flake as an input of your own system flake:

```nix
{
  inputs.shojiwm-win7 = {
    url = "github:sachahjkl/shojiwm-win7";
    inputs.nixpkgs.follows = "nixpkgs";
  };
}
```

`packages.<system>.default` is the configuration tree. Link its directories
into `~/.config/shojiwm` with whatever manages your home files, or run
`apps.<system>.install`.

### Without Nix

```sh
git clone https://github.com/sachahjkl/shojiwm-win7.git
cd shojiwm-win7
mkdir -p ~/.config/shojiwm
rm -rf ~/.config/shojiwm/src ~/.config/shojiwm/assets ~/.config/shojiwm/shell
cp -R src assets shell package.json tsconfig.json ~/.config/shojiwm/
```

Then start the ShojiWM session, or press `Super`+`Shift`+`R` to reload a
running one.

## Keys

| Key                               | Action                                         |
| --------------------------------- | ---------------------------------------------- |
| `Super` (tap)                     | Show or hide the Start menu                    |
| `Super`+`Return`                  | Terminal                                       |
| `Super`+`E`                       | File manager                                   |
| `Super`+`B`                       | Browser                                        |
| `Alt`+`F4`                        | Close the focused window                       |
| `Alt`+`Tab` / `Alt`+`Shift`+`Tab` | Switch window                                  |
| `Super`+`Up`                      | Maximise                                       |
| `Super`+`Down`                    | Restore                                        |
| `Super`+`Left` / `Super`+`Right`  | Put the window against that side               |
| `Super`+`D`                       | Show the desktop, then put the windows back    |
| `Super`+drag                      | Move a window from anywhere on it              |
| `Super`+`Shift`+`R`               | Reload this configuration (built into ShojiWM) |
| `Super`+`Shift`+`Q`               | Quit the session (built into ShojiWM)          |

## The files

| File                                                                                         | What it holds                                                                                                                                  |
| -------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `src/index.tsx`                                                                              | The entry point: environment, keys, the programs that start with the session, the blur, the snap preview, and the composition of every window. |
| `src/theme.ts`                                                                               | Every size and colour of the window frame.                                                                                                     |
| `src/decoration.tsx`                                                                         | The frame itself: the caption bar, the sheen, the title with its glow, the caption buttons and their labels.                                   |
| `src/window-manager.ts`                                                                      | The floating window manager: placement, move, resize, maximise, minimise, fullscreen, double click, and Aero Snap.                             |
| `src/alt-tab.tsx`                                                                            | The Alt+Tab panel, built from render textures and an input grab.                                                                               |
| `assets/frame.frag`                                                                          | The paint shader that rounds the top corners of the frame and leaves the bottom two square.                                                    |
| `assets/layer-mask.frag`                                                                     | Clips the blur behind a panel to the shape the panel draws.                                                                                    |
| `assets/*.svg`                                                                               | The caption glyphs and the two sheens. Drawn by hand.                                                                                          |
| `shell/shell.qml`                                                                            | The Quickshell root: the wallpaper, the taskbar and the Start menu, one of each per screen.                                                    |
| `shell/theme.js`                                                                             | Every size and colour of the shell, and the combine setting.                                                                                   |
| `shell/Apps.qml`                                                                             | The window list, the grouping, the program list, and what a click does. The programs the places open are named here.                           |
| `shell/StartMenuState.qml`                                                                   | Which screen shows the Start menu.                                                                                                             |
| `shell/StartMenuIpc.qml`                                                                     | The IPC entry point the compositor calls to open the menu.                                                                                     |
| `shell/components/Taskbar.qml`                                                               | The bar, and the three popups its buttons ask for.                                                                                             |
| `shell/components/TaskButton.qml`                                                            | One program: its stack, its states, its clicks.                                                                                                |
| `shell/components/ThumbnailPopup.qml`                                                        | The live pictures over a button.                                                                                                               |
| `shell/components/JumpList.qml`                                                              | The tasks of a program, on a right click.                                                                                                      |
| `shell/components/TrayMenu.qml`                                                              | A notification-area item's own menu.                                                                                                           |
| `shell/components/MenuItemRow.qml`                                                           | One line of either menu.                                                                                                                       |
| `shell/components/StartMenu.qml`, `StartOrb.qml`, `Clock.qml`, `TrayArea.qml`, `MenuRow.qml` | The Start menu, the orb, the clock, the notification area, and one menu line.                                                                  |
| `shell/assets/*.svg`                                                                         | The orb, the place icons, the account tile and the wallpaper. Drawn by hand.                                                                   |
| `tools/screenshots.sh`                                                                       | Runs the whole desktop on a machine with no display and takes the pictures in this file.                                                       |
| `flake.nix`                                                                                  | The package, the install command, and the checks.                                                                                              |

## Changing it

- **Programs:** `TERMINAL`, `FILE_MANAGER` and `BROWSER` at the top of
  `src/index.tsx`, and `terminal`, `fileManager`, `browser`, `controlPanel`
  and `pinnedIds` at the top of `shell/Apps.qml`.
- **Frame colours and sizes:** `src/theme.ts`.
- **Shell colours and sizes:** `shell/theme.js`. Set `combineWindows` to
  `false` there for one taskbar button per window.
- **Wallpaper:** replace `shell/assets/wallpaper.svg`, or point the `Image` in
  `shell/shell.qml` at your own file.

Press `Super`+`Shift`+`R` after a change to `src`. The shell reloads itself
when a QML file changes.

## Chromium and Firefox

Chromium and Firefox draw their own title bar by default, so they keep it. To
give Chromium the Aero frame, right click its tab strip and turn off **Use
system title bar and borders**... in Chromium's wording, switch _on_ the
system title bar. Firefox has the same setting under **Customise toolbar →
Title bar**.

## Checks

```sh
nix flake check
```

runs three checks:

- `typecheck` — `tsc` over `src`, against the ShojiWM TypeScript package
  pinned in `flake.lock`.
- `qmllint` — `qmllint` over `shell`, against the Quickshell QML types.
- `pre-commit` — `alejandra`, `deadnix`, `statix` and `prettier`.

## Real limits

Four things that do not work, each with the file that shows why.

- **A label has no shadow and no blur.** `SSDStyle` carries `boxShadow`,
  which is the shape of a box, and nothing for text
  (`packages/shoji_wm/src/types.ts`). The glow under the caption text is
  therefore ten offset copies of the label at low opacity, not a blurred one.
  It reads as a glow at 12 px; it is not the real thing.
- **The Alt+Tab panel has no text.** An output composition is built from
  `<Layers>`, `<Windows>`, `<LayerPopups>`, `<TextureView>`, `<Solid>`,
  `<Scene3D>` and `<Plane>`; there is no label node
  (`docs/configuration/output-composition-reference.md`). The panel shows the
  thumbnails without the title of the selected window.
- **The Alt+Tab panel has square corners.** `<Solid>` takes a colour and a
  rectangle, with no radius (same page). Rounding it would mean drawing the
  card into a texture through a shader.
- **The taskbar thumbnails are not live pictures.** The code for them is in
  `shell/components/ThumbnailPopup.qml`: a `ScreencopyView` per window, which
  is how Quickshell captures one. ShojiWM implements
  `hyprland_toplevel_export_manager_v1` for exactly that client
  (`src/shojiwm_lib/src/protocols/hyprland_toplevel_export.rs` says so in its
  first line). It still does not work here: with Quickshell 0.3.0 and with
  0.3.2, against ShojiWM `9e778dc` in its nested backend with software
  rendering, the capture ends the shell's Wayland connection — the shell logs
  "The Wayland connection experienced a fatal error: Invalid argument", the
  compositor logs "error in client communication", and the whole bar
  disappears. The thumbnails therefore carry the program's icon, and
  `liveThumbnails` in `shell/theme.js` turns the capture on for anyone whose
  pair handles it.

## Not done yet

Possible with the APIs above, simply not written:

- **Aero Peek**, the desktop seen through glass outlines while you rest on the
  show-desktop strip. The shell would have to ask the compositor to hide the
  windows; `COMPOSITOR` can serve a socket (`shoji_wm/ipc`) and Quickshell can
  speak to one (`Quickshell.Io.Socket`), so the way is open.
- **Submenus of a notification-area item.** An entry with children is
  triggered instead of opened in place (`QsMenuEntry.hasChildren`).
- **Recent files in a jump list**, from `recently-used.xbel`.
- **Pinned programs in the taskbar.** The Start menu has a pinned list; the
  taskbar shows only what is open.
- **An "All Programs" tree** in the Start menu. The list is flat, and the
  search box filters it.
- **A menu that closes on a press outside it.** A layer-shell popup is told
  nothing about such a press, so the jump list and the tray menus close when
  the pointer has been away from them for 900 ms.

## Tested, and not tested

Run, on a server with no display and no GPU:

- `nix flake check` — the TypeScript against the ShojiWM package pinned in
  `flake.lock`, the QML against the Quickshell types, and the formatters.
- `tools/screenshots.sh` — the whole session: the compositor in its nested
  backend inside Xvfb with Mesa's software renderer, the shell, two `foot`
  windows and a tray item. Every picture above comes from that run.
- In that session, by hand through xdotool: opening and closing the Start
  menu, the thumbnails of a taskbar button, a jump list, a tray menu, Aero
  Snap by dragging a window to an edge, and Alt+Tab.

Not run:

- **A real GPU session.** Everything was drawn by llvmpipe at 1280x800 and
  scale 1. Fractional scale, a second monitor, and anything that depends on
  speed are untried.
- **The programs the keys start.** kitty, Thunar and Chromium are not
  installed on the test machine; the pictures use `foot`.
- **Real input devices.** The pointer and the keys came from xdotool through
  the X server.
- **The caption-button labels, the double click on the caption bar, and the
  minimise animation.** They are wired up, but no picture proves them.

## Licence

MIT. See [LICENSE](./LICENSE).

The artwork in `assets/` and `shell/assets/` is original SVG, written by hand
for this repository. It imitates a visual style; it copies no file, no icon
and no font from Microsoft.
