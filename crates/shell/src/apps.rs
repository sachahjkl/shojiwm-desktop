//! What the taskbar and the Start menu know about programs and windows (QML `Apps.qml`).
//!
//! Windows come from `wlr_foreign_toplevel_management_v1` (AntiWebUI's `toplevels()`), so
//! nothing here is specific to ShojiWM. Programs come from the system's `.desktop` files
//! (AntiWebUI's `apps()`). The state lives once per UI thread, made the first time it is read:
//! every screen's bar reads the same lists.
//!
//! A program is named by its `.desktop` file name (`kitty.desktop`), as the settings keep it
//! (`start_pinned`, `taskbar_pinned`, `jump_pins`) and as `Application::id` gives it.

use crate::paths::{asset, state_dir};
use crate::settings::{Settings, change, home, settings};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use serde_json::{Map, Value};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

/// The icon of a program that has none.
pub const FALLBACK_ICON: &str = "application-x-executable";
/// How long a click on a pinned program's button after it was started is not a new start.
const STARTING: Duration = Duration::from_millis(3000);
/// How many programs the Start menu lists under the pinned ones.
pub const FREQUENT_LIMIT: usize = 7;
/// How many program ids the history of started programs keeps, more than the launcher shows.
const RECENT_PROGRAMS_KEPT: usize = 100;
/// How many recent files a jump list shows.
pub const RECENT_LIMIT: usize = 8;
/// How often `recently-used.xbel` and `user-dirs.dirs` are looked at for changes.
const WATCH: Duration = Duration::from_secs(2);

/// The MIME type of a program dragged in the shell (from the Start menu, the launcher, the
/// desktop): its `.desktop` file name. A drag of a program carries
/// `DragData::new(ProgramDrag(id)).data(PROGRAM_MIME, id)`, so that another surface of the shell
/// takes it.
pub const PROGRAM_MIME: &str = "application/x-shojiwm-desktop-program";

/// A program dragged in the shell: its `.desktop` file name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramDrag(pub String);

// ----- windows ---------------------------------------------------------------------------------

/// A button of the taskbar: a program pinned to it, with the windows of it that are open, or
/// windows of one program (or one window, when they are not combined).
#[derive(Clone, Debug, PartialEq)]
pub struct WindowGroup {
    /// The same for as long as the button is there: `entry:<id>` for a program, `appid:<id>`
    /// for windows of a program with no `.desktop` file, `window:<n>` for a window of its own,
    /// `controlpanel` for the shell's Control Panel.
    pub key: String,
    pub name: String,
    /// A name of the icon theme, or a file.
    pub icon: String,
    pub entry: Option<Application>,
    /// Kept on the taskbar while it has no window.
    pub pinned: bool,
    /// Its windows, oldest first. Empty for a pinned program that is not running.
    pub windows: Vec<Toplevel>,
}

/// A file of `recently-used.xbel`.
#[derive(Clone, Debug, PartialEq)]
pub struct RecentFile {
    pub path: String,
    pub name: String,
    /// When it was last visited or changed, as the file writes it (ISO 8601, so it sorts).
    pub visited: String,
    /// The names and the first word of the commands it was opened with, in lower case.
    pub apps: Vec<String>,
}

/// A file pinned to a jump list, or one of its recent files.
#[derive(Clone, Debug, PartialEq)]
pub struct JumpFile {
    pub name: String,
    pub path: String,
}

/// A folder of All Programs: its name and its programs, by name.
#[derive(Clone, Debug, PartialEq)]
pub struct Folder {
    pub name: String,
    pub entries: Vec<Application>,
}

/// The state that is not a setting: what was started and how often, the order the buttons were
/// dragged into, the recent files, the user's folders and name.
#[derive(Clone, Copy)]
struct Model {
    /// How often each program was started from the shell, by `.desktop` file name.
    launch_counts: Signal<HashMap<String, i64>>,
    /// The programs started from the shell, last first.
    recent_program_ids: Signal<Vec<String>>,
    /// The order the buttons were dragged into, as group keys, for this session.
    button_order: Signal<Vec<String>>,
    recent_files: Signal<Vec<RecentFile>>,
    /// The folders of xdg-user-dirs, by key (DOCUMENTS, PICTURES, ...).
    user_dirs: Signal<HashMap<String, String>>,
    /// The full name from the password database, else the login name.
    user_name: Signal<String>,
    groups: Memo<Vec<WindowGroup>>,
}

