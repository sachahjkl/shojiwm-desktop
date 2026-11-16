# shojiwm-desktop: intent

A desktop in two looks (Kernel, the default, and Aero, Windows 7's) on the ShojiWM compositor.

## The parts

- **The compositor configuration** (`src/*.ts`, `src/themes/`): TypeScript that ShojiWM runs. It
  places windows, draws their frames, keeps window groups as tabs and binds the keys. It must work
  alone: the shell can start later, crash or restart.
- **The shell** (`crates/shell`, Rust on AntiWebUI): the taskbar, the Start menu, the dialogs, the
  Control Panel, the lock screen and the log-on screen. One process, one UI thread.
- **The NixOS module** (`nix/*.mod.nix`, `services.shojiwm-desktop`): the whole session, its
  programs, its GTK and Qt themes, the portal.
- **The forks**: ShojiWM (`sachahjkl/ShojiWM`, branch `integration` = upstream + the branches in its
  `FORK.md`) and AntiWebUI (`sachahjkl/AntiWebUI`, branch `integration`). Each fix lives on its own
  branch for its upstream pull request.

## Subtleties

- **The shell talks to the compositor configuration** over a socket (`compositor.rs` `share(...)`
  to `peek.tsx` `server.handle(...)`). Every value has a default on the TypeScript side, so the
  compositor behaves well before the shell sends anything.
- **A reload closes that socket**, and the new configuration runs with the settings it kept (see
  below) until the shell sends them again. The shell's reader sees the end of file at once, tries the
  socket again when inotify sees it made (and after a backoff of 10 ms up to 250 ms, for a socket
  made but not listening yet), then sends the kept settings first.
- **Files the shell follows are watched, never polled** (`watch.rs`: settings, recent files,
  `user-dirs.dirs`, desktop folders). inotify watches the file's folder, not the file: an editor
  or an atomic writer replaces the file by a rename. The event only wakes the worker; the file's
  time decides whether it is read again, which also ignores the shell's own writes.
- **The UI thread never touches the disk, a process or the network.** Work goes to
  `background::off_thread` or a watcher thread, and the result comes back through a UI channel.
- **The compositor keeps the shell's last settings through a reload** (`peek.tsx`, `setting(...)`,
  `event.persist`) and applies them in `onEnable`, before the shell reconnects: without this the
  theme, the glass, the corners and the tab choices fall back to their defaults for a moment. The
  portal's shortcuts are not kept: a key bound with no service to tell would swallow its presses.
- **A configuration reload** (Super+Shift+R) calls `onDisable` then `onEnable`, not `onOpen`: the
  window manager keeps its state through `snapshot()` / `restoreSnapshot()`, and `onFirstCommit`
  registers the windows it does not know.
- **The restore after a reload ends on frames, not on events.** ShojiWM marks every live window
  dirty and evaluates them all in the next frame, so each kept window comes back there. A window
  closed during the reload never comes back, and no event names it. So the restore ends when all
  kept windows are back, or at the first frame 500 ms after the reload (`createPollForEachOutput`);
  its groups then re-form without it. Refused: the first `onOpen` of a new window (none may come,
  and one can arrive in the same pass as the kept ones); a wall-clock timer (it fires with the
  screens asleep, before any frame has brought the windows back).
- **Window tabs never take a window that draws its own title bar** (client-side decorations):
  our tab strip would sit over or under that bar. The Control Panel's choices cannot lift this.
- **The look is one setting** (`settings.theme`) that reaches the compositor (frames), the shell
  (its own widgets) and the GTK theme (`shell/gtk-switch.sh`). Qt takes its palette from GTK.
- **Programs of other projects** (inspecteur, classeur, papier, cliche) fill the desktop's roles
  by default; each role is a command the session can change.

## File Index

- [features/click-catcher.md](features/click-catcher.md): the one layer per screen that closes
  the Start menu, the launcher and the popover of hidden icons on a press outside them.
- [features/taskbar-thumbnails.md](features/taskbar-thumbnails.md): the live previews over a
  taskbar button, who hides them and when, and the live picture shared with the tab preview.

Add the intent of a feature in `features/` when its design is decided.
