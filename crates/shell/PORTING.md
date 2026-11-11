# Porting the Quickshell shell to AntiWebUI

This file is the brief of the people (and agents) who port the QML shell (`shell/`) to the Rust
crate `crates/shell`. It holds the rules, the shared API and the contracts between modules.

## Rules

- Write code only. Do not run `cargo`, `nix` or any build: the assembler builds and fixes.
- Write only the files of your part. Do not edit `main.rs` or another part's files. If you need
  something from the core or from another part, write it in your final report.
- Read the QML files of your part fully and port both the look and the behaviour, in both
  themes (Aero and Kernel). The theme values come from `theme::theme()` and the colour tokens
  from `theme::scheme()`, read in reactive closures (`dyn_style`, `text(move || ..)`, `memo`,
  `fill(move || ..)`), so that a theme change applies at once. Kernel's own look (bar at the top,
  launcher plate, section titles, category colours) is in the QML too: port it.
- Verify every AntiWebUI API you call against its source:
  `/home/sacha/Projects/ref/AntiWebUI` (rev c54748e). Read `docs/design/` (37 for the Wayland
  shell: layers, outputs, services; 21 and 33 for widgets; 09 for popups) and the crates'
  source. `anti_webui::shell::prelude::*` has the shell API (`open_layer`, `LayerOptions`,
  `Output`, `for_each_output`, `toplevels()`, `audio()`, `network()`, `bluetooth()`,
  `notification_server()`, `tray()`, `media()`, `session()`, `apps()`, `system()`, `Process`,
  `lock_session`, `local_time`, `idle`, `capture`, `thumbnail`).
- A real shell on AntiWebUI, for idioms: `/home/sacha/Projects/ref/shoji-bar-4/src` (bar.rs,
  panels.rs, launcher.rs, quick_settings.rs, notify.rs, lock.rs, pam.rs, calendar.rs, dock.rs,
  ui.rs, settings.rs, preview.rs).
- English identifiers and doc comments. A doc comment says what the thing is and why, in short
  complete sentences, as in the core files. One concept, one name: keep the QML's names
  (snake_case) unless they are unclear.
- The simplest implementation that fully works. No compatibility shims, no speculative options.
- No new colours or sizes in the code when the theme has them. A colour the QML writes literally
  becomes a named `const` with a comment, or a field you report as missing from the theme.

## The core (read it; do not change it)

- `settings.rs`: `settings() -> Signal<Settings>` (every key of `settings.json`, snake_case;
  `change(|s| ..)` writes), `dark()`, `set_dark()`, `animations()`, `motion(ms) -> u64`,
  `home()`, `config_home()`.
- `theme.rs`: `theme() -> Theme`, `scheme() -> Scheme`, `is_kernel()`, `corner(r)`,
  `popup_radius()`, `menu_radius()`, `item_radius()`, `field_radius()`, `button_radius()`,
  `bar_at_top()`, `bar_above()`, `bar_below()`, `away_from_bar(Align) -> Placement`,
  `parse_hex()`. `theme_values.rs` has the fields.
- `compositor.rs`: `send(method, params)`, `request(method, params) -> Task<Option<Value>>`,
  `available()`. The methods the compositor answers are in `src/peek.tsx` (`peek`,
  `show-desktop`, `screens`, `display`, …): read them there.
- `ipc.rs`: `handle(target, |function, args| -> Result<String, String>)`. The targets and
  functions keep the QML `IpcHandler` names, since the compositor calls them
  (`src/index.tsx`).
- `programs.rs`: `program(key)`, `session_program(key)`, `run(command)`, `BROWSER`.
- `paths.rs`: `asset(name)` (the pictures of `shell/assets`), `shell_file(name)`,
  `state_dir()`, `runtime_dir()`.

## The UI kit (`crate::ui::*`)

Theme helpers:

