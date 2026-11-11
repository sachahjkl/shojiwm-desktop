//! What is on the desktop (QML `Desktop`, its model part): the Recycle Bin, the system icons, and
//! whatever is in the desktop folder (`XDG_DESKTOP_DIR`, `~/Desktop` by default), sorted as the
//! settings say. The folder and the Recycle Bin are watched (`gio monitor`), so a file added or
//! trashed elsewhere shows at once. What the desktop does to its files is here too: open, trash,
//! rename, new folder, new text document.
//!
//! The model is one for every screen: each desktop shows the part of it that is placed on it.

use crate::paths::asset;
use crate::programs::{program, run};
use crate::settings::{change, home, settings};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// What the desktop runs to open, launch, trash and move files (`tools.js`).
const GIO: &str = "gio";

/// How long the folder is left to settle after a change before it is read again: a copy of many
/// files is one reading.
const SETTLE: Duration = Duration::from_millis(150);

/// The suffixes of pictures, which show themselves as their icon.
const PICTURES: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg"];

/// What an icon is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Trash,
    Computer,
    UserFiles,
    ControlPanel,
    Folder,
    /// A `.desktop` file: it launches its program and wears the shortcut arrow.
    Shortcut,
    File,
}

impl Kind {
    /// The system icons are no files: they are not renamed, trashed or dragged out.
    pub fn is_system(self) -> bool {
        matches!(self, Kind::Trash | Kind::Computer | Kind::UserFiles | Kind::ControlPanel)
    }

    /// The Settings name of a system icon (`desktop_system_icons`).
    fn setting(self) -> &'static str {
        match self {
            Kind::Computer => "computer",
            Kind::UserFiles => "userfiles",
            Kind::ControlPanel => "controlpanel",
            _ => "",
        }
    }
}

/// One icon of the desktop.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    /// The full path of a file; `trash:`, `computer:`, `userfiles:`, `controlpanel:` for the
    /// system icons. It is also the key of its place in `desktop_places`.
    pub key: String,
    pub kind: Kind,
    pub path: String,
    /// The file's name, with its extension.
    pub name: String,
    /// The extension in lower case, "" for a folder.
    pub suffix: String,
    pub size: u64,
    pub modified: Option<SystemTime>,
    /// A shortcut's `Name` and the icon to show for it, read from its `.desktop` file.
    pub shortcut_name: String,
    pub shortcut_icon: String,
}

impl Item {
    fn system(kind: Kind, name: &str, path: &str) -> Self {
        Self {
            key: format!("{}:", if kind == Kind::Trash { "trash" } else { kind.setting() }),
            kind,
            path: path.to_owned(),
            name: name.to_owned(),
            suffix: String::new(),
            size: 0,
            modified: None,
            shortcut_name: String::new(),
            shortcut_icon: String::new(),
        }
    }

    pub fn is_dir(&self) -> bool {
        self.kind == Kind::Folder
    }

    /// What shows under the icon: a shortcut's own name, else the file's.
    pub fn label(&self) -> String {
        if self.kind == Kind::Shortcut {
            if !self.shortcut_name.is_empty() {
                return self.shortcut_name.clone();
            }
            return strip_suffix_ignore_case(&self.name, ".desktop").to_owned();
        }
        self.name.clone()
    }

    /// The picture of the icon: a path, or a name of the icon theme. Reading it tracks whether
    /// the Recycle Bin is full.
    pub fn icon(&self) -> String {
        let file = |name: &str| asset(&format!("icons/{name}")).to_string_lossy().into_owned();
        match self.kind {
            Kind::Trash => {
                file(if trash_full() { "recycle-bin-full.png" } else { "recycle-bin.png" })
            }
            Kind::Folder => file("folder.png"),
            Kind::Computer => file("computer.png"),
            Kind::UserFiles => file("home.png"),
            Kind::ControlPanel => file("control-panel.png"),
            Kind::Shortcut => self.shortcut_icon.clone(),
            Kind::File if PICTURES.contains(&self.suffix.as_str()) => self.path.clone(),
            Kind::File => file_icon(&self.suffix),
        }
    }
}

fn strip_suffix_ignore_case<'a>(name: &'a str, suffix: &str) -> &'a str {
    let cut = name.len().saturating_sub(suffix.len());
    if name.is_char_boundary(cut) && name[cut..].eq_ignore_ascii_case(suffix) {
        &name[..cut]
    } else {
        name
    }
}

