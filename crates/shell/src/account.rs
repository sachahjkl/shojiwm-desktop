//! The account the shell shows: its login name, its full name and its picture (Apps.qml's
//! `loginName`, `userName`; Settings.qml's `userPictureSource`). The Start menu, the Shut Down
//! dialog, the lock and log-on screens, the desktop and the Control Panel read them here.
//!
//! The password database and the picture files are read on worker threads: the name and the
//! picture are signals that change once the answer comes.

use crate::paths::asset;
use crate::settings::{home, settings};
use anti_webui::prelude::*;
use std::cell::Cell;
use std::path::PathBuf;

/// The login name of the user that runs the shell: `$USER`, else the last part of `$HOME`.
pub fn login_name() -> String {
    std::env::var("USER")
        .ok()
        .filter(|u| !u.is_empty())
        .or_else(|| home().and_then(|h| h.file_name().map(|n| n.to_string_lossy().into_owned())))
        .unwrap_or_default()
}

/// The first part of the GECOS field of a line of `getent passwd`, if it is not empty.
fn gecos_name(line: &str) -> Option<String> {
    // The GECOS field: the full name first, then comma-separated extras nobody fills in.
    line.trim()
        .split(':')
        .nth(4)
        .and_then(|gecos| gecos.split(',').next())
        .map(str::trim)
        .filter(|full| !full.is_empty())
        .map(str::to_owned)
}

/// The full name of `login` from the password database (`getent`, so NSS accounts count too):
/// the first part of its GECOS field. It is `login` until `getent` answers, and stays `login`
/// when the field is empty. Call it under an owner that lives as long as the name is shown.
pub fn full_name(login: &str) -> Signal<String> {
    let name = signal(login.to_owned());
    if login.is_empty() {
        return name;
    }
    let args = vec!["passwd".into(), login.into()];
    task(move || {
        crate::status::output_lines("getent", &args)
            .first()
            .and_then(|line| gecos_name(line))
    })
    .then(move |full| {
        if let Some(full) = full {
            name.set(full);
        }
    });
    name
}

/// The first readable file of a list. It opens files: call it on a worker thread only.
fn first_readable(paths: impl IntoIterator<Item = PathBuf>) -> Option<PathBuf> {
    paths.into_iter().find(|p| std::fs::File::open(p).is_ok())
}

/// The file of AccountsService's picture of `login`.
fn accounts_service_file(login: &str) -> PathBuf {
    PathBuf::from("/var/lib/AccountsService/icons").join(login)
}

/// AccountsService's picture of `login`, which a log-on screen can read (not `~/.face`, in a
/// private home). `None` until a worker has found it readable, and when it is not. Call it
/// under an owner that lives as long as the picture is shown.
pub fn accounts_service_picture(login: &str) -> Signal<Option<PathBuf>> {
    let picture = signal(None);
    if login.is_empty() {
        return picture;
    }
    let file = accounts_service_file(login);
    task(move || first_readable([file])).then(move |found| picture.set(found));
    picture
}

/// The system's pictures of the account, in the order they are tried: AccountsService's icon,
/// `~/.face`, `~/.face.icon`.
fn system_pictures(login: &str, home: Option<PathBuf>) -> Vec<PathBuf> {
    let mut places = Vec::new();
    if !login.is_empty() {
        places.push(accounts_service_file(login));
    }
    if let Some(h) = home {
        places.push(h.join(".face"));
        places.push(h.join(".face.icon"));
    }
    places
}

thread_local! {
    static PICTURE: Cell<Option<Signal<PathBuf>>> = const { Cell::new(None) };
}

/// The account picture as a signal, made the first time. A worker finds the system's picture
/// again only when the picture chosen in the settings changes.
pub fn picture() -> Signal<PathBuf> {
    PICTURE.with(|cell| {
        cell.get().unwrap_or_else(|| {
            let (s, _) = untrack(|| create_root(watch_picture));
            cell.set(Some(s));
            s
        })
    })
}

/// The picture's signal, and the effect that resolves it each time the chosen one changes.
fn watch_picture() -> Signal<PathBuf> {
    let picture = signal(asset("user.svg"));
    let chosen = memo(|| settings().with(|s| s.user_picture.clone()));
    // The task belongs to the effect's run: a later choice re-runs the effect, which drops the
    // answer of a worker started for an earlier one.
    effect(move || {
        let chosen = chosen.get();
        if !chosen.is_empty() {
            picture.set(PathBuf::from(chosen));
            return;
        }
        let places = system_pictures(&login_name(), home());
        task(move || first_readable(places)).then(move |found| {
            picture.set(found.unwrap_or_else(|| asset("user.svg")));
        });
    });
    picture
}

/// The account picture: the one chosen in the Control Panel, else the system's
/// (AccountsService's icon, `~/.face`, `~/.face.icon`), else the drawn one. Reading it tracks
/// it; it reads no file.
pub fn user_picture() -> PathBuf {
    picture().get()
}

thread_local! {
    static USER_NAME: Cell<Option<Signal<String>>> = const { Cell::new(None) };
}

/// The full name of the user that runs the shell, else the login name. Reading it tracks it:
/// it is the login name until the password database answers.
pub fn user_name() -> String {
    USER_NAME.with(|cell| {
        cell.get()
            .unwrap_or_else(|| {
                let (name, _) = untrack(|| create_root(|| full_name(&login_name())));
                cell.set(Some(name));
                name
            })
            .get()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gecos_gives_the_first_part_of_the_fifth_field() {
        assert_eq!(
            gecos_name("ann:x:1000:100:Ann Smith,,,:/home/ann:/bin/sh\n").as_deref(),
            Some("Ann Smith")
        );
        assert_eq!(gecos_name("ann:x:1000:100::/home/ann:/bin/sh"), None);
        assert_eq!(gecos_name(""), None);
    }

    #[test]
    fn system_pictures_come_in_order() {
        let list = system_pictures("ann", Some(PathBuf::from("/home/ann")));
        assert_eq!(
            list,
            vec![
                PathBuf::from("/var/lib/AccountsService/icons/ann"),
                PathBuf::from("/home/ann/.face"),
                PathBuf::from("/home/ann/.face.icon"),
            ]
        );
        assert_eq!(system_pictures("", None), Vec::<PathBuf>::new());
    }
}
