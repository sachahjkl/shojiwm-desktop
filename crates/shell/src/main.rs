//! shojiwm-shell: the desktop shell of shojiwm-desktop on AntiWebUI. It replaces the Quickshell
//! shell (`shell/`), surface by surface: the taskbar, the Start menu and the launcher, the
//! notification area and its panels, the desktop and its gadgets, the dialogs, the Control
//! Panel and the lock screen, in the theme chosen in the Control Panel (Aero or Kernel).
//!
//! `shojiwm-shell` runs the shell; `shojiwm-shell msg <target> <function> [args]` sends it a
//! command (see `ipc.rs`); `shojiwm-shell pick-source` is the screen-cast portal's picker.

#![recursion_limit = "256"]

mod account;
mod apps;
mod background;
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
mod spawn;
mod start_menu;
mod status;
mod sun;
mod tab_preview;
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
    if args.first().map(String::as_str) == Some("pick-source") {
        std::process::exit(pick_source());
    }
    keep_crashes();
    if args.first().map(String::as_str) == Some("greeter") {
        if let Err(e) = greeter::run() {
            eprintln!("shojiwm-shell greeter: {e}");
            std::process::exit(1);
        }
        return;
    }
    // Read on the settings thread while the shell connects to the compositor.
    settings::preload();
    let result = Shell::new()
        .namespace("shojiwm-shell")
        .app_id("shojiwm-shell")
        .run(|| {
            ipc::serve();
            compositor::share_settings();
            sun::install();
            tab_preview::install();
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

/// `shojiwm-shell pick-source`: the screen-cast portal's picker (`SHOJIWM_PORTAL_PICKER`). It
/// reads the portal's request, one JSON object, on its input, asks the running shell to show
/// its "Share your screen" dialog, and prints the chosen source's id. The exit code is 1 when the
/// user cancels or the shell cannot ask; the portal then refuses the request.
fn pick_source() -> i32 {
    use std::io::Read;
    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        eprintln!("shojiwm-shell pick-source: {e}");
        return 1;
    }
    let request: serde_json::Value = match serde_json::from_str(&input) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("shojiwm-shell pick-source: {e}");
            return 1;
        }
    };
    // Compact JSON is one line: the socket takes one argument per line.
    let args = [
        "screenshare".to_owned(),
        "pick".to_owned(),
        request.to_string(),
    ];
    match ipc::ask(&args) {
        Ok(id) if !id.is_empty() => {
            println!("{id}");
            0
        }
        Ok(_) => 1,
        Err(e) => {
            if e.to_string() != "cancelled" {
                eprintln!("shojiwm-shell pick-source: {e}");
            }
            1
        }
    }
}

/// A crash of the shell is written to `$XDG_STATE_HOME/shojiwm-desktop/shell-crashes.log`, its
/// time, its message and its backtrace: the compositor starts the shell with its output on
/// /dev/null and starts it again at once, so nothing else says it stopped. The file is written as
/// the shell dies, never while it runs.
fn keep_crashes() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        use std::io::Write;
        let dir = paths::state_dir();
        let _ = std::fs::create_dir_all(&dir);
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("shell-crashes.log"))
        {
            let at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs());
            let thread = std::thread::current();
            let _ = writeln!(
                file,
                "--- {at} (unix time), thread {}\n{info}\n{}",
                thread.name().unwrap_or("?"),
                std::backtrace::Backtrace::force_capture()
            );
        }
        default(info);
    }));
}