/// The icon of a file by its kind (Apps.qml `fileIcon`).
fn file_icon(suffix: &str) -> String {
    const KINDS: &[(&str, &[&str])] = &[
        (
            "file-image",
            &["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "tif", "tiff", "avif", "heic"],
        ),
        ("file-audio", &["mp3", "flac", "ogg", "opus", "wav", "m4a", "aac"]),
        ("file-video", &["mp4", "mkv", "webm", "avi", "mov", "m4v"]),
    ];
    let name = KINDS
        .iter()
        .find(|(_, list)| list.contains(&suffix))
        .map_or("file", |(kind, _)| kind);
    asset(&format!("icons/{name}.png")).to_string_lossy().into_owned()
}

// ----- the shared state ------------------------------------------------------------------------

struct Model {
    /// The files of the folder, unsorted.
    files: Signal<Vec<Item>>,
    trash_full: Signal<bool>,
    /// A reading of the folder is waiting for it to settle.
    pending: Cell<Option<Timeout>>,
}

thread_local! {
    static MODEL: Cell<Option<&'static Model>> = const { Cell::new(None) };
}

fn model() -> &'static Model {
    MODEL.with(|cell| {
        cell.get().unwrap_or_else(|| {
            let (model, _) = create_root(|| {
                let model: &'static Model = Box::leak(Box::new(Model {
                    files: signal(scan(&folder())),
                    trash_full: signal(false),
                    pending: Cell::new(None),
                }));
                model
            });
            cell.set(Some(model));
            watch(model);
            model
        })
    })
}

/// Starts reading the folder and the Recycle Bin, for as long as the shell runs. Call it from
/// the shell's root, before the first desktop opens: the watchers and their timers live there.
pub fn install() {
    model();
}

/// The watchers, in a root of their own: they outlive the screen that started them.
fn watch(model: &'static Model) {
    create_root(|| {
        let folder = folder();
        let _ = std::fs::create_dir_all(&folder);
        let changes = Process::new(GIO).arg("monitor").arg(&folder).spawn();
        changes.on_line(move |_| rescan_later());
        count_trash(model);
        let trash = Process::new(GIO).args(["monitor", "trash:///"]).spawn();
        trash.on_line(move |_| count_trash(model));
    });
}

/// Whether the Recycle Bin has anything in it. `gio` counts what is in every trash folder
/// (`~/.local/share/Trash`, and a `.Trash-<uid>` at the top of each other mount).
fn count_trash(model: &'static Model) {
    let count = Process::new(GIO)
        .args(["info", "-a", "trash::item-count", "trash:///"])
        .spawn();
    count.on_exit(move |_| {
        let full = count.lines().iter().any(|line| {
            line.split_once("trash::item-count:")
                .and_then(|(_, n)| n.trim().parse::<u64>().ok())
                .is_some_and(|n| n > 0)
        });
        model.trash_full.set_if_changed(full);
    });
}

fn rescan_later() {
    let model = model();
    if let Some(t) = model.pending.take() {
        t.cancel();
    }
    model.pending.set(Some(set_timeout(SETTLE, move || {
        model.pending.set(None);
        model.files.set_if_changed(scan(&folder()));
    })));
}

/// Reads the folder again now (the menu's Refresh).
pub fn refresh() {
    model().files.set(scan(&folder()));
}

pub fn trash_full() -> bool {
    model().trash_full.get()
}

