//! What the Start menu's search box and the launcher find (QML `Search`): the programs, the
//! Control Panel's tasks and the files, as rows ready to show, and the list of those rows.
//!
//! Before anything is typed, the rows are the programs started last, as many as the Control
//! Panel says, then every program by name: the launcher shows them, the Start menu shows its
//! own lists instead. While typing: the programs that match (AntiWebUI's ranking, best first),
//! the Control Panel's tasks, then the files of the home folder whose names hold what is typed.

use super::power::quoted;
use crate::paths::asset;
use crate::settings::{home, settings};
use crate::theme::scheme;
use crate::ui::{MenuLine, RowLook, menu_row, with_context_menu};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use std::cell::Cell;
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
    Task { text: String, page: String },
    /// A file or a folder, by path.
    File(String),
}

/// A row of the results. `pick` numbers the rows that can be chosen; a heading has none.
#[derive(Clone, PartialEq)]
pub(super) struct Row {
    pub found: Found,
    pub pick: Option<usize>,
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

impl Search {
    pub fn new() -> Self {
        let query = signal(String::new());
        let files = signal(Vec::<String>::new());
        find_files(query, files);
        let rows = memo(move || {
            let typed = query.with(|q| q.trim().to_owned());
            let needle = typed.to_lowercase();
            let mut out = Vec::new();
            let mut pick = 0;
            let mut group = |title: &'static str, list: Vec<Found>| {
                if list.is_empty() {
                    return;
                }
                out.push(Row {
                    found: Found::Heading(title),
                    pick: None,
                });
                for found in list {
                    out.push(Row {
                        found,
                        pick: Some(pick),
                    });
                    pick += 1;
                }
            };
            if needle.is_empty() {
                let count = settings().with(|s| s.launcher_recent_count).max(0) as usize;
                group(
                    "Recent",
                    crate::apps::recent_programs()
                        .into_iter()
                        .take(count)
                        .map(Found::Program)
                        .collect(),
                );
                group(
                    "All Programs",
                    apps().all().into_iter().map(Found::Program).collect(),
                );
            } else {
                group(
                    "Programs",
                    apps()
                        .search(&needle)
                        .into_iter()
                        .take(PROGRAM_LIMIT)
                        .map(Found::Program)
                        .collect(),
                );
                group(
                    "Control Panel",
                    crate::control_panel::state::find_tasks(&typed)
                        .into_iter()
                        .take(TASK_LIMIT)
                        .map(|t| Found::Task {
                            text: t.text.to_string(),
                            page: t.page.to_string(),
                        })
                        .collect(),
                );
                group(
                    "Files",
                    files.get().into_iter().map(Found::File).collect(),
                );
            }
            out
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
        untrack(|| self.rows.with(|r| r.iter().find(|r| r.pick == Some(pick)).cloned()))
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

/// Looks for the files with `fd` in the home folder, a moment after the last key, for names that
/// hold what is typed. Hidden folders stay out (fd's default). An answer to an older query is
/// dropped.
fn find_files(query: Signal<String>, files: Signal<Vec<String>>) {
    let pending: Rc<Cell<Option<Timeout>>> = Rc::default();
    effect(move || {
        let wanted = query.with(|q| q.trim().to_owned());
        files.set(Vec::new());
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
                    .args(["--fixed-strings", "--"])
                    .arg(&pattern)
                    .arg(&home)
                    .output();
                let Ok(out) = out else {
                    return Vec::new();
                };
                String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(|l| l.trim_end_matches('/').to_owned())
                    .collect::<Vec<_>>()
            })
            .then(move |found| {
                if query.is_alive() && query.with_untracked(|q| q.trim() == wanted) {
                    files.set(found);
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
            if on_taskbar { "Unpin from Taskbar" } else { "Pin to Taskbar" },
            move || crate::apps::set_taskbar_pinned(&taskbar, !on_taskbar),
        ),
        MenuLine::item(
            if pinned { "Unpin from Start Menu" } else { "Pin to Start Menu" },
            move || crate::apps::set_pinned(&start, !pinned),
        ),
    ]
}

/// The rows of a search: a heading over each group, then its rows, the one picked lit; with
/// nothing found while typing, "Run “…”", which Return runs. Choosing a row runs `done`.
/// `program_menus`: a program row has its menu on a right click (the Start menu's).
pub(super) fn results(
    search: Search,
    pick: Signal<usize>,
    done: fn(),
    program_menus: bool,
) -> Div {
    let list = dyn_view(move || {
        let rows = search.rows.get();
        div()
            .flex()
            .flex_col()
            .children(
                rows.into_iter()
                    .map(|row| result_row(row, pick, done, program_menus))
                    .collect::<Vec<_>>(),
            )
    });
    let run = show(
        move || search.active() && search.pick_count.get() == 0,
        move || {
            menu_row(
                asset("icons/run.png").to_string_lossy().into_owned(),
                move || format!("Run \u{201C}{}\u{201D}", search.query.with(|q| q.trim().to_owned())),
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

fn result_row(row: Row, pick: Signal<usize>, done: fn(), program_menus: bool) -> Div {
    let (icon, label) = match &row.found {
        Found::Heading(title) => (String::new(), (*title).to_owned()),
        Found::Program(app) => (app_icon(app), app.name.clone()),
        Found::Task { text, .. } => (
            asset("icons/control-panel.png").to_string_lossy().into_owned(),
            text.clone(),
        ),
        Found::File(path) => (
            crate::apps::file_icon(path),
            file_name(path),
        ),
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
            if query.with(String::is_empty) { s } else { s.hidden() }
        })
        .child(words)
}
