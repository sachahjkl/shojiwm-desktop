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
- **The UI thread never touches the disk, a process or the network.** Work goes to
  `background::off_thread` or a watcher thread, and the result comes back through a UI channel.
- **A configuration reload** (Super+Shift+R) calls `onDisable` then `onEnable`, not `onOpen`: the
  window manager keeps its state through `snapshot()` / `restoreSnapshot()`, and `onFirstCommit`
  registers the windows it does not know.
- **Window tabs never take a window that draws its own title bar** (client-side decorations):
  our tab strip would sit over or under that bar. The Control Panel's choices cannot lift this.
- **The look is one setting** (`settings.theme`) that reaches the compositor (frames), the shell
  (its own widgets) and the GTK theme (`shell/gtk-switch.sh`). Qt takes its palette from GTK.
- **Programs of other projects** (inspecteur, classeur, papier, cliche) fill the desktop's roles
  by default; each role is a command the session can change.

## File Index

No other file yet. Add the intent of a feature in `features/` when its design is decided.
