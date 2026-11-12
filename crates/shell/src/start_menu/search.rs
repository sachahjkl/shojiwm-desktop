//! What the Start menu's search box and the launcher find (QML `Search`): the programs, the
//! Control Panel's tasks and the files, as rows ready to show, and the list of those rows.
//!
//! Before anything is typed, the rows are the programs started last, as many as the Control
//! Panel says, then every program by name: the launcher shows them, the Start menu shows its
//! own lists instead. While typing: the programs that match (AntiWebUI's ranking, best first),
//! the Control Panel's tasks, then the files of the home folder whose names hold what is typed.
//!
//! The ranking runs in a task for each query; an answer to an older query is dropped, and the
//! rows of the last answer stay until the next one comes.

use crate::apps::{Latest, ProgramIndex};
use crate::control_panel::state::find_tasks;
use crate::fuzzy;
use crate::paths::asset;
use crate::power::quoted;
use crate::settings::{home, settings};
use crate::theme::scheme;
use crate::ui::{MenuLine, RowLook, menu_row, with_context_menu};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use std::cell::Cell;
use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;
use std::time::Duration;

/// The most programs a search lists.
const PROGRAM_LIMIT: usize = 8;
/// The most Control Panel tasks a search lists.
const TASK_LIMIT: usize = 5;
/// The most files a search lists.
const FILE_LIMIT: usize = 8;
/// The files are looked for this long after the last key.
const FILE_WAIT: Duration = Duration::from_millis(250);
/// The fewest characters the files are looked for with.
const FILE_MIN_CHARS: usize = 2;

/// What a row of the results stands for.
#[derive(Clone, PartialEq)]
pub(super) enum Found {
    /// The title of a group: not a choice.
    Heading(&'static str),
    Program(Application),
    /// A Control Panel task: what it says, and the page it opens.
    Task {
        text: String,
        page: String,
    },
    /// A file or a folder, by path.
    File(String),
}

/// A row of the results. `pick` numbers the rows that can be chosen; a heading has none.
#[derive(Clone, PartialEq)]
pub(super) struct Row {
    pub found: Found,
    pub pick: Option<usize>,
    /// The letters of its label the typed text matched, highlighted (`fuzzy`).
    pub marks: Vec<usize>,
}

/// A search: what is typed, and what it finds. Each search box has its own.
#[derive(Clone, Copy)]
pub(super) struct Search {
    pub query: Signal<String>,
    pub rows: Memo<Vec<Row>>,
    pub pick_count: Memo<usize>,
}

/// The program's icon, a name of the icon theme or a file.
pub(super) fn app_icon(app: &Application) -> String {
    app.icon
        .clone()
        .unwrap_or_else(|| "application-x-executable".to_owned())
}

/// The last part of a path.
fn file_name(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_owned()
}

/// Opens a file with the program the desktop chose for it (a folder in the file manager).
pub(super) fn open_file(path: &str) {
    crate::programs::run(&format!("xdg-open {}", quoted(path)));
}

/// What a query found, ranked in a task: the programs and the Control Panel's tasks, each with
/// the letters matched.
#[derive(Clone, Default, PartialEq)]
struct Ranked {
    /// The query, trimmed.
    query: String,
    programs: Vec<(Found, Vec<usize>)>,
    tasks: Vec<(Found, Vec<usize>)>,
}

/// The files `fd` found for a query, ranked.
#[derive(Clone, Default, PartialEq)]
struct Files {
    /// The query, trimmed.
    query: String,
    found: Vec<(Found, Vec<usize>)>,
}

/// The rows of groups: a heading over each group that has rows, the rows numbered for the pick.
#[derive(Default)]
struct Rows {
    out: Vec<Row>,
    pick: usize,
}

impl Rows {
    fn group(&mut self, title: &'static str, list: impl IntoIterator<Item = (Found, Vec<usize>)>) {
        let mut list = list.into_iter().peekable();
        if list.peek().is_none() {
            return;
        }
        self.out.push(Row {
            found: Found::Heading(title),
            pick: None,
            marks: Vec::new(),
        });
        for (found, marks) in list {
            self.out.push(Row {
                found,
                pick: Some(self.pick),
                marks,
            });
            self.pick += 1;
        }
    }
}

impl Search {
    pub fn new() -> Self {
        let query = signal(String::new());
        let typed = memo(move || query.with(|q| q.trim().to_owned()));
        let files = signal(Files::default());
        let ranked = signal(Ranked::default());
        find_files(typed, files);
        rank_queries(typed, ranked);
        let recent_count = memo(|| settings().with(|s| s.launcher_recent_count).max(0) as usize);
        let rows = memo(move || {
            let typed = typed.get();
            let mut rows = Rows::default();
            let plain = |a: Application| (Found::Program(a), Vec::new());
            if typed.is_empty() {
                let recent = crate::apps::recent_programs();
                rows.group(
                    "Recent",
                    recent.into_iter().take(recent_count.get()).map(plain),
                );
                let index = crate::apps::program_index();
                rows.group(
                    "All Programs",
                    index.programs.iter().map(|p| plain(p.entry.clone())),
                );
            } else {
                ranked.with(|r| {
                    rows.group("Programs", r.programs.iter().cloned());
                    rows.group("Control Panel", r.tasks.iter().cloned());
                });
                files.with(|f| {
                    if f.query == typed {
                        rows.group("Files", f.found.iter().cloned());
                    }
                });
            }
            rows.out
        });
        let pick_count = memo(move || rows.with(|r| r.iter().filter(|r| r.pick.is_some()).count()));
        Self {
            query,
            rows,
            pick_count,
        }
    }

