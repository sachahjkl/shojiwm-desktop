//! The Start menu and the launcher (QML `StartMenu`, `Launcher`, `StartMenuState`,
//! `StartMenuIpc` and their layers in `shell.qml`).
//!
//! One piece of state serves the whole shell: the menu has a layer on each screen, but only one
//! of them is open at a time. The launcher (Super+Shift+D) is the menu's search alone, near the
//! top of the focused screen. The menu and the launcher are never open together.
//!
//! The layers:
//!
//! - the click-catcher of each screen, under the menu and over everything else, which closes the
//!   menu. It stays mapped and empties its input region while the menu is closed, as the QML
//!   shell's did: unmapping two layers in one frame ended the shell's Wayland connection;
//! - the menu of each screen, on the bar's side, also mapped at all times and empty while closed;
//! - while the launcher is open: a catcher over its screen, and the launcher's panel.

mod launcher;
pub mod launcher_entries;
mod menu;
mod places;
pub mod power;
mod search;
mod user_tile;

pub use power::power_split_button;
pub use user_tile::{user_picture, user_tile};

use crate::settings::settings;
use crate::theme::{bar_at_top, theme};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use std::cell::Cell;

/// Which screen shows what, by connector name.
#[derive(Clone, Copy)]
struct State {
    /// The screen that shows the Start menu; "" for none.
    open_on: Signal<String>,
    launcher_open: Signal<bool>,
    /// The screen of the launcher (the focused one).
    launcher_screen: Signal<String>,
}

thread_local! {
    static STATE: Cell<Option<State>> = const { Cell::new(None) };
}

fn state() -> State {
    STATE.with(|cell| {
        cell.get().unwrap_or_else(|| {
            let (s, _) = create_root(|| State {
                open_on: signal(String::new()),
                launcher_open: signal(false),
                launcher_screen: signal(String::new()),
            });
            cell.set(Some(s));
            s
        })
    })
}

/// The screen named, or the main screen for "".
fn screen_or_main(output_name: &str) -> String {
    if output_name.is_empty() {
        crate::desktop::primary_output_name()
    } else {
        output_name.to_owned()
    }
}

/// Opens the Start menu on a screen, by name ("": the main screen). The launcher closes.
pub fn open_on(output_name: &str) {
    let s = state();
    s.launcher_open.set(false);
    s.open_on.set(screen_or_main(output_name));
}

/// Closes the Start menu, wherever it is open.
pub fn close() {
    state().open_on.set(String::new());
}

/// Opens the Start menu on a screen, by name ("": the main screen), or closes it there.
pub fn toggle_on(output_name: &str) {
    let name = screen_or_main(output_name);
    if untrack(|| is_open_on(&name)) {
        close();
    } else {
        open_on(&name);
    }
}

/// The Start menu is open on that screen. Reading it tracks it.
pub fn is_open_on(output_name: &str) -> bool {
    state().open_on.with(|o| o == output_name)
}

/// Opens the launcher on a screen, by name ("": the main screen). The Start menu closes.
pub fn open_launcher(output_name: &str) {
    let s = state();
    s.open_on.set(String::new());
    s.launcher_screen.set(screen_or_main(output_name));
    s.launcher_open.set(true);
}

pub fn close_launcher() {
    state().launcher_open.set(false);
}

pub fn toggle_launcher(output_name: &str) {
    if state().launcher_open.get_untracked() {
        close_launcher();
    } else {
        open_launcher(output_name);
    }
}

/// The height of a row of the menu, large or compact (the "large icons" setting).
fn row_height() -> f32 {
    let t = theme();
    if settings().with(|s| s.start_menu_large_icons) {
        t.menu_row_height
    } else {
        t.menu_row_height_small
    }
}

/// The height of the menu's glass, large or compact.
fn menu_height() -> f32 {
    let t = theme();
    if settings().with(|s| s.start_menu_large_icons) {
        t.menu_height
    } else {
        t.menu_height_small
    }
}

/// The IPC targets, the menu's layers on each screen and the launcher's. Call once, from the
/// root, after the taskbars have opened: the click-catcher must stand over the bar.
pub fn install() {
    let _ = state();
    ipc();
    for_each_output(|output| {
        catcher_layer(&output);
        menu_layer(output);
    });
    launcher_layers();
}

fn ipc() {
    crate::ipc::handle("startmenu", |function, args| {
        let screen = args.first().map(String::as_str).unwrap_or("");
        match function {
            "open" => open_on(""),
            "openOn" => open_on(screen),
            "close" | "closeOn" => close(),
            "toggle" => toggle_on(""),
            "toggleOn" => toggle_on(screen),
            _ => return Err(format!("no function {function}")),
        }
        Ok(String::new())
    });
    crate::ipc::handle("launcher", |function, args| {
        let screen = args.first().map(String::as_str).unwrap_or("");
        match function {
            "open" => open_launcher(""),
            "openOn" => open_launcher(screen),
            "close" => close_launcher(),
            "toggle" => toggle_launcher(""),
            "toggleOn" => toggle_launcher(screen),
            _ => return Err(format!("no function {function}")),
        }
        Ok(String::new())
    });
}