/// The icons of every screen, in sort order: the Recycle Bin, the system icons asked for, then
/// the folders and the files. Reading it tracks the folder and the settings.
pub fn items() -> Vec<Item> {
    let mut files = model().files.get();
    let (sort, system) = settings().with(|s| (s.desktop_sort.clone(), s.desktop_system_icons.clone()));
    files.sort_by(|a, b| {
        use std::cmp::Ordering;
        let by_kind = b.is_dir().cmp(&a.is_dir());
        if by_kind != Ordering::Equal {
            return by_kind;
        }
        let by_sort = match sort.as_str() {
            "size" => b.size.cmp(&a.size),
            "type" => a.suffix.cmp(&b.suffix),
            "date" => match (a.modified, b.modified) {
                (Some(x), Some(y)) => y.cmp(&x),
                _ => Ordering::Equal,
            },
            _ => Ordering::Equal,
        };
        by_sort.then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    let home = home().map(|h| h.to_string_lossy().into_owned()).unwrap_or_default();
    // The system icons as Windows 7's Desktop Icon Settings lists them, the Recycle Bin on top.
    let mut list = vec![Item::system(Kind::Trash, "Recycle Bin", "trash:///")];
    for (kind, name, path) in [
        (Kind::Computer, "Computer".to_owned(), "/".to_owned()),
        (Kind::UserFiles, user_name(), home),
        (Kind::ControlPanel, "Control Panel".to_owned(), String::new()),
    ] {
        if system.iter().any(|s| s == kind.setting()) {
            list.push(Item::system(kind, &name, &path));
        }
    }
    list.extend(files);
    list
}

/// Removes a system icon from the desktop (its menu's "Remove from the desktop").
pub fn remove_system_icon(kind: Kind) {
    change(|s| s.desktop_system_icons.retain(|k| k != kind.setting()));
}

// ----- the folder ------------------------------------------------------------------------------

/// The desktop folder: `XDG_DESKTOP_DIR` of `user-dirs.dirs`, else `~/Desktop`.
pub fn folder() -> PathBuf {
    let home = home().unwrap_or_else(|| PathBuf::from("/"));
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"));
    std::fs::read_to_string(config.join("user-dirs.dirs"))
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                let value = line.trim().strip_prefix("XDG_DESKTOP_DIR")?.trim_start();
                let value = value.strip_prefix('=')?.trim().trim_matches('"');
                let value = match value.strip_prefix("$HOME") {
                    Some(rest) => format!("{}{rest}", home.to_string_lossy()),
                    None => value.to_owned(),
                };
                (!value.is_empty()).then(|| PathBuf::from(value))
            })
        })
        .unwrap_or_else(|| home.join("Desktop"))
}

pub fn folder_string() -> String {
    folder().to_string_lossy().into_owned()
}

/// The files of the folder, hidden ones left out.
fn scan(folder: &Path) -> Vec<Item> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                return None;
            }
            let path = entry.path();
            let meta = std::fs::metadata(&path).ok();
            let dir = meta.as_ref().is_some_and(|m| m.is_dir());
            let dot = name.rfind('.').filter(|&i| i > 0);
            let suffix = match dot {
                Some(i) if !dir => name[i + 1..].to_lowercase(),
                _ => String::new(),
            };
            let kind = if dir {
                Kind::Folder
            } else if suffix == "desktop" {
                Kind::Shortcut
            } else {
                Kind::File
            };
            let path_text = path.to_string_lossy().into_owned();
            let (shortcut_name, shortcut_icon) = if kind == Kind::Shortcut {
                read_shortcut(&path)
            } else {
                (String::new(), String::new())
            };
            Some(Item {
                key: path_text.clone(),
                kind,
                path: path_text,
                name,
                suffix,
                size: meta.as_ref().map_or(0, |m| m.len()),
                modified: meta.as_ref().and_then(|m| m.modified().ok()),
                shortcut_name,
                shortcut_icon,
            })
        })
        .collect()
}

/// The `Name` of a shortcut and its icon: its own if the theme has it, else that of the
/// program it starts, else a generic one.
fn read_shortcut(path: &Path) -> (String, String) {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let mut section = "";
    let (mut name, mut icon, mut exec) = (String::new(), String::new(), String::new());
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            section = line;
            continue;
        }
        if section != "[Desktop Entry]" {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim().to_owned();
        match key.trim() {
            "Name" => name = value,
            "Icon" => icon = value,
            "Exec" => exec = value,
            _ => {}
        }
    }
    (name, shortcut_icon(&icon, &exec))
}

fn shortcut_icon(icon: &str, exec: &str) -> String {
    const PIXELS: u32 = 48;
    if icon.starts_with('/') || (!icon.is_empty() && anti_webui::shell::icon_path(icon, PIXELS).is_some()) {
        return icon.to_owned();
    }
    let command = exec
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_matches(|c| c == '"' || c == '\'');
    let command = command.rsplit('/').next().unwrap_or("").to_lowercase();
    if !command.is_empty() {
        let found = apps().all().into_iter().find(|app| {
            app.id.trim_end_matches(".desktop").eq_ignore_ascii_case(&command)
                || app.name.eq_ignore_ascii_case(&command)
        });
        if let Some(name) = found.and_then(|app| app.icon) {
            return name;
        }
    }
    asset("icons/run.png").to_string_lossy().into_owned()
}