thread_local! {
    static MODEL: Cell<Option<Model>> = const { Cell::new(None) };
    /// When each pinned program was last started from its button.
    static STARTED: RefCell<HashMap<String, Instant>> = RefCell::new(HashMap::new());
    /// The keys of the `[Desktop Entry]` group of each `.desktop` file read, by file name.
    static DESKTOP_KEYS: RefCell<HashMap<String, HashMap<String, String>>> =
        RefCell::new(HashMap::new());
}

/// Makes the state now, outside of any surface: its timers (the watch of the recent files)
/// then belong to the shell, not to a surface that can close. Call once, from the root, before
/// anything reads the lists.
pub fn install() {
    model();
}

fn model() -> Model {
    MODEL.with(|cell| {
        cell.get().unwrap_or_else(|| {
            let (m, _) = create_root(|| {
                let counts = read_json(&counts_path())
                    .and_then(|v| v.as_object().cloned())
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|(k, v)| Some((k, v.as_i64()?)))
                    .collect();
                let recent = read_json(&recent_programs_path())
                    .and_then(|v| v.as_array().cloned())
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                let button_order = signal(Vec::<String>::new());
                let m = Model {
                    launch_counts: signal(counts),
                    recent_program_ids: signal(recent),
                    button_order,
                    recent_files: signal(Vec::new()),
                    user_dirs: signal(HashMap::new()),
                    user_name: signal(login_name()),
                    groups: memo(move || {
                        let open = toplevels().all();
                        let (combine, pins) =
                            settings().with(|s| (s.combine_windows, s.taskbar_pinned.clone()));
                        button_order.with(|order| build_groups(&open, &pins, combine, order))
                    }),
                };
                follow_file(recent_xbel_path(), move |text| {
                    m.recent_files
                        .set(text.map(parse_recent).unwrap_or_default());
                });
                follow_file(user_dirs_path(), move |text| {
                    m.user_dirs
                        .set(text.map(parse_user_dirs).unwrap_or_default());
                });
                read_user_name(m.user_name);
                m
            });
            cell.set(Some(m));
            m
        })
    })
}

/// The buttons of the taskbar, left to right: the pinned programs first, in the order of
/// `taskbar_pinned`, then the others in the order their first window opened, as the buttons
/// were dragged into. A pinned program with no window open has no `windows`. Reading it tracks
/// it.
pub fn window_groups() -> Vec<WindowGroup> {
    model().groups.get()
}

/// The shell's own window: the Control Panel.
fn is_control_panel(t: &Toplevel) -> bool {
    t.title == "Control Panel" && t.app_id.to_lowercase().contains("shojiwm-shell")
}

/// The key of the program a window belongs to, whether windows are combined or not.
fn program_key(t: &Toplevel) -> String {
    if is_control_panel(t) {
        return "controlpanel".into();
    }
    match entry_for(&t.app_id) {
        Some(entry) => format!("entry:{}", entry.id),
        None if t.app_id.is_empty() => "appid:<none>".into(),
        None => format!("appid:{}", t.app_id.to_lowercase()),
    }
}

