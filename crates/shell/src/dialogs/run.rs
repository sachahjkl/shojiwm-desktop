//! The Run dialog (Super+R), as Windows 7 shows it above the Start button (QML `RunDialog`,
//! `RunState`): the glass frame with its caption, the page with the run icon and the sentence,
//! the Open field with the history under its arrow, then OK and Cancel.
//!
//! The field starts with the last command, selected. Typing completes from the history; Down,
//! F4 or the arrow opens the history; Return runs, Escape closes the list, then the dialog. What
//! a line does follows from its first word: an address opens, an executable file or a program
//! of `PATH` runs (the whole line, through the shell), another path that exists opens. A
//! command whose program cannot be found stays in the field, with a message under it.

use super::frame::{button_band, dialog_frame, dialog_page};
use super::run_close;
use crate::paths::{asset, state_dir};
use crate::theme::{bar_at_top, scheme, theme};
use crate::ui::{cp_button, dark_look, font, ink_asset};
use anti_webui::prelude::*;
use serde_json::Value;
use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

/// How many commands the history keeps, as Windows' RunMRU.
const HISTORY_KEPT: usize = 26;
/// The width of the dialog.
const WIDTH: f32 = 420.0;
/// How far the dialog keeps from the bar and from the screen's left edge.
const MARGIN: f32 = 12.0;
/// The height of the field, and of a row of the history.
const FIELD_HEIGHT: f32 = 24.0;
const HISTORY_ROW_HEIGHT: f32 = 20.0;
/// The width of the field's arrow.
const ARROW_WIDTH: f32 = 18.0;
/// The arrow under the pointer or while the list is open.
const ARROW_HOT_DARK: Color = rgb(0x2a4660);
const ARROW_HOT_LIGHT: Color = rgb(0xe3f4fc);
/// The chosen row of the history, its text, and the selection in the field: Windows' blue.
const PICKED: Color = rgb(0x3399ff);
const PICKED_TEXT: Color = Color::WHITE;
/// The "Cannot find" message.
const NOT_FOUND_DARK: Color = rgb(0xff8a80);
const NOT_FOUND_LIGHT: Color = rgb(0xc42b1c);
/// The band of buttons.
const BAND_DARK: Color = rgb(0x1b242d);
const BAND_LIGHT: Color = rgb(0xf0f0f0);

const SENTENCE: &str = "Type the name of a program, folder, document, or Internet resource, and it will be opened for you.";

// ----- the history -------------------------------------------------------------------------------

fn history_path() -> PathBuf {
    state_dir().join("run-history.json")
}

