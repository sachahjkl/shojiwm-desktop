//! shojiwm-shell: the desktop shell of shojiwm-desktop on AntiWebUI. It replaces the Quickshell
//! shell (`shell/`), surface by surface: the taskbar, the Start menu and the launcher, the
//! notification area and its panels, the desktop and its gadgets, the dialogs, the Control
//! Panel and the lock screen, in the theme chosen in the Control Panel (Aero or Kernel).
//!
//! `shojiwm-shell` runs the shell; `shojiwm-shell msg <target> <function> [args]` sends it a
//! command (see `ipc.rs`).

#![recursion_limit = "256"]

mod account;
mod apps;
mod compositor;
mod control_panel;
mod desktop;
mod dialogs;
mod fuzzy;
mod greeter;
mod ipc;
mod lock;
mod paths;
mod power;
mod programs;
mod settings;
mod sound;
mod start_menu;
mod status;
mod sun;
mod taskbar;
mod theme;
mod theme_values;
mod ui;

use anti_webui::shell::prelude::*;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("msg") {
        if let Err(e) = ipc::send(&args[1..]) {
            eprintln!("shojiwm-shell: {e}");
            std::process::exit(1);
        }
        return;
    }
    if args.first().map(String::as_str) == Some("greeter") {
        if let Err(e) = greeter::run() {
            eprintln!("shojiwm-shell greeter: {e}");
            std::process::exit(1);
        }
        return;
    }
    let result = Shell::new()
        .namespace("shojiwm-shell")
        .app_id("shojiwm-shell")
        .run(|| {
            ipc::serve();
            compositor::share_settings();
            sun::install();
            apps::install();
            status::install();
            taskbar::install();
            dialogs::install();
            lock::install();
            control_panel::install();
            sound::install();
            for_each_output(|output| {
                desktop::open(output.clone());
                taskbar::open(output);
            });
            // After the bars: its click-catcher stands over them.
            start_menu::install();
            ipc::handle("appearance", |function, args| match function {
                "setDark" => {
                    settings::choose_dark(args.first().is_some_and(|a| a == "true"));
                    Ok(String::new())
                }
                "toggle" => {
                    settings::choose_dark(!settings::dark());
                    Ok(String::new())
                }
                "isDark" => Ok(settings::dark().to_string()),
                _ => Err(format!("no function {function}")),
            });
        });
    if let Err(e) = result {
        eprintln!("shojiwm-shell: {e}");
        std::process::exit(1);
    }
}