fn build_groups(
    open: &[Toplevel],
    pins: &[String],
    combine: bool,
    order: &[String],
) -> Vec<WindowGroup> {
    let mut list: Vec<WindowGroup> = Vec::new();
    // The group each program's next window goes into, or after, by program key.
    let mut latest: HashMap<String, String> = HashMap::new();
    for entry in entries_of(pins) {
        let key = format!("entry:{}", entry.id);
        if latest.contains_key(&key) {
            continue;
        }
        latest.insert(key.clone(), key.clone());
        list.push(WindowGroup {
            key,
            name: entry.name.clone(),
            icon: entry_icon(Some(&entry)),
            entry: Some(entry),
            pinned: true,
            windows: Vec::new(),
        });
    }
    for t in open {
        let program = program_key(t);
        let at = latest
            .get(&program)
            .and_then(|key| list.iter().position(|g| &g.key == key));
        // Not combined: the first window takes the pinned button's place, each later one gets
        // its own button just after the last one of the same program.
        if let Some(i) = at
            && !combine
            && !list[i].windows.is_empty()
        {
            let own = WindowGroup {
                key: format!("window:{}", t.id()),
                windows: vec![t.clone()],
                pinned: false,
                ..list[i].clone()
            };
            latest.insert(program, own.key.clone());
            list.insert(i + 1, own);
            continue;
        }
        let i = match at {
            Some(i) => i,
            None => {
                let ours = is_control_panel(t);
                let entry = if ours { None } else { entry_for(&t.app_id) };
                let key = if combine {
                    program.clone()
                } else {
                    format!("window:{}", t.id())
                };
                let name = match &entry {
                    _ if ours => "Control Panel".to_owned(),
                    Some(e) if !e.name.is_empty() => e.name.clone(),
                    _ => title_of(t),
                };
                let icon = if ours {
                    asset("icons/control-panel.png")
                        .to_string_lossy()
                        .into_owned()
                } else {
                    icon_for(&t.app_id)
                };
                latest.insert(program, key.clone());
                list.push(WindowGroup {
                    key,
                    name,
                    icon,
                    entry,
                    pinned: false,
                    windows: Vec::new(),
                });
                list.len() - 1
            }
        };
        list[i].windows.push(t.clone());
    }
    let mut ranked: Vec<(usize, WindowGroup)> = list
        .into_iter()
        .enumerate()
        .map(|(natural, g)| {
            let rank = order
                .iter()
                .position(|k| *k == g.key)
                .unwrap_or(order.len() + natural);
            (rank, g)
        })
        .collect();
    ranked.sort_by_key(|(rank, _)| *rank);
    ranked.into_iter().map(|(_, g)| g).collect()
}

/// Puts the button of `key` before the button now at `index`, or last for the count of
/// buttons. The pinned programs keep that order in the settings.
pub fn move_button(key: &str, index: usize) {
    let m = model();
    let groups = untrack(|| m.groups.get());
    let mut keys: Vec<String> = groups.iter().map(|g| g.key.clone()).collect();
    let Some(from) = keys.iter().position(|k| k == key) else {
        return;
    };
    let moved = keys.remove(from);
    let to = if from < index { index - 1 } else { index };
    keys.insert(to.min(keys.len()), moved);
    let pins: Vec<String> = keys
        .iter()
        .filter_map(|k| groups.iter().find(|g| &g.key == k))
        .filter(|g| g.pinned)
        .filter_map(|g| g.entry.as_ref().map(|e| e.id.clone()))
        .collect();
    m.button_order.set(keys);
    change(|s| s.taskbar_pinned = pins);
}

/// Pins a program dropped on the taskbar, before the button now at `index`.
pub fn pin_dropped_at(id: &str, index: usize) {
    let Some(entry) = entry_by_id(id) else {
        return;
    };
    let m = model();
    let key = format!("entry:{}", entry.id);
    let mut keys: Vec<String> = untrack(|| m.groups.get())
        .iter()
        .map(|g| g.key.clone())
        .filter(|k| *k != key)
        .collect();
    keys.insert(index.min(keys.len()), key);
    m.button_order.set(keys);
    if !is_taskbar_pinned(&entry) {
        set_taskbar_pinned(&entry, true);
    }
}

/// The `.desktop` file name a drop of files carries, if one of them is a `.desktop` file.
pub fn dropped_program_id(files: &[PathBuf]) -> Option<String> {
    files
        .iter()
        .filter_map(|f| f.file_name()?.to_str().map(String::from))
        .find(|name| name.ends_with(".desktop"))
}

/// A window's title, else its app id, else "Window".
pub fn title_of(t: &Toplevel) -> String {
    if !t.title.is_empty() {
        t.title.clone()
    } else if !t.app_id.is_empty() {
        t.app_id.clone()
    } else {
        "Window".into()
    }
}

/// `true` while a window of the group has the keyboard and is not minimized.
pub fn group_is_current(group: &WindowGroup) -> bool {
    group.windows.iter().any(|t| t.activated && !t.minimized)
}

/// A left click on a button. No window: start the program, unless it was started from there
/// moments ago. One window: show it, or put it away when it is the one in front. (With several
/// windows the taskbar shows the thumbnails instead.)
pub fn activate_group(group: &WindowGroup) {
    if let Some(first) = group.windows.first() {
        toggle_window(first);
        return;
    }
    let Some(entry) = &group.entry else {
        return;
    };
    let now = Instant::now();
    let again = STARTED.with(|s| {
        let mut s = s.borrow_mut();
        let recent = s
            .get(&entry.id)
            .is_some_and(|at| now.duration_since(*at) < STARTING);
        if !recent {
            s.insert(entry.id.clone(), now);
        }
        recent
    });
    if !again {
        launch(entry);
    }
}

