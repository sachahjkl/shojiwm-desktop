//! What is on the desktop (QML `Desktop`, its model part): the Recycle Bin, the system icons, and
//! whatever is in the desktop folder (`XDG_DESKTOP_DIR`, `~/Desktop` by default), sorted as the
//! settings say. The folder and the Recycle Bin are watched (`gio monitor`), so a file added or
//! trashed elsewhere shows at once. What the desktop does to its files is here too: open, trash,
//! rename, new folder, new text document.
//!
//! The model is one for every screen: each desktop shows the part of it that is placed on it.

use crate::account::user_name;
use crate::background::off_thread_in;
use crate::paths::asset;
use crate::programs::{program, run, run_helper};
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
        matches!(
            self,
            Kind::Trash | Kind::Computer | Kind::UserFiles | Kind::ControlPanel
        )
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
    /// The name in lower case, what the icons are sorted by.
    pub sort_name: String,
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
            key: format!(
                "{}:",
                if kind == Kind::Trash {
                    "trash"
                } else {
                    kind.setting()
                }
            ),
            kind,
            path: path.to_owned(),
            name: name.to_owned(),
            sort_name: name.to_lowercase(),
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
        let file = |name: &str| {
            asset(&format!("icons/{name}"))
                .to_string_lossy()
                .into_owned()
        };
        match self.kind {
            Kind::Trash => file(if trash_full() {
                "recycle-bin-full.png"
            } else {
                "recycle-bin.png"
            }),
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
            &[
                "png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "tif", "tiff", "avif", "heic",
            ],
        ),
        (
            "file-audio",
            &["mp3", "flac", "ogg", "opus", "wav", "m4a", "aac"],
        ),
        ("file-video", &["mp4", "mkv", "webm", "avi", "mov", "m4v"]),
    ];
    let name = KINDS
        .iter()
        .find(|(_, list)| list.contains(&suffix))
        .map_or("file", |(kind, _)| kind);
    asset(&format!("icons/{name}.png"))
        .to_string_lossy()
        .into_owned()
}

// ----- the shared state ------------------------------------------------------------------------

struct Model {
    /// The root the model lives in, and the results of its tasks.
    owner: Owner,
    /// The desktop folder, once read from `user-dirs.dirs` (on a worker).
    folder: Signal<Option<PathBuf>>,
    /// The files of the folder, folders first, then by name (see [`scan`]).
    files: Signal<Vec<Item>>,
    /// Every icon in sort order: the system icons, then the files. One for every screen.
    items: Memo<Vec<Item>>,
    trash_full: Signal<bool>,
    /// A reading of the folder is waiting for it to settle.
    pending: Cell<Option<Timeout>>,
    /// A count of the Recycle Bin is waiting for it to settle.
    trash_pending: Cell<Option<Timeout>>,
    /// The last reading of the folder asked for: an older one that ends later is dropped.
    generation: Cell<u64>,
}

thread_local! {
    static MODEL: Cell<Option<&'static Model>> = const { Cell::new(None) };
}

