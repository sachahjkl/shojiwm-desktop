//! The account the shell shows: its login name, its full name and its picture (Apps.qml's
//! `loginName`, `userName`; Settings.qml's `userPictureSource`). The Start menu, the Shut Down
//! dialog, the lock and log-on screens, the desktop and the Control Panel read them here.

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
        .or_else(|| home().and_then(|h| h.file_name().map(|n| n.to_string_lossy().into_owned())))
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
    let _ = lookup;
    name
}

/// The first readable file of a list.
fn first_readable(paths: impl IntoIterator<Item = PathBuf>) -> Option<PathBuf> {
    paths.into_iter().find(|p| std::fs::File::open(p).is_ok())
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

thread_local! {
    static USER_NAME: std::cell::Cell<Option<Signal<String>>> = const { std::cell::Cell::new(None) };
}

/// The full name of the user that runs the shell, else the login name. Reading it tracks it:
/// it is the login name until the password database answers.
pub fn user_name() -> String {
    USER_NAME.with(|cell| {
        cell.get()
            .unwrap_or_else(|| {
                let (name, _) = create_root(|| full_name(&login_name()));
                cell.set(Some(name));
                name
            })
            .get()
    })
}