/// Super+1 to Super+9, as Windows 7: the button `number` (from 1). Not running: start it. One
/// window: show it, or put it away when it is in front. Several: the next one each time.
/// `another` starts a new instance instead (Super+Shift+N).
pub fn activate_button(number: usize, another: bool) {
    let groups = untrack(|| model().groups.get());
    let Some(group) = number.checked_sub(1).and_then(|i| groups.get(i)) else {
        return;
    };
    if another {
        if let Some(entry) = &group.entry {
            launch(entry);
        }
        return;
    }
    match group.windows.len() {
        0 => activate_group(group),
        1 => toggle_window(&group.windows[0]),
        n => {
            let active = group.windows.iter().position(|t| t.activated);
            let next = &group.windows[active.map_or(0, |i| (i + 1) % n)];
            let _ = next.set_minimized(false);
            let _ = next.activate();
        }
    }
}

/// Shows the window, or puts it away when it is the one in front.
pub fn toggle_window(t: &Toplevel) {
    if t.activated && !t.minimized {
        let _ = t.set_minimized(true);
        return;
    }
    let _ = t.set_minimized(false);
    let _ = t.activate();
}

/// Shows the desktop without the compositor: puts every window away.
pub fn minimize_all() {
    for t in toplevels().all() {
        let _ = t.set_minimized(true);
    }
}

// ----- programs --------------------------------------------------------------------------------

/// Every program with a `.desktop` file that asks to be listed, by name. Reading it tracks it.
pub fn applications() -> Vec<Application> {
    apps().all()
}

/// The program of a `.desktop` file name such as `kitty.desktop` (the extension may be left
/// out).
pub fn entry_by_id(id: &str) -> Option<Application> {
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    let file = if id.ends_with(".desktop") {
        id.to_owned()
    } else {
        format!("{id}.desktop")
    };
    apps().get(&file)
}

/// The programs of a list of file names, in order, leaving out those not installed.
pub fn entries_of(ids: &[String]) -> Vec<Application> {
    ids.iter().filter_map(|id| entry_by_id(id)).collect()
}

/// The `.desktop` file name without its extension, in lower case.
fn stem(id: &str) -> String {
    id.trim_end_matches(".desktop").to_lowercase()
}

/// The program a window's app id belongs to: its `.desktop` file by name, else by the last
/// part of a reverse-DNS name, by `StartupWMClass`, or by the program's name.
pub fn entry_for(app_id: &str) -> Option<Application> {
    if app_id.is_empty() {
        return None;
    }
    if let Some(entry) = entry_by_id(app_id) {
        return Some(entry);
    }
    let wanted = app_id.to_lowercase();
    let short = wanted.rsplit('.').next().unwrap_or(&wanted).to_owned();
    let all = apps().all();
    let last_part = |id: &str| {
        let s = stem(id);
        s.rsplit('.').next().unwrap_or(&s).to_owned()
    };
    all.iter()
        .find(|a| stem(&a.id) == wanted)
        .or_else(|| all.iter().find(|a| last_part(&a.id) == wanted))
        .or_else(|| {
            all.iter().find(|a| {
                desktop_key(&a.id, "StartupWMClass").is_some_and(|c| c.to_lowercase() == wanted)
            })
        })
        .or_else(|| {
            all.iter()
                .find(|a| stem(&a.id) == short || last_part(&a.id) == short)
        })
        .or_else(|| all.iter().find(|a| a.name.to_lowercase() == wanted))
        .cloned()
}

/// The icon of a window's program: its `.desktop` file's, else the app id's in the icon
/// theme, else the generic one.
pub fn icon_for(app_id: &str) -> String {
    if let Some(entry) = entry_for(app_id) {
        return entry_icon(Some(&entry));
    }
    let lower = app_id.to_lowercase();
    if !lower.is_empty() && anti_webui::shell::icon_path(&lower, 32).is_some() {
        return lower;
    }
    FALLBACK_ICON.into()
}

/// The icon of a program, else the generic one.
pub fn entry_icon(entry: Option<&Application>) -> String {
    entry
        .and_then(|e| e.icon.clone())
        .filter(|i| !i.is_empty())
        .unwrap_or_else(|| FALLBACK_ICON.into())
}

