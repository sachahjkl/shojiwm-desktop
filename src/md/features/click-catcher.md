# The click-catcher

Code: `crates/shell/src/catcher.rs`.

## Purpose

The Start menu, the launcher and the popover of hidden icons are layers, not popups. A popup
holds a pointer grab, and the compositor starts a drag only from a press of the grab it is in:
no icon could be dragged out of a popup. With no grab, a press outside these layers must still
close them. A transparent layer over the screen catches that press.

## Design

- **One catcher per screen**, opened by `start_menu::install` for each output. It is a
  `Layer::Top` layer over the whole screen, with no blur and no keyboard.
- **Claims.** What can be open calls `catcher::claim(open, reach, close)`: whether it is open on a
  screen (a tracked read), how much of the screen the catcher takes for it, and how to close it.
  A claim lives as long as the owner that registered it (the tray's lives with its bar).
- **The input region is the union of the open claims.** `Reach::OutsideBar` leaves the bar, so
  that a press on the bar reaches the bar (it closes its popovers itself) and a drop on the bar
  is the bar's. `Reach::Screen` takes the bar too: the launcher is not the bar's. Nothing open:
  the region is empty.
- **A press closes every claim open on that screen.**
- **The screen size is read when it changes** (`Output::current`), so a mode or scale change
  moves the edge of the bar.

## Subtleties

- **The catcher stays mapped** with an empty input region while nothing is open. Unmapping two
  layers in one frame once ended the shell's Wayland connection. AntiWebUI has nothing that makes
  this safe now, so the rule stays.
- **Stacking follows the order of opening.** The catcher opens after the bar (over it) and
  before the Start menu's layer (under it). The popover of hidden icons and the launcher's panel
  open later, on demand, so they stand over it; the launcher's panel is on `Layer::Overlay`.
- **The Start menu's claim is open on every screen** while the menu is open anywhere: a press on
  another screen closes it, as before. The launcher's and the popover's claims are open on their
  own screen only.

## Refused

- **One catcher per open thing** (three layers before): two stayed mapped on each screen, each
  with its own copy of the size and of the rule for the bar.
- **Moving the catcher to `Layer::Overlay` while the launcher is open:** a compositor may restack
  a layer that changes layer, and the catcher could come back over the Start menu.