/// The surface the size of the screen that closes the menu on a press anywhere outside it. It
/// asks for no blur: there is nothing of it to see.
fn catcher_layer(output: &Output) {
    let layer = open_layer(
        LayerOptions::new()
            .layer(Layer::Top)
            .anchor(Anchor::ALL)
            .exclusive(Exclusive::Ignore)
            .keyboard(Keyboard::None)
            .input_region(InputRegion::Empty)
            .output(output)
            .namespace("no_blur"),
        || div().size_full().on_pointer_down(|_| close()),
    );
    effect(move || {
        let open = state().open_on.with(|o| !o.is_empty());
        layer.set_input_region(if open { InputRegion::Full } else { InputRegion::Empty });
    });
}

/// Where the menu's layer stands: on the bar's side, by the screen's left edge, past the bar.
fn menu_place() -> (Anchor, Margin) {
    let t = theme();
    let past_bar = (t.taskbar_height + t.menu_margin).round() as i32;
    let margin = Margin {
        top: if bar_at_top() { past_bar } else { 0 },
        bottom: if bar_at_top() { 0 } else { past_bar },
        left: t.menu_margin.round() as i32,
        right: 0,
    };
    let edge = if bar_at_top() { Anchor::TOP } else { Anchor::BOTTOM };
    (edge | Anchor::LEFT, margin)
}

/// The menu's size: room above the glass for the account picture, which stands over its top
/// edge.
fn menu_size() -> (i32, i32) {
    let t = theme();
    (
        t.menu_width.round() as i32,
        (menu_height() + t.menu_tile_overhang).round() as i32,
    )
}

/// The menu's layer on a screen: mapped at all times, empty and letting clicks through while
/// the menu is closed. The compositor blurs what it draws only, so a closed menu shows nothing.
fn menu_layer(output: Output) {
    let name = output.name.clone();
    let open = memo(move || is_open_on(&name));
    let (anchor, margin) = untrack(menu_place);
    let (width, height) = untrack(menu_size);
    let content_name = output.name.clone();
    let layer = open_layer(
        LayerOptions::new()
            .layer(Layer::Top)
            .anchor(anchor)
            .margin(margin)
            .size(width, height)
            .exclusive(Exclusive::Ignore)
            .keyboard(Keyboard::None)
            .input_region(InputRegion::Empty)
            .output(&output)
            .namespace("shojiwm-win7-start"),
        move || {
            div()
                .size_full()
                .child(show(move || open.get(), move || menu::start_menu(content_name.clone())))
        },
    );
    {
        let layer = layer.clone();
        effect(move || {
            let (anchor, margin) = menu_place();
            let (width, height) = menu_size();
            layer.set_anchor(anchor);
            layer.set_margin(margin);
            layer.set_size(LayerSize::Fixed(width), LayerSize::Fixed(height));
        });
    }
    effect(move || {
        if open.get() {
            layer.set_input_region(InputRegion::Full);
            layer.set_keyboard(Keyboard::OnDemand);
        } else {
            layer.set_input_region(InputRegion::Empty);
            layer.set_keyboard(Keyboard::None);
        }
    });
}

/// The launcher's two layers, while it is open: a catcher over its screen without glass, and
/// the panel, as tall as it ever gets so that it is not resized at each key.
fn launcher_layers() {
    effect(|| {
        let s = state();
        if !s.launcher_open.get() {
            return;
        }
        let wanted = s.launcher_screen.get();
        let all = outputs();
        let Some(output) = all
            .iter()
            .find(|o| o.name == wanted)
            .or_else(|| all.first())
            .cloned()
        else {
            return;
        };
        untrack(move || {
            open_layer(
                LayerOptions::new()
                    .layer(Layer::Overlay)
                    .anchor(Anchor::ALL)
                    .exclusive(Exclusive::Ignore)
                    .keyboard(Keyboard::None)
                    .output(&output)
                    .namespace("no_blur"),
                || div().size_full().on_pointer_down(|_| close_launcher()),
            );
            let screen_height = output.logical_size.height as f32;
            let top = (screen_height * 0.18).round() as i32;
            let (width, height) = launcher::window_size(&output);
            let content_output = output.clone();
            let layer = open_layer(
                LayerOptions::new()
                    .layer(Layer::Overlay)
                    .anchor(Anchor::TOP)
                    .margin(Margin { top, ..Default::default() })
                    .size(width, height)
                    .exclusive(Exclusive::Ignore)
                    .keyboard(Keyboard::Exclusive)
                    .output(&output)
                    .namespace("shojiwm-win7-launcher"),
                move || launcher::launcher(content_output),
            );
            effect(move || {
                let (width, height) = launcher::window_size(&output);
                layer.set_size(LayerSize::Fixed(width), LayerSize::Fixed(height));
            });
        });
    });
}