/// The programs the Start menu pins, in order.
pub fn pinned() -> Vec<Application> {
    entries_of(&settings().with(|s| s.start_pinned.clone()))
}

pub fn is_pinned(entry: &Application) -> bool {
    settings().with(|s| s.start_pinned.contains(&entry.id))
}

pub fn is_taskbar_pinned(entry: &Application) -> bool {
    settings().with(|s| s.taskbar_pinned.contains(&entry.id))
}

pub fn set_pinned(entry: &Application, on: bool) {
    change(|s| s.start_pinned = Settings::with_member(&s.start_pinned, &entry.id, on));
}

pub fn set_taskbar_pinned(entry: &Application, on: bool) {
    change(|s| s.taskbar_pinned = Settings::with_member(&s.taskbar_pinned, &entry.id, on));
}

/// Starts a program, and counts it for the Start menu's lists.
pub fn launch(entry: &Application) {
    let name = entry.id.trim_end_matches(".desktop");
    let _ = entry.launch_in(crate::spawn::scope_prefix(name));
    count_launch(&entry.id);
}

/// Runs what was typed in the Start menu's search box, as the Run dialog does: through the
/// shell, in the home folder, so that arguments, pipes and `~` work. `false` for nothing typed.
pub fn run_typed(text: &str) -> bool {
    let command = text.trim();
    if command.is_empty() {
        return false;
    }
    crate::spawn::start_command_line(command, Some(&home_dir()));
    true
}

fn count_launch(id: &str) {
    let m = model();
    m.launch_counts
        .update(|c| *c.entry(id.to_owned()).or_insert(0) += 1);
    let counts: Map<String, Value> = m
        .launch_counts
        .get_untracked()
        .into_iter()
        .map(|(k, v)| (k, Value::from(v)))
        .collect();
    write_json(&counts_path(), &Value::Object(counts));
    m.recent_program_ids.update(|r| {
        r.retain(|other| other != id);
        r.insert(0, id.to_owned());
        r.truncate(RECENT_PROGRAMS_KEPT);
    });
    let recent = m.recent_program_ids.get_untracked();
    write_json(&recent_programs_path(), &Value::from(recent));
}

/// The programs started from the shell, last first, that are still installed.
pub fn recent_programs() -> Vec<Application> {
    entries_of(&model().recent_program_ids.get())
}

/// The programs started most often, most first, leaving out the pinned ones and those no
/// longer installed.
pub fn frequent() -> Vec<Application> {
    let counts = model().launch_counts.get();
    let pins = settings().with(|s| s.start_pinned.clone());
    let mut ids: Vec<(&String, &i64)> =
        counts.iter().filter(|(id, _)| !pins.contains(id)).collect();
    ids.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    ids.into_iter()
        .filter_map(|(id, _)| entry_by_id(id))
        .take(FREQUENT_LIMIT)
        .collect()
}

/// The main categories of the freedesktop menu specification, under the names Windows gave its
/// folders where it had one.
const FOLDER_NAMES: &[(&str, &str)] = &[
    ("AudioVideo", "Multimedia"),
    ("Audio", "Multimedia"),
    ("Video", "Multimedia"),
    ("Development", "Development"),
    ("Education", "Education"),
    ("Game", "Games"),
    ("Graphics", "Graphics"),
    ("Network", "Internet"),
    ("Office", "Office"),
    ("Science", "Science"),
    ("Settings", "Settings"),
    ("System", "System Tools"),
    ("Utility", "Accessories"),
];

/// The folder of All Programs a program goes in.
pub fn folder_of(entry: &Application) -> &'static str {
    entry
        .categories
        .iter()
        .find_map(|c| {
            FOLDER_NAMES
                .iter()
                .find(|(k, _)| *k == c.as_str())
                .map(|(_, n)| *n)
        })
        .unwrap_or("Other")
}

/// Every listed program, in folders, by name.
pub fn folders() -> Vec<Folder> {
    let mut list: Vec<Folder> = Vec::new();
    for entry in applications() {
        let name = folder_of(&entry);
        match list.iter_mut().find(|f| f.name == name) {
            Some(folder) => folder.entries.push(entry),
            None => list.push(Folder {
                name: name.into(),
                entries: vec![entry],
            }),
        }
    }
    list.sort_by(|a, b| a.name.cmp(&b.name));
    list
}

// ----- the user --------------------------------------------------------------------------------

pub fn home_dir() -> PathBuf {
    home().unwrap_or_else(|| PathBuf::from("/"))
}

