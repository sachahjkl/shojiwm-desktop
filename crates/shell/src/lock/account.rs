//! The account the lock screen and the log-on screen show: its login name, its full name, its
//! picture, and the power actions their power button offers (Apps.qml's `loginName`,
//! `userName`, `powerActions`; Settings.qml's `userPictureSource`).

use crate::paths::asset;
use crate::settings::{home, settings};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use std::path::PathBuf;

/// The login name of the user that runs the shell: `$USER`, else the last part of `$HOME`.
pub fn login_name() -> String {
    std::env::var("USER")
        .ok()
        .filter(|u| !u.is_empty())
        .or_else(|| {
            home().and_then(|h| h.file_name().map(|n| n.to_string_lossy().into_owned()))
        })
        .unwrap_or_default()
}

/// The full name of `login` from the password database (`getent`, so NSS accounts count too):
/// the first part of its GECOS field. It is `login` until `getent` answers, and stays `login`
/// when the field is empty. Call it under an owner that lives as long as the name is shown.
pub fn full_name(login: &str) -> Signal<String> {
    let name = signal(login.to_owned());
    if login.is_empty() {
        return name;
    }
    let lookup = Process::new("getent").args(["passwd", login]).spawn();
    lookup.on_line(move |line| {
        // The GECOS field: the full name first, then comma-separated extras nobody fills in.
        let full = line
            .trim()
            .split(':')
            .nth(4)
            .and_then(|gecos| gecos.split(',').next())
            .map(str::trim)
            .unwrap_or_default();
        if !full.is_empty() {
            name.set(full.to_owned());
        }
    });
    // It answers once and ends; its line handler lives with the owner.
    std::mem::forget(lookup);
    name
}

/// The first readable file of a list.
fn first_readable(paths: impl IntoIterator<Item = PathBuf>) -> Option<PathBuf> {
    paths
        .into_iter()
        .find(|p| std::fs::File::open(p).is_ok())
}

/// AccountsService's picture of `login`, which a log-on screen can read (not `~/.face`, in a
/// private home).
pub fn accounts_service_picture(login: &str) -> Option<PathBuf> {
    if login.is_empty() {
        return None;
    }
    first_readable([PathBuf::from("/var/lib/AccountsService/icons").join(login)])
}

/// The account picture: the one chosen in the Control Panel, else the system's
/// (AccountsService's icon, `~/.face`, `~/.face.icon`), else the drawn one. Reading it tracks
/// the settings.
pub fn user_picture() -> PathBuf {
    let chosen = settings().with(|s| s.user_picture.clone());
    if !chosen.is_empty() {
        return PathBuf::from(chosen);
    }
    let login = login_name();
    let mut places = Vec::new();
    if !login.is_empty() {
        places.push(PathBuf::from("/var/lib/AccountsService/icons").join(&login));
    }
    if let Some(h) = home() {
        places.push(h.join(".face"));
        places.push(h.join(".face.icon"));
    }
    first_readable(places).unwrap_or_else(|| asset("user.svg"))
}

/// The ids of the power actions (Apps.qml's `powerActions`), in their order.
const POWER_IDS: [&str; 6] = ["logoff", "lock", "restart", "sleep", "hibernate", "shutdown"];

/// The command line of a power action, by id; `None` for "lock" (the shell's own lock screen)
/// and for an unknown id.
pub fn power_command(id: &str) -> Option<String> {
    let command = match id {
        "logoff" => match std::env::var("XDG_SESSION_ID") {
            Ok(session) if !session.is_empty() => format!("loginctl terminate-session {session}"),
            _ => format!("loginctl terminate-user {}", login_name()),
        },
        "restart" => "systemctl reboot".into(),
        "sleep" => "systemctl suspend".into(),
        "hibernate" => "systemctl hibernate".into(),
        "shutdown" => "systemctl poweroff".into(),
        _ => return None,
    };
    Some(command)
}

/// What the red power button of the lock and log-on screens runs: the Control Panel's choice
/// for the power button, except "lock" (it is locked already, or there is no session to lock),
/// which becomes "shutdown". Reading it tracks the settings.
pub fn power_button_action() -> String {
    let chosen = settings().with(|s| s.power_button_action.clone());
    if POWER_IDS.contains(&chosen.as_str()) && chosen != "lock" {
        chosen
    } else {
        "shutdown".into()
    }
}

/// Runs a power action by id. The screen is locked already, or there is no session at all, so
/// nothing locks first.
pub fn run_power(id: &str) {
    if let Some(command) = power_command(id) {
        crate::programs::run(&command);
    }
}
