//! Default Programs (QML `DefaultApps` and the page of `ControlPanel.qml`), as Windows 7's:
//!
//! - for each kind of file or link, the program that opens it, from the programs that say in
//!   their `.desktop` file that they can. `xdg-mime` reads and writes the choices
//!   (`~/.config/mimeapps.list`);
//! - the programs the desktop starts (`settings().programs`, `programs::program`): the session's,
//!   an installed program of the right category, or a command typed here.

use super::common::{
    FIELD_HEIGHT, control_row, cp_field, data_dirs, desktop_entry_keys, desktop_file, field_border,
    muted_small, output_of, page_column, tinted_text,
};
use crate::control_panel::{heading, note};
use crate::programs::{program, session_program};
use crate::settings::{change, settings};
use crate::theme::scheme;
use crate::ui::{cp_combo, shell_icon};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use serde_json::Value;
use std::collections::HashMap;

/// The width of the labels' column, of the combo boxes, and of the command's field.
const LABEL_WIDTH: f32 = 120.0;
const COMBO_WIDTH: f32 = 280.0;
const COMMAND_WIDTH: f32 = 420.0;

/// A kind of file or link: the first type decides, and a choice sets all of them.
struct Kind {
    label: &'static str,
    types: &'static [&'static str],
}

const KINDS: &[Kind] = &[
    Kind {
        label: "Web browser",
        types: &[
            "x-scheme-handler/https",
            "x-scheme-handler/http",
            "text/html",
        ],
    },
    Kind {
        label: "E-mail",
        types: &["x-scheme-handler/mailto"],
    },
    Kind {
        label: "Folders",
        types: &["inode/directory"],
    },
    Kind {
        label: "Text files",
        types: &["text/plain"],
    },
    Kind {
        label: "Pictures",
        types: &["image/png", "image/jpeg", "image/webp", "image/gif"],
    },
    Kind {
        label: "Music",
        types: &["audio/mpeg", "audio/flac", "audio/ogg"],
    },
    Kind {
        label: "Videos",
        types: &["video/mp4", "video/x-matroska", "video/webm"],
    },
    Kind {
        label: "PDF documents",
        types: &["application/pdf"],
    },
];

/// A program the desktop starts: its key in `settings().programs`, and the categories of the
/// installed programs offered for it. Clipboard and screenshot have none to offer, only a
/// command.
struct ProgramKind {
    key: &'static str,
    label: &'static str,
    categories: &'static [&'static str],
    note: &'static str,
}

const PROGRAM_KINDS: &[ProgramKind] = &[
    ProgramKind {
        key: "terminal",
        label: "Terminal",
        categories: &["TerminalEmulator"],
        note: "Super+Enter",
    },
    ProgramKind {
        key: "fileManager",
        label: "File manager",
        categories: &["FileManager"],
        note: "Super+E, and the places",
    },
    ProgramKind {
        key: "taskManager",
        label: "Task manager",
        categories: &["Monitor"],
        note: "Ctrl+Shift+Escape",
    },
    ProgramKind {
        key: "systemMonitor",
        label: "System monitor",
        categories: &["Monitor"],
        note: "CPU Meter's button",
    },
    ProgramKind {
        key: "clipboard",
        label: "Clipboard history",
        categories: &[],
        note: "Super+V",
    },
    ProgramKind {
        key: "screenshot",
        label: "Screenshot",
        categories: &[],
        note: "Print Screen",
    },
];

/// The default of a type and the programs that can open it, by `.desktop` file name.
#[derive(Clone, Debug, Default, PartialEq)]
struct Handlers {
    current: String,
    candidates: Vec<String>,
}

impl Handlers {
    /// The programs to choose from, the current one first.
    fn choices(&self) -> Vec<String> {
        let mut list = self.candidates.clone();
        if !self.current.is_empty() && !list.contains(&self.current) {
            list.insert(0, self.current.clone());
        }
        list
    }
}

/// For the first type of each kind: its default (`xdg-mime query default`), and the
/// `.desktop` files that list it, from your applications and the system's.
fn read_handlers() -> HashMap<String, Handlers> {
    let mut files: Vec<(String, String)> = Vec::new();
    for dir in data_dirs() {
        let Ok(entries) = std::fs::read_dir(dir.join("applications")) else {
            continue;
        };
        let mut paths: Vec<_> = entries.flatten().map(|e| e.path()).collect();
        paths.sort();
        for path in paths {
            if path.extension().is_none_or(|e| e != "desktop") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Some(line) = text.lines().find(|l| l.starts_with("MimeType=")) else {
                continue;
            };
            let id = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            files.push((id, line["MimeType=".len()..].to_owned()));
        }
    }
    KINDS
        .iter()
        .map(|kind| {
            let ty = kind.types[0];
            let current = output_of("xdg-mime", &["query", "default", ty])
                .trim()
                .to_owned();
            let mut candidates: Vec<String> = Vec::new();
            for (id, types) in &files {
                if types.split(';').any(|t| t == ty) && !candidates.contains(id) {
                    candidates.push(id.clone());
                }
            }
            (
                ty.to_owned(),
                Handlers {
                    current,
                    candidates,
                },
            )
        })
        .collect()
}

