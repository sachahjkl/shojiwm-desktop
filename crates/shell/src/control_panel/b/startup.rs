//! Startup Programs (QML `Startup` and the page of `ControlPanel.qml`): the XDG autostart
//! entries, which systemd's xdg-autostart generator starts in this session.
//!
//! An entry is a `.desktop` file in an autostart folder: the system's
//! (`$XDG_CONFIG_DIRS/autostart`, the first folder wins) or yours (`~/.config/autostart`). A
//! file of yours replaces the system's of the same name, and `Hidden=true` in it turns the
//! entry off. Changes apply from the next log-on.

use super::common::{
    control_row, cp_field, data_dirs, desktop_entry_keys, env_or, field_border, icon_or,
    muted_small, page_column, tinted_text,
};
use crate::control_panel::{heading, note};
use crate::theme::{item_radius, scheme, theme};
use crate::ui::{cp_button, cp_check, cp_link, font, shell_icon};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// How many installed programs the list to add one from shows.
const CANDIDATES: usize = 12;
const FALLBACK_ICON: &str = "application-x-executable";

/// An autostart entry.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Entry {
    /// The file name, as `foo.desktop`.
    id: String,
    name: String,
    exec: String,
    icon: String,
    enabled: bool,
    /// `OnlyShowIn` and `NotShowIn` let it start in this desktop.
    here: bool,
    /// "" when there is none.
    system_path: String,
    user_path: String,
}

/// `~/.config/autostart`.
fn user_dir() -> PathBuf {
    crate::settings::config_home().join("autostart")
}

/// The system's autostart folders, the first one winning.
fn system_dirs() -> Vec<PathBuf> {
    env_or("XDG_CONFIG_DIRS", || "/etc/xdg".into())
        .split(':')
        .filter(|d| !d.is_empty())
        .map(|d| Path::new(d).join("autostart"))
        .collect()
}

/// The `.desktop` files of a folder, by name.
fn desktop_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "desktop") && p.is_file())
        .collect();
    files.sort();
    files
}

/// The keys of an entry that matter here.
fn keys_of(path: &Path) -> HashMap<String, String> {
    const WANTED: &[&str] = &[
        "Name",
        "Comment",
        "Exec",
        "Icon",
        "Hidden",
        "OnlyShowIn",
        "NotShowIn",
        "X-GNOME-Autostart-enabled",
    ];
    let text = std::fs::read_to_string(path).unwrap_or_default();
    desktop_entry_keys(&text)
        .into_iter()
        .filter(|(k, _)| WANTED.contains(&k.as_str()))
        .collect()
}