/// The user's full name from the password database, else the login name.
fn user_name() -> String {
    thread_local! {
        static NAME: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
    }
    NAME.with(|cell| {
        cell.borrow_mut()
            .get_or_insert_with(|| {
                let login = std::env::var("USER").ok().filter(|u| !u.is_empty()).unwrap_or_else(|| {
                    home()
                        .and_then(|h| h.file_name().map(|n| n.to_string_lossy().into_owned()))
                        .unwrap_or_default()
                });
                let full = std::process::Command::new("getent")
                    .args(["passwd", &login])
                    .output()
                    .ok()
                    .and_then(|out| {
                        let line = String::from_utf8_lossy(&out.stdout).trim().to_owned();
                        // The GECOS field: the full name first, then extras nobody fills in.
                        let gecos = line.split(':').nth(4)?.split(',').next()?.trim().to_owned();
                        (!gecos.is_empty()).then_some(gecos)
                    });
                full.unwrap_or(login)
            })
            .clone()
    })
}

// ----- doing things ----------------------------------------------------------------------------

/// A word for `sh`, quoted.
fn quote(word: &str) -> String {
    format!("'{}'", word.replace('\'', r"'\''"))
}

/// Runs `gio` with these words, apart from the shell.
pub fn gio(args: &[&str]) {
    let line = std::iter::once(GIO.to_owned())
        .chain(args.iter().map(|a| quote(a)))
        .collect::<Vec<_>>()
        .join(" ");
    run(&line);
}

/// Opens a folder in the file manager (Apps.qml `openPath`).
pub fn open_path(path: &str) {
    run(&format!("{} {}", program("fileManager"), quote(path)));
}

/// Opens an icon: a folder or a place in the file manager, a shortcut's program, a file in its
/// program.
pub fn open(item: &Item) {
    match item.kind {
        Kind::ControlPanel => crate::control_panel::open(),
        Kind::Trash | Kind::Folder | Kind::Computer | Kind::UserFiles => open_path(&item.path),
        Kind::Shortcut => gio(&["launch", &item.path]),
        Kind::File => gio(&["open", &item.path]),
    }
}

/// Moves files to the Recycle Bin; the system icons are left alone.
pub fn trash(items: &[Item]) {
    let paths: Vec<&str> = items
        .iter()
        .filter(|i| !i.kind.is_system())
        .map(|i| i.path.as_str())
        .collect();
    trash_paths(&paths);
}

pub fn trash_paths(paths: &[&str]) {
    if paths.is_empty() {
        return;
    }
    let mut args = vec!["trash"];
    args.extend_from_slice(paths);
    gio(&args);
}

pub fn empty_trash() {
    gio(&["trash", "--empty"]);
}

/// Moves (or copies) files into a folder.
pub fn move_into(paths: &[String], folder: &str, copy: bool) {
    let target = format!("{folder}/");
    let mut args = vec![if copy { "copy" } else { "move" }];
    args.extend(paths.iter().map(String::as_str));
    args.push(&target);
    gio(&args);
}

/// The folder a path is in.
pub fn parent_of(path: &str) -> String {
    match path.rfind('/') {
        Some(0) | None => "/".to_owned(),
        Some(i) => path[..i].to_owned(),
    }
}

/// Renames a file. A name that is empty, the same, or has a slash in it changes nothing.
/// Returns the new path, for the icon to keep its place under its new name.
pub fn rename(item: &Item, name: &str) -> Option<String> {
    let name = name.trim();
    if name.is_empty() || name == item.name || name.contains('/') || item.kind.is_system() {
        return None;
    }
    let path = format!("{}/{name}", parent_of(&item.path));
    std::fs::rename(&item.path, &path).ok()?;
    rescan_later();
    Some(path)
}

/// The first "New folder", "New folder (2)" and so on that is free.
fn free_name(base: &str, extension: &str) -> String {
    let taken: Vec<String> = untrack(|| model().files.with(|f| f.iter().map(|i| i.name.clone()).collect()));
    let mut name = format!("{base}{extension}");
    let mut n = 2;
    while taken.contains(&name) {
        name = format!("{base} ({n}){extension}");
        n += 1;
    }
    name
}

pub fn new_folder() {
    let _ = std::fs::create_dir(folder().join(free_name("New folder", "")));
    rescan_later();
}

pub fn new_text_document() {
    let path = folder().join(free_name("New Text Document", ".txt"));
    let _ = std::fs::OpenOptions::new().create(true).append(true).open(path);
    rescan_later();
}

/// The terminal, in the desktop folder.
pub fn open_terminal() {
    run(&format!("cd {} && {}", quote(&folder_string()), program("terminal")));
}
