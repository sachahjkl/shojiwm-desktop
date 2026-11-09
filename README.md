# shojiwm-win7

A Windows 7 "Aero" desktop for [ShojiWM](https://github.com/bea4dev/ShojiWM),
the Wayland compositor you configure in TypeScript.

The compositor draws the window frames: a blurred glass caption bar, the
caption buttons at the top right, and a thin glossy border. A
[Quickshell](https://quickshell.org) shell draws the rest of the desktop: the
taskbar, the Start menu and the wallpaper.

No file from Windows is copied, embedded or required. The artwork is
original, except the colour icons of the desktop, the Start menu and the jump
lists, which are built from KDE's LGPL-licensed Oxygen icons (see "Licence").

## What you get

|                           |                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------- | ------------------------ |
| **Glass window frames**   | The compositor blurs what is behind the caption bar, the taskbar and the menus, at the strength the Control Panel sets (Clear to Frosted), and tints the caption bar blue-grey. A drawn sheen lies over the blur. The focused window has a stronger tint, a lighter outline and a deeper shadow. All four corners are rounded, 8 px, and the lines round it are translucent black and white, so they take the colour of what is behind. The caption bars', the taskbar's and the Start menu's glass take a colour from a palette of fifteen, at an intensity you set, as Windows 7's Window Color does.                  |
| **Caption buttons**       | Minimise, maximise or restore, and a wider close button. Each one lights up under the pointer; the close button glows red. A label appears when the pointer rests on one.                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| **Floating windows**      | Every window floats. New windows open in a cascade in the middle of the screen.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| **Fast animations**       | Windows open in 140 ms and close in 110 ms. Minimise and restore take 130 ms.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| **Alt+Tab**               | A panel of live thumbnails over the desktop. Tab, the arrow keys and the pointer pick a window.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| **Aero Snap**             | Drag a window against the top, left or right edge, or hold `Super` and press an arrow key. The rectangle the window would take is drawn while you drag.                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| **Double click**          | On the caption bar: maximise, and again to restore.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| **Taskbar**               | Glass bar at the bottom: the orb, glass buttons that fill its height (icons alone by default, as Windows 7 does; labels in the Control Panel), the notification area, the clock over the full date, and the show-desktop button at the far right.                                                                                                                                                                                                                                                                                                                                                                        |
| **Sticky notes**          | Desktop menu, New, Sticky Note: a yellow note on the desktop to type in, moved by its band, sized by its corner, with + for another and × to remove it. The notes are kept in `~/.local/state/shojiwm-win7/notes.json`. The desktop holds the keyboard while you type in one and gives it back when the pointer goes to a window.                                                                                                                                                                                                                                                                                        |
| **Gadgets**               | On the first screen's desktop: the CPU Meter, two dials for the processor and the memory in use read from `/proc` every 1.5 s, and the Clock, an analog face with a seconds hand and the day. Drag them anywhere; the desktop menu's Gadgets submenu shows or hides each.                                                                                                                                                                                                                                                                                                                                                |
| **Desktop icons**         | The Recycle Bin and everything in the desktop folder, in columns from the top left. Click to pick, Ctrl+click or drag a box for several, double click to open; a `.desktop` file is a shortcut with the arrow. Right click an icon to open, delete or rename it; right click the desktop for View, Sort by, Refresh, New folder or text document, a terminal there, and Personalize. Computer, your files and the Control Panel can stand beside the Recycle Bin (Taskbar and Start menu, Desktop icons); Computer does by default.                                                                                      |
| **Pinned programs**       | A pinned program keeps its icon on the taskbar while it is closed; a click starts it. Once it runs, its windows gather behind that same button. A middle click, Ctrl+click or Shift+click on any button starts another instance; clicks while a program starts do not start it again.                                                                                                                                                                                                                                                                                                                                    |
| **Button progress**       | A program's progress fills its taskbar button green, as Windows 7 shows a download; a count shows in a red badge, and an urgent program turns its button orange. Programs send these with the `com.canonical.Unity.LauncherEntry` signal (Chromium, Firefox, KDE and GTK programs).                                                                                                                                                                                                                                                                                                                                      |
| **Combined buttons**      | Every window of one program sits behind one button, drawn as a stack. One setting in `shell/theme.js` gives one button per window instead.                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| **Window thumbnails**     | Rest on a taskbar button and its windows are listed with their titles. A click raises one, a middle click closes it. The live picture of each window is written but off; see "Real limits". A media player's thumbnails carry its toolbar, as on Windows 7: previous, play or pause, and next, through MPRIS. A thumbnail under the pointer shows a red close button at its top right.                                                                                                                                                                                                                                   |
| **Jump lists**            | Right click a taskbar button for the files the program opened last (from `recently-used.xbel`), the tasks of its `.desktop` file, a second instance, and close. A file's pin, at the right of its line under the pointer, keeps it in a Pinned section over the recent ones, in the taskbar's and the Start menu's jump lists.                                                                                                                                                                                                                                                                                           |
| **Notification area**     | A tray item's own menu, drawn in the same style, with its submenus, check boxes and radio buttons.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| **Launcher**              | `Super`+`D` opens [Vicinae](https://github.com/vicinaehq/vicinae), a fast search for programs, files and commands, in an Aero theme that matches the Start menu. The Nix package brings Vicinae with it, from nixpkgs unstable.                                                                                                                                                                                                                                                                                                                                                                                          |
| **Action Center**         | The flag of the notification area reports what Windows 7's Action Center would: system and user services that failed (systemd) and disks 90 % full, with a red or yellow mark. Its panel lists them; a failed service of yours can be cleared, a full disk opened. `quickshell ipc call actioncenter open                                                                                                                                                                                                                                                                                                                | close`.     |
| **Notifications**         | The bell of the notification area keeps the notifications, across sessions, in `~/.local/state/shojiwm-win7/notifications.json` (Hokum draws them and keeps none): a red count for the unseen ones, a click for the list with each program's icon, title, text and time, a cross per card and Clear all. The panel's switch, or a middle click on the bell, is Hokum's Do not disturb, which slashes the bell.                                                                                                                                                                                                           |
| **Network**               | The network icon beside the speaker shows a wired screen or wireless bars, with a yellow mark without Internet and a red one without a connection. A click opens the Windows 7 network panel: the connection in use, the wireless networks with their bars, Connect and Disconnect, the wireless switch and the connection editor. NetworkManager asks for a new network's key through its agent (nm-applet).                                                                                                                                                                                                            |
| **Volume**                | The speaker beside the clock shows the level of the default output. A click opens the volume panel: a slider, the mute button and the link to the mixer (`pwvucontrol`). The wheel steps it by 2 %, a middle click mutes, and the media keys work. With more than one output, a click on the device's name lists them to switch. `quickshell ipc call volume open                                                                                                                                                                                                                                                        | openDevices | close` drives the panel. |
| **Screen zoom**           | `Super`+`Shift` and the wheel magnify the screen around the pointer; `Super`+`Shift`+`0` resets it.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| **Additional clocks**     | Up to two more time zones, set in Date and Time with a field that offers the zones as you type, shown under the calendar and in the clock's tooltip with the day they are in.                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| **Calendar**              | Click the clock for the long date and a month, today framed, with arrows to the months around it.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| **Lock screen**           | The Windows 7 log-on screen: your picture behind (or the desktop's), the time, your account picture in its glass frame, your name, and the password field with the round blue arrow. `Super`+`L`, the Start menu, `loginctl lock-session`, going to sleep and a delay of inactivity lock it; PAM checks the password.                                                                                                                                                                                                                                                                                                    |
| **Control Panel**         | A window laid out as the Windows 7 one: light or dark, desktop background, lock screen picture and delay, account picture, taskbar and Start menu, clock format. Every change applies at once.                                                                                                                                                                                                                                                                                                                                                                                                                           |
| **Screen resolution**     | The Control Panel shows the screens as they are placed and sets the resolution, refresh rate and scale of each one, live. A change goes back on its own after 15 seconds unless you keep it. The choices replace `SHOJIWM_WIN7_OUTPUTS` for the screens you change.                                                                                                                                                                                                                                                                                                                                                      |
| **Devices and Printers**  | The Control Panel shows the computer, its screens, the USB devices (lsusb, hubs left out) with an icon that fits each, and the printers (CUPS) with a tick on the default one.                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| **Sound**                 | The Control Panel lists the playback and recording devices PipeWire knows, marks the default ones, and makes another the default; a link opens the volume mixer.                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| **Default programs**      | The Control Panel sets the program that opens web links, mail, folders, text, pictures, music, videos and PDF documents, from the programs whose .desktop file says they can, through xdg-mime (~/.config/mimeapps.list).                                                                                                                                                                                                                                                                                                                                                                                                |
| **Startup programs**      | The Control Panel lists the XDG autostart entries the session starts: turn one off, restore a system entry, or add any installed program. Changes apply at the next log-on.                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| **Background slide show** | Desktop Background can change the picture on its own from a folder (your Pictures by default), every 1 minute to 1 day, in name order or shuffled, with a Next picture button.                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| **Light and dark**        | The system's own preference (`color-scheme`, which the settings portal passes to GTK, libadwaita, Qt, Firefox, Chromium and Vicinae). The menus, the Start menu and the Control Panel follow it too.                                                                                                                                                                                                                                                                                                                                                                                                                     |
| **Aero Peek**             | Rest on the show-desktop strip at the far right and every window turns into a glass outline over the desktop. Click it to put the windows away; click again to bring them back.                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| **Start menu**            | Pinned programs, then the ones you start most, and **All Programs** with every program in folders. Type to search; the arrow keys and Return pick a result, and Return with nothing found runs what you typed. On the right: your full name, your folders (as `xdg-user-dirs` names them), and a power button whose arrow offers log off, lock, restart, sleep and hibernate. Its search lists, under headings as Windows 7 does, the programs, the Control Panel tasks and the files of your home folder (fd). A program with recent files has an arrow; resting on it shows them in the right column, to open with it. |

## Screenshots

Every picture here is a capture of this configuration running. None is drawn
or mocked up. `tools/screenshots.sh` makes them on a machine with no display:
ShojiWM's nested backend inside an X server with no screen, Mesa rendering in
software, and the keys and the pointer driven by xdotool.

![The desktop](assets/site/desktop.png)

The wallpaper with its icons, two windows, and the taskbar: one combined
button for the two `foot` windows, the pinned `footclient` as an icon alone
since none of its windows is open, the notification area and the clock.

![The desktop's menu](assets/site/desktop-menu.png)

A right click on the desktop: the menu, with View open. The desktop icons are
on the left.

![Two window frames](assets/site/frames.png)

The frames, cut out of the picture above: the focused window on top with a
lighter outline, the other behind it.

![The Start menu](assets/site/start-menu.png)

The Start menu: the pinned programs on the left (`foot` and `footclient` here,
pinned by the capture script), your folders on the right, the search box at
the bottom and the power button with its arrow in the corner. The name at the
top is cut short with an ellipsis when it does not fit.

![All Programs](assets/site/start-menu-all.png)

**All Programs**: every program in a folder named after its main category.
One folder opens at a time.

![The power menu](assets/site/start-menu-power.png)

The arrow beside the power button: log off, lock, restart, sleep, hibernate.

![Running a command](assets/site/start-menu-run.png)

A search that finds no program offers to run the text as a command.

![The thumbnails of a taskbar button](assets/site/taskbar-thumbnail.png)

Resting on a taskbar button lists the windows behind it, with their titles.
Each card carries the program's icon; see "Real limits" for why it is not a
live picture of the window.

![A jump list](assets/site/jump-list.png)

A right click on the same button: the files the program opened last, then
the program itself and "Close all windows". `foot` declares no tasks in its
`.desktop` file, so there is no "Tasks" section. Nothing on the test machine
opens files through GTK, so `tools/screenshots.sh` writes the
`recently-used.xbel` the list is read from, naming `foot`; and since Thunar,
Chromium and kitty are not installed there, it pins `foot` and `footclient`
instead.

![A notification-area menu](assets/site/tray-menu.png)

The menu of a tray item, drawn in the same style.

![Aero Snap](assets/site/snap.png)

Dragging a window against the left edge draws the rectangle it would take.

![Aero Peek](assets/site/peek.png)

Aero Peek: the pointer rests on the show-desktop strip, and the two windows
are left as glass outlines over the wallpaper.

![The launcher](assets/site/launcher-search.png)

`Super`+`D`: Vicinae in the Aero theme, searching as you type.

![The calendar](assets/site/calendar.png)

The calendar over the clock. The short date under the clock reads "7 11
2026" in these pictures because the test machine's system locale is
`C.UTF-8`; ShojiWM starts the shell through a login shell, which takes the
system's locale.

![A place in the user's tile](assets/site/start-menu-place.png)

Resting on a place shows its icon in the user's tile, as Windows 7 does.

![The lock screen](assets/site/lock-screen-wrong.png)

The lock screen, after a wrong password. The name is the test machine's;
yours comes from your account.

![The lock screen in dark](assets/site/lock-screen-dark.png)

In dark: the night wallpaper, the frame in smoked glass, a dark field and a
darker bead. `quickshell -p ~/.config/shojiwm/shell ipc call appearance
setDark true` (or `toggle`) switches it, as the Control Panel does.

![The Control Panel](assets/site/control-panel.png)

The Control Panel's home page.

![The Control Panel in dark](assets/site/control-panel-dark.png)

Light or dark: the system's preference. The Control Panel, the menus and the
Start menu follow it.

![The Start menu in dark](assets/site/start-menu-dark.png)

The Start menu in dark. With the built-in wallpaper, the desktop and the lock
screen take its night version, `shell/assets/wallpaper-dark.svg`; a picture
of your own stays as it is.

![The lock screen settings](assets/site/control-panel-lock.png)

The lock screen's page: its picture, the delay of inactivity, locking on
sleep.

![Alt+Tab](assets/site/alt-tab.png)

Alt+Tab: a panel of live thumbnails, the selected window lit.

### Close up

The same captures, cut up and blown up eight times with no smoothing, so
every pixel of the finish can be checked. `assets/site/details/` holds the
whole set; `tools/details.py` makes them.

|                                                               |                                                             |
| ------------------------------------------------------------- | ----------------------------------------------------------- |
| ![Top right corner](assets/site/details/corner-top-right.png) | ![Top left corner](assets/site/details/corner-top-left.png) |
| The caption buttons hanging from the frame's own outline.     | The rounded top corner: dark line, light line, glass.       |
| ![The title](assets/site/details/caption-title.png)           | ![The start orb](assets/site/details/taskbar-orb.png)       |
| Crisp dark text over a soft light shape, with no outline.     | The orb in its seat on the taskbar.                         |

## Where the numbers come from

The frame, the caption buttons and the title are not drawn by eye. Real
Windows 7 screenshots at 1:1 were read pixel by pixel with Pillow, and every
size, colour and alpha is written down in
[docs/aero-measurements.md](docs/aero-measurements.md) with the image it came
from. The references themselves are not in this repository; the document
links them.

| Tool                         | What it does                                                                                                                                    |
| ---------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| `tools/make-caption-svgs.py` | Draws the sixteen caption-button files from the measured gradients and glyph boxes.                                                             |
| `tools/screenshots.sh`       | Runs the whole desktop headless and captures it.                                                                                                |
| `tools/details.py`           | Finds each window in a capture and cuts the zoomed crops in `assets/site/details/`.                                                             |
| `tools/compare.py`           | Puts a reference next to one of our captures and prints the error per part of the frame. Its output stays in `/tmp`: it holds reference pixels. |

Two states are **not** measured, because no screenshot catches them: a
caption button under the pointer, and one being pressed. They are drawn from
the resting gradient.

## Requirements

- **ShojiWM**, installed and registered as a session.
- **Quickshell 0.3.0 or later** — it draws the taskbar, the Start menu and the
  wallpaper.
- **A font.** The text is set in **Inter**, and falls back to Noto Sans and
  then DejaVu Sans.
- **An icon theme** with an `application-x-executable` icon, such as
  **Adwaita** or **Papirus**. Window and taskbar icons come from it.
- **Vicinae**, the launcher `Super`+`D` opens. The Nix package of this
  configuration depends on it (the latest, from nixpkgs unstable) and writes
  its path into `src/index.tsx`; without Nix, install it and keep `vicinae`
  in `PATH`.
- The programs the keys start: **kitty**, **Thunar** and **Chromium**. Change
  them in `src/index.tsx` and `shell/Apps.qml` if you use others.

- **The lock screen** checks the password with the `login` PAM service
  (`/etc/pam.d/login`), and needs the compositor's `ext-session-lock`, which
  ShojiWM has.
- **Light and dark** for programs needs **dconf** (on NixOS
  `programs.dconf.enable = true`) and a settings portal that reads it,
  such as **xdg-desktop-portal-gtk**, which ShojiWM's NixOS module installs.
  The Nix package brings the `dconf` and `gdbus` commands itself.

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

| Key                               | Action                                                                                       |
| --------------------------------- | -------------------------------------------------------------------------------------------- |
| `Super` (tap)                     | Show or hide the Start menu                                                                  |
| `Super`+`Return`                  | Terminal                                                                                     |
| `Super`+`E`                       | File manager                                                                                 |
| `Super`+`B`                       | Browser                                                                                      |
| `Alt`+`F4`                        | Close the focused window; on the desktop, Shut Down Windows                                  |
| `Super`+`Shift`+`E`               | Shut Down Windows                                                                            |
| `Alt`+`Tab` / `Alt`+`Shift`+`Tab` | Switch window                                                                                |
| `Super`+`Up`                      | Maximise                                                                                     |
| `Super`+`Down`                    | Restore a maximised window, else minimise                                                    |
| `Super`+`Left` / `Super`+`Right`  | Snap to that side; again, to the next screen; from the other side, restore                   |
| `Super`+`Shift`+`Left` / `Right`  | Move the window to the next screen that way                                                  |
| `Super`+`Home`                    | Minimise the other windows, then bring them back                                             |
| `Print Screen`, `Ctrl`+`Print`    | Screenshot (`SHOJIWM_WIN7_SCREENSHOT`)                                                       |
| `Super`+`D`                       | The launcher (Vicinae)                                                                       |
| `Super`+`M`                       | Show the desktop, then put the windows back                                                  |
| `Super`+`L`                       | Lock                                                                                         |
| `Super`+drag                      | Move a window from anywhere on it                                                            |
| `Super`+double click              | Maximise, and again to restore                                                               |
| `Super`+`Shift`+`R`               | Reload this configuration (built into ShojiWM)                                               |
| `Super`+`Shift`+`Q`               | Close the focused window                                                                     |
| `Super`+`1` … `9`                 | The taskbar button at that place: start it, show it, put it away, or the next of its windows |
| `Super`+`Shift`+`1` … `9`         | Another instance of that button's program                                                    |
| `Super`+`V`                       | Clipboard history (cliphist, shown in Vicinae); the chosen entry is copied again             |
| `Super`+`R`                       | Run: the Start menu's search box                                                             |
| `Ctrl`+`Shift`+`Escape`           | Task manager (`SHOJIWM_WIN7_TASK_MANAGER`, else `kitty btop`)                                |
| `Super`+`Shift`+wheel             | Zoom the screen around the pointer; `Super`+`Shift`+`0` resets it                            |
| Volume, mute and player keys      | Volume of the default output (`wpctl`), microphone mute, `playerctl`                         |

## The files

| File                                                                                                | What it holds                                                                                                                                                             |
| --------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src/index.tsx`                                                                                     | The entry point: environment, keys, the programs that start with the session, the blur, the snap preview, and the composition of every window.                            |
| `src/theme.ts`                                                                                      | Every size and colour of the window frame.                                                                                                                                |
| `src/decoration.tsx`                                                                                | The frame itself: the painted glass, the caption, the title with its glow, the caption buttons and their labels.                                                          |
| `src/window-manager.ts`                                                                             | The floating window manager: placement, move, resize, maximise, minimise, fullscreen, double click, and Aero Snap.                                                        |
| `src/alt-tab.tsx`                                                                                   | The Alt+Tab panel, built from render textures and an input grab.                                                                                                          |
| `src/peek.tsx`                                                                                      | Aero Peek, and the socket the shell asks for it and for the show-desktop toggle through.                                                                                  |
| `assets/frame.frag`                                                                                 | The paint shader that draws the frame: the dark outer line, the light line, the glass, the two lines around the client, and the corner radii that a style cannot express. |
| `assets/caption/*.svg`                                                                              | The three caption buttons, one file per state, drawn at their exact pixel size.                                                                                           |
| `docs/aero-measurements.md`                                                                         | Every measured number, and the reference screenshot each came from.                                                                                                       |
| `tools/make-caption-svgs.py`                                                                        | Draws `assets/caption/*.svg` from those numbers.                                                                                                                          |
| `tools/details.py`                                                                                  | Finds the windows in a capture and cuts the zoomed details out of it.                                                                                                     |
| `tools/compare.py`                                                                                  | Puts a reference and a capture side by side and measures the difference.                                                                                                  |
| `assets/layer-mask.frag`                                                                            | Clips the blur behind a panel to the shape the panel draws.                                                                                                               |
| `assets/*.svg`                                                                                      | The caption glyphs and the two sheens. Drawn by hand.                                                                                                                     |
| `shell/shell.qml`                                                                                   | The Quickshell root: the wallpaper, the taskbar and the Start menu, one of each per screen.                                                                               |
| `shell/theme.js`                                                                                    | Every size and colour of the shell, and the combine setting.                                                                                                              |
| `shell/Apps.qml`                                                                                    | The window list, the grouping, the pinned programs, the recent files, the program list, and what a click does. The programs the places open are named here.               |
| `shell/StartMenuState.qml`                                                                          | Which screen shows the Start menu.                                                                                                                                        |
| `shell/StartMenuIpc.qml`                                                                            | The IPC entry point the compositor calls to open the menu.                                                                                                                |
| `shell/Settings.qml`                                                                                | What the Control Panel changes, kept in `~/.config/shojiwm/settings.json`, and the system's light or dark preference.                                                     |
| `shell/Scheme.qml`                                                                                  | The colours that change between light and dark.                                                                                                                           |
| `shell/LockState.qml`, `shell/components/LockScreen.qml`                                            | Everything that locks the session, the PAM check, and the lock screen itself.                                                                                             |
| `shell/ControlPanelState.qml`, `shell/components/ControlPanel.qml`, `Cp*.qml`, `PictureChooser.qml` | The Control Panel window, its pages and its controls.                                                                                                                     |
| `shell/tools.js`                                                                                    | The paths of `dconf` and `gdbus`, which the Nix package fills in.                                                                                                         |
| `shell/CompositorIpc.qml`                                                                           | The other way round: the socket the shell uses to ask the compositor for Aero Peek and the show-desktop toggle.                                                           |
| `shell/components/Taskbar.qml`                                                                      | The bar, and the three popups its buttons ask for.                                                                                                                        |
| `shell/components/TaskButton.qml`                                                                   | One program: its stack, its states, its clicks.                                                                                                                           |
| `shell/components/ThumbnailPopup.qml`                                                               | The live pictures over a button.                                                                                                                                          |
| `shell/components/JumpList.qml`                                                                     | The recent files and the tasks of a program, on a right click.                                                                                                            |
| `shell/components/TrayMenu.qml`                                                                     | A notification-area item's own menu, and each of its submenus.                                                                                                            |
| `shell/components/MenuItemRow.qml`                                                                  | One line of either menu.                                                                                                                                                  |
| `shell/components/StartMenu.qml`, `StartOrb.qml`, `Clock.qml`, `TrayArea.qml`, `MenuRow.qml`        | The Start menu, the orb, the clock, the notification area, and one menu line.                                                                                             |
| `shell/components/AeroScrollBar.qml`                                                                | The Windows 7 scroll bar of the Start menu and the Control Panel.                                                                                                         |
| `shell/components/PowerMenu.qml`, `PowerPart.qml`                                                   | The menu behind the power button's arrow, and the two halves of that button.                                                                                              |
| `shell/components/CalendarPopup.qml`, `CalendarArrow.qml`                                           | The calendar over the clock.                                                                                                                                              |
| `shell/assets/icons/`                                                                               | The colour icons (Oxygen, LGPL-3.0), built by `tools/fetch-icons.sh`.                                                                                                     |
| `assets/vicinae/`                                                                                   | Vicinae's settings (`aero.json`) and its Aero theme (`vicinae/themes/aero.toml`).                                                                                         |
| `shell/assets/*.svg`                                                                                | The orb, the place icons, the account tile and the wallpaper. Drawn by hand.                                                                                              |
| `tools/screenshots.sh`                                                                              | Runs the whole desktop on a machine with no display and takes the pictures in this file.                                                                                  |
| `flake.nix`                                                                                         | The package, the install command, and the checks.                                                                                                                         |

## Changing it

- **Programs:** `TERMINAL`, `FILE_MANAGER` and `BROWSER` at the top of
  `src/index.tsx`, and `terminal`, `fileManager` and `browser` at the top of
  `shell/Apps.qml`. `pinnedIds` there is the Start menu's
  pinned list and `taskbarPinnedIds` the taskbar's, both as `.desktop` file
  names.
- **Most things:** the **Control Panel** (Start menu → Control Panel). It
  writes `~/.config/shojiwm/settings.json`, which an update of the
  configuration leaves alone and which can be edited by hand.
- **Power:** `powerActions` in `shell/Apps.qml` holds each command (`loginctl`
  and `systemctl` by default); the Control Panel picks which one the button
  itself runs.
- **Screens:** set `SHOJIWM_WIN7_OUTPUTS` in the session environment to
  Hyprland monitor rules `name,mode,position,scale` separated by `;`, for
  example `DP-2,2560x1440@180,0x0,1.25;HDMI-A-1,preferred,auto,1.875`.
  Without it, ShojiWM uses the best mode of each screen at scale 1.
- **Terminal:** set `SHOJIWM_WIN7_TERMINAL` to the command that opens one;
  kitty without it.
- **More keys:** set `SHOJIWM_WIN7_KEYS` to a JSON object of shortcut to
  shell command, for example `{"Ctrl+Shift+Space": "hex dictation toggle"}`,
  for programs that would register a global shortcut through a portal.
- **Notifications:** `assets/hokum/aero.kdl` is a Windows 7 balloon theme
  for [Hokum](https://hokum.app). Point Hokum at it in
  `~/.config/hokum/config.kdl` with
  `theme "~/.config/shojiwm/assets/hokum/aero.kdl"`, and put the cards over
  the notification area with `behaviour { position "bottom-right" }`.
- **Frame colours and sizes:** `src/theme.ts`.
- **Shell colours and sizes:** `shell/theme.js`, and `shell/Scheme.qml` for
  the colours that change between light and dark.
- **Wallpaper, lock screen picture, account picture:** the Control Panel. With
  no account picture chosen there, the system's is used, from the first of
  `/var/lib/AccountsService/icons/$USER`, `~/.face` and `~/.face.icon`.

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

- **A label has no shadow and no blur, and ignores its own alpha.**
  Windows 7 lays a blurred white copy under the caption text. ShojiWM can
  neither blur a label nor draw one translucent, and an effect under it
  only sees what is behind the window. The title is plain dark text, as on
  the ShojiWM author's own Windows 7 setup, over a light enough glass.
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

- **Aero Peek on one window**, from its taskbar thumbnail. The shell knows a
  window as a `zwlr_foreign_toplevel` handle and the compositor as a
  `WaylandWindow`; nothing ties the two together over the socket yet.
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
- The capture pipeline itself was checked before any tuning: Xvfb reports
  96x96 DPI, the nested window is 1280x800 at 0,0, and the capture is
  1280x800, so nothing is scaled. `fc-match` in that session resolves the
  faces that are asked for and reports grey antialiasing. A plain Wayland
  client's text comes out crisp in the same pipeline, which is what ruled
  the pipeline out as the cause of the title text defect.
- In that session, through xdotool: moving a window by its caption bar,
  opening and closing the Start menu, the thumbnails of a taskbar button, a
  jump list with its recent files, the menu of a tray item, Aero Peek by
  resting on the show-desktop strip, Aero Snap by dragging a window to the
  left edge, and Alt+Tab.

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
- **Tray submenus.** The only tray item on the test machine, CopyQ, has none.
- **The power actions, running a typed command, and the programs used most.**
  The menus are captured; nothing was shut down or run from them.
- **Opening a recent file, starting a pinned program, the show-desktop
  click.** Wired up; the captures show the lists and the peek, not a click on
  them.

## Licence

MIT. See [LICENSE](./LICENSE).

The artwork in `assets/` and `shell/assets/` is original SVG, written by hand
for this repository, except `shell/assets/icons/`. It imitates a visual
style; it copies no file, no icon and no font from Microsoft.

`shell/assets/icons/` holds icons built by `tools/fetch-icons.sh` from
KDE's [Oxygen icons](https://invent.kde.org/frameworks/oxygen-icons),
LGPL-3.0-or-later: glossy icons of the Windows 7 era, with Oxygen's yellow
folder and emblems over it for the libraries. Their licence is next to them.