/// Makes a program the default for every type of a kind.
fn set_default(handlers: Signal<HashMap<String, Handlers>>, kind: &Kind, id: String) {
    let types = kind.types;
    task(move || {
        for ty in types.iter().copied() {
            output_of("xdg-mime", &["default", id.as_str(), ty]);
        }
        read_handlers()
    })
    .then(move |read| handlers.set(read));
}

/// What to show for a `.desktop` file name: its program's name, else the name without the
/// extension.
fn name_of(id: &str) -> String {
    apps()
        .get(id)
        .map(|a| a.name)
        .unwrap_or_else(|| id.trim_end_matches(".desktop").to_owned())
}

/// A program offered for a program kind: its name and the command line that starts it.
#[derive(Clone, Debug, PartialEq)]
struct Choice {
    name: String,
    command: String,
}

/// The words of a `.desktop` file's `Exec=`, without the field codes (`%f`, `%U`, ...), as
/// Quickshell's `DesktopEntry.command` gives them.
fn exec_words(id: &str) -> Option<Vec<String>> {
    let text = std::fs::read_to_string(desktop_file(id)?).ok()?;
    let exec = desktop_entry_keys(&text)
        .into_iter()
        .find(|(k, _)| k == "Exec")?
        .1;
    // The string's own escapes first, then the quoting of the arguments.
    let exec = exec
        .replace("\\s", " ")
        .replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\r", "\r")
        .replace("\\\\", "\\");
    let mut words = Vec::new();
    let mut word = String::new();
    let mut started = false;
    let mut quoted = false;
    let mut chars = exec.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                quoted = !quoted;
                started = true;
            }
            '\\' if quoted => {
                if let Some(next) = chars.next() {
                    word.push(next);
                }
            }
            ' ' | '\t' if !quoted => {
                if started {
                    words.push(std::mem::take(&mut word));
                    started = false;
                }
            }
            _ => {
                word.push(c);
                started = true;
            }
        }
    }
    if started {
        words.push(word);
    }
    const CODES: &[&str] = &[
        "%f", "%F", "%u", "%U", "%d", "%D", "%n", "%N", "%i", "%c", "%k", "%v", "%m",
    ];
    Some(
        words
            .into_iter()
            .filter(|w| !CODES.contains(&w.as_str()))
            .map(|w| w.replace("%%", "%"))
            .collect(),
    )
}