fn model() -> &'static Model {
    MODEL.with(|cell| {
        cell.get().unwrap_or_else(|| {
            let (model, _) = create_root(|| {
                let files = signal(Vec::new());
                let sort = memo(|| settings().with(|s| s.desktop_sort.clone()));
                let system = memo(|| settings().with(|s| s.desktop_system_icons.clone()));
                let items = memo(move || {
                    let mut list = system_items(&system.get());
                    let start = list.len();
                    files.with(|f: &Vec<Item>| list.extend(f.iter().cloned()));
                    sort_files(&mut list[start..], &sort.get());
                    list
                });
                let model: &'static Model = Box::leak(Box::new(Model {
                    owner: Owner::current().expect("a root"),
                    folder: signal(None),
                    files,
                    items,
                    trash_full: signal(false),
                    pending: Cell::new(None),
                    trash_pending: Cell::new(None),
                    generation: Cell::new(0),
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

/// Finds the folder and makes it if it is not there (on a worker), then reads it and watches it
/// and the Recycle Bin. The watchers live in the model's root: they outlive the screen that
/// started them.
fn watch(model: &'static Model) {
    off_thread_in(
        Some(model.owner),
        || {
            let folder = read_folder();
            let _ = std::fs::create_dir_all(&folder);
            folder
        },
        move |folder| {
            let changes = Process::new(GIO).arg("monitor").arg(&folder).spawn();
            changes.on_line(move |_| rescan_later());
            model.folder.set(Some(folder));
            rescan_now();
            count_trash();
            let trash = Process::new(GIO).args(["monitor", "trash:///"]).spawn();
            trash.on_line(move |_| count_trash_later());
            // A shortcut takes the icon of its program when the programs are read: read the
            // folder again once they are.
            let loaded = memo(|| apps().loaded());
            let first = Cell::new(true);
            effect(move || {
                let now = loaded.get();
                if !first.replace(false) && now {
                    rescan_later();
                }
            });
        },
    );
}

/// Whether the Recycle Bin has anything in it, counted on a worker. `gio` counts what is in
/// every trash folder (`~/.local/share/Trash`, and a `.Trash-<uid>` at the top of each other
/// mount).
fn count_trash() {
    let model = model();
    off_thread_in(
        Some(model.owner),
        || {
            std::process::Command::new(GIO)
                .args(["info", "-a", "trash::item-count", "trash:///"])
                .stdin(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .output()
                .map(|out| trash_has_items(&String::from_utf8_lossy(&out.stdout)))
                .ok()
        },
        move |full| {
            if let Some(full) = full {
                model.trash_full.set_if_changed(full);
            }
        },
    );
}

/// The Recycle Bin changed: count it once it has settled (a trashing of many files is one count).
fn count_trash_later() {
    let model = model();
    if let Some(t) = model.trash_pending.take() {
        t.cancel();
    }
    let timer = model.owner.with(|| {
        set_timeout(SETTLE, move || {
            model.trash_pending.set(None);
            count_trash();
        })
    });
    model.trash_pending.set(Some(timer));
}

/// Whether `gio info -a trash::item-count` says there is something in the Recycle Bin.
fn trash_has_items(output: &str) -> bool {
    output.lines().any(|line| {
        line.split_once("trash::item-count:")
            .and_then(|(_, n)| n.trim().parse::<u64>().ok())
            .is_some_and(|n| n > 0)
    })
}

fn rescan_later() {
    let model = model();
    if let Some(t) = model.pending.take() {
        t.cancel();
    }
    let timer = model.owner.with(|| {
        set_timeout(SETTLE, move || {
            model.pending.set(None);
            rescan_now();
        })
    });
    model.pending.set(Some(timer));
}

/// Reads the folder on a worker. A reading asked for later wins over this one.
fn rescan_now() {
    let model = model();
    let Some(folder) = model.folder.get_untracked() else {
        return;
    };
    let generation = model.generation.get() + 1;
    model.generation.set(generation);
    off_thread_in(
        Some(model.owner),
        move || scan(&folder),
        move |files| {
            if model.generation.get() == generation {
                model.files.set_if_changed(files);
            }
        },
    );
}

/// Reads the folder again now (the menu's Refresh).
pub fn refresh() {
    rescan_now();
}

pub fn trash_full() -> bool {
    model().trash_full.get()
}

/// The icons of every screen, in sort order: the Recycle Bin, the system icons asked for, then
/// the folders and the files. Reading it tracks the folder and the settings that order it.
pub fn items() -> Vec<Item> {
    model().items.get()
}

/// The Recycle Bin and the system icons asked for, as Windows 7's Desktop Icon Settings lists
/// them, the Recycle Bin on top.
fn system_items(system: &[String]) -> Vec<Item> {
    let home = home()
        .map(|h| h.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut list = vec![Item::system(Kind::Trash, "Recycle Bin", "trash:///")];
    for kind in [Kind::Computer, Kind::UserFiles, Kind::ControlPanel] {
        if !system.iter().any(|s| s == kind.setting()) {
            continue;
        }
        let (name, path) = match kind {
            Kind::Computer => ("Computer".to_owned(), "/".to_owned()),
            Kind::UserFiles => (user_name(), home.clone()),
            _ => ("Control Panel".to_owned(), String::new()),
        };
        list.push(Item::system(kind, &name, &path));
    }
    list
}

/// Puts files that [`scan`] gave (folders first, then by name) in the order of the "Sort by"
/// setting. The sort is stable: what is equal stays in name order.
fn sort_files(files: &mut [Item], sort: &str) {
    use std::cmp::Ordering;
    let by: fn(&Item, &Item) -> Ordering = match sort {
        "size" => |a, b| b.size.cmp(&a.size),
        "type" => |a, b| a.suffix.cmp(&b.suffix),
        "date" => |a, b| match (a.modified, b.modified) {
            (Some(x), Some(y)) => y.cmp(&x),
            _ => Ordering::Equal,
        },
        _ => return,
    };
    files.sort_by(|a, b| b.is_dir().cmp(&a.is_dir()).then_with(|| by(a, b)));
}

/// Removes a system icon from the desktop (its menu's "Remove from the desktop").
pub fn remove_system_icon(kind: Kind) {
    change(|s| s.desktop_system_icons.retain(|k| k != kind.setting()));
}

// ----- the folder ------------------------------------------------------------------------------

/// The desktop folder, once it is known (soon after the shell starts).
pub fn folder() -> Option<PathBuf> {
    model().folder.get_untracked()
}

pub fn folder_string() -> Option<String> {
    folder().map(|f| f.to_string_lossy().into_owned())
}

/// Reads the desktop folder: `XDG_DESKTOP_DIR` of `user-dirs.dirs`, else `~/Desktop`. On a
/// worker only.
fn read_folder() -> PathBuf {
    let home = home().unwrap_or_else(|| PathBuf::from("/"));
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"));
    let text = std::fs::read_to_string(config.join("user-dirs.dirs")).ok();
    desktop_dir(text.as_deref(), &home)
}

/// The desktop folder that a `user-dirs.dirs` says, else `~/Desktop`.
fn desktop_dir(user_dirs: Option<&str>, home: &Path) -> PathBuf {
    user_dirs
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

/// What a shortcut's program is found by, from the programs as they are when the folder is read.
struct Program {
    /// The id without `.desktop`, in lower case.
    id: String,
    name: String,
    icon: Option<String>,
}

/// The files of the folder, hidden ones left out: folders first, then by name. On a worker
/// only: it reads the disk and the icon theme.
fn scan(folder: &Path) -> Vec<Item> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Vec::new();
    };
    // The programs, read once for every shortcut, and only if there is one.
    let mut programs: Option<Vec<Program>> = None;
    let mut files: Vec<Item> = entries
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
                let programs = programs.get_or_insert_with(read_programs);
                read_shortcut(&path, programs)
            } else {
                (String::new(), String::new())
            };
            Some(Item {
                key: path_text.clone(),
                kind,
                path: path_text,
                sort_name: name.to_lowercase(),
                name,
                suffix,
                size: meta.as_ref().map_or(0, |m| m.len()),
                modified: meta.as_ref().and_then(|m| m.modified().ok()),
                shortcut_name,
                shortcut_icon,
            })
        })
        .collect();
    sort_by_name(&mut files);
    files
}

/// Folders first, then by name without regard to case.
fn sort_by_name(files: &mut [Item]) {
    files.sort_by(|a, b| {
        b.is_dir()
            .cmp(&a.is_dir())
            .then_with(|| a.sort_name.cmp(&b.sort_name))
    });
}

fn read_programs() -> Vec<Program> {
    apps()
        .all()
        .into_iter()
        .map(|app| Program {
            id: app.id.trim_end_matches(".desktop").to_lowercase(),
            name: app.name.to_lowercase(),
            icon: app.icon,
        })
        .collect()
}

/// The `Name` of a shortcut and its icon: its own if the theme has it, else that of the
/// program it starts, else a generic one.
fn read_shortcut(path: &Path, programs: &[Program]) -> (String, String) {
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
    (name, shortcut_icon(&icon, &exec, programs))
}

fn shortcut_icon(icon: &str, exec: &str, programs: &[Program]) -> String {
    const PIXELS: u32 = 48;
    if icon.starts_with('/')
        || (!icon.is_empty() && anti_webui::shell::icon_path(icon, PIXELS).is_some())
    {
        return icon.to_owned();
    }
    let command = exec
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_matches(|c| c == '"' || c == '\'');
    let command = command.rsplit('/').next().unwrap_or("").to_lowercase();
    if !command.is_empty() {
        let found = programs
            .iter()
            .find(|p| p.id == command || p.name == command);
        if let Some(name) = found.and_then(|p| p.icon.clone()) {
            return name;
        }
    }
    asset("icons/run.png").to_string_lossy().into_owned()
}

// ----- doing things ----------------------------------------------------------------------------

/// A word for `sh`, quoted.
fn quote(word: &str) -> String {
    format!("'{}'", word.replace('\'', r"'\''"))
}

/// Runs `gio` with these words as a helper, apart from the shell: to trash and move files.
fn gio(args: &[&str]) {
    let line = std::iter::once(GIO.to_owned())
        .chain(args.iter().map(|a| quote(a)))
        .collect::<Vec<_>>()
        .join(" ");
    run_helper(&line);
}

/// Starts a program with `gio` (`launch` a shortcut, `open` a file): the program runs in a scope
/// of its own (`crate::spawn`).
fn gio_start(verb: &str, path: &str) {
    crate::spawn::start_program(&[GIO.to_owned(), verb.to_owned(), path.to_owned()], None);
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
        Kind::Shortcut => gio_start("launch", &item.path),
        Kind::File => gio_start("open", &item.path),
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

/// Renames a file, on a worker. A name that is empty, the same, or has a slash in it changes
/// nothing. Once it is renamed, `done` gets the new path, for the icon to keep its place under
/// its new name.
pub fn rename(item: &Item, name: &str, done: impl FnOnce(String) + 'static) {
    let name = name.trim();
    if name.is_empty() || name == item.name || name.contains('/') || item.kind.is_system() {
        return;
    }
    let from = item.path.clone();
    let to = format!("{}/{name}", parent_of(&item.path));
    off_thread_in(
        Some(model().owner),
        move || std::fs::rename(&from, &to).ok().map(|()| to),
        move |renamed| {
            if let Some(path) = renamed {
                done(path);
                rescan_later();
            }
        },
    );
}

/// The first "New folder", "New folder (2)" and so on that is free.
fn free_name(base: &str, extension: &str) -> String {
    let taken: Vec<String> = untrack(|| {
        model()
            .files
            .with(|f| f.iter().map(|i| i.name.clone()).collect())
    });
    let mut name = format!("{base}{extension}");
    let mut n = 2;
    while taken.contains(&name) {
        name = format!("{base} ({n}){extension}");
        n += 1;
    }
    name
}

/// Does `make` with a path free in the folder, named after `base`, on a worker, then reads the
/// folder again.
fn make_in_folder(base: &str, extension: &str, make: impl FnOnce(&Path) + Send + 'static) {
    let Some(folder) = folder() else { return };
    let path = folder.join(free_name(base, extension));
    off_thread_in(
        Some(model().owner),
        move || make(&path),
        |()| rescan_later(),
    );
}

pub fn new_folder() {
    make_in_folder("New folder", "", |path| {
        let _ = std::fs::create_dir(path);
    });
}

pub fn new_text_document() {
    make_in_folder("New Text Document", ".txt", |path| {
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path);
    });
}

/// The terminal, in the desktop folder.
pub fn open_terminal() {
    crate::spawn::start_command_line(&program("terminal"), folder().as_deref());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(name: &str, dir: bool, size: u64) -> Item {
        let mut item = Item::system(Kind::Trash, name, "");
        item.kind = if dir { Kind::Folder } else { Kind::File };
        item.key = name.to_owned();
        item.sort_name = name.to_lowercase();
        item.suffix = if dir {
            String::new()
        } else {
            name.rsplit_once('.').map_or("", |(_, s)| s).to_lowercase()
        };
        item.size = size;
        item
    }

    fn names(files: &[Item]) -> Vec<&str> {
        files.iter().map(|f| f.name.as_str()).collect()
    }

    #[test]
    fn folders_come_first_then_names_without_case() {
        let mut files = vec![
            file("b.txt", false, 1),
            file("Zed", true, 0),
            file("A.txt", false, 2),
            file("alpha", true, 0),
        ];
        sort_by_name(&mut files);
        assert_eq!(names(&files), ["alpha", "Zed", "A.txt", "b.txt"]);
    }

    #[test]
    fn a_sort_keeps_name_order_among_equals() {
        let mut files = vec![
            file("dir", true, 0),
            file("a.png", false, 5),
            file("B.txt", false, 9),
            file("c.png", false, 5),
        ];
        sort_by_name(&mut files);
        sort_files(&mut files, "size");
        assert_eq!(names(&files), ["dir", "B.txt", "a.png", "c.png"]);
        sort_by_name(&mut files);
        sort_files(&mut files, "type");
        assert_eq!(names(&files), ["dir", "a.png", "c.png", "B.txt"]);
        sort_by_name(&mut files);
        sort_files(&mut files, "name");
        assert_eq!(names(&files), ["dir", "a.png", "B.txt", "c.png"]);
    }

    #[test]
    fn the_desktop_folder_is_read_from_user_dirs() {
        let home = Path::new("/home/u");
        let text =
            "# comment\nXDG_DOWNLOAD_DIR=\"$HOME/Downloads\"\nXDG_DESKTOP_DIR=\"$HOME/Bureau\"\n";
        assert_eq!(desktop_dir(Some(text), home), Path::new("/home/u/Bureau"));
        assert_eq!(
            desktop_dir(Some("XDG_DESKTOP_DIR=\"/data/desk\""), home),
            Path::new("/data/desk")
        );
        assert_eq!(desktop_dir(None, home), Path::new("/home/u/Desktop"));
        assert_eq!(
            desktop_dir(Some("XDG_DESKTOP_DIR=\"\""), home),
            Path::new("/home/u/Desktop")
        );
    }

    #[test]
    fn the_trash_count_is_read_from_gio() {
        assert!(trash_has_items(
            "uri: trash:///\nattributes:\n  trash::item-count: 3\n"
        ));
        assert!(!trash_has_items("attributes:\n  trash::item-count: 0\n"));
        assert!(!trash_has_items(""));
    }
}