- `dark_look() -> bool`: Kernel, or Aero with the system dark. Use it for every light or dark choice.
- `font()`, `heading_font()`, `clock_font() -> FontFamily`.
- `ink_asset(name) -> PathBuf`: `shell/assets/<name>` on dark, `ink-<name>` on light.
- `lighter(color, factor) -> Color` (Qt's `Qt.lighter`).

Gradients, for `Div::fill(move || ..)`:

- `vertical_gradient(&[(offset, Color)]) -> ShaderPaint`, and `horizontal_gradient(..)`. Up to 4 stops.

Frames:

- `popup_frame() -> Div`: the glass popup box. Add padding and size, and put it in `popup()`.
- `tooltip_frame() -> Div`.
- `shell_tooltip(target: Div, placement, label: impl Fn() -> String) -> Div`.

Menus:

- `MenuLine::item(label, run)`, `::separator()`, `::heading(label)`, `::submenu(label, lines)`.
  Modifiers: `.icon(name_or_path)`, `.checked(bool)`, `.radio(bool)`, `.enabled(bool)`,
  `.pin(pinned, toggle)`.
- `popup_menu(open: impl Fn() -> bool, on_close, options: MenuOptions, lines: impl Fn() -> Vec<MenuLine>) -> Show`.
  Add it as a child of the element the menu belongs to.
- `bar_menu_options(anchor: NodeRef) -> MenuOptions`.
- `with_context_menu(target: Div, lines) -> Div`.
- `RowLook { bold, heading, arrow, indent, places, pin }` and
  `menu_row(icon, label, look, selected: impl Fn() -> bool) -> Div`: a Start menu row.

Text:

- `label_text(label) -> Text`.
- `section_title(label) -> Div`, and `section_title_with(label, color, accent) -> Div`.
- `shadow_text(label) -> Div`.

Icons and scrolling:

- `shell_icon(name_or_path, size) -> Div`.
- `aero_scroll(content) -> Div`. Give it a height.

Controls:

- `cp_button(label, on_click)` and `cp_default_button(label, on_click)`.
- `cp_check(label, Binding<bool>)`.
- `cp_radio(label, Binding<T>, value)`.
- `cp_combo(labels, Binding<usize>)`.
- `cp_link(label, large, on_click)` and `cp_link_tinted(label, large, tint, on_click)`.

For a setting: `Binding::new(|| settings().with(|s| s.x), |v| change(|s| s.x = v))`.

The kit names its types `MenuLine` and `RowLook`, since the prelude already has `MenuEntry` and
`MenuItem`.

## The parts and their contracts

Each part owns a module directory under `src/`. The public functions below are the contracts
the other parts call: write them with exactly these signatures (more public functions are fine).
`output_name` is the connector's name (`Output::name`).

1. **taskbar** (`src/taskbar/`, `src/apps.rs`).
   - Sources: `Taskbar`, `TaskButton`, `StartOrb`, `KernelLauncher`, `TrayArea`, `TrayIcon`,
     `TrayOverflow`, `TrayMenu`, `ThumbnailPopup`, `JumpList`, `MediaControls`, `Apps.qml`,
     `DragState.qml`, and the taskbar part of `shell.qml` (the bar's layer, its side, auto-hide,
     the click-catcher).
   - `taskbar::open(output: Output)`.
   - IPC target `taskbar`.
   - `apps::window_groups()` and the rest of Apps.qml's model, for the other parts.
2. **status** (`src/status/`).
   - Sources: `VolumeIcon`/`VolumePopup`, `NetworkIcon`/`NetworkPopup`,
     `BluetoothIcon`/`BluetoothPopup`, `NotificationIcon`/`NotificationPanel`,
     `ActionCenterIcon`/`ActionCenterPanel`, `Clock`, `CalendarPopup`, `CalendarArrow`, and the
     services `Audio`, `Network`, `Maintenance`, `NotificationHistory`, `WorldClocks`.
   - `status::area(output: Output) -> Div`: the shell's own icons of the notification area, in
     the taskbar's order (action center, notifications, bluetooth, network, volume).
   - `status::clock(output: Output) -> Div`: the clock with its calendar.
   - IPC targets `volume`, `bluetooth`, `actioncenter`.
3. **start_menu** (`src/start_menu/`).
   - Sources: `StartMenu`, `PlaceRow`, `UserTile`, `PowerPart`, `PowerSplitButton`,
     `PowerMenu`, `StartMenuState`, `StartMenuIpc`, `Search`, `LauncherEntries`, `Launcher`, and
     the Start menu, dismiss and launcher layers of `shell.qml`.
   - `start_menu::install()`, `start_menu::toggle_on(output_name: &str)`,
     `start_menu::close()`, `start_menu::is_open_on(output_name: &str) -> bool`,
     `start_menu::open_launcher(output_name: &str)`.
   - IPC targets `startmenu`, `launcher`.
4. **desktop** (`src/desktop/`).
   - Sources: `Desktop`, `DesktopMenu`, `GadgetFrame`, `ClockGadget`, `CpuMeter`, `StickyNote`,
     `StickyNotes`, `SystemStats`, `Slideshow`, `Screens.qml`, and the wallpaper and desktop
     layers of `shell.qml`.
   - `desktop::open(output: Output)`.
   - `desktop::primary_output_name() -> String`.
5. **dialogs** (`src/dialogs/`).
   - Sources: `RunDialog`/`RunState`, `ShutdownDialog`/`ShutdownState`, `AdminMenu`,
     `AltTabTitles`, `ScreenSaver`, and their layers in `shell.qml`.
   - `dialogs::install()`, `dialogs::run_toggle_on(output_name: &str)`,
     `dialogs::shutdown_open_on(output_name: &str)`,
     `dialogs::admin_menu_toggle_on(output_name: &str)`.
   - IPC targets `run`, `shutdown`, `adminmenu`.
6. **lock** (`src/lock/`, `src/greeter/`).
   - Sources: `LockScreen`, `LockState`, `LogonButton`, `SessionPicker`, `greeter.qml`,
     `GreeterState`, and PAM (see shoji-bar-4's `pam.rs`).
   - `lock::install()`, `lock::lock()`.
   - IPC target `lock`.
   - `greeter::run() -> Result<(), String>`: the log-on screen, run as `shojiwm-shell greeter`
     under greetd.
7. **control_panel, window and first pages** (`src/control_panel/mod.rs`, `state.rs`,
   `home.rs`, `pages_a.rs`, `picture_chooser.rs`).
   - Sources: `ControlPanel.qml`, `ControlPanelState`, `PictureChooser`: the window (a normal
     toplevel, `open_window`), its navigation, side bar, search and home page, and the pages
     appearance, desktop background (with the slide show settings), lock screen, account
     picture, taskbar and Start menu, notification area icons.
   - `control_panel::open()`, `control_panel::open_page(page: &str)`.
   - IPC target `controlpanel`.
   - It calls the second part's pages by these names: `pages_b::display_page()`,
     `pages_b::devices_page()`, `pages_b::sound_page()`, `pages_b::default_programs_page()`,
     `pages_b::startup_page()`, `pages_b::date_time_page()`, `pages_b::ease_page()`,
     `pages_b::about_page()`. Each returns a `Div`.
8. **control_panel, second pages** (`src/control_panel/pages_b.rs`, and files under
   `src/control_panel/b/`).
   - Sources: the pages of `ControlPanel.qml` named above, and `DisplaySettings`,
     `DefaultApps`, `Startup`.
   - The page functions named in part 7. Use the page heading and note styles of part 7: they
     are `control_panel::heading(label)`, `control_panel::sub_heading(label)` and
     `control_panel::note(label)`, which part 7 writes.

## Report

End with:

- the files you wrote;
- the public functions;
- the IPC targets;
- what you need from the core or from other parts;
- the AntiWebUI APIs you were not sure of.