pub use crate::account::{login_name, user_name};

/// Reads the full name from the password database: the GECOS field's first part.
fn read_user_name(name: Signal<String>) {
    let login = login_name();
    if login.is_empty() {
        return;
    }
    let lookup = Process::new("getent")
        .args(["passwd", login.as_str()])
        .spawn();
    lookup.on_line(move |line| {
        let full = line
            .split(':')
            .nth(4)
            .and_then(|g| g.split(',').next())
            .unwrap_or("")
            .trim();
        if !full.is_empty() {
            name.set(full.to_owned());
        }
    });
}

/// A folder of xdg-user-dirs by key (DOCUMENTS, PICTURES, ...), else `~/<fallback>`.
pub fn user_dir(key: &str, fallback: &str) -> String {
    model()
        .user_dirs
        .with(|d| d.get(key).cloned())
        .unwrap_or_else(|| home_dir().join(fallback).to_string_lossy().into_owned())
}

fn user_dirs_path() -> PathBuf {
    crate::settings::config_home().join("user-dirs.dirs")
}

/// `XDG_<KEY>_DIR="<path>"` lines, with `$HOME` put in.
fn parse_user_dirs(text: String) -> HashMap<String, String> {
    let home = home_dir().to_string_lossy().into_owned();
    text.lines()
        .filter_map(|line| {
            let (key, value) = line.trim().split_once('=')?;
            let key = key.trim().strip_prefix("XDG_")?.strip_suffix("_DIR")?;
            let value = value.trim().strip_prefix('"')?.strip_suffix('"')?;
            let value = match value.strip_prefix("$HOME") {
                Some(rest) => format!("{home}{rest}"),
                None => value.to_owned(),
            };
            Some((key.to_owned(), value))
        })
        .collect()
}

/// Opens a folder in the file manager: its command line gets the folder as its last word.
pub fn open_path(path: &str) {
    let manager = crate::programs::program("fileManager");
    crate::spawn::start_command_line(&format!("{manager} {}", shell_quote(path)), None);
}

// ----- recent files ----------------------------------------------------------------------------

fn data_home() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| home_dir().join(".local/share"))
}

/// The list GTK, Qt's KDE platform and most file managers write.
fn recent_xbel_path() -> PathBuf {
    data_home().join("recently-used.xbel")
}

/// Every file the desktop remembers opening, newest first. Reading it tracks it.
pub fn recent_files() -> Vec<RecentFile> {
    model().recent_files.get()
}