    /// Something is typed.
    pub fn active(&self) -> bool {
        self.query.with(|q| !q.trim().is_empty())
    }

    /// The row with that pick number.
    pub fn row_at(&self, pick: usize) -> Option<Row> {
        untrack(|| {
            self.rows
                .with(|r| r.iter().find(|r| r.pick == Some(pick)).cloned())
        })
    }

    /// Return in a search box: the row picked, else the text run as a command, as Run... does.
    /// `true` when something was done.
    pub fn accept(&self, pick: usize) -> bool {
        if !untrack(|| self.active()) {
            return false;
        }
        let count = untrack(|| self.pick_count.get());
        if count > 0
            && let Some(row) = self.row_at(pick.min(count - 1))
        {
            choose(&row);
            return true;
        }
        crate::apps::run_typed(&self.query.get_untracked())
    }
}

/// Does what a row stands for: starts the program, opens the task's page, or opens the file.
pub(super) fn choose(row: &Row) {
    match &row.found {
        Found::Heading(_) => {}
        Found::Program(app) => crate::apps::launch(app),
        Found::Task { page, .. } => crate::control_panel::open_page(page),
        Found::File(path) => open_file(path),
    }
}

/// Ranks the programs and the Control Panel's tasks in a task each time the query (or the
/// programs) change. An answer to an older query is dropped.
fn rank_queries(typed: Memo<String>, ranked: Signal<Ranked>) {
    let latest = Latest::new();
    effect(move || {
        let query = typed.get();
        let index = crate::apps::program_index();
        let ticket = latest.next();
        if query.is_empty() {
            ranked.set_if_changed(Ranked::default());
            return;
        }
        task(move || rank(&query, &index)).then(move |found| {
            if latest.is_current(ticket) {
                ranked.set_if_changed(found);
            }
        });
    });
}

/// What `typed` (trimmed, not empty) finds. It goes over every program: run it in a task.
fn rank(typed: &str, index: &ProgramIndex) -> Ranked {
    // Fuzzy on the names, best first; then what only the programs' other words (generic name,
    // keywords) find, as the desktop's own search ranks them.
    let by_name: Vec<(Application, Vec<usize>)> =
        fuzzy::rank(typed, index.programs.iter(), |p| p.entry.name.clone())
            .into_iter()
            .map(|m| (m.item.entry.clone(), m.marks))
            .collect();
    let others = apps().search(&typed.to_lowercase());
    let programs = merge_found(by_name, others, |a| a.id.clone(), PROGRAM_LIMIT)
        .into_iter()
        .map(|(a, marks)| (Found::Program(a), marks))
        .collect();
    let tasks = find_tasks(typed)
        .into_iter()
        .take(TASK_LIMIT)
        .map(|f| {
            let found = Found::Task {
                text: f.task.text.to_string(),
                page: f.task.page.to_string(),
            };
            (found, f.marks)
        })
        .collect();
    Ranked {
        query: typed.to_owned(),
        programs,
        tasks,
    }
}

/// The ranked items, then those of `more` not among them (no letters matched), at most `limit`.
fn merge_found<T, K: Hash + Eq>(
    ranked: Vec<(T, Vec<usize>)>,
    more: Vec<T>,
    key: impl Fn(&T) -> K,
    limit: usize,
) -> Vec<(T, Vec<usize>)> {
    let mut seen: HashSet<K> = ranked.iter().map(|(t, _)| key(t)).collect();
    let mut out = ranked;
    for item in more {
        if out.len() >= limit {
            break;
        }
        if seen.insert(key(&item)) {
            out.push((item, Vec::new()));
        }
    }
    out.truncate(limit);
    out
}

/// Looks for the files with `fd` in the home folder, a moment after the last key, for names that
/// hold what is typed, and ranks them in the same task. Hidden folders stay out (fd's default).
/// An answer to an older query is dropped.
fn find_files(typed: Memo<String>, files: Signal<Files>) {
    let pending: Rc<Cell<Option<Timeout>>> = Rc::default();
    effect(move || {
        let wanted = typed.get();
        if let Some(t) = pending.take() {
            t.cancel();
        }
        if wanted.chars().count() < FILE_MIN_CHARS {
            return;
        }
        let Some(home) = home() else {
            return;
        };
        pending.set(Some(set_timeout(FILE_WAIT, move || {
            let pattern = wanted.clone();
            task(move || {
                let out = std::process::Command::new("fd")
                    .args(["--max-results", &FILE_LIMIT.to_string(), "--ignore-case"])
                    // Every name with the typed letters in order: `fuzzy` ranks them.
                    .arg("--")
                    .arg(fuzzy::file_pattern(&pattern))
                    .arg(&home)
                    .output();
                let paths: Vec<String> = match out {
                    Ok(out) => String::from_utf8_lossy(&out.stdout)
                        .lines()
                        .filter(|l| !l.is_empty())
                        .map(|l| l.trim_end_matches('/').to_owned())
                        .collect(),
                    Err(_) => Vec::new(),
                };
                let found = fuzzy::rank(&pattern, paths, |p| file_name(p))
                    .into_iter()
                    .map(|m| (Found::File(m.item), m.marks))
                    .collect();
                Files {
                    query: pattern,
                    found,
                }
            })
            .then(move |found| {
                if files.is_alive() && untrack(|| typed.with(|t| *t == found.query)) {
                    files.set_if_changed(found);
                }
            });
        })));
    });
}

/// The menu of a program, on a right click: open it, and pin it to the taskbar or the Start
/// menu, or take it off. `done` runs after Open.
pub(super) fn program_lines(app: &Application, done: fn()) -> Vec<MenuLine> {
    let pinned = crate::apps::is_pinned(app);
    let on_taskbar = crate::apps::is_taskbar_pinned(app);
    let (open, taskbar, start) = (app.clone(), app.clone(), app.clone());
    vec![
        MenuLine::item("Open", move || {
            crate::apps::launch(&open);
            done();
        }),
        MenuLine::separator(),
        MenuLine::item(
            if on_taskbar {
                "Unpin from Taskbar"
            } else {
                "Pin to Taskbar"
            },
            move || crate::apps::set_taskbar_pinned(&taskbar, !on_taskbar),
        ),
        MenuLine::item(
            if pinned {
                "Unpin from Start Menu"
            } else {
                "Pin to Start Menu"
            },
            move || crate::apps::set_pinned(&start, !pinned),
        ),
    ]
}

/// The rows of a search: a heading over each group, then its rows, the one picked lit; with
/// nothing found while typing, "Run “…”", which Return runs. Choosing a row runs `done`.
/// `program_menus`: a program row has its menu on a right click (the Start menu's).
pub(super) fn results(search: Search, pick: Signal<usize>, done: fn(), program_menus: bool) -> Div {
    // Keyed: a row whose key stays keeps its node, so that only new rows are built.
    let list = div().flex().flex_col().child(for_each(
        move || search.rows.get(),
        row_key,
        move |row| result_row(row, pick, done, program_menus),
    ));
    let run = show(
        move || search.active() && search.pick_count.get() == 0,
        move || {
            menu_row(
                asset("icons/run.png").to_string_lossy().into_owned(),
                move || {
                    format!(
                        "Run \u{201C}{}\u{201D}",
                        search.query.with(|q| q.trim().to_owned())
                    )
                },
                RowLook::default(),
                || true,
            )
            .on_click(move |_| {
                if search.accept(0) {
                    done();
                }
            })
        },
    );
    div().flex().flex_col().child(list).child(run)
}

/// What a row is built from: its kind, what it stands for, its pick and its marks.
type RowKey = (u8, String, Option<usize>, Vec<usize>);

fn row_key(row: &Row) -> RowKey {
    let (kind, what) = match &row.found {
        Found::Heading(title) => (0, (*title).to_owned()),
        Found::Program(app) => (1, format!("{}\n{}", app.id, app.name)),
        Found::Task { page, text } => (2, format!("{page}:{text}")),
        Found::File(path) => (3, path.clone()),
    };
    (kind, what, row.pick, row.marks.clone())
}

fn result_row(row: Row, pick: Signal<usize>, done: fn(), program_menus: bool) -> Div {
    let (icon, label) = match &row.found {
        Found::Heading(title) => (String::new(), (*title).to_owned()),
        Found::Program(app) => (app_icon(app), app.name.clone()),
        Found::Task { text, .. } => (
            asset("icons/control-panel.png")
                .to_string_lossy()
                .into_owned(),
            text.clone(),
        ),
        Found::File(path) => (crate::apps::file_icon(path), file_name(path)),
    };
    let heading = matches!(row.found, Found::Heading(_));
    let mine = row.pick;
    let selected = move || mine.is_some() && mine == Some(pick.get());
    let el = node_ref();
    // The picked row is kept in sight as the keys move the pick.
    effect(move || {
        if selected() {
            el.scroll_into_view();
        }
    });
    let chosen = row.clone();
    let line = menu_row(
        icon,
        label,
        RowLook {
            heading,
            marks: row.marks.clone(),
            ..Default::default()
        },
        selected,
    )
    .node_ref(el);
    if heading {
        return line;
    }
    let line = line.on_click(move |_| {
        choose(&chosen);
        done();
    });
    match (&row.found, program_menus) {
        (Found::Program(app), true) => {
            let app = app.clone();
            with_context_menu(line, move || program_lines(&app, done))
        }
        _ => line,
    }
}

/// The search box's placeholder, in the scheme's own colour: the field's own one is a fainter
/// copy of the text's colour.
pub(super) fn placeholder(
    query: Signal<String>,
    words: &'static str,
    size: impl Fn() -> f32 + 'static,
) -> Div {
    div()
        .absolute()
        .inset(0.0)
        .flex()
        .items_center()
        .overflow_clip()
        .whitespace_nowrap()
        .pointer_events_none()
        .dyn_style(move |s| {
            let s = s
                .font_family(crate::ui::font())
                .text_size(size())
                .text_color(scheme().placeholder);
            if query.with(String::is_empty) {
                s
            } else {
                s.hidden()
            }
        })
        .child(words)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_the_other_matches_after_the_ranked_ones_once() {
        let ranked = vec![("firefox", vec![0, 1]), ("fish", vec![0])];
        let more = vec!["fish", "files", "thunderbird", "fonts"];
        let merged = merge_found(ranked, more, |s| *s, 4);
        assert_eq!(
            merged,
            [
                ("firefox", vec![0, 1]),
                ("fish", vec![0]),
                ("files", vec![]),
                ("thunderbird", vec![]),
            ]
        );
    }

    #[test]
    fn merging_keeps_at_most_the_limit() {
        let ranked = vec![("a", vec![]), ("b", vec![]), ("c", vec![])];
        assert_eq!(merge_found(ranked, vec!["d"], |s| *s, 2).len(), 2);
    }

    #[test]
    fn groups_have_a_heading_and_numbered_rows() {
        let mut rows = Rows::default();
        let file = |p: &str| (Found::File(p.to_owned()), Vec::new());
        rows.group("Empty", Vec::new());
        rows.group("Files", vec![file("/a"), file("/b")]);
        rows.group("More", vec![file("/c")]);
        let picks: Vec<Option<usize>> = rows.out.iter().map(|r| r.pick).collect();
        assert_eq!(picks, [None, Some(0), Some(1), None, Some(2)]);
        assert!(rows.out[0].found == Found::Heading("Files"));
    }
}
