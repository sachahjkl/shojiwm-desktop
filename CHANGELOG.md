# Changelog

All notable changes to shojiwm-desktop. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project follows
[semantic versioning](https://semver.org).

## [0.2.0] - 2026-11-11

### Added

- **Legal page** on the site: a clean-room implementation, what it uses and under which licences, trademarks, and what the site does with your data.
- **Window groups for programs that draw their own title bar**: a tab strip is put over their title bar while they are in a group.
- **The window under a thumbnail stands out**: while the pointer rests on a taskbar thumbnail, its window comes to the front with a gold glow and outline.
- **Light or dark by the sun** (Control Panel > Light, dark and glass): dark from sunset to sunrise at the place of the time zone, found without the network. The page gives today's times. This is the default.
- **Taskbar location** (Control Panel > Taskbar and Start Menu): bottom or top, in both themes. The bottom is the default.
- **Network and Sharing Center** in the Control Panel: the connection map, the active networks with their addresses, gateway and DNS, the adapters (enable, disable, Wi-Fi on or off), the saved connections (connect, disconnect, connect automatically, forget, properties: name, Wi-Fi key, IPv4 and IPv6 settings) and a hidden network to join. It replaces nm-connection-editor.
- **A Sound window** (`shojiwm-shell msg sound open`), with a sidebar and five pages: Playback (each program's volume, mute and output device), Recording (each program's input device), Output devices and Input devices (default device, volume, mute, port, balance), and Configuration (each card's profile). It replaces pwvucontrol: the volume panel and the Control Panel open it.

### Changed

- **The site's footer** is Froment Software's, with its logo, and the page you are on is a soft plate in the navigation.
- **The compositor starts the shell again if it stops.**
- **A window maximises when its top edge reaches the top of the work area** while dragged: the bar's edge when the bar is at the top, the screen's top otherwise.
- **Fuzzy search in the Control Panel**, with the matched letters highlighted, as in the Start menu.
- **Kernel's tabs read as tabs**: the shown tab joins the window under it; the others are darker and set back, with readable titles.
- **Kernel's taskbar buttons**: no plate at rest; under the pointer and for the active window, a flat light plate as tall as the bar, the icon inside it.
- **Kernel draws square corners everywhere**: window frames, tabs, menus, popups and buttons.
- **Kernel is the default theme**, of the shell, of the compositor and of the site.

### Fixed

- **One thumbnail per window**: a window opened on an empty desktop, or just after the shell started, showed twice in its button's thumbnails.
- **Sound cards' profiles**: only one card listed its profiles.
- **The network map's red crosses** showed while the network worked.
- **The shell no longer stops** when a thumbnail is shown for a window that just closed.
- **A program that quits leaves no ghost** in the taskbar or its thumbnails.
- **No more stretching while a window grows**: the last picture of the program stays at its size until the next one, instead of being stretched over the new size.
- **A window's icon is over its title's glow** (Aero).
- **White text is white again**: the lock screen's clock, the session picker, the gauges and the desktop's icon names looked silver, their shadow drawn over them.
- **The clock fits the bar**: the time over the date, with room above and below them, in both themes and with the bar at the top or the bottom; Kernel's text is a little smaller.

## [0.1.0] - 2026-11-11

### Added

- **The desktop**: window frames and window groups in the compositor; a shell with a taskbar,
  a Start menu, a notification area, desktop icons and gadgets, a Control Panel, a lock screen
  and a log-on screen; two themes, Aero and Kernel.
- **The site**, at https://desktop.froment.software: an overview, a gallery and the
  documentation.

[0.2.0]: https://github.com/sachahjkl/shojiwm-desktop/tree/v0.2.0