/// The command line that starts an installed program, as a shell reads it: its `.desktop`
/// file's command, in the terminal when it says so.
fn command_of(app: &Application) -> Option<String> {
    let words = exec_words(&app.id)?;
    if words.is_empty() {
        return None;
    }
    let plain = |w: &str| {
        !w.is_empty()
            && w.chars()
                .all(|c| c.is_ascii_alphanumeric() || "_/.:=+,@%-".contains(c))
    };
    let line = words
        .iter()
        .map(|w| {
            if plain(w) {
                w.clone()
            } else {
                format!("'{}'", w.replace('\'', "'\\''"))
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    Some(if app.terminal {
        format!("{} -e {line}", session_program("terminal"))
    } else {
        line
    })
}

/// The installed programs a program kind offers, by name.
fn program_choices(kind: &ProgramKind) -> Vec<Choice> {
    if kind.categories.is_empty() {
        return Vec::new();
    }
    let mut out: Vec<Choice> = apps()
        .all()
        .into_iter()
        .filter(|a| {
            a.categories
                .iter()
                .any(|c| kind.categories.contains(&c.as_str()))
        })
        .filter_map(|a| {
            Some(Choice {
                command: command_of(&a)?,
                name: a.name,
            })
        })
        .collect();
    out.sort_by_key(|a| a.name.to_lowercase());
    out
}

/// Chooses a program's command; "" goes back to the session's.
fn set_program(key: &str, command: &str) {
    change(|s| {
        if command.is_empty() {
            s.programs.remove(key);
        } else {
            s.programs
                .insert(key.to_owned(), Value::String(command.to_owned()));
        }
    });
}

pub fn page() -> Div {
    let handlers = signal(HashMap::<String, Handlers>::new());
    task(read_handlers).then(move |read| handlers.set(read));
    page_column(12.0)
        .child(heading(
            "Choose the programs that open your files and links",
        ))
        .child(note(
            "The choice applies to every program that opens files through the desktop's \
             standard (xdg-open), and is kept in ~/.config/mimeapps.list.",
        ))
        .children(
            KINDS
                .iter()
                .map(|kind| kind_row(handlers, kind))
                .collect::<Vec<_>>(),
        )
        .child(heading("Choose the programs the desktop starts"))
        .child(note(
            "The terminal, the task manager and the others the keys and the taskbar start. \
             \"As the session sets it\" keeps the one your system's configuration names.",
        ))
        .children(PROGRAM_KINDS.iter().map(program_row).collect::<Vec<_>>())
}

/// A kind of file: its name, the icon of its program, the programs to choose from, and "Not
/// set" when it has none.
fn kind_row(handlers: Signal<HashMap<String, Handlers>>, kind: &'static Kind) -> Div {
    let info = memo(move || handlers.with(|h| h.get(kind.types[0]).cloned().unwrap_or_default()));
    let icon = move || {
        info.with(|i| {
            if i.current.is_empty() {
                return String::new();
            }
            apps()
                .get(&i.current)
                .and_then(|a| a.icon)
                .unwrap_or_else(|| "application-x-executable".into())
        })
    };
    control_row(12.0)
        .child(
            div()
                .flex_none()
                .w(LABEL_WIDTH)
                .child(tinted_text(kind.label, || scheme().text)),
        )
        .child(shell_icon(icon, 24.0))
        .child(dyn_view(move || {
            let choices = info.with(Handlers::choices);
            let labels = if choices.is_empty() {
                vec!["No program says it can".to_owned()]
            } else {
                choices.iter().map(|id| name_of(id)).collect()
            };
            let current = Binding::new(
                move || {
                    info.with(|i| i.choices().iter().position(|id| *id == i.current))
                        .unwrap_or(0)
                },
                move |index: usize| {
                    if let Some(id) = choices.get(index) {
                        set_default(handlers, kind, id.clone());
                    }
                },
            );
            untrack(|| cp_combo(labels, current).w(COMBO_WIDTH))
        }))
        .child(show(
            move || info.with(|i| i.current.is_empty()),
            || muted_small("Not set"),
        ))
}

/// A program the desktop starts: its name, the choice (the session's, an installed program, or
/// another command), and under it the command that runs, or the field to type one in.
fn program_row(kind: &'static ProgramKind) -> Div {
    let chosen = memo(move || {
        settings().with(|s| {
            s.programs
                .get(kind.key)
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned()
        })
    });
    let choices = memo(move || program_choices(kind));
    // The choice shown: 0 the session's, 1.. an installed program, the last another command.
    let shown = memo(move || {
        let command = chosen.get();
        if command.is_empty() {
            return 0;
        }
        choices.with(|list| {
            list.iter()
                .position(|c| c.command == command)
                .map_or(list.len() + 1, |i| i + 1)
        })
    });
    // "Another command…" was picked: its field shows even before anything is typed.
    let editing = signal(false);
    let custom = move || editing.get() || shown.get() == choices.with(Vec::len) + 1;
    let combo = dyn_view(move || {
        let mut labels = vec!["As the session sets it".to_owned()];
        labels.extend(choices.with(|l| l.iter().map(|c| c.name.clone()).collect::<Vec<_>>()));
        labels.push("Another command\u{2026}".to_owned());
        let current = Binding::new(
            move || {
                if editing.get() {
                    choices.with(Vec::len) + 1
                } else {
                    shown.get()
                }
            },
            move |index: usize| {
                editing.set(false);
                let count = untrack(|| choices.with(Vec::len));
                if index == 0 {
                    set_program(kind.key, "");
                } else if index <= count {
                    let command = untrack(|| choices.with(|l| l[index - 1].command.clone()));
                    set_program(kind.key, &command);
                } else {
                    editing.set(true);
                }
            },
        );
        untrack(|| cp_combo(labels, current).w(COMBO_WIDTH))
    });
    let command_line = show(
        move || !custom(),
        move || {
            div().flex_none().w(COMMAND_WIDTH).overflow_clip().child(
                muted_small(move || {
                    let command = program(kind.key);
                    if command.is_empty() {
                        "None".to_owned()
                    } else {
                        command
                    }
                })
                .whitespace_nowrap(),
            )
        },
    );
    let command_field = show(custom, move || {
        let typed = signal(untrack(|| chosen.get()));
        let input = text_input()
            .value(typed)
            .placeholder("A command, then Enter")
            .on_submit(move |text| {
                set_program(kind.key, text.trim());
                editing.set(false);
            });
        cp_field(input, COMMAND_WIDTH, field_border)
    });
    div()
        .flex()
        .flex_col()
        .gap(4.0)
        .child(
            control_row(12.0)
                .child(
                    div()
                        .flex_none()
                        .w(LABEL_WIDTH)
                        .child(tinted_text(kind.label, || scheme().text)),
                )
                .child(combo)
                .child(muted_small(kind.note)),
        )
        .child(
            control_row(12.0)
                .min_h(FIELD_HEIGHT)
                .child(div().flex_none().w(LABEL_WIDTH))
                .child(command_line)
                .child(command_field),
        )
}