/// Whether `OnlyShowIn` and `NotShowIn` let the entry start in this desktop.
fn shows_here(keys: &HashMap<String, String>) -> bool {
    let current = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    let desktops: Vec<&str> = current.split(':').collect();
    let names = |key: &str| -> Vec<String> {
        keys.get(key)
            .map(|v| {
                v.split(';')
                    .filter(|n| !n.is_empty())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default()
    };
    let listed = |names: &[String]| desktops.iter().any(|d| names.iter().any(|n| n == d));
    let only = names("OnlyShowIn");
    if !only.is_empty() && !listed(&only) {
        return false;
    }
    !listed(&names("NotShowIn"))
}

/// Every entry, by name.
fn read_entries() -> Vec<Entry> {
    // By file name: the system's file and yours; yours replaces the system's as a whole.
    let mut found: Vec<(String, String, String, HashMap<String, String>)> = Vec::new();
    for dir in system_dirs() {
        for path in desktop_files(&dir) {
            let id = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            if found.iter().any(|(seen, ..)| *seen == id) {
                continue;
            }
            let keys = keys_of(&path);
            found.push((id, path.display().to_string(), String::new(), keys));
        }
    }
    for path in desktop_files(&user_dir()) {
        let id = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let keys = keys_of(&path);
        let user_path = path.display().to_string();
        match found.iter_mut().find(|(seen, ..)| *seen == id) {
            Some(entry) => {
                entry.2 = user_path;
                entry.3 = keys;
            }
            None => found.push((id, String::new(), user_path, keys)),
        }
    }
    let mut list: Vec<Entry> = found
        .into_iter()
        .map(|(id, system_path, user_path, keys)| {
            let get = |k: &str| keys.get(k).cloned().unwrap_or_default();
            Entry {
                name: keys
                    .get("Name")
                    .cloned()
                    .unwrap_or_else(|| id.trim_end_matches(".desktop").to_owned()),
                exec: get("Exec"),
                icon: get("Icon"),
                enabled: get("Hidden") != "true" && get("X-GNOME-Autostart-enabled") != "false",
                here: shows_here(&keys),
                id,
                system_path,
                user_path,
            }
        })
        .collect();
    list.sort_by_key(|a| a.name.to_lowercase());
    list
}

/// Turns an entry on or off: your copy of it, without the keys that turn it off, and with
/// `Hidden=true` when it must not start.
fn write_enabled(entry: &Entry, on: bool) {
    let source = if entry.user_path.is_empty() {
        &entry.system_path
    } else {
        &entry.user_path
    };
    let Ok(text) = std::fs::read_to_string(source) else {
        return;
    };
    let mut out = String::new();
    for line in text.lines() {
        if line.starts_with("Hidden=") || line.starts_with("X-GNOME-Autostart-enabled=") {
            continue;
        }
        out.push_str(line);
        out.push('\n');
        if !on && line.trim_end() == "[Desktop Entry]" {
            out.push_str("Hidden=true\n");
        }
    }
    let dir = user_dir();
    let target = dir.join(&entry.id);
    let partial = dir.join(format!("{}.new", entry.id));
    if std::fs::create_dir_all(&dir).is_ok() && std::fs::write(&partial, out).is_ok() {
        let _ = std::fs::rename(&partial, &target);
    }
}

/// Starts a program at log-on: a copy of its `.desktop` file in your autostart folder,
/// writable by you.
fn write_added(file_id: &str) {
    let Some(source) = data_dirs()
        .into_iter()
        .map(|dir| dir.join("applications").join(file_id))
        .find(|path| path.is_file())
    else {
        return;
    };
    let dir = user_dir();
    let target = dir.join(file_id);
    if std::fs::create_dir_all(&dir).is_err() || std::fs::copy(&source, &target).is_err() {
        return;
    }
    if let Ok(meta) = std::fs::metadata(&target) {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = meta.permissions();
        permissions.set_mode(permissions.mode() | 0o200);
        let _ = std::fs::set_permissions(&target, permissions);
    }
}

/// The entries, read again after each change.
#[derive(Clone, Copy)]
struct Startup {
    entries: Signal<Vec<Entry>>,
}

impl Startup {
    fn refresh(self) {
        task(read_entries).then(move |list| self.entries.set(list));
    }

    /// Does `work` on a thread, then reads the folders again.
    fn change(self, work: impl FnOnce() + Send + 'static) {
        task(move || {
            work();
            read_entries()
        })
        .then(move |list| self.entries.set(list));
    }
}

pub fn page() -> Div {
    let startup = Startup {
        entries: signal(Vec::new()),
    };
    startup.refresh();
    let adding = signal(false);
    let user_dir = user_dir().display().to_string();
    page_column(10.0)
        .child(heading("Choose the programs that start when you log on"))
        .child(note(format!(
            "These are the XDG autostart entries of the system and your own, in {user_dir}. \
             A change applies the next time you log on."
        )))
        .child(div().h(4.0))
        .child(for_each(
            move || startup.entries.get(),
            |e| e.clone(),
            move |e| entry_row(startup, e),
        ))
        .child(show(
            move || startup.entries.with(Vec::is_empty),
            || note("No program starts when you log on."),
        ))
        .child(control_row(10.0).pt(8.0).child(cp_button(
            move || {
                let label = if adding.get() {
                    "Done"
                } else {
                    "Add a program\u{2026}"
                };
                label.to_owned()
            },
            move || adding.update(|a| *a = !*a),
        )))
        .child(show(move || adding.get(), move || add_list(startup)))
}

/// An entry: its check box, icon, name and command, whose it is, and a link to remove yours.
/// Dimmed when it does not start in this desktop.
fn entry_row(startup: Startup, entry: Entry) -> Div {
    let checked = {
        let enabled = entry.enabled;
        let entry = entry.clone();
        Binding::new(
            move || enabled,
            move |on: bool| {
                let entry = entry.clone();
                startup.change(move || write_enabled(&entry, on));
            },
        )
    };
    let name = if entry.here {
        entry.name.clone()
    } else {
        format!("{} (not in this desktop)", entry.name)
    };
    let words = div()
        .flex()
        .flex_col()
        .flex_none()
        .w(330.0)
        .overflow_clip()
        .child(text(name).whitespace_nowrap().dyn_style(|s| {
            s.font_family(font())
                .text_size(theme().font_size)
                .text_color(scheme().text)
        }))
        .child(muted_small(entry.exec.clone()).whitespace_nowrap());
    let whose = div()
        .flex_none()
        .w(70.0)
        .child(muted_small(if entry.system_path.is_empty() {
            "Yours"
        } else {
            "System"
        }));
    let mut row = control_row(12.0)
        .opacity(if entry.here { 1.0 } else { 0.55 })
        .child(cp_check("", checked))
        .child(shell_icon(icon_or(&entry.icon, FALLBACK_ICON), 24.0))
        .child(words)
        .child(whose);
    // An entry you added goes; a system entry you changed goes back to the system's.
    if !entry.user_path.is_empty() {
        let label = if entry.system_path.is_empty() {
            "Remove"
        } else {
            "Restore"
        };
        let path = entry.user_path.clone();
        row = row.child(cp_link(label, false, move || {
            let path = path.clone();
            startup.change(move || {
                let _ = std::fs::remove_file(path);
            });
        }));
    }
    row
}

/// The installed programs not yet in the list, found by name, to pick one that starts at
/// log-on.
fn add_list(startup: Startup) -> Div {
    let filter = signal(String::new());
    let field = cp_field(
        text_input().value(filter).placeholder("Find a program"),
        260.0,
        field_border,
    );
    let candidates = move || {
        let typed = filter.get().to_lowercase();
        startup.entries.with(|entries| {
            apps()
                .all()
                .into_iter()
                .filter(|a| entries.iter().all(|e| e.id != a.id))
                .filter(|a| typed.is_empty() || a.name.to_lowercase().contains(&typed))
                .take(CANDIDATES)
                .collect::<Vec<_>>()
        })
    };
    div()
        .flex()
        .flex_col()
        .gap(6.0)
        .w_full()
        .child(field)
        .child(for_each(
            candidates,
            |a| a.id.clone(),
            move |a| candidate_row(startup, a),
        ))
}

fn candidate_row(startup: Startup, app: Application) -> Div {
    let ui = Interaction::new();
    let id = app.id.clone();
    let icon = icon_or(app.icon.as_deref().unwrap_or(""), FALLBACK_ICON);
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(8.0)
        .w(360.0)
        .h(28.0)
        .pl(6.0)
        .border(1.0)
        .select_none()
        .interaction(ui)
        .on_click(move |_| {
            let id = id.clone();
            startup.change(move || write_added(&id));
        })
        .dyn_style(move |s| {
            let c = scheme();
            let s = s.rounded(item_radius());
            if ui.hovered() {
                s.bg(c.hover).border_color(c.selection_border)
            } else {
                s.bg(Color::TRANSPARENT).border_color(Color::TRANSPARENT)
            }
        })
        .child(shell_icon(icon, 20.0))
        .child(tinted_text(app.name, || scheme().text))
}