/// The commands run from the dialog, last first.
fn load_history() -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(history_path()) else {
        return Vec::new();
    };
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Array(list)) => list
            .into_iter()
            .map(|v| match v {
                Value::String(s) => s,
                other => other.to_string(),
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Puts `command` first in the history, once, and writes it out.
fn remember(command: &str) {
    let mut list = load_history();
    list.retain(|entry| entry != command);
    list.insert(0, command.to_owned());
    list.truncate(HISTORY_KEPT);
    let path = history_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, Value::from(list).to_string());
}

/// The most recent command that starts with `prefix`, whatever the case, and is longer.
fn completion(history: &[String], prefix: &str) -> Option<String> {
    if prefix.is_empty() {
        return None;
    }
    let lower = prefix.to_lowercase();
    let length = prefix.chars().count();
    history
        .iter()
        .find(|entry| entry.chars().count() > length && entry.to_lowercase().starts_with(&lower))
        .cloned()
}

// ----- running -----------------------------------------------------------------------------------

/// Prints what to do with the first word: open, run or missing.
const DECIDE: &str = r#"w=$1
case $w in "~") w=$HOME ;; "~/"*) w=$HOME/${w#"~/"} ;; esac
case $w in
  [a-zA-Z]*:*) echo open ;;
  /*|./*|../*|"$HOME"*)
    if [ -f "$w" ] && [ -x "$w" ]; then echo run
    elif [ -e "$w" ]; then echo open
    else echo missing; fi ;;
  *) if command -v -- "$w" >/dev/null; then echo run; else echo missing; fi ;;
esac"#;

fn home() -> PathBuf {
    crate::settings::home().unwrap_or_else(|| PathBuf::from("/"))
}

/// `~` and `~/...` in the home folder, for xdg-open.
fn expand_home(text: &str) -> String {
    let home = home().to_string_lossy().into_owned();
    if text == "~" {
        home
    } else if let Some(rest) = text.strip_prefix("~/") {
        format!("{home}/{rest}")
    } else {
        text.to_owned()
    }
}

/// Starts a program from the home folder, detached from the shell.
fn spawn_detached(program: &str, args: &[&str]) {
    let _ = std::process::Command::new("setsid")
        .arg("-f")
        .arg(program)
        .args(args)
        .current_dir(home())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

/// What to do with the line, from its first word: "open", "run" or "missing".
fn decide(line: &str) -> Task<String> {
    let word = line.split_whitespace().next().unwrap_or("").to_owned();
    task(move || {
        std::process::Command::new("sh")
            .args(["-c", DECIDE, "sh", &word])
            .current_dir(home())
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
            .unwrap_or_else(|_| "missing".into())
    })
}

// ----- the dialog --------------------------------------------------------------------------------

/// What the dialog holds while it is open.
#[derive(Clone, Copy)]
struct State {
    history: Signal<Vec<String>>,
    typed: Signal<String>,
    list_open: Signal<bool>,
    /// The history row picked with the keys or the pointer while the list is open.
    pick: Signal<usize>,
    /// The last line that could not be found, for the message.
    not_found: Signal<String>,
    field: NodeRef,
}

impl State {
    /// Puts `text` in the field, all of it selected, and gives the field the keyboard.
    fn fill(&self, text: &str, last_length: &Cell<usize>) {
        self.field.set_text(text, (0, text.len()));
        self.typed.set(text.to_owned());
        last_length.set(text.chars().count());
        self.field.focus();
    }

    fn choose(&self, index: usize, last_length: &Cell<usize>) {
        let Some(entry) = self.history.with_untracked(|h| h.get(index).cloned()) else {
            return;
        };
        self.list_open.set(false);
        self.fill(&entry, last_length);
    }

    /// Opens or closes the history, on its first row.
    fn toggle_list(&self) {
        self.pick.set(0);
        let any = self.history.with_untracked(|h| !h.is_empty());
        self.list_open.update(|open| *open = !*open && any);
    }
}

/// The dialog over its screen: a click outside of it closes it.
pub fn view() -> Div {
    let history = load_history();
    let first = history.first().cloned().unwrap_or_default();
    let state = State {
        history: signal(history),
        typed: signal(first.clone()),
        list_open: signal(false),
        pick: signal(0usize),
        not_found: signal(String::new()),
        field: node_ref(),
    };
    // The length before the last edit: completion follows typing, not deleting.
    let last_length = Rc::new(Cell::new(first.chars().count()));
    {
        let last_length = last_length.clone();
        request_frame(move |_| state.fill(&first, &last_length));
    }
    // A second Run while the first is checked waits for it.
    let checking = Rc::new(Cell::new(false));
    let accept = Rc::new(move || {
        state.list_open.set(false);
        let line = state.typed.get_untracked().trim().to_owned();
        if line.is_empty() || checking.get() {
            return;
        }
        state.not_found.set(String::new());
        checking.set(true);
        let checking = checking.clone();
        decide(&line).then(move |verdict| {
            checking.set(false);
            match verdict.as_str() {
                "missing" => state.not_found.set(line),
                verdict => {
                    remember(&line);
                    if verdict == "open" {
                        spawn_detached("xdg-open", &[&expand_home(&line)]);
                    } else {
                        spawn_detached("sh", &["-c", &line]);
                    }
                    run_close();
                }
            }
        });
    });

    let page = dialog_page()
        .child(
            div()
                .flex()
                .flex_col()
                .gap(14.0)
                .pt(16.0)
                .px(14.0)
                .pb(14.0)
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(14.0)
                        .child(
                            img(asset("icons/run.png"))
                                .flex_none()
                                .size(32.0)
                                .object_fit(ObjectFit::Contain),
                        )
                        .child(div().flex_1().min_w(0.0).child(SENTENCE)),
                )
                .child(open_row(state, last_length, accept.clone()))
                .child(not_found_line(state)),
        )
        .child(button_band(
            (BAND_DARK, BAND_LIGHT),
            None,
            fragment([
                cp_button("OK", {
                    let accept = accept.clone();
                    move || accept()
                })
                .into_view(),
                cp_button("Cancel", run_close).into_view(),
            ]),
        ));

    let frame = dialog_frame("Run", WIDTH, page)
        .absolute()
        .left(MARGIN)
        .dyn_style(|s| {
            let away = theme().taskbar_height + MARGIN;
            if bar_at_top() {
                s.top(away)
            } else {
                s.bottom(away)
            }
        })
        // A press in the window keeps it, and closes the list.
        .on_pointer_down(move |e| {
            e.stop_propagation();
            state.list_open.set(false);
        });

    div()
        .size_full()
        .relative()
        .on_pointer_down(|_| run_close())
        .child(frame)
}

/// "Open:" and the field, with its arrow and the history under it.
fn open_row(state: State, last_length: Rc<Cell<usize>>, accept: Rc<dyn Fn()>) -> Div {
    let field_ui = Interaction::new();
    let keys_length = last_length.clone();
    let list_length = last_length.clone();
    let field = text_input()
        .value(state.typed)
        .node_ref(state.field)
        .interaction(field_ui)
        .flex_1()
        .min_w(0.0)
        .ml(5.0)
        .mr(4.0)
        .dyn_style(|s| {
            s.font_family(font())
                .text_size(theme().font_size)
                .text_color(scheme().text)
                .selection_color(PICKED)
        })
        .on_input(move |typed| {
            state.not_found.set(String::new());
            let length = typed.chars().count();
            let grew = length > last_length.get();
            last_length.set(length);
            if !grew {
                return;
            }
            let Some(found) = state.history.with_untracked(|h| completion(h, typed)) else {
                return;
            };
            // The rest of the command, selected: typing on replaces it.
            let cut = found
                .char_indices()
                .nth(length)
                .map_or(found.len(), |(at, _)| at);
            state.field.set_text(&found, (cut, found.len()));
            state.typed.set(found);
        })
        .on_key_down(move |e| {
            let count = state.history.with_untracked(Vec::len);
            let open = state.list_open.get_untracked();
            match e.key {
                Key::Named(NamedKey::Enter) => {
                    if open {
                        state.choose(state.pick.get_untracked(), &keys_length);
                    } else {
                        accept();
                    }
                }
                Key::Named(NamedKey::Escape) => {
                    if open {
                        state.list_open.set(false);
                    } else {
                        run_close();
                    }
                }
                Key::Named(NamedKey::ArrowUp) => {
                    if open {
                        state.pick.update(|p| *p = p.saturating_sub(1));
                    }
                }
                Key::Named(NamedKey::ArrowDown) => {
                    if open {
                        state
                            .pick
                            .update(|p| *p = (*p + 1).min(count.saturating_sub(1)));
                    } else {
                        state.pick.set(0);
                        state.list_open.set(count > 0);
                    }
                }
                Key::Named(NamedKey::F(4)) => state.toggle_list(),
                _ => return,
            }
            e.prevent_default();
        });

    let arrow_ui = Interaction::new();
    let arrow = div()
        .flex_none()
        .w(ARROW_WIDTH)
        .self_stretch()
        .m(1.0)
        .flex()
        .items_center()
        .justify_center()
        .interaction(arrow_ui)
        .on_pointer_down(move |e| {
            e.stop_propagation();
            state.toggle_list();
            state.field.focus();
        })
        .dyn_style(move |s| {
            if arrow_ui.hovered() || state.list_open.get() {
                s.bg(if dark_look() {
                    ARROW_HOT_DARK
                } else {
                    ARROW_HOT_LIGHT
                })
            } else {
                s.bg(Color::TRANSPARENT)
            }
        })
        .child(
            img(|| ink_asset("combo-arrow.svg"))
                .w(7.0)
                .h(4.0)
                .object_fit(ObjectFit::Contain),
        );

    let combo = div()
        .flex()
        .flex_row()
        .items_center()
        .size_full()
        .border(1.0)
        .dyn_style(move |s| {
            let c = scheme();
            s.bg(c.field).border_color(if field_ui.focused() {
                c.field_focus
            } else {
                c.field_border
            })
        })
        .child(field)
        .child(arrow);

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(10.0)
        .child(
            div()
                .flex_none()
                .h(FIELD_HEIGHT)
                .flex()
                .items_center()
                .child("Open:"),
        )
        .child(
            div()
                .relative()
                .flex_1()
                .min_w(0.0)
                .h(FIELD_HEIGHT)
                .child(combo)
                .child(history_list(state, list_length)),
        )
}

/// The history, under the field and over the rest of the page, while it is open.
fn history_list(state: State, last_length: Rc<Cell<usize>>) -> Show {
    show(
        move || state.list_open.get(),
        move || {
            let last_length = last_length.clone();
            div()
                .absolute()
                .left(0.0)
                .right(0.0)
                .top(FIELD_HEIGHT)
                .z_index(10)
                .flex()
                .flex_col()
                .p(1.0)
                .border(1.0)
                .dyn_style(|s| {
                    let c = scheme();
                    s.bg(c.field).border_color(c.field_border)
                })
                .child(dyn_view(move || {
                    let rows: Vec<Div> = state
                        .history
                        .get()
                        .into_iter()
                        .enumerate()
                        .map(|(i, entry)| history_row(state, i, entry, last_length.clone()))
                        .collect();
                    div().flex().flex_col().children(rows)
                }))
        },
    )
}

fn history_row(state: State, index: usize, entry: String, last_length: Rc<Cell<usize>>) -> Div {
    let picked = move || state.pick.get() == index;
    div()
        .flex_none()
        .flex()
        .items_center()
        .h(HISTORY_ROW_HEIGHT)
        .pl(5.0)
        .overflow_clip()
        .whitespace_nowrap()
        .on_hover(move |on| {
            if on {
                state.pick.set(index);
            }
        })
        // On the press: the frame closes the list on it.
        .on_pointer_down(move |e| {
            e.stop_propagation();
            state.choose(index, &last_length);
        })
        .dyn_style(move |s| {
            if picked() {
                s.bg(PICKED).text_color(PICKED_TEXT)
            } else {
                s.bg(Color::TRANSPARENT).text_color(scheme().text)
            }
        })
        .child(entry)
}

/// "Cannot find …", while the line that could not be found is still in the field.
fn not_found_line(state: State) -> Div {
    div()
        .pl(50.0)
        .dyn_style(move |s| {
            let shown = state.not_found.with(|missing| {
                !missing.is_empty() && state.typed.with(|typed| typed.trim() == missing.as_str())
            });
            let s = s.text_color(if dark_look() {
                NOT_FOUND_DARK
            } else {
                NOT_FOUND_LIGHT
            });
            if shown { s } else { s.hidden() }
        })
        .child(text(move || {
            let missing = state.not_found.get();
            let word = missing.split_whitespace().next().unwrap_or("");
            format!("Cannot find \u{201c}{word}\u{201d}. Check the name, then try again.")
        }))
}
