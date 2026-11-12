//! What the taskbar and the Start menu know about programs and windows (QML `Apps.qml`).
//!
//! Windows come from `wlr_foreign_toplevel_management_v1` (AntiWebUI's `toplevels()`), so
//! nothing here is specific to ShojiWM. Programs come from the system's `.desktop` files
//! (AntiWebUI's `apps()`). The state lives once per UI thread, made the first time it is read:
//! every screen's bar reads the same lists.
//!
//! A program is named by its `.desktop` file name (`kitty.desktop`), as the settings keep it
//! (`start_pinned`, `taskbar_pinned`, `jump_pins`) and as `Application::id` gives it.
//!
//! Nothing here touches the disk on the UI thread. When the programs change, a task reads their
//! `.desktop` files and builds the [`ProgramIndex`]: a window's app id, a file name, the folders
//! of All Programs are then lookups. A thread watches `recently-used.xbel` and `user-dirs.dirs`
//! and posts what it parsed; the jump lists of every program are found in a task when the recent
//! files or the programs change. The state files are written by one thread, in order
//! ([`write_later`], [`run_in_order`]).

use crate::paths::{asset, state_dir};
use crate::settings::{Settings, change, home, settings};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use serde_json::{Map, Value};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, mpsc};
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
/// dragged into, the programs' index, the recent files, the user's folders.
#[derive(Clone, Copy)]
struct Model {
    /// The owner of the state: the tasks started from a memo answer under it.
    root: Owner,
    /// How often each program was started from the shell, by `.desktop` file name.
    launch_counts: Signal<HashMap<String, i64>>,
    /// The programs started from the shell, last first.
    recent_program_ids: Signal<Vec<String>>,
    /// The two lists above have been read from their files: until then nothing is written.
    launch_history_read: Signal<bool>,
    /// The order the buttons were dragged into, as group keys, for this session.
    button_order: Signal<Vec<String>>,
    /// The programs with what their `.desktop` files say, built in a task when they change.
    index: Signal<Arc<ProgramIndex>>,
    recent_files: Signal<Arc<Vec<RecentFile>>>,
    /// The recent files of each program, by `.desktop` file name, at most [`RECENT_LIMIT`];
    /// found in a task when the recent files or the programs change.
    jump_recent: Signal<Arc<HashMap<String, Vec<RecentFile>>>>,
    /// The folders of xdg-user-dirs, by key (DOCUMENTS, PICTURES, ...).
    user_dirs: Signal<HashMap<String, String>>,
    /// Whether the icon theme has an icon of each name [`icon_for`] asked about.
    theme_icons: Signal<HashMap<String, bool>>,
    groups: Memo<Vec<WindowGroup>>,
}

