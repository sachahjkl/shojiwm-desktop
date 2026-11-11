//! The menu of the Start orb, on a right click or Super+X (QML `AdminMenu`): the tools a power
//! user wants at hand, as Windows' Win+X menu has them. A terminal, a root one, the task manager,
//! the Control Panel, the files, the system's pages, the network and disk tools, the system log,
//! Search and Run, the power actions and the desktop.
//!
//! It hangs above the orb (below it when the bar is at the top), in a layer over the whole
//! screen that has the keyboard: Up and Down pick an entry, Return starts it, Escape or a press
//! outside closes it.

use super::{admin_menu_close, run_open_on};
use crate::power::run_power;
use crate::programs::{program, run};
use crate::theme::{Align, away_from_bar, bar_at_top, theme};
use crate::ui::{MenuLine, popup_menu};
use anti_webui::prelude::*;
use serde_json::json;

/// A word for a shell's command line, quoted.
fn quoted(word: &str) -> String {
    format!("'{}'", word.replace('\'', r"'\''"))
}

/// A command in the terminal: `-e` runs it in kitty, Ghostty, foot and Alacritty alike.
fn in_terminal(command: &str) {
    run(&format!("{} -e {command}", program("terminal")));
}

fn entries(output_name: &str) -> Vec<MenuLine> {
    let launcher_on = output_name.to_owned();
    let run_on = output_name.to_owned();
    let power = |label: &'static str, id: &'static str| MenuLine::item(label, move || run_power(id));
    vec![
        MenuLine::item("Terminal", || run(&program("terminal"))),
        MenuLine::item("Terminal (Administrator)", || in_terminal("sudo -i")),
        MenuLine::separator(),
        MenuLine::item("Task Manager", || run(&program("taskManager"))),
        MenuLine::item("Control Panel", crate::control_panel::open),
        MenuLine::item("File Explorer", || {
            let home = crate::settings::home().unwrap_or_default();
            run(&format!(
                "{} {}",
                program("fileManager"),
                quoted(&home.to_string_lossy())
            ));
        }),
        MenuLine::item("System", || crate::control_panel::open_page("about")),
        MenuLine::item("Devices and Printers", || crate::control_panel::open_page("devices")),
        MenuLine::item("Network Connections", || run("nm-connection-editor")),
        MenuLine::item("Disk Management", || run("gparted")),
        MenuLine::item("System Log", || in_terminal("journalctl -b -e")),
        MenuLine::separator(),
        MenuLine::item("Search", move || crate::start_menu::open_launcher(&launcher_on)),
        MenuLine::item("Run", move || run_open_on(&run_on)),
        MenuLine::separator(),
        power("Lock", "lock"),
        power("Log off", "logoff"),
        power("Sleep", "sleep"),
        power("Restart", "restart"),
        power("Shut down", "shutdown"),
        MenuLine::separator(),
        MenuLine::item("Desktop", || crate::compositor::send("show-desktop", json!({}))),
    ]
}

/// The layer's content on the screen `output_name`: the room of the orb, which the menu hangs
/// from, and a press anywhere else closes it.
pub fn view(output_name: String) -> Div {
    let orb = node_ref();
    let options = untrack(|| {
        MenuOptions::new()
            .anchor(orb)
            .placement(away_from_bar(Align::Start))
            .gap(theme().popup_gap as i32)
    });
    let menu = popup_menu(
        || true,
        admin_menu_close,
        options,
        move || entries(&output_name),
    );
    div()
        .size_full()
        .relative()
        .on_pointer_down(|_| admin_menu_close())
        .child(
            div()
                .absolute()
                .node_ref(orb)
                .dyn_style(|s| {
                    let t = theme();
                    let s = s.left(t.orb_left).w(t.orb_size).h(t.taskbar_height);
                    if bar_at_top() { s.top(0.0) } else { s.bottom(0.0) }
                })
                .child(menu),
        )
}
