<p align="center"><img src="assets/logo.svg" alt="The shojiwm-desktop logo: a glass window with two shoji panels, one slid open on a mountain" width="128" height="128"></p>

# shojiwm-desktop

Site and documentation: <https://desktop.froment.software/>

A complete desktop for [ShojiWM](https://github.com/bea4dev/ShojiWM), the
Wayland compositor you configure in TypeScript, in two themes: **Aero**
(Windows 7's glass) and **Kernel**.

It has three parts:

- **The compositor configuration** (`src/`, TypeScript). ShojiWM runs it. It
  draws the window frames and manages the windows: placement, Aero Snap,
  Alt+Tab, Aero Peek, screen zoom, window groups and the keys.
- **The shell** `shojiwm-shell` (`crates/shell/`, Rust). It draws the rest:
  the taskbar, the Start menu, the launcher, the notification area, the
  desktop, the dialogs, the Control Panel, the lock screen and the log-on
  screen. It is built on [AntiWebUI](https://github.com/bea4dev/AntiWebUI), a
  GPU-rendered native UI toolkit by bea4dev, the author of ShojiWM.
- **The portal** (`portal/`, Rust): an xdg-desktop-portal GlobalShortcuts
  backend, so that programs can bind global keys.

It is an independent, clean-room implementation: no code, image, icon, font,
sound or other file from Microsoft Windows is copied, embedded, decompiled or
required. The artwork is original, except the colour icons of the desktop, the
Start menu and the jump lists, built from KDE's LGPL-licensed Oxygen icons, and
the NixOS snowflake (see "Licence").

## What you get

|                           |                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| ------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Two themes**            | Control Panel, Appearance and Personalization: **Aero**, Windows 7's glass, or **Kernel**: a deep blue-black desk under a dotted grid, steel-blue caption plates with a line of light and darkened edges, a red close button and vector caption icons, titles in JetBrains Mono, a thin bar at the top of the screen with a launcher plate, flat buttons and a monospaced clock, section titles in monospaced spaced capitals after an accent bar, and the Control Panel's categories each in its own colour. The change applies at once, frames included.                                                                                                                                                                                                                                                        |
| **Glass window frames**   | In Aero, the compositor blurs what is behind the caption bar, the taskbar and the menus, at the strength the Control Panel sets (Clear to Frosted), and tints the caption bar blue-grey. A drawn sheen lies over the blur. The focused window has a stronger tint, a lighter outline and a deeper shadow. All four corners are rounded, 8 px (square in the square variant, which squares the menus, buttons and fields too). The lines round it are translucent black and white, so they take the colour of what is behind. The glass takes a colour from a palette of fifteen, at an intensity you set, as Windows 7's Window Color does.                                                                                                                                                                       |
| **Window groups**         | Drop a window's caption on another window's caption: it becomes a tab of that window's frame, as in a browser. While you drag, the window turns see-through and a ghost tab shows where it will go. Click a tab to show its window; drag it along the strip to move it, out of the strip to take the window out; its × closes it. `Super`+`Ctrl`+`Left`/`Right` changes tab, `Super`+`Ctrl`+`Down` takes the shown one out. The taskbar still lists every window, and choosing one shows its tab. Ghostty gets our frame, and its new-tab key (`Ctrl`+`Shift`+`T`, set to a new window) opens a tab of the window it was pressed in.                                                                                                                                                                              |
| **Caption buttons**       | Minimise, maximise or restore, and a wider close button. Each one lights up under the pointer; the close button glows red. A label appears when the pointer rests on one.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| **Floating windows**      | Every window floats. A program's new window opens where its last one closed; other new windows open in a cascade in the middle of the screen.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| **Fast animations**       | Windows open in 140 ms and close in 110 ms. Minimise and restore take 130 ms. The Control Panel sets the speed; the system's `enable-animations` turns them off.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| **Alt+Tab**               | A panel of live thumbnails over the desktop, each with its window's title. Tab, the arrow keys and the pointer pick a window.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| **Aero Snap**             | Drag a window against the top, left or right edge, or hold `Super` and press an arrow key. While you drag, a rectangle shows the place the window will take.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| **Double click**          | On the caption bar: maximise, and again to restore.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| **Taskbar**               | The orb (Aero) or the launcher plate (Kernel), buttons that fill its height (icons alone by default, as Windows 7 does; labels in the Control Panel), the notification area, the clock over the full date, and the show-desktop button at the far end. Aero puts it at the bottom, Kernel at the top. Auto-hide slides it away to a 2 px line that brings it back, and keeps it up while one of its menus is open.                                                                                                                                                                                                                                                                                                                                                                                                |
| **Sticky notes**          | Desktop menu, New, Sticky Note: a yellow note on the desktop to type in, moved by its band, sized by its corner, with + for another and × to remove it. The notes are kept in `~/.local/state/shojiwm-desktop/notes.json`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| **Gadgets**               | On the main screen's desktop: the CPU Meter, two dials for the processor and the memory in use read from `/proc`, and the Clock, an analog face with a seconds hand and the day. Drag them anywhere; the desktop menu's Gadgets submenu shows or hides each.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| **Desktop icons**         | The Recycle Bin and everything in the desktop folder. Drag an icon anywhere, on any screen; View → Align icons to grid snaps it to a cell, Auto arrange icons fills each screen's columns in sort order. The icons of a screen that is not connected show on the main one and go back when it returns. Click to pick, Ctrl+click or drag a box for several, double click to open; a `.desktop` file is a shortcut with the arrow. Right click an icon to open, delete or rename it; right click the desktop for View, Sort by, Refresh, New folder or text document, a terminal there, and Personalize. Computer, your files and the Control Panel can stand beside the Recycle Bin; Computer does by default.                                                                                                    |
| **Pinned programs**       | A pinned program keeps its icon on the taskbar while it is closed; a click starts it. Once it runs, its windows gather behind that same button. A middle click, Ctrl+click or Shift+click on any button starts another instance; clicks while a program starts do not start it again.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| **Button progress**       | A program's progress fills its taskbar button green, as Windows 7 shows a download; a count shows in a red badge, and an urgent program turns its button orange. Programs send these with the `com.canonical.Unity.LauncherEntry` signal (Chromium, Firefox, KDE and GTK programs).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| **Combined buttons**      | Every window of one program sits behind one button, drawn as a stack. The Control Panel's Taskbar and Start Menu page gives one button per window instead (`combineWindows`).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| **Window thumbnails**     | Rest on a taskbar button and its windows are listed as cards with their titles and a live picture. A click raises one, a middle click closes it. A media player's cards carry its toolbar, as on Windows 7: previous, play or pause, and next, through MPRIS. A card under the pointer shows a red close button at its top right.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| **Jump lists**            | Right click a taskbar button for the files the program opened last (from `recently-used.xbel`), the tasks of its `.desktop` file, a second instance, and close. A file's pin, at the right of its line under the pointer, keeps it in a Pinned section over the recent ones, in the taskbar's and the Start menu's jump lists.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| **Notification area**     | A tray item's own menu, drawn in the same style, with its submenus, check boxes and radio buttons. The icons you choose show on the bar, the others behind the arrow; new icons are hidden. A right click on the shell's own icons opens their menu: the bell (Clear all, Do not disturb), the volume (Volume Mixer, playback devices, Mute), the network (Wi-Fi, connections), the flag, Bluetooth and the clock; on the show-desktop strip (Peek at desktop) and on the bar itself (Task Manager, Properties).                                                                                                                                                                                                                                                                                                  |
| **Launcher**              | `Super`+`Shift`+`D` opens the Start menu's search alone, on the focused screen. Before you type, it lists the programs you started last (25 by default, set in the Control Panel), then every program. As you type, it finds programs, Control Panel tasks and files. The search is fuzzy: the letters you type must come in order in a name, so "fx" finds Firefox; letters side by side, at the start of a word and at the start of the name rank first, and the matched letters are highlighted. Return starts the one picked, or runs what you typed. Drag its sides and bottom edge to resize it; it keeps the size, and a list of height 0 leaves the field alone.                                                                                                                                          |
| **Run dialog**            | `Super`+`R`, or Run... in the Start menu: the Open field with the last 26 commands, completion from them, and the list under its arrow. A folder, a file or an address opens with `xdg-open`; anything else runs in a shell. A program that is not found stays in the field, with a message under it.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| **Start orb menu**        | A right click on the orb, or `Super`+`X`: terminals (one as root), the task manager, the Control Panel pages, the files, the network connections, the disks, the system log, the launcher, the power actions and the desktop.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| **Action Center**         | The flag of the notification area reports the problems that Windows 7's Action Center reports: system and user services that failed (systemd) and disks 90 % full, with a red or yellow mark. Its panel lists them; a failed service of yours can be cleared, a full disk opened. `shojiwm-shell msg actioncenter open\|close`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| **Notifications**         | The bell of the notification area keeps the notifications, across sessions, in `~/.local/state/shojiwm-desktop/notifications.json` (Hokum draws them and keeps none): a red count for the unseen ones, a click for the list with each program's icon, title, text and time, a cross per card and Clear all. The panel's switch, or a middle click on the bell, is Hokum's Do not disturb, which slashes the bell.                                                                                                                                                                                                                                                                                                                                                                                                 |
| **Network**               | The network icon shows a wired screen or wireless bars, with a yellow mark without Internet and a red one without a connection. A click opens the network panel: the connection in use, the wireless networks with their bars, Connect and Disconnect, the wireless switch and the connection editor. NetworkManager asks for a new network's key through its agent (nm-applet).                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| **Bluetooth**             | With a BlueZ adapter, a Bluetooth icon shows in the notification area, grey when Bluetooth is off. A click opens its panel: the adapter and its switch, the paired devices (connected ones first, with their battery), then the devices in reach. A click on a device offers Connect or Disconnect, Pair or Remove. `shojiwm-shell msg bluetooth open\|close`.                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| **Volume**                | The speaker shows the level of the default output. A click opens the volume panel: a slider, the mute button and the link to the Sound window's Playback page. The wheel steps it by 2 %, a middle click mutes, and the media keys work. With more than one output, a click on the device's name lists them to switch. `shojiwm-shell msg volume open\|openDevices\|close` drives the panel.                                                                                                                                                                                                                                                                                                                                                                                                                      |
| **Screen zoom**           | `Super`+`Shift` and the wheel magnify the screen around the pointer; `Super`+`Shift`+`0` resets it.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| **Additional clocks**     | Up to two more time zones, set in Date and Time with a field that offers the zones as you type, shown under the calendar and in the clock's tooltip with the day they are in.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| **Calendar**              | Click the clock for the long date and a month, today framed, with arrows to the months around it.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| **Screen saver**          | Mystify, drawn here: two bouncing shapes with fading trails, after 1 to 30 minutes without input (the idle protocol), until the mouse or a key; set and previewed on the Lock Screen page. Off by default, as on Windows 7.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| **Lock screen**           | The Windows 7 log-on screen: your picture behind (or the desktop's), the time, your account picture in its glass frame, your name, and the password field with the round blue arrow. `Super`+`L`, the Start menu, `loginctl lock-session`, going to sleep and a delay of inactivity lock it; PAM checks the password. It shows on the main screen only; the others stay black, and what is typed while one of them has the keyboard still goes to the password.                                                                                                                                                                                                                                                                                                                                                   |
| **Log-on screen**         | The lock screen as a log-on screen for greetd (`shojiwm-shell greeter`), with the session to start in the bottom-left corner. `nixosModules.greeter` sets it up. See "The log-on screen".                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| **Control Panel**         | A window laid out as the Windows 7 one: the theme, light or dark, the glass, the desktop background and its slide show, the lock screen picture and delay, the account picture, the taskbar and Start menu, the notification icons, the clock format and extra clocks, ease of access, and the pages below. Every change applies at once.                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| **Screen resolution**     | The Control Panel shows the screens as they are placed and sets the resolution, refresh rate and scale of each one, live. A change goes back on its own after 15 seconds unless you keep it. The choices replace `SHOJIWM_DESKTOP_OUTPUTS` for the screens you change. "Make this my main display" picks the main screen.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| **Devices and Printers**  | The Control Panel shows the computer, its screens, the USB devices (lsusb, hubs left out) with an icon that fits each, and the printers (CUPS) with a tick on the default one.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| **Sound**                 | The Control Panel lists the playback and recording devices PipeWire knows, marks the default ones, and makes another the default; its links open the Sound window. That window has a pane of pages: Playback and Recording (each program with its volume, its mute button and the device it uses, System Sounds grouped), Output and Input devices (the default one, volume, mute, port, balance) and Configuration (each card's profile). Everything follows PipeWire live (`shojiwm-shell msg sound open [playback\|recording\|outputs\|inputs\|configuration]`).                                                                                                                                                                                                                                               |
| **Default programs**      | The Control Panel sets the program that opens web links, mail, folders, text, pictures, music, videos and PDF documents, from the programs whose .desktop file says they can, through xdg-mime (`~/.config/mimeapps.list`). It also chooses the terminal, the file manager, the task manager, the system monitor, the clipboard history and the screenshot command.                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| **Startup programs**      | The Control Panel lists the XDG autostart entries the session starts: turn one off, restore a system entry, or add any installed program. Changes apply at the next log-on.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| **Background slide show** | Desktop Background can change the picture on its own from a folder (your Pictures by default), every 1 minute to 1 day, in name order or shuffled, with a Next picture button. Picture position sets Fill, Fit, Stretch, Tile or Center.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| **Light and dark**        | The system's own preference (`color-scheme`, which the settings portal passes to GTK, libadwaita, Qt, Firefox and Chromium). The shell follows it in both themes.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| **Aero Peek**             | Rest on the show-desktop strip and every window turns into a glass outline over the desktop. Click it to put the windows away; click again to bring them back.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| **Start menu**            | Pinned programs, then the ones you start most, and **All Programs** with every program in folders. Type to search, with the launcher's fuzzy matching; the arrow keys and Return pick a result, and Return with nothing found runs what you typed. On the right: your full name, your folders (as `xdg-user-dirs` names them), and a power button whose arrow offers log off, lock, restart, sleep and hibernate. Its search lists, under headings as Windows 7 does, the programs, the Control Panel tasks and the files of your home folder (fd). A program with recent files has an arrow; resting on it shows them in the right column. Recent Items in the right column opens a submenu of the files used lately. Compact by default; Taskbar and Start menu → Use large icons gives Windows 7's large rows. |

## Screenshots

Every picture here is a capture of this configuration running. None is drawn
or mocked up. `tools/screenshots.sh` makes them on a machine with no display:
ShojiWM's nested backend inside an X server with no screen, Mesa rendering in
software, and the keys and the pointer driven by xdotool. The Aero captures
are in `assets/site/`, the Kernel captures under the same names in
`assets/site/kernel/`. The [gallery](https://desktop.froment.software/gallery.html)
shows them all.

![The desktop](assets/site/desktop.png)

The wallpaper with its icons, two windows, the gadgets, and the taskbar: one
combined button for the two `foot` windows, the pinned `footclient` as an
icon alone since none of its windows is open, the notification area and the
clock.

![The desktop in the Kernel theme](assets/site/kernel/desktop.png)

The same session in the Kernel theme: the bar at the top, the steel-blue
caption plates, the dotted desk.

![Two window frames](assets/site/frames.png)

The frames: the focused window on top with a lighter outline, the other
behind it.

![Window groups: the ghost tab](assets/site/group-ghost.png)

A window dragged by its caption over the caption of another: it turns
see-through, and a ghost tab shows where it will go.

![Window groups: two tabs](assets/site/group-tabs.png)

After the drop: the two windows in one frame, as tabs.

![The Start menu](assets/site/start-menu.png)

The Start menu: the pinned programs on the left (`foot` and `footclient` here,
pinned by the capture script), your folders on the right, the search box at
the bottom and the power button with its arrow in the corner.

![All Programs](assets/site/start-menu-all.png)

**All Programs**: every program in a folder named after its main category.
One folder opens at a time.

![The power menu](assets/site/start-menu-power.png)

The arrow beside the power button: log off, lock, restart, sleep, hibernate.

![Running a command](assets/site/start-menu-run.png)

A search that finds no program offers to run the text as a command.

![The launcher](assets/site/launcher-search.png)

`Super`+`Shift`+`D`: the launcher, the Start menu's search on the focused
screen, here while typing, with the matched letters highlighted.

![The thumbnails of a taskbar button](assets/site/taskbar-thumbnail.png)

Resting on a taskbar button lists the windows behind it, with their titles.
See "Real limits" for why these cards show the program's icon.

![A jump list](assets/site/jump-list.png)

A right click on the same button: the files the program opened last, then
the program itself and "Close all windows". Nothing on the test machine
opens files through GTK, so `tools/screenshots.sh` writes the
`recently-used.xbel` the list is read from, naming `foot`.

![A notification-area menu](assets/site/tray-menu.png)

The menu of a tray item, drawn in the same style.

![Aero Snap](assets/site/snap.png)

Dragging a window against the left edge shows the rectangle it will take.

![Aero Peek](assets/site/peek.png)

Aero Peek: the pointer rests on the show-desktop strip, and the two windows
are left as glass outlines over the wallpaper.

![The calendar](assets/site/calendar.png)

The calendar over the clock.

![Alt+Tab](assets/site/alt-tab.png)

Alt+Tab: a panel of live thumbnails, the selected window lit.

![The Control Panel](assets/site/control-panel.png)

The Control Panel's home page.

![The Control Panel in the Kernel theme](assets/site/kernel/control-panel.png)

The same page in Kernel: each category in its own colour.

![The Control Panel in dark](assets/site/control-panel-dark.png)

Light or dark: the system's preference. The Control Panel, the menus and the
Start menu follow it.

![The lock screen](assets/site/lock-screen-wrong.png)

The lock screen, after a wrong password. The name is the test machine's;
yours comes from your account.

![The lock screen in dark](assets/site/lock-screen-dark.png)

In dark: the night wallpaper, the frame in smoked glass, a dark field and a
darker bead. `shojiwm-shell msg appearance setDark true` (or `toggle`)
switches it, as the Control Panel does.

### Close up

The Aero captures, cut up and blown up eight times with no smoothing, so
every pixel of the finish can be checked. `assets/site/details/` holds the
whole set; `tools/details.py` makes them.

|                                                               |                                                             |
| ------------------------------------------------------------- | ----------------------------------------------------------- |
| ![Top right corner](assets/site/details/corner-top-right.png) | ![Top left corner](assets/site/details/corner-top-left.png) |
| The caption buttons hanging from the frame's own outline.     | The rounded top corner: dark line, light line, glass.       |
| ![The title](assets/site/details/caption-title.png)           | ![The start orb](assets/site/details/taskbar-orb.png)       |
| Dark text on the glass.                                       | The orb in its seat on the taskbar.                         |

## Where the numbers come from

The Aero frame, the caption buttons and the title are not drawn by eye. Real
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

- **ShojiWM** from the `integration` branch of
  [sachahjkl/ShojiWM](https://github.com/sachahjkl/ShojiWM/blob/integration/FORK.md),
  installed and registered as a session. It carries what upstream does not
  have yet and this config uses: the `ext-data-control-v1` global (Super+V)
  and labels' `textShadow` (the caption titles). Upstream ShojiWM runs it
  without these.
- **`shojiwm-shell`** in the session's `PATH` (`packages.<system>.shell`). The
  configuration starts it.
- **A GPU driver with Vulkan or OpenGL**: AntiWebUI renders with wgpu.
- **Fonts.** The text is set in **Inter**, and falls back to Noto Sans. The
  Kernel theme sets its titles and clock in **JetBrainsMono Nerd Font**
  (`JetBrainsMono NF`).
- **An icon theme** with an `application-x-executable` icon, such as
  **Adwaita** or **Papirus**. Window and taskbar icons come from it.
- **fd**, for the files the Start menu and the launcher find.
- **[inspecteur](https://inspecteur.sacha.house)**, the task manager, unless
  the Control Panel's Default Programs names another.
- The programs the keys start by default: **kitty**, **Thunar** and
  **Chromium**. Default Programs or the session's environment chooses others
  (see "Changing it").
- **The lock screen** checks the password with the `login` PAM service
  (`/etc/pam.d/login`), and needs the compositor's `ext-session-lock`, which
  ShojiWM has.
- **Light and dark** for programs needs **dconf** (on NixOS
  `programs.dconf.enable = true`) and a settings portal that reads it,
  such as **xdg-desktop-portal-gtk**, which ShojiWM's NixOS module installs.
  The Nix package of the shell brings the `dconf` and `gdbus` commands itself.

Optional: `systemd` for the power actions and the Action Center,
NetworkManager, BlueZ, PipeWire (`wpctl`), `playerctl`, Hokum for the
notifications, and any status-notifier application for the notification
area.

### The Aero icon theme

`packages.<system>.icon-theme` installs the icon theme "Aero": Oxygen with
the desktop's yellow folders, its libraries and the glass Recycle Bin, so a
file manager such as Thunar shows the same icons as the desktop. Install it
with `kdePackages.oxygen-icons`, which it inherits, and choose it for the
session (`org.gnome.desktop.interface icon-theme`).

### Global shortcuts

Programs such as Hex, OBS or Discord bind their keys through the
GlobalShortcuts portal, which ShojiWM does not have. `packages.<system>.portal`
(`portal/`, xdg-desktop-portal-shojiwm-desktop) provides it. It takes each
program's preferred keys, asks the configuration to bind them, and sends the
program `Activated` when they go down and `Deactivated` when any of them goes
up. Hold-to-talk shortcuts therefore work. There is no dialog to choose other
keys: change them in the program.

On NixOS:

```nix
xdg.portal = {
  extraPortals = [inputs.shojiwm-desktop.packages.${system}.portal];
  # The desktop name in lowercase: xdg-desktop-portal reads
  # shojiwm-portals.conf.
  config.shojiwm."org.freedesktop.impl.portal.GlobalShortcuts" = "shojiwm-desktop";
};
```

### The log-on screen

`shojiwm-shell greeter` is the lock screen as a log-on screen for
[greetd](https://sr.ht/~kennylevinsen/greetd/): the same picture, clock,
account picture and password field on the main screen, black on the others,
and the session to start in the bottom-left corner. It remembers the last
session chosen. If PAM asks a second question after the password (a one-time
code), the field asks it.

On NixOS, `nixosModules.greeter` sets it up: greetd, a Hyprland of its own
with your screens, and the greeter account.

```nix
imports = [inputs.shojiwm-desktop.nixosModules.greeter];
services.displayManager.ly.enable = false; # or whichever display manager you had
services.shojiwm-desktop-greeter = {
  enable = true;
  user = "alice";
  defaultSession = "shojiwm-uwsm";
  mainScreen = "HDMI-A-1";
  monitors = [{output = "HDMI-A-1"; mode = "3840x2160@60"; position = "0x0"; scale = 1.875;}];
};
```

The module also has `keyboard.layout`, `keyboard.variant` and
`keyboard.options`, which default to `services.xserver.xkb`.

If the log-on screen leaves without starting a session (a crash, or
Ctrl+Alt+Delete on a log-on screen that drew nothing), greetd shows its text
log-on, which starts the default session. Keep `/var/lib/greeter` if your
root is not persistent: the last session chosen lives there.

Elsewhere, run `shojiwm-shell greeter` in a compositor with wlr-layer-shell
that greetd starts as the greeter account, and stop the compositor when the
greeter exits. It reads:

| Variable                          | Value                                                                                           |
| --------------------------------- | ----------------------------------------------------------------------------------------------- |
| `SHOJIWM_GREETER_USER`            | The account to log on.                                                                          |
| `SHOJIWM_GREETER_SESSIONS`        | The folder of the sessions' `.desktop` files (`share/wayland-sessions`).                        |
| `SHOJIWM_GREETER_DEFAULT_SESSION` | The session chosen the first time: its file name without `.desktop`.                            |
| `SHOJIWM_DESKTOP_PRIMARY_SCREEN`  | The connector of the main screen (`HDMI-A-1`). Without it, the first screen.                    |
| `SHOJIWM_SHELL_DIR`               | The folder of the pictures: the configuration's `shell`. Without it, `~/.config/shojiwm/shell`. |

The account picture is AccountsService's (`/var/lib/AccountsService/icons/`):
the greeter cannot read `~/.face`. To try it without logging out, run it
under greetd's `fakegreet`, which accepts the account `user`, the password
`password`, then `9`:

```sh
SHOJIWM_GREETER_USER=user SHOJIWM_GREETER_SESSIONS=/run/current-system/sw/share/wayland-sessions \
  fakegreet "shojiwm-shell greeter"
```

`checks.<system>.greeter` runs the module end to end in VMs: a password
typed into the log-on screen starts the session, the greeter's Hyprland
leaves and the session chosen is kept; and a log-on screen killed gives way
to the text log-on, which starts the default session.

## Install

### With Nix

Install the shell, then the configuration into `~/.config/shojiwm`:

```sh
nix profile install github:sachahjkl/shojiwm-desktop#shell
nix run github:sachahjkl/shojiwm-desktop#install
```

The second command copies `src`, `assets`, `shell` (the shell's pictures),
`package.json` and `tsconfig.json`. It does not touch `node_modules`, which
ShojiWM manages, or `settings.json`.

To use the flake as an input of your own system flake:

```nix
{
  inputs.shojiwm-desktop = {
    url = "github:sachahjkl/shojiwm-desktop";
    inputs.nixpkgs.follows = "nixpkgs";
  };
}
```

Put `packages.<system>.shell` in `environment.systemPackages`.
`packages.<system>.default` is the configuration tree. Link its directories
into `~/.config/shojiwm` with whatever manages your home files, or run
`apps.<system>.install`.

### Without Nix

```sh
git clone https://github.com/sachahjkl/shojiwm-desktop.git
cd shojiwm-desktop
(cd crates/shell && cargo build --release)
install -Dm755 crates/shell/target/release/shojiwm-shell ~/.local/bin/shojiwm-shell
mkdir -p ~/.config/shojiwm
rm -rf ~/.config/shojiwm/src ~/.config/shojiwm/assets ~/.config/shojiwm/shell
cp -R src assets shell package.json tsconfig.json ~/.config/shojiwm/
```

The shell links fontconfig, freetype, libxkbcommon, wayland, PAM and libxcb,
and loads Vulkan or OpenGL, Wayland and PipeWire at run time.

Then start the ShojiWM session, or press `Super`+`Shift`+`R` to reload a
running one.

## Keys

| Key                               | Action                                                                                                |
| --------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `Super` (tap)                     | Show or hide the Start menu                                                                           |
| `Super`+`Return`                  | Terminal                                                                                              |
| `Super`+`E`                       | File manager                                                                                          |
| `Super`+`B`                       | Browser                                                                                               |
| `Alt`+`F4`                        | Close the focused window; on the desktop, Shut Down Windows                                           |
| `Super`+`Shift`+`E`               | Shut Down Windows                                                                                     |
| `Alt`+`Tab` / `Alt`+`Shift`+`Tab` | Switch window                                                                                         |
| `Super`+`Up`                      | Maximise                                                                                              |
| `Super`+`Down`                    | Restore a maximised window, else minimise                                                             |
| `Super`+`Left` / `Super`+`Right`  | Snap to that side; again, to the next screen; from the other side, restore                            |
| `Super`+`Shift`+`Left` / `Right`  | Move the window to the next screen that way                                                           |
| `Super`+`Home`                    | Minimise the other windows, then bring them back                                                      |
| `Super`+`Ctrl`+`Left` / `Right`   | The tab before or after in the focused window's group                                                 |
| `Super`+`Ctrl`+`Down`             | Take the shown tab out of its group, as a window of its own                                           |
| `Print Screen`, `Ctrl`+`Print`    | Screenshot (the Control Panel's Default Programs; by default the whole screen to the clipboard)       |
| `Super`+`D`                       | Show the desktop, then put the windows back                                                           |
| `Super`+`Shift`+`D`               | The launcher: the Start menu's search, on the focused screen                                          |
| `Super`+`M`                       | The same as `Super`+`D`                                                                               |
| `Super`+`L`                       | Lock                                                                                                  |
| `Super`+drag                      | Move a window from anywhere on it                                                                     |
| `Super`+double click              | Maximise, and again to restore                                                                        |
| `Super`+`Shift`+`R`               | Reload this configuration (built into ShojiWM)                                                        |
| `Super`+`Shift`+`Q`               | Close the focused window                                                                              |
| `Super`+`1` … `9`                 | The taskbar button at that place: start it, show it, put it away, or the next of its windows          |
| `Super`+`Shift`+`1` … `9`         | Another instance of that button's program                                                             |
| `Super`+`V`                       | Clipboard history: the command Default Programs names, for example `papier toggle`                    |
| `Super`+`R`                       | The Run dialog, on the focused screen: its history (26), completion from it, Down or F4 for the list  |
| `Super`+`X`                       | The Start orb's menu (also a right click on the orb): terminals, tools, power actions                 |
| `Ctrl`+`Shift`+`Escape`           | Task manager (Default Programs, else inspecteur); also in the bar's and the clock's right-click menus |
| `Super`+`Shift`+wheel             | Zoom the screen around the pointer; `Super`+`Shift`+`0` resets it                                     |
| Volume, mute and player keys      | Volume of the default output (`wpctl`), microphone mute, `playerctl`                                  |

## Commands

The compositor's keys drive the shell with `shojiwm-shell msg <target>
<function> [args]`, over `$XDG_RUNTIME_DIR/shojiwm-shell.$WAYLAND_DISPLAY.sock`.
A script can send the same commands. A screen argument is a connector name
(`DP-2`); an empty one means the main screen.

| Target         | Functions                                                         |
| -------------- | ----------------------------------------------------------------- |
| `taskbar`      | `activate <n>`, `launchNew <n>`                                   |
| `startmenu`    | `open`, `openOn <screen>`, `close`, `toggle`, `toggleOn <screen>` |
| `launcher`     | `open`, `openOn <screen>`, `close`, `toggle`, `toggleOn <screen>` |
| `run`          | `open`, `openOn <screen>`, `toggleOn <screen>`, `close`           |
| `shutdown`     | `openOn <screen>`, `close`                                        |
| `adminmenu`    | `toggle`, `toggleOn <screen>`, `close`                            |
| `lock`         | `lock`, `isLocked`                                                |
| `volume`       | `open`, `openDevices`, `close`                                    |
| `bluetooth`    | `open`, `openOn <screen>`, `close`                                |
| `actioncenter` | `open`, `close`                                                   |
| `controlpanel` | `open`, `openPage <page>`, `close`                                |
| `appearance`   | `setDark true\|false`, `toggle`, `isDark`                         |

The Control Panel's pages are `appearance`, `background`, `display`,
`lockscreen`, `account`, `taskbar`, `notifications`, `clock`, `ease`,
`devices`, `sound`, `defaults`, `startup` and `about`.

## The files

| File                                            | What it holds                                                                                                                                               |
| ----------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src/index.tsx`                                 | The entry point: environment, keys, the shell process, the blur, the snap preview, and the composition of every window.                                     |
| `src/themes/choice.ts`, `src/themes/frames.tsx` | The theme chosen in the Control Panel (Aero or Kernel), the room each theme's frame takes, and the frame each window gets.                                  |
| `src/themes/aero/`, `src/themes/kernel/`        | Each theme's frame: `metrics.ts` (every size and colour), `frame.tsx` (the caption, its buttons and labels).                                                |
| `src/themes/tabs.tsx`                           | The tab strip of a group's caption, in the theme's style, with the ghost tab of a drop.                                                                     |
| `src/groups.ts`                                 | Window groups: the windows of each group, the one shown, the tab layout, the programs whose new windows open as tabs.                                       |
| `src/window-manager.ts`                         | The floating window manager: placement, move, resize, maximise, minimise, fullscreen, double click, and Aero Snap.                                          |
| `src/alt-tab.tsx`                               | The Alt+Tab panel, built from render textures and an input grab; the shell draws its titles.                                                                |
| `src/peek.tsx`                                  | Aero Peek, and the socket (`$SHOJIWM_DESKTOP_SOCKET`) the shell and the portal talk to the configuration through.                                           |
| `src/display.ts`, `src/programs.ts`             | The screens (`SHOJIWM_DESKTOP_OUTPUTS` and the Control Panel's choices), and the programs the keys start.                                                   |
| `src/shortcuts.ts`                              | The keys the GlobalShortcuts portal binds, over the same socket.                                                                                            |
| `src/zoom.tsx`                                  | The screen zoom.                                                                                                                                            |
| `crates/shell/src/main.rs`, `ipc.rs`            | The shell's entry point and its command socket (`shojiwm-shell msg`).                                                                                       |
| `crates/shell/src/compositor.rs`                | The shell's line to the configuration: Peek, show desktop, screens, and the settings the frames are drawn with.                                             |
| `crates/shell/src/settings.rs`                  | What the Control Panel changes, kept in `~/.config/shojiwm/settings.json`, and the system's light or dark preference.                                       |
| `crates/shell/src/theme.rs`, `theme_values.rs`  | Every size and colour of the shell, per theme.                                                                                                              |
| `crates/shell/src/taskbar/`                     | The bar, its buttons, thumbnails, jump lists, tray and media controls.                                                                                      |
| `crates/shell/src/start_menu/`                  | The Start menu, the launcher, the search, the places and the power menu.                                                                                    |
| `crates/shell/src/fuzzy.rs`                     | The fuzzy matching of the searches (nucleo).                                                                                                                |
| `crates/shell/src/status/`                      | The notification area's panels: volume, network, Bluetooth, notifications, Action Center, calendar.                                                         |
| `crates/shell/src/desktop/`                     | The wallpaper, the desktop icons, the gadgets and the sticky notes.                                                                                         |
| `crates/shell/src/dialogs/`                     | Run, Shut Down Windows, the orb's menu, the Alt+Tab titles, the screen saver.                                                                               |
| `crates/shell/src/control_panel/`               | The Control Panel window and its pages.                                                                                                                     |
| `crates/shell/src/lock/`, `greeter/`            | The lock screen with its PAM check, and the log-on screen for greetd.                                                                                       |
| `crates/shell/src/ui/`                          | The controls the surfaces share: frames, menus, scroll bars, text, icons.                                                                                   |
| `shell/assets/`                                 | The shell's pictures: the orb, the place icons, the account tile, the wallpapers, and the colour icons (Oxygen, LGPL-3.0, built by `tools/fetch-icons.sh`). |
| `shell/gtk-switch.sh`                           | Moves GTK 3 to the light or dark variant of the theme in use.                                                                                               |
| `portal/`                                       | The GlobalShortcuts portal service (Rust), with its D-Bus and systemd files in `portal/data`.                                                               |
| `assets/frame.frag`, `assets/kernel/`           | The paint shaders that draw the Aero frame and the Kernel plate, and Kernel's caption icons.                                                                |
| `assets/caption/*.svg`                          | The Aero caption buttons, one file per state, drawn at their exact pixel size.                                                                              |
| `assets/layer-mask.frag`                        | Clips the blur behind a panel to the shape the panel draws.                                                                                                 |
| `docs/aero-measurements.md`                     | Every measured number, and the reference screenshot each came from.                                                                                         |
| `nix/greeter.nix`, `tests/greeter.nix`          | The log-on screen's NixOS module, and its test in VMs.                                                                                                      |
| `tools/screenshots.sh`                          | Runs the whole desktop on a machine with no display and takes the pictures in this file.                                                                    |
| `tools/dev.sh`                                  | Runs ShojiWM nested in a window, on this checkout.                                                                                                          |
| `assets/logo.svg`, `.project/`                  | The project logo (drawn by hand), its 512 px rendering, and the project description that sacha.house reads.                                                 |
| `site/`, `.github/workflows/pages.yml`          | The project site (overview, gallery, documentation), and the workflow that publishes it to GitHub Pages with the logo and the screenshots.                  |
| `flake.nix`                                     | The packages, the install command, the log-on screen module, and the checks.                                                                                |

## Changing it

- **Most things:** the **Control Panel** (Start menu → Control Panel). It
  writes `~/.config/shojiwm/settings.json`, which an update of the
  configuration leaves alone and which can be edited by hand; the shell reads
  it again when it changes. The site's documentation lists every key.
- **Pinned programs:** `startPinned` and `taskbarPinned` in `settings.json`,
  as `.desktop` file names.
- **Programs:** the Control Panel's Default Programs chooses the terminal,
  the file manager, the task manager, the system monitor, the clipboard
  history and the screenshot: an installed program or any command. "As the
  session sets it" takes `SHOJIWM_DESKTOP_TERMINAL`,
  `SHOJIWM_DESKTOP_FILE_MANAGER`, `SHOJIWM_DESKTOP_TASK_MANAGER`,
  `SHOJIWM_DESKTOP_SYSTEM_MONITOR`, `SHOJIWM_DESKTOP_CLIPBOARD` or
  `SHOJIWM_DESKTOP_SCREENSHOT` from the session's environment, else kitty,
  Thunar, inspecteur, inspecteur on its Performance section, none, and the
  whole screen to the clipboard. The browser is `BROWSER` at the top of
  `src/index.tsx`.
- **Power:** the Control Panel picks what the Start menu's power button does
  (`powerButtonAction`).
- **Main screen:** Control Panel → Screen Resolution → Make this my main
  display. Without a choice, `SHOJIWM_DESKTOP_PRIMARY_SCREEN`, else the first
  screen. The lock screen, the gadgets, and the Start menu and Shut Down
  dialog asked without a screen go there.
- **Screens:** set `SHOJIWM_DESKTOP_OUTPUTS` in the session environment to
  Hyprland monitor rules `name,mode,position,scale` separated by `;`, for
  example `DP-2,2560x1440@180,0x0,1.25;HDMI-A-1,preferred,auto,1.875`.
  Without it, ShojiWM uses the best mode of each screen at scale 1.
- **More keys:** set `SHOJIWM_DESKTOP_KEYS` to a JSON object of shortcut to
  shell command, for example `{"Ctrl+Shift+u": "papier toggle"}`, for
  programs that do not use the GlobalShortcuts portal.
- **Notifications:** `assets/hokum/aero.kdl` is a Windows 7 balloon theme
  for [Hokum](https://hokum.app). Point Hokum at it in
  `~/.config/hokum/config.kdl` with
  `theme "~/.config/shojiwm/assets/hokum/aero.kdl"`, and put the cards over
  the notification area with `behaviour { position "bottom-right" }`.
- **Frame colours and sizes:** `src/themes/<theme>/metrics.ts`.
- **Shell colours and sizes:** `crates/shell/src/theme_values.rs`.
- **Wallpaper, lock screen picture, account picture:** the Control Panel. With
  no account picture chosen there, the system's is used, from the first of
  `/var/lib/AccountsService/icons/$USER`, `~/.face` and `~/.face.icon`.

Press `Super`+`Shift`+`R` after a change to `src`. After a change to
`crates/shell`, build the shell again and restart it.

## Chromium and Firefox

Chromium and Firefox draw their own title bar by default, so they keep it. To
give Chromium the theme's frame, right click its tab strip and switch _on_
**Use system title bar and borders**. Firefox has the same setting under
**Customise toolbar → Title bar**.

## Development

`nix develop` gives Rust, TypeScript, Odin, prettier, the shell's libraries and the
pre-commit hooks. `tools/dev.sh` runs ShojiWM nested in a window of the
current session, with `src`, `assets` and `shell` of `~/.config/shojiwm`
linked to the checkout.

`tools/screenshots.sh` takes every picture. `THEME=kernel` runs the session
in Kernel, `OUT` names the folder (`assets/site` by default), `UNTIL` stops
after the picture of that name, `KEEP=1` keeps the session up at the end, and
`SHOJIWM_FLAKE` points at another ShojiWM:

```sh
tools/screenshots.sh
THEME=kernel OUT=assets/site/kernel tools/screenshots.sh
```

## Checks

```sh
nix flake check
```

runs eight checks:

- `shell` — the shell builds.
- `shell-lint` — `clippy` without warnings and `rustfmt` over `crates/shell`.
- `portal` — the portal builds and passes its tests.
- `portal-lint` — `clippy` without warnings and `rustfmt` over `portal`.
- `typecheck` — `tsc` over `src`, against the ShojiWM TypeScript package
  pinned in `flake.lock`.
- `greeter` — the log-on screen module in VMs.
- `pre-commit` — `alejandra`, `deadnix`, `statix` and `prettier`.
- `changelog` — the tests of `tools/desktoptool`, then its `changelog check`
  (x86_64 only).

`CHANGELOG.md` is made of fragments in `changelog.d/`, one per change:
`tools/desktoptool` writes, checks and assembles them (see
`changelog.d/README.md`).

## Real limits

- **The Alt+Tab panel has square corners.** `<Solid>` takes a colour and a
  rectangle, with no radius
  (`docs/configuration/output-composition-reference.md` in ShojiWM). To
  round it, a shader must draw the card into a texture.
- **The taskbar thumbnails of the screenshots show the program's icon.** The
  shell asks AntiWebUI for a live picture of each window
  (`ext_image_copy_capture` or ShojiWM's `hyprland_toplevel_export`,
  `crates/shell/src/taskbar/thumbnails.rs`). In the headless session that
  takes the screenshots, no picture arrives, and the card keeps the icon.

## Not done yet

Possible with the APIs above, simply not written:

- **Aero Peek on one window**, from its taskbar thumbnail. The shell knows a
  window as a foreign-toplevel handle and the compositor as a
  `WaylandWindow`; nothing ties the two together over the socket yet.

## Tested, and not tested

Run, on a server with no display and no GPU:

- `nix flake check` — the shell and the portal with their lints, the
  TypeScript against the ShojiWM package pinned in `flake.lock`, the log-on
  screen in VMs, and the formatters.
- `tools/screenshots.sh` — the whole session (Kernel with `THEME=kernel`): the compositor
  in its nested backend inside Xvfb with Mesa's software renderer, the shell,
  two `foot` windows and a tray item. Every picture above comes from that
  run.
- In that session, through xdotool: moving a window by its caption bar,
  dropping one caption on another to group them, opening and closing the
  Start menu, the thumbnails of a taskbar button, a jump list with its recent
  files, the menu of a tray item, Aero Peek by resting on the show-desktop
  strip, Aero Snap by dragging a window to the left edge, Alt+Tab, the
  Control Panel, and a wrong password on the lock screen.

Not run:

- **A real GPU session in the captures.** Everything there was drawn by
  llvmpipe at 1280x800 and scale 1.
- **The programs the keys start.** kitty, Thunar and Chromium are not
  installed on the test machine; the pictures use `foot`.
- **Real input devices.** The pointer and the keys came from xdotool through
  the X server.
- **The power actions, running a typed command, and the programs used most.**
  The menus are captured; nothing was shut down or run from them.

## Licence

MIT. See [LICENSE](./LICENSE).

The artwork in `assets/` and `shell/assets/` is original SVG, written by hand
for this repository, except `shell/assets/icons/` and the NixOS snowflake. The
Aero theme reconstructs a look from measured screenshots
([docs/aero-measurements.md](docs/aero-measurements.md)); it copies no file,
no icon and no font from Microsoft. No font is bundled: Inter and JetBrains
Mono (SIL OFL) come from the system.

`shell/assets/nixos-snowflake.svg` is the NixOS logo from the NixOS artwork
(nixos-icons), CC-BY-4.0, shown on the Control Panel's System page.
`shell/assets/start-orb-logo.svg` is the same logo, plain white and bolder,
on the Start orb.

`shell/assets/icons/` holds icons built by `tools/fetch-icons.sh` from
KDE's [Oxygen icons](https://invent.kde.org/frameworks/oxygen-icons),
LGPL-3.0-or-later: glossy icons of the Windows 7 era, with Oxygen's yellow
folder and emblems over it for the libraries. Their licence is next to them.

Windows, Windows 7 and Aero are trademarks of Microsoft Corporation. This
project is not affiliated with, endorsed or sponsored by Microsoft. The
[legal page](https://desktop.froment.software/legal.html) gives the details:
how it was made, the licences, trademarks and the site's privacy.