thread_local! {
    static MODEL: Cell<Option<Model>> = const { Cell::new(None) };
    /// When each pinned program was last started from its button.
    static STARTED: RefCell<HashMap<String, Instant>> = RefCell::new(HashMap::new());
    /// The icon names looked for in the icon theme, answered or not.
    static THEME_ICONS_ASKED: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

/// Makes the state now, outside of any surface: its tasks and its watch of the recent files then
/// belong to the shell, not to a surface that can close. Call once, from the root, before
/// anything reads the lists.
pub fn install() {
    model();
}

/// What the watch thread posts.
enum Watched {
    RecentFiles(Vec<RecentFile>),
    UserDirs(HashMap<String, String>),
}

fn model() -> Model {
    MODEL.with(|cell| {
        cell.get().unwrap_or_else(|| {
            let (m, _) = create_root(|| {
                let button_order = signal(Vec::<String>::new());
                let index = signal(Arc::new(ProgramIndex::default()));
                let theme_icons = signal(HashMap::new());
                let root = Owner::current().expect("create_root makes an owner");
                let m = Model {
                    root,
                    launch_counts: signal(HashMap::new()),
                    recent_program_ids: signal(Vec::new()),
                    launch_history_read: signal(false),
                    button_order,
                    index,
                    recent_files: signal(Arc::new(Vec::new())),
                    jump_recent: signal(Arc::new(HashMap::new())),
                    user_dirs: signal(HashMap::new()),
                    theme_icons,
                    groups: memo(move || {
                        let open = toplevels().all();
                        let (combine, pins) =
                            settings().with(|s| (s.combine_windows, s.taskbar_pinned.clone()));
                        let index = index.get();
                        let icon = |app_id: &str| icon_in(&index, theme_icons, root, app_id);
                        button_order
                            .with(|order| build_groups(&index, &icon, &open, &pins, combine, order))
                    }),
                };
                read_launch_history(m);
                follow_programs(m);
                follow_jump_lists(m);
                watch_files(ui_channel(move |watched: Watched| match watched {
                    Watched::RecentFiles(list) => m.recent_files.set(Arc::new(list)),
                    Watched::UserDirs(dirs) => m.user_dirs.set(dirs),
                }));
                m
            });
            cell.set(Some(m));
            m
        })
    })
}

/// The answer of the last request only: each request takes a ticket, and an answer counts when
/// its ticket is still the last one taken and the owner of the requests is still there.
#[derive(Clone, Copy)]
pub(crate) struct Latest(Signal<u64>);

impl Latest {
    pub fn new() -> Self {
        Self(signal(0))
    }

    /// A ticket for a new request: the answers of those before it no longer count.
    pub fn next(&self) -> u64 {
        self.0.update_untracked(|n| {
            *n += 1;
            *n
        })
    }

    /// The answer of the request with `ticket` counts.
    pub fn is_current(&self, ticket: u64) -> bool {
        self.0.is_alive() && self.0.get_untracked() == ticket
    }
}

/// Builds the index again in a task each time the programs change.
fn follow_programs(m: Model) {
    let latest = Latest::new();
    effect(move || {
        let list = apps().all();
        let ticket = latest.next();
        task(move || build_index(list)).then(move |index| {
            if latest.is_current(ticket) {
                m.index.set(Arc::new(index));
            }
        });
    });
}

/// Finds the jump lists again in a task each time the recent files or the programs change.
fn follow_jump_lists(m: Model) {
    let latest = Latest::new();
    effect(move || {
        let (index, files) = (m.index.get(), m.recent_files.get());
        let ticket = latest.next();
        task(move || jump_lists(&index, &files)).then(move |found| {
            if latest.is_current(ticket) {
                m.jump_recent.set(Arc::new(found));
            }
        });
    });
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
fn program_key(index: &ProgramIndex, t: &Toplevel) -> String {
    if is_control_panel(t) {
        return "controlpanel".into();
    }
    match index.find(&t.app_id) {
        Some(program) => format!("entry:{}", program.entry.id),
        None if t.app_id.is_empty() => "appid:<none>".into(),
        None => format!("appid:{}", t.app_id.to_lowercase()),
    }
}

fn build_groups(
    index: &ProgramIndex,
    icon_for: &dyn Fn(&str) -> String,
    open: &[Toplevel],
    pins: &[String],
    combine: bool,
    order: &[String],
) -> Vec<WindowGroup> {
    let mut list: Vec<WindowGroup> = Vec::new();
    // The group each program's next window goes into, or after, by program key.
    let mut latest: HashMap<String, String> = HashMap::new();
    let pinned = pins
        .iter()
        .filter_map(|id| index.get(id).map(|p| p.entry.clone()));
    for entry in pinned {
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
        let program = program_key(index, t);
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
                let entry = if ours {
                    None
                } else {
                    index.find(&t.app_id).map(|p| p.entry.clone())
                };
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
                    match &entry {
                        Some(e) => entry_icon(Some(e)),
                        None => icon_for(&t.app_id),
                    }
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

/// The programs with what their `.desktop` files say. Reading it tracks it.
pub fn program_index() -> Arc<ProgramIndex> {
    model().index.get()
}

/// The program of a `.desktop` file name such as `kitty.desktop` (the extension may be left
/// out).
pub fn entry_by_id(id: &str) -> Option<Application> {
    model().index.with(|i| i.get(id).map(|p| p.entry.clone()))
}

/// The programs of a list of file names, in order, leaving out those not installed.
pub fn entries_of(ids: &[String]) -> Vec<Application> {
    model().index.with(|i| {
        ids.iter()
            .filter_map(|id| i.get(id).map(|p| p.entry.clone()))
            .collect()
    })
}

/// The program a window's app id belongs to: its `.desktop` file by name, else by the last
/// part of a reverse-DNS name, by `StartupWMClass`, or by the program's name.
pub fn entry_for(app_id: &str) -> Option<Application> {
    model()
        .index
        .with(|i| i.find(app_id).map(|p| p.entry.clone()))
}

/// The icon of a window's program: its `.desktop` file's, else the app id's in the icon
/// theme, else the generic one.
pub fn icon_for(app_id: &str) -> String {
    let m = model();
    m.index
        .with(|index| icon_in(index, m.theme_icons, m.root, app_id))
}

/// [`icon_for`] with the index given. Whether the icon theme has the app id is looked up in a
/// task the first time, under `root`; the generic icon shows until the answer, which `icons`
/// then carries.
fn icon_in(
    index: &ProgramIndex,
    icons: Signal<HashMap<String, bool>>,
    root: Owner,
    app_id: &str,
) -> String {
    if let Some(program) = index.find(app_id) {
        return entry_icon(Some(&program.entry));
    }
    let lower = app_id.to_lowercase();
    if lower.is_empty() {
        return FALLBACK_ICON.into();
    }
    match icons.with(|i| i.get(&lower).copied()) {
        Some(true) => return lower,
        Some(false) => {}
        None => {
            let first = THEME_ICONS_ASKED.with(|asked| asked.borrow_mut().insert(lower.clone()));
            if first {
                let name = lower.clone();
                root.with(|| {
                    task(move || {
                        let found = anti_webui::shell::icon_path(&name, 32).is_some();
                        (name, found)
                    })
                    .then(move |(name, found)| {
                        icons.update(|i| {
                            i.insert(name, found);
                        })
                    })
                });
            }
        }
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
    m.recent_program_ids.update(|r| {
        r.retain(|other| other != id);
        r.insert(0, id.to_owned());
        r.truncate(RECENT_PROGRAMS_KEPT);
    });
    // Before the files are read, writing them would lose what they hold: the lists are merged
    // with them, then written, once they are read.
    if m.launch_history_read.get_untracked() {
        save_launch_history(m);
    }
}

/// Writes the launch counts and the programs started last, on the writer thread.
fn save_launch_history(m: Model) {
    let counts: Map<String, Value> = m.launch_counts.with_untracked(|c| {
        c.iter()
            .map(|(k, v)| (k.clone(), Value::from(*v)))
            .collect()
    });
    write_later(counts_path(), Value::Object(counts).to_string());
    let recent = m
        .recent_program_ids
        .with_untracked(|r| Value::from(r.clone()));
    write_later(recent_programs_path(), recent.to_string());
}

/// Reads the launch counts and the programs started last in a task. What was started before the
/// answer is added to them, and then written.
fn read_launch_history(m: Model) {
    task(|| {
        let counts: HashMap<String, i64> = read_json(&counts_path())
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(k, v)| Some((k, v.as_i64()?)))
            .collect();
        let recent: Vec<String> = read_json(&recent_programs_path())
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        (counts, recent)
    })
    .then(move |(counts, recent)| {
        let started_before = m.launch_counts.with_untracked(|c| !c.is_empty());
        m.launch_counts.update(|c| merge_counts(c, counts));
        m.recent_program_ids.update(|r| merge_recent(r, recent));
        m.launch_history_read.set(true);
        if started_before {
            save_launch_history(m);
        }
    });
}

/// Adds the counts read from the file to those of this session.
fn merge_counts(session: &mut HashMap<String, i64>, read: HashMap<String, i64>) {
    for (id, count) in read {
        *session.entry(id).or_insert(0) += count;
    }
}

/// The programs of this session first, then those read from the file, each once.
fn merge_recent(session: &mut Vec<String>, read: Vec<String>) {
    for id in read {
        if !session.contains(&id) {
            session.push(id);
        }
    }
    session.truncate(RECENT_PROGRAMS_KEPT);
}

/// The programs started from the shell, last first, that are still installed.
pub fn recent_programs() -> Vec<Application> {
    entries_of(&model().recent_program_ids.get())
}

/// [`frequent`], leaving out the programs `pins` names.
pub fn frequent_except(pins: &[String]) -> Vec<Application> {
    let m = model();
    let ids = m.launch_counts.with(|counts| most_started(counts, pins));
    m.index.with(|i| {
        ids.iter()
            .filter_map(|id| i.get(id).map(|p| p.entry.clone()))
            .take(FREQUENT_LIMIT)
            .collect()
    })
}

/// The program ids of `counts`, started most first (then by id), without those of `pins`.
fn most_started(counts: &HashMap<String, i64>, pins: &[String]) -> Vec<String> {
    let mut ids: Vec<(&String, &i64)> =
        counts.iter().filter(|(id, _)| !pins.contains(id)).collect();
    ids.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    ids.into_iter().map(|(id, _)| id.clone()).collect()
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

/// Every listed program, in folders, by name: made with the index. Reading it tracks it.
pub fn folders() -> Vec<Folder> {
    model().index.with(|i| i.folders.clone())
}

/// The programs in folders, the folders by name, the programs in the order they come.
fn make_folders(entries: impl IntoIterator<Item = Application>) -> Vec<Folder> {
    let mut list: Vec<Folder> = Vec::new();
    for entry in entries {
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

pub use crate::account::user_name;

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
    model().recent_files.with(|files| files.as_ref().clone())
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

/// The names a program goes by in `recently-used.xbel`, from its file name, its name and the
/// `StartupWMClass` and `Exec` of its `.desktop` file.
fn names_of(id: &str, name: &str, class: Option<&str>, exec: Option<&str>) -> Vec<String> {
    let id = id.trim_end_matches(".desktop");
    let mut names = vec![id.to_lowercase()];
    if !name.is_empty() {
        names.push(name.to_lowercase());
    }
    if let Some(class) = class {
        names.push(class.to_lowercase());
    }
    if let Some(exec) = exec {
        names.push(command_name(exec));
    }
    // Reverse-DNS ids, such as org.gnome.TextEditor, also go by their last part.
    if let Some((_, last)) = id.rsplit_once('.') {
        names.push(last.to_lowercase());
    }
    names
}

/// The files opened by a program of `names`, newest first, at most [`RECENT_LIMIT`].
fn recent_matching(names: &[String], files: &[RecentFile]) -> Vec<RecentFile> {
    files
        .iter()
        .filter(|f| f.apps.iter().any(|a| names.contains(a)))
        .take(RECENT_LIMIT)
        .cloned()
        .collect()
}

/// The recent files of each program that has some, by `.desktop` file name. It goes over every
/// program and every file: run it in a task.
fn jump_lists(index: &ProgramIndex, files: &[RecentFile]) -> HashMap<String, Vec<RecentFile>> {
    index
        .programs
        .iter()
        .filter_map(|p| {
            let found = recent_matching(&p.names, files);
            (!found.is_empty()).then(|| (p.entry.id.clone(), found))
        })
        .collect()
}

/// The files a program opened most recently, newest first, at most [`RECENT_LIMIT`]. Reading it
/// tracks the jump lists.
pub fn recent_for(entry: &Application) -> Vec<RecentFile> {
    model()
        .jump_recent
        .with(|lists| lists.get(&entry.id).cloned().unwrap_or_default())
}

/// The program has recent files or pinned files to show in a jump list. Reading it tracks them.
pub fn has_jump_list(entry: &Application) -> bool {
    model()
        .jump_recent
        .with(|lists| lists.contains_key(&entry.id))
        || settings().with(|s| {
            s.jump_pins
                .get(&entry.id)
                .and_then(Value::as_array)
                .is_some_and(|l| !l.is_empty())
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
    let Some((exec, dir)) = model().index.with_untracked(|i| {
        i.get(&entry.id)
            .and_then(|p| Some((p.exec.clone()?, p.dir.clone())))
    }) else {
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
    let dir = dir.unwrap_or_else(home_dir);
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
/// dashes of its name (`kde-foo.desktop` is `kde/foo.desktop`). It looks at the disk: call it on
/// a worker thread only.
fn desktop_file(dirs: &[PathBuf], id: &str) -> Option<PathBuf> {
    dirs.iter().find_map(|dir| {
        let direct = dir.join(id);
        if direct.is_file() {
            return Some(direct);
        }
        let nested = dir.join(id.replace('-', "/"));
        nested.is_file().then_some(nested)
    })
}

/// The keys of the `[Desktop Entry]` group of a `.desktop` file, as written (not localized): the
/// first of each.
fn desktop_entry_keys(text: &str) -> HashMap<String, String> {
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
}

/// A program, with the keys of its `.desktop` file that AntiWebUI's `Application` does not give.
#[derive(Clone, Debug)]
pub struct Program {
    pub entry: Application,
    /// `Exec`, as written, field codes and all.
    pub exec: Option<String>,
    /// `Path`, when it is a folder.
    pub dir: Option<PathBuf>,
    /// `StartupWMClass`.
    pub class: Option<String>,
    /// The names it goes by in `recently-used.xbel`, in lower case.
    names: Vec<String>,
}

/// Every program, by name, and the ways to find one: by `.desktop` file name, by a window's app
/// id, and the folders of All Programs. Built in a task ([`build_index`]); every lookup is a hash.
#[derive(Debug, Default)]
pub struct ProgramIndex {
    pub programs: Vec<Program>,
    lookup: Lookup,
    folders: Vec<Folder>,
}

impl ProgramIndex {
    fn new(programs: Vec<Program>) -> Self {
        let lookup = Lookup::build(programs.iter().map(|p| MatchKeys {
            id: &p.entry.id,
            name: &p.entry.name,
            class: p.class.as_deref(),
        }));
        let folders = make_folders(programs.iter().map(|p| p.entry.clone()));
        Self {
            programs,
            lookup,
            folders,
        }
    }

    /// The program of a `.desktop` file name such as `kitty.desktop` (the extension may be left
    /// out).
    pub fn get(&self, id: &str) -> Option<&Program> {
        self.lookup.file(id).map(|i| &self.programs[i])
    }

    /// The program a window's app id belongs to (see [`entry_for`]).
    pub fn find(&self, app_id: &str) -> Option<&Program> {
        self.lookup.find(app_id).map(|i| &self.programs[i])
    }
}

/// What a program is found by.
struct MatchKeys<'a> {
    /// The `.desktop` file name.
    id: &'a str,
    name: &'a str,
    /// `StartupWMClass`.
    class: Option<&'a str>,
}

/// The place of a program in the list by each key it is found by, in lower case but the file
/// name. Each key keeps the first program that has it.
#[derive(Debug, Default)]
struct Lookup {
    by_file: HashMap<String, usize>,
    by_stem: HashMap<String, usize>,
    by_last_part: HashMap<String, usize>,
    by_class: HashMap<String, usize>,
    /// The stem or the last part of the stem.
    by_short: HashMap<String, usize>,
    by_name: HashMap<String, usize>,
}

impl Lookup {
    fn build<'a>(keys: impl IntoIterator<Item = MatchKeys<'a>>) -> Self {
        let mut l = Self::default();
        for (i, k) in keys.into_iter().enumerate() {
            let stem = k.id.trim_end_matches(".desktop").to_lowercase();
            let last = stem.rsplit('.').next().unwrap_or(&stem).to_owned();
            l.by_file.entry(k.id.to_owned()).or_insert(i);
            l.by_stem.entry(stem.clone()).or_insert(i);
            l.by_last_part.entry(last.clone()).or_insert(i);
            if let Some(class) = k.class.filter(|c| !c.is_empty()) {
                l.by_class.entry(class.to_lowercase()).or_insert(i);
            }
            l.by_short.entry(stem).or_insert(i);
            l.by_short.entry(last).or_insert(i);
            l.by_name.entry(k.name.to_lowercase()).or_insert(i);
        }
        l
    }

    /// A `.desktop` file name, trimmed, the extension added when it is not there.
    fn file(&self, id: &str) -> Option<usize> {
        let id = id.trim();
        if id.is_empty() {
            return None;
        }
        if id.ends_with(".desktop") {
            self.by_file.get(id).copied()
        } else {
            self.by_file.get(&format!("{id}.desktop")).copied()
        }
    }

    /// A window's app id: the file name, then the stem, the last part of a reverse-DNS stem,
    /// `StartupWMClass`, the last part of the app id, then the program's name.
    fn find(&self, app_id: &str) -> Option<usize> {
        if app_id.is_empty() {
            return None;
        }
        if let Some(i) = self.file(app_id) {
            return Some(i);
        }
        let wanted = app_id.to_lowercase();
        let short = wanted.rsplit('.').next().unwrap_or(&wanted);
        self.by_stem
            .get(&wanted)
            .or_else(|| self.by_last_part.get(&wanted))
            .or_else(|| self.by_class.get(&wanted))
            .or_else(|| self.by_short.get(short))
            .or_else(|| self.by_name.get(&wanted))
            .copied()
    }
}

/// Reads the `.desktop` file of each program and makes the index. It reads the disk: run it in a
/// task.
fn build_index(list: Vec<Application>) -> ProgramIndex {
    let dirs = application_dirs();
    let programs = list
        .into_iter()
        .map(|entry| {
            let keys = desktop_file(&dirs, &entry.id)
                .and_then(|p| std::fs::read_to_string(p).ok())
                .map(|text| desktop_entry_keys(&text))
                .unwrap_or_default();
            let key = |k: &str| keys.get(k).filter(|v| !v.is_empty()).cloned();
            let exec = key("Exec");
            let class = key("StartupWMClass");
            let dir = key("Path").map(PathBuf::from).filter(|p| p.is_dir());
            let names = names_of(&entry.id, &entry.name, class.as_deref(), exec.as_deref());
            Program {
                entry,
                exec,
                dir,
                class,
                names,
            }
        })
        .collect();
    ProgramIndex::new(programs)
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

fn write_file(path: &Path, text: &str) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, text);
}

/// Work for the writer thread.
enum WriteJob {
    /// Writes a file, unless a later job of the same batch writes it again.
    File(PathBuf, String),
    Run(Box<dyn FnOnce() + Send>),
}

/// The writer thread: it does the jobs in the order they come. A batch is what has come while it
/// was busy; of the writes of one file in a batch, only the last is done.
fn writer() -> &'static mpsc::Sender<WriteJob> {
    static WRITER: OnceLock<mpsc::Sender<WriteJob>> = OnceLock::new();
    WRITER.get_or_init(|| {
        let (send, receive) = mpsc::channel::<WriteJob>();
        let _ = std::thread::Builder::new()
            .name("shell state writer".into())
            .spawn(move || {
                while let Ok(first) = receive.recv() {
                    let mut batch = vec![first];
                    batch.extend(receive.try_iter());
                    for job in latest_writes(batch) {
                        match job {
                            WriteJob::File(path, text) => write_file(&path, &text),
                            WriteJob::Run(job) => job(),
                        }
                    }
                }
            });
        send
    })
}

/// The jobs of a batch, in order, without the writes of a file that a later write replaces.
fn latest_writes(batch: Vec<WriteJob>) -> Vec<WriteJob> {
    let mut written = HashSet::new();
    let mut kept: Vec<WriteJob> = batch
        .into_iter()
        .rev()
        .filter(|job| match job {
            WriteJob::File(path, _) => written.insert(path.clone()),
            WriteJob::Run(_) => true,
        })
        .collect();
    kept.reverse();
    kept
}

/// Writes `text` to the state file `path` on the writer thread, after what was asked before.
pub(crate) fn write_later(path: PathBuf, text: String) {
    let _ = writer().send(WriteJob::File(path, text));
}

/// Runs `job` on the writer thread, after what was asked before: for a file to read, change and
/// write again.
pub(crate) fn run_in_order(job: impl FnOnce() + Send + 'static) {
    let _ = writer().send(WriteJob::Run(Box::new(job)));
}

fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// A thread that reads `recently-used.xbel` and `user-dirs.dirs` now, then again each time their
/// time changes, looked at every [`WATCH`], and posts what it parsed.
fn watch_files(to_ui: UiSender<Watched>) {
    let _ = std::thread::Builder::new()
        .name("shell file watch".into())
        .spawn(move || {
            let (recent, dirs) = (recent_xbel_path(), user_dirs_path());
            // `None` until looked at once, then the time seen.
            let mut seen_recent: Option<Option<SystemTime>> = None;
            let mut seen_dirs: Option<Option<SystemTime>> = None;
            loop {
                let now = modified(&recent);
                if seen_recent != Some(now) {
                    seen_recent = Some(now);
                    let text = std::fs::read_to_string(&recent).ok();
                    to_ui.send(Watched::RecentFiles(
                        text.map(parse_recent).unwrap_or_default(),
                    ));
                }
                let now = modified(&dirs);
                if seen_dirs != Some(now) {
                    seen_dirs = Some(now);
                    let text = std::fs::read_to_string(&dirs).ok();
                    to_ui.send(Watched::UserDirs(
                        text.map(parse_user_dirs).unwrap_or_default(),
                    ));
                }
                std::thread::sleep(WATCH);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup(list: &[(&str, &str, Option<&str>)]) -> Lookup {
        Lookup::build(
            list.iter()
                .map(|&(id, name, class)| MatchKeys { id, name, class }),
        )
    }

    #[test]
    fn finds_a_program_by_each_key_in_order() {
        let l = lookup(&[
            ("Alacritty.desktop", "Alacritty", None),
            ("org.gnome.TextEditor.desktop", "Text Editor", None),
            ("code.desktop", "Visual Studio Code", Some("Code-Insiders")),
            ("firefox.desktop", "Firefox", None),
            ("texteditor.desktop", "Other", None),
        ]);
        // The file name, with or without its extension, as written.
        assert_eq!(l.find("Alacritty"), Some(0));
        assert_eq!(l.file(" firefox.desktop "), Some(3));
        assert_eq!(l.file(""), None);
        // The stem, whatever the case, before the last part of another one.
        assert_eq!(l.find("TEXTEDITOR"), Some(4));
        assert_eq!(l.find("org.gnome.texteditor"), Some(1));
        // StartupWMClass.
        assert_eq!(l.find("code-insiders"), Some(2));
        // The last part of a reverse-DNS app id.
        assert_eq!(l.find("org.mozilla.firefox"), Some(3));
        // The name.
        assert_eq!(l.find("visual studio code"), Some(2));
        assert_eq!(l.find("nothing"), None);
        assert_eq!(l.find(""), None);
    }

    #[test]
    fn the_first_program_keeps_a_key() {
        let l = lookup(&[
            ("a.term.desktop", "One", None),
            ("b.term.desktop", "Two", None),
        ]);
        assert_eq!(l.find("term"), Some(0));
    }

    #[test]
    fn reads_the_desktop_entry_group_only() {
        let keys = desktop_entry_keys(
            "[Desktop Entry]\nExec=foo %u\nExec=bar\nName = Foo \n[Desktop Action new]\nExec=baz\n",
        );
        assert_eq!(keys.get("Exec").map(String::as_str), Some("foo %u"));
        assert_eq!(keys.get("Name").map(String::as_str), Some("Foo"));
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn names_of_a_program() {
        assert_eq!(
            names_of(
                "org.gnome.TextEditor.desktop",
                "Text Editor",
                Some("Gnome-text-editor"),
                Some("/usr/bin/gnome-text-editor %U"),
            ),
            [
                "org.gnome.texteditor",
                "text editor",
                "gnome-text-editor",
                "gnome-text-editor",
                "texteditor",
            ]
        );
    }

    #[test]
    fn recent_files_of_a_program() {
        let file = |name: &str, apps: &[&str]| RecentFile {
            path: format!("/home/me/{name}"),
            name: name.to_owned(),
            visited: String::new(),
            apps: apps.iter().map(|a| (*a).to_owned()).collect(),
        };
        let mut files = vec![file("a.txt", &["gedit"]), file("b.png", &["eog"])];
        files.extend((0..20).map(|i| file(&format!("{i}.txt"), &["gedit"])));
        let found = recent_matching(&["gedit".to_owned()], &files);
        assert_eq!(found.len(), RECENT_LIMIT);
        assert_eq!(found[0].name, "a.txt");
        assert!(recent_matching(&["vim".to_owned()], &files).is_empty());
    }

    #[test]
    fn merges_the_launch_history_read_later() {
        let mut counts = HashMap::from([("a.desktop".to_owned(), 1)]);
        merge_counts(
            &mut counts,
            HashMap::from([("a.desktop".to_owned(), 4), ("b.desktop".to_owned(), 2)]),
        );
        assert_eq!(counts["a.desktop"], 5);
        assert_eq!(counts["b.desktop"], 2);
        let mut recent = vec!["b".to_owned()];
        merge_recent(
            &mut recent,
            vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
        );
        assert_eq!(recent, ["b", "a", "c"]);
    }

    #[test]
    fn most_started_first_without_the_pinned() {
        let counts = HashMap::from([
            ("a".to_owned(), 3),
            ("b".to_owned(), 5),
            ("c".to_owned(), 3),
            ("d".to_owned(), 9),
        ]);
        assert_eq!(most_started(&counts, &["d".to_owned()]), ["b", "a", "c"]);
    }

    #[test]
    fn a_batch_writes_each_file_once_with_its_last_text() {
        let file = |p: &str, t: &str| WriteJob::File(PathBuf::from(p), t.to_owned());
        let kept = latest_writes(vec![
            file("/a", "1"),
            file("/b", "1"),
            WriteJob::Run(Box::new(|| {})),
            file("/a", "2"),
        ]);
        let shown: Vec<String> = kept
            .iter()
            .map(|job| match job {
                WriteJob::File(p, t) => format!("{}={t}", p.display()),
                WriteJob::Run(_) => "run".to_owned(),
            })
            .collect();
        assert_eq!(shown, ["/b=1", "run", "/a=2"]);
    }

    #[test]
    fn only_the_last_ticket_counts() {
        let (latest, owner) = create_root(Latest::new);
        let first = latest.next();
        let second = latest.next();
        assert!(!latest.is_current(first));
        assert!(latest.is_current(second));
        owner.dispose();
        assert!(!latest.is_current(second));
    }
}
