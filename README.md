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

| | |
| --- | --- |
| **Glass window frames** | The compositor blurs what is behind the caption bar and tints it blue-grey. A drawn sheen lies over the blur. The focused window has a stronger tint, a lighter outline and a deeper shadow. |
| **Caption buttons** | Minimise, maximise or restore, and a wider close button. Each one lights up under the pointer; the close button glows red. Each shows a label after half a second. |
| **Floating windows** | Every window floats. New windows open in a cascade in the middle of the screen. |
| **Fast animations** | Windows open in 140 ms and close in 110 ms. Minimise and restore take 130 ms. |
| **Alt+Tab** | A panel of live thumbnails over the desktop. Tab, the arrow keys and the pointer pick a window. |
| **Aero Snap keys** | `Super`+`Left` and `Super`+`Right` put the window against one side. `Super`+`Up` maximises it. |
| **Taskbar** | Glass bar at the bottom: the orb, one button per window, the notification area, the clock, and the show-desktop strip at the far right. |
| **Start menu** | Programs on the left with a search box, places on the right, and the shut-down button in the corner. |

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

| Key | Action |
| --- | --- |
| `Super` (tap) | Show or hide the Start menu |
| `Super`+`Return` | Terminal |
| `Super`+`E` | File manager |
| `Super`+`B` | Browser |
| `Alt`+`F4` | Close the focused window |
| `Alt`+`Tab` / `Alt`+`Shift`+`Tab` | Switch window |
| `Super`+`Up` | Maximise |
| `Super`+`Down` | Restore |
| `Super`+`Left` / `Super`+`Right` | Put the window against that side |
| `Super`+`D` | Show the desktop, then put the windows back |
| `Super`+drag | Move a window from anywhere on it |
| `Super`+`Shift`+`R` | Reload this configuration (built into ShojiWM) |
| `Super`+`Shift`+`Q` | Quit the session (built into ShojiWM) |

## The files

| File | What it holds |
| --- | --- |
| `src/index.tsx` | The entry point: environment, keys, the programs that start with the session, the blur, and the composition of every window. |
| `src/theme.ts` | Every size and colour of the window frame. |
| `src/decoration.tsx` | The frame itself: the caption bar, the sheen, the caption buttons and their labels. |
| `src/window-manager.ts` | The floating window manager: placement, move, resize, maximise, minimise, fullscreen and Aero Snap. |
| `src/alt-tab.tsx` | The Alt+Tab panel, built from render textures and an input grab. |
| `assets/*.svg` | The caption glyphs and the two sheens. Drawn by hand. |
| `shell/shell.qml` | The Quickshell root: the wallpaper, the taskbar and the Start menu, one of each per screen. |
| `shell/theme.js` | Every size and colour of the shell. |
| `shell/Apps.qml` | The window list, the program list, and what a click does. The programs the places open are named here. |
| `shell/StartMenuState.qml` | Which screen shows the Start menu. |
| `shell/StartMenuIpc.qml` | The IPC entry point the compositor calls to open the menu. |
| `shell/components/*.qml` | The taskbar, the orb, a task button, the clock, the notification area, the Start menu and one menu row. |
| `shell/assets/*.svg` | The orb, the place icons, the account tile and the wallpaper. Drawn by hand. |
| `flake.nix` | The package, the install command, and the checks. |

## Changing it

- **Programs:** `TERMINAL`, `FILE_MANAGER` and `BROWSER` at the top of
  `src/index.tsx`, and `terminal`, `fileManager`, `browser`, `controlPanel`
  and `pinnedIds` at the top of `shell/Apps.qml`.
- **Frame colours and sizes:** `src/theme.ts`.
- **Shell colours and sizes:** `shell/theme.js`.
- **Wallpaper:** replace `shell/assets/wallpaper.svg`, or point the `Image` in
  `shell/shell.qml` at your own file.

Press `Super`+`Shift`+`R` after a change to `src`. The shell reloads itself
when a QML file changes.

## Chromium and Firefox

Chromium and Firefox draw their own title bar by default, so they keep it. To
give Chromium the Aero frame, right click its tab strip and turn off **Use
system title bar and borders**... in Chromium's wording, switch *on* the
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

## What is not here

- **No window grouping in the taskbar.** One button per window, as in the
  "never combine" setting of Windows.
- **No menus on the notification area.** A left click activates an item and a
  right click asks it for its secondary action. The DBus menu of an item is
  not shown.
- **No jump lists, no Aero Peek, no thumbnail preview over a taskbar button.**
- **No snap by dragging a window to an edge.** The keys above do snap.
- **No double click on the caption bar to maximise.** Use the button or
  `Super`+`Up`.
- **The frame corners are rounded on all four sides.** A ShojiWM style has one
  radius for the whole element, so the bottom corners are rounded like the
  top ones. Windows 7 squares off the bottom two.
- **The title has no glow behind it.** ShojiWM draws no text shadow.
- **The blur behind the Start menu is a rectangle.** The menu surface is
  exactly the size of the menu, so its rounded corners show a little blur
  outside them.

## Tested, and not tested

This configuration was written and checked on a machine with no display:

- The TypeScript is type checked against ShojiWM itself (`nix flake check`).
- The QML is checked against the Quickshell types (`nix flake check`).
- The flake builds.

**Nothing was run on screen.** A compositor needs a GPU session, so the
colours, the layout, the animations and the input handling have never been
seen. Expect to adjust numbers in `src/theme.ts` and `shell/theme.js`.

## Licence

MIT. See [LICENSE](./LICENSE).

The artwork in `assets/` and `shell/assets/` is original SVG, written by hand
for this repository. It imitates a visual style; it copies no file, no icon
and no font from Microsoft.