fn unescape_xml(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// The value of the attribute `name` in the attributes of a tag, unescaped.
fn attribute(attributes: &str, name: &str) -> Option<String> {
    let wanted = format!("{name}=\"");
    let mut from = 0;
    while let Some(at) = attributes[from..].find(&wanted) {
        let start = from + at;
        let before = attributes[..start].chars().next_back();
        let value_start = start + wanted.len();
        if before.is_none_or(char::is_whitespace) {
            let end = attributes[value_start..].find('"')?;
            return Some(unescape_xml(&attributes[value_start..value_start + end]));
        }
        from = value_start;
    }
    None
}

/// `%xx` escapes to bytes, read as UTF-8.
fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Some(b) = std::str::from_utf8(&bytes[i + 1..i + 3])
                .ok()
                .and_then(|hex| u8::from_str_radix(hex, 16).ok())
        {
            out.push(b);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The first word of an exec string, without its quotes and its path, in lower case.
fn command_name(exec: &str) -> String {
    let word = exec
        .trim()
        .trim_start_matches(['\'', '"'])
        .split(|c: char| c.is_whitespace() || c == '\'' || c == '"')
        .next()
        .unwrap_or("");
    word.rsplit('/').next().unwrap_or(word).to_lowercase()
}

fn parse_recent(text: String) -> Vec<RecentFile> {
    let mut found = Vec::new();
    let mut rest = text.as_str();
    while let Some(start) = rest.find("<bookmark ") {
        let after = &rest[start + "<bookmark ".len()..];
        let Some(head_end) = after.find('>') else {
            break;
        };
        let attributes = &after[..head_end];
        let tail = &after[head_end + 1..];
        let (body, next) = if attributes.ends_with('/') {
            ("", tail)
        } else {
            match tail.find("</bookmark>") {
                Some(end) => (&tail[..end], &tail[end + "</bookmark>".len()..]),
                None => (tail, ""),
            }
        };
        rest = next;
        let Some(uri) = attribute(attributes, "href") else {
            continue;
        };
        // Only local files: the jump list hands a path to the program.
        let Some(path) = uri.strip_prefix("file://").map(percent_decode) else {
            continue;
        };
        let visited = attribute(attributes, "visited")
            .or_else(|| attribute(attributes, "modified"))
            .unwrap_or_default();
        let mut apps = Vec::new();
        let mut body_rest = body;
        while let Some(at) = body_rest.find("<bookmark:application ") {
            let tag = &body_rest[at + "<bookmark:application ".len()..];
            let end = tag.find('>').unwrap_or(tag.len());
            let attributes = &tag[..end];
            if let Some(name) = attribute(attributes, "name") {
                apps.push(name.to_lowercase());
            }
            if let Some(exec) = attribute(attributes, "exec") {
                apps.push(command_name(&exec));
            }
            body_rest = &tag[end..];
        }
        let name = path.rsplit('/').next().unwrap_or(&path).to_owned();
        found.push(RecentFile {
            path,
            name,
            visited,
            apps,
        });
    }
    found.sort_by(|a, b| b.visited.cmp(&a.visited));
    found
}

/// The names a program goes by in `recently-used.xbel`.
fn names_of(entry: &Application) -> Vec<String> {
    let id = entry.id.trim_end_matches(".desktop");
    let mut names = vec![id.to_lowercase()];
    if !entry.name.is_empty() {
        names.push(entry.name.to_lowercase());
    }
    if let Some(class) = desktop_key(&entry.id, "StartupWMClass") {
        names.push(class.to_lowercase());
    }
    if let Some(exec) = desktop_key(&entry.id, "Exec") {
        names.push(command_name(&exec));
    }
    // Reverse-DNS ids, such as org.gnome.TextEditor, also go by their last part.
    if let Some((_, last)) = id.rsplit_once('.') {
        names.push(last.to_lowercase());
    }
    names
}

/// The files a program opened most recently, newest first, at most [`RECENT_LIMIT`].
pub fn recent_for(entry: &Application) -> Vec<RecentFile> {
    let names = names_of(entry);
    model().recent_files.with(|files| {
        files
            .iter()
            .filter(|f| f.apps.iter().any(|a| names.contains(a)))
            .take(RECENT_LIMIT)
            .cloned()
            .collect()
    })
}

/// The paths pinned to a program's jump list, in the order pinned.
fn pinned_paths(entry: &Application) -> Vec<String> {
    settings().with(|s| {
        s.jump_pins
            .get(&entry.id)
            .and_then(Value::as_array)
            .map(|l| {
                l.iter()
                    .filter_map(|p| p.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    })
}

/// The files pinned to a program's jump list, in the order pinned.
pub fn pinned_files_for(entry: &Application) -> Vec<JumpFile> {
    pinned_paths(entry)
        .into_iter()
        .map(|path| JumpFile {
            name: path.rsplit('/').next().unwrap_or(&path).to_owned(),
            path,
        })
        .collect()
}

/// The recent files of a program, without the pinned ones.
pub fn recent_unpinned(entry: &Application) -> Vec<JumpFile> {
    let pinned = pinned_paths(entry);
    recent_for(entry)
        .into_iter()
        .filter(|f| !pinned.contains(&f.path))
        .map(|f| JumpFile {
            name: f.name,
            path: f.path,
        })
        .collect()
}

pub fn set_file_pinned(entry: &Application, path: &str, on: bool) {
    let id = entry.id.clone();
    let path = path.to_owned();
    change(move |s| {
        let mut list: Vec<String> = s
            .jump_pins
            .get(&id)
            .and_then(Value::as_array)
            .map(|l| {
                l.iter()
                    .filter_map(|p| p.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        list.retain(|other| *other != path);
        if on {
            list.push(path);
        }
        if list.is_empty() {
            s.jump_pins.remove(&id);
        } else {
            s.jump_pins.insert(id, Value::from(list));
        }
    });
}

/// The colour icon of a file, from its extension: a file of `shell/assets/icons`.
pub fn file_icon(name: &str) -> String {
    let extension = name
        .rsplit_once('.')
        .filter(|(stem, _)| !stem.is_empty())
        .map(|(_, e)| e.to_lowercase())
        .unwrap_or_default();
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
    let kind = KINDS
        .iter()
        .find(|(_, exts)| exts.contains(&extension.as_str()))
        .map_or("file", |(k, _)| *k);
    asset(&format!("icons/{kind}.png"))
        .to_string_lossy()
        .into_owned()
}

/// Single quotes for a shell.
fn shell_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}

/// Opens a file with a program, as its `.desktop` file says to start it: the file goes where
/// the field codes of `Exec` are, else last.
pub fn open_with(entry: &Application, path: &str) {
    let Some(exec) = desktop_key(&entry.id, "Exec") else {
        return;
    };
    let file = shell_quote(path);
    let mut placed = false;
    let mut words: Vec<String> = Vec::new();
    for word in exec.split_whitespace() {
        match word {
            "%f" | "%F" | "%u" | "%U" => {
                if !placed {
                    words.push(file.clone());
                    placed = true;
                }
            }
            w if w.len() == 2 && w.starts_with('%') && w != "%%" => {}
            w => words.push(w.replace("%%", "%")),
        }
    }
    if !placed {
        words.push(file);
    }
    let dir = desktop_key(&entry.id, "Path")
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .unwrap_or_else(home_dir);
    crate::spawn::start_command_line(&words.join(" "), Some(&dir));
}

// ----- .desktop files --------------------------------------------------------------------------

/// Where `.desktop` files are, most important first.
fn application_dirs() -> Vec<PathBuf> {
    let dirs = std::env::var("XDG_DATA_DIRS").unwrap_or_default();
    let mut list: Vec<PathBuf> = dirs
        .split(':')
        .filter(|d| d.starts_with('/'))
        .map(PathBuf::from)
        .collect();
    if list.is_empty() {
        list = vec!["/usr/local/share".into(), "/usr/share".into()];
    }
    std::iter::once(data_home())
        .chain(list)
        .map(|d| d.join("applications"))
        .collect()
}

/// The file of a `.desktop` file name: in a folder of applications, or in a subfolder for the
/// dashes of its name (`kde-foo.desktop` is `kde/foo.desktop`).
fn desktop_file(id: &str) -> Option<PathBuf> {
    application_dirs().into_iter().find_map(|dir| {
        let direct = dir.join(id);
        if direct.is_file() {
            return Some(direct);
        }
        let nested = dir.join(id.replace('-', "/"));
        nested.is_file().then_some(nested)
    })
}

/// A key of a program's `[Desktop Entry]` group, as written (not localized). AntiWebUI's
/// `Application` does not give `Exec`, `Path` or `StartupWMClass`; the file is read once.
fn desktop_key(id: &str, key: &str) -> Option<String> {
    DESKTOP_KEYS.with(|cache| {
        let mut cache = cache.borrow_mut();
        let keys = cache.entry(id.to_owned()).or_insert_with(|| {
            let Some(text) = desktop_file(id).and_then(|p| std::fs::read_to_string(p).ok()) else {
                return HashMap::new();
            };
            let mut in_entry = false;
            let mut keys = HashMap::new();
            for line in text.lines() {
                let line = line.trim();
                if line.starts_with('[') {
                    in_entry = line == "[Desktop Entry]";
                } else if in_entry && let Some((k, v)) = line.split_once('=') {
                    keys.entry(k.trim().to_owned())
                        .or_insert_with(|| v.trim().to_owned());
                }
            }
            keys
        });
        keys.get(key).cloned().filter(|v| !v.is_empty())
    })
}

// ----- files -----------------------------------------------------------------------------------

fn counts_path() -> PathBuf {
    state_dir().join("launch-counts.json")
}

fn recent_programs_path() -> PathBuf {
    state_dir().join("launch-recent.json")
}

fn read_json(path: &Path) -> Option<Value> {
    serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()
}

fn write_json(path: &Path, value: &Value) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, value.to_string());
}

fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// Calls `read` with the file's text now (`None` when it cannot be read), and again each time
/// its time changes.
fn follow_file(path: PathBuf, read: impl Fn(Option<String>) + 'static) {
    fn look(
        path: PathBuf,
        seen: Option<SystemTime>,
        first: bool,
        read: std::rc::Rc<dyn Fn(Option<String>)>,
    ) {
        let now = modified(&path);
        if first || now != seen {
            read(std::fs::read_to_string(&path).ok());
        }
        set_timeout(WATCH, move || look(path, now, false, read));
    }
    look(path, None, true, std::rc::Rc::new(read));
}
