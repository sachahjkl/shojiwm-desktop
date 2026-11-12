//! The Run dialog (Super+R), as Windows 7 shows it above the Start button (QML `RunDialog`,
//! `RunState`): the glass frame with its caption, the page with the run icon and the sentence,
//! the Open field with the history under its arrow, then OK and Cancel.
//!
//! The field starts with the last command, selected. Typing opens the suggestions under it
//! (`completion`): the history, the programs, the commands of `PATH`, the names of a path; the
//! first one that continues the text completes it inline, the rest selected. Down and Up move in
//! the suggestions, Tab (or Right on a picked one) puts one in the field, Return runs the picked
//! one, else the text. Without suggestions, Down, F4 or the arrow opens the history. Escape
//! closes the list, then the dialog. What
//! a line does follows from its first word: an address opens, an executable file or a program
//! of `PATH` runs (the whole line, through the shell), another path that exists opens. A
//! command whose program cannot be found stays in the field, with a message under it.
//!
//! A drag on the caption moves the dialog, as a window, and it opens there again on that screen
//! (`place`). The suggestions open over the field when their rows do not fit under it; the best
//! one is then at the bottom, next to the field, and Up goes on to the next one.

mod completion;
mod place;

use super::frame::{button_band, caption_grip, dialog_frame, dialog_page};
use super::run_close;
use crate::paths::{asset, state_dir};
use crate::theme::{bar_at_top, item_radius, scheme, theme};
use crate::ui::{cp_button, dark_look, font, ink_asset, marked_text, popup_frame, shell_icon};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use completion::{Entry, Kind, Suggestion};
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

/// Starts a program from the home folder, in a scope of its own (`crate::spawn`).
fn spawn_detached(program: &str, args: &[&str]) {
    let argv: Vec<String> = std::iter::once(program)
        .chain(args.iter().copied())
        .map(str::to_owned)
        .collect();
    crate::spawn::start_program(&argv, Some(&home()));
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
    /// What was typed, without the inline completion: what the suggestions match.
    query: Signal<String>,
    suggestions: Signal<Vec<Suggestion>>,
    /// The suggestion picked with the keys or the pointer; none until then.
    chosen: Signal<Option<usize>>,
    /// Escape closed the suggestions: the next edit opens them again.
    hushed: Signal<bool>,
    /// The commands of `PATH`, read once in a task when the dialog opens.
    commands: Signal<Rc<Vec<String>>>,
    /// The names of the folder last read for a path, and the folder being read.
    listing: Signal<Option<(PathBuf, Rc<Vec<Entry>>)>>,
    listing_wanted: Signal<Option<PathBuf>>,
    /// The field with its arrow, for where it is on the screen.
    combo: NodeRef,
    /// The size of the screen, in logical pixels: the layer covers all of it.
    screen: (f32, f32),
    /// The suggestions open under the field, else over it.
    below: Signal<bool>,
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
        self.hushed.set(true);
        self.fill(&entry, last_length);
    }

    /// Opens or closes the history, on its first row.
    fn toggle_list(&self) {
        self.pick.set(0);
        let any = self.history.with_untracked(|h| !h.is_empty());
        self.list_open.update(|open| *open = !*open && any);
    }

    /// The suggestions show: something matches, and neither the history nor Escape hides them.
    fn suggesting(&self) -> bool {
        !self.list_open.get() && !self.hushed.get() && self.suggestions.with(|s| !s.is_empty())
    }

    /// Finds the suggestions of the query again, none picked.
    fn refresh(&self) {
        if !self.query.is_alive() {
            return;
        }
        let query = self.query.get_untracked();
        let mut found = if query.trim().is_empty() {
            Vec::new()
        } else if let Some((dir, start)) = completion::split_path(&query) {
            let folder = completion::resolve_dir(dir, &home());
            let entries = self.listing.with_untracked(|l| match l {
                Some((read, entries)) if *read == folder => Some(entries.clone()),
                _ => None,
            });
            match entries {
                Some(entries) => completion::path_suggestions(dir, start, &entries),
                None => {
                    self.read_folder(folder);
                    Vec::new()
                }
            }
        } else {
            completion::rank(&query, self.candidates())
        };
        found.retain(|s| s.app.is_some() || s.text != query);
        self.choose_side();
        self.chosen.set(None);
        self.suggestions.set(found);
    }

    /// Opens the suggestions under the field when the most rows they show fit there, else over
    /// it. The most rows, not the rows now: the side stays the same while typing.
    fn choose_side(&self) {
        let Some(field) = self.combo.logical_rect() else {
            return;
        };
        let t = untrack(theme);
        let list = place::list_height(completion::SHOWN, t.popup_item_height);
        self.below.set(place::list_below(
            field.y,
            field.y + field.height,
            list,
            self.screen.1,
            t.taskbar_height,
            untrack(bar_at_top),
        ));
    }

    /// Moves the pick in the suggestions: `further` to the next one, else back to the field.
    fn step(&self, further: bool) {
        let chosen = self.chosen.get_untracked();
        let shown = self.suggestions.with_untracked(Vec::len);
        self.chosen.set(if further {
            Some(chosen.map_or(0, |i| (i + 1).min(shown.saturating_sub(1))))
        } else {
            chosen.and_then(|i| i.checked_sub(1))
        });
    }

    /// The history, the programs, then the commands of `PATH`, in that order for the ties.
    fn candidates(&self) -> Vec<Suggestion> {
        let commands = self.commands.get_untracked();
        let on_path = |name: &str| commands.binary_search_by(|c| c.as_str().cmp(name)).is_ok();
        let mut out: Vec<Suggestion> = self
            .history
            .get_untracked()
            .into_iter()
            .map(|entry| Suggestion::plain(Kind::Recent, entry))
            .collect();
        for app in untrack(crate::apps::applications) {
            let Some(exec) = crate::apps::exec_line(&app) else {
                continue;
            };
            out.push(Suggestion {
                kind: Kind::Application,
                label: app.name.clone(),
                text: completion::app_command(&exec, on_path),
                marks: Vec::new(),
                app: Some(app),
            });
        }
        out.extend(
            commands
                .iter()
                .map(|name| Suggestion::plain(Kind::Command, name.clone())),
        );
        out
    }

    /// Reads the names of `folder` in a task, then finds the suggestions again, unless another
    /// folder is wanted by then.
    fn read_folder(&self, folder: PathBuf) {
        if self.listing_wanted.get_untracked().as_ref() == Some(&folder) {
            return;
        }
        self.listing_wanted.set(Some(folder.clone()));
        let state = *self;
        let wanted = folder.clone();
        task(move || completion::list_dir(&wanted)).then(move |entries| {
            if !state.query.is_alive()
                || state.listing_wanted.get_untracked().as_ref() != Some(&folder)
            {
                return;
            }
            state.listing_wanted.set(None);
            state.listing.set(Some((folder, Rc::new(entries))));
            state.refresh();
        });
    }

    /// Puts a suggestion in the field, the caret at its end, and finds the suggestions of it:
    /// the names inside a folder.
    fn take(&self, index: usize, last_length: &Cell<usize>) {
        let Some(text) = self
            .suggestions
            .with_untracked(|s| s.get(index).map(|s| s.text.clone()))
        else {
            return;
        };
        self.field.set_text(&text, (text.len(), text.len()));
        self.typed.set(text.clone());
        last_length.set(text.chars().count());
        self.query.set(text);
        self.hushed.set(false);
        self.refresh();
    }
}

/// The dialog over its screen: a click outside of it closes it.
pub fn view(output: Output) -> Div {
    let screen = untrack(|| {
        output
            .current()
            .map_or(output.logical_size, |o| o.logical_size)
    });
    let screen = (screen.width as f32, screen.height as f32);
    let history = load_history();
    let first = history.first().cloned().unwrap_or_default();
    let state = State {
        history: signal(history),
        typed: signal(first.clone()),
        list_open: signal(false),
        pick: signal(0usize),
        not_found: signal(String::new()),
        field: node_ref(),
        query: signal(String::new()),
        suggestions: signal(Vec::new()),
        chosen: signal(None),
        hushed: signal(false),
        commands: signal(Rc::new(Vec::new())),
        listing: signal(None),
        listing_wanted: signal(None),
        combo: node_ref(),
        screen,
        below: signal(true),
    };
    task(|| completion::scan_commands(&completion::path_dirs())).then(move |names| {
        if state.query.is_alive() {
            state.commands.set(Rc::new(names));
            if !state.hushed.get_untracked() && !state.query.with_untracked(String::is_empty) {
                state.refresh();
            }
        }
    });
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
        state.hushed.set(true);
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

    // Where it is: the place kept for this screen, else above the Start button. A drag on the
    // caption moves it and keeps the new place.
    let name = output.name.clone();
    let at: Signal<Option<(f32, f32)>> = signal(place::load(&name));
    let frame_ref = node_ref();
    let grab_offset: Rc<Cell<(f32, f32)>> = Rc::default();
    let grabbed = move |e: &GrabEvent| {
        let scale = window_scale().as_f32();
        let pointer = (e.position.x / scale, e.position.y / scale);
        let Some(rect) = frame_ref.logical_rect() else {
            return;
        };
        match e.phase {
            GrabPhase::Start => {
                grab_offset.set((pointer.0 - rect.x, pointer.1 - rect.y));
            }
            GrabPhase::Move => {
                let off = grab_offset.get();
                at.set(Some(place::clamp(
                    (pointer.0 - off.0, pointer.1 - off.1),
                    (rect.width, rect.height),
                    screen,
                )));
            }
            GrabPhase::End | GrabPhase::Cancel => {
                if let Some(now) = at.get_untracked() {
                    place::save(&name, now);
                }
                // The field takes the keys back, once the press is over: typing goes on where
                // it was.
                let field = state.field;
                request_frame(move |_| field.focus());
            }
        }
    };
    let frame = dialog_frame("Run", WIDTH, page)
        .child(caption_grip(grabbed))
        .node_ref(frame_ref)
        .absolute()
        .dyn_style(move |s| match at.get() {
            Some((x, y)) => s.left(x).top(y),
            None => {
                let away = theme().taskbar_height + MARGIN;
                let s = s.left(MARGIN);
                if bar_at_top() {
                    s.top(away)
                } else {
                    s.bottom(away)
                }
            }
        })
        // A press in the window keeps it, and closes the lists.
        .on_pointer_down(move |e| {
            e.stop_propagation();
            state.list_open.set(false);
            state.hushed.set(true);
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
    // Runs a suggestion: an application as the Start menu starts it, anything else as if typed.
    let run_suggestion: Rc<dyn Fn(usize)> = {
        let last_length = last_length.clone();
        let accept = accept.clone();
        Rc::new(move |index| {
            let Some(chosen) = state.suggestions.with_untracked(|s| s.get(index).cloned()) else {
                return;
            };
            match &chosen.app {
                Some(app) => {
                    remember(&chosen.text);
                    crate::apps::launch(app);
                    run_close();
                }
                None => {
                    state.fill(&chosen.text, &last_length);
                    accept();
                }
            }
        })
    };
    let keys_run = run_suggestion.clone();
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
            state.list_open.set(false);
            state.hushed.set(false);
            state.query.set(typed.to_owned());
            state.refresh();
            // Completion follows typing, not deleting.
            if !grew {
                return;
            }
            let Some(found) = state
                .suggestions
                .with_untracked(|s| completion::inline(typed, s))
            else {
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
            let suggesting = untrack(|| state.suggesting());
            let chosen = state.chosen.get_untracked();
            match e.key {
                Key::Named(NamedKey::Enter) => {
                    if open {
                        state.choose(state.pick.get_untracked(), &keys_length);
                    } else if suggesting && let Some(index) = chosen {
                        keys_run(index);
                    } else {
                        accept();
                    }
                }
                Key::Named(NamedKey::Escape) => {
                    if open {
                        state.list_open.set(false);
                    } else if suggesting {
                        state.hushed.set(true);
                    } else {
                        run_close();
                    }
                }
                Key::Named(NamedKey::ArrowUp) => {
                    if open {
                        state.pick.update(|p| *p = p.saturating_sub(1));
                    } else if suggesting {
                        // Over the field the list is upside down: Up goes on to the next one.
                        state.step(!state.below.get_untracked());
                    }
                }
                Key::Named(NamedKey::ArrowDown) => {
                    if open {
                        state
                            .pick
                            .update(|p| *p = (*p + 1).min(count.saturating_sub(1)));
                    } else if suggesting {
                        state.step(state.below.get_untracked());
                    } else {
                        state.pick.set(0);
                        state.list_open.set(count > 0);
                    }
                }
                Key::Named(NamedKey::Tab) if !open && suggesting => {
                    state.take(chosen.unwrap_or(0), &keys_length);
                }
                Key::Named(NamedKey::ArrowRight) if !open && suggesting && chosen.is_some() => {
                    state.take(chosen.unwrap_or(0), &keys_length);
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
        .node_ref(state.combo)
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
                .child(history_list(state, list_length))
                .child(suggestion_list(state, run_suggestion)),
        )
}

/// The size of a suggestion's icon.
const SUGGESTION_ICON: f32 = 16.0;

/// The suggestions, under the field and over the rest of the page, in a popup's frame.
fn suggestion_list(state: State, run: Rc<dyn Fn(usize)>) -> Show {
    show(
        move || state.suggesting(),
        move || {
            let run = run.clone();
            popup_frame()
                .absolute()
                .left(0.0)
                .right(0.0)
                .dyn_style(move |s| {
                    if state.below.get() {
                        s.top(FIELD_HEIGHT + place::LIST_GAP)
                    } else {
                        s.bottom(FIELD_HEIGHT + place::LIST_GAP)
                    }
                })
                .z_index(10)
                .px(1.0)
                .py(2.0)
                // A press between the rows keeps the list.
                .on_pointer_down(|e| e.stop_propagation())
                .child(dyn_view(move || {
                    let rows: Vec<Div> = state
                        .suggestions
                        .get()
                        .into_iter()
                        .enumerate()
                        .map(|(i, s)| suggestion_row(state, i, s, run.clone()))
                        .collect();
                    // Over the field, the best one is at the bottom, next to it.
                    let column = div().flex();
                    if state.below.get() {
                        column.flex_col()
                    } else {
                        column.flex_col_reverse()
                    }
                    .children(rows)
                }))
        },
    )
}

/// The icon of a suggestion: the program's, a command's, the run icon for the history, a
/// folder's or a file's.
fn suggestion_icon(s: &Suggestion) -> String {
    let file = |name: &str| asset(name).to_string_lossy().into_owned();
    match s.kind {
        Kind::Application => crate::apps::entry_icon(s.app.as_ref()),
        Kind::Command => crate::apps::FALLBACK_ICON.to_owned(),
        Kind::Recent => file("icons/run.png"),
        Kind::Folder => file("icons/folder.png"),
        Kind::File => crate::apps::file_icon(&s.text),
    }
}

/// A suggestion: its icon, its label with the matched letters marked, and its kind, muted.
fn suggestion_row(state: State, index: usize, s: Suggestion, run: Rc<dyn Fn(usize)>) -> Div {
    let lit = move || state.chosen.get() == Some(index);
    div()
        .flex_none()
        .flex()
        .flex_row()
        .items_center()
        .gap(7.0)
        .mx(1.0)
        .px(4.0)
        .border(1.0)
        .on_hover(move |on| {
            if on {
                state.chosen.set(Some(index));
            }
        })
        // On the press: the frame closes the lists on it.
        .on_pointer_down(move |e| {
            e.stop_propagation();
            run(index);
        })
        .dyn_style(move |st| {
            let t = theme();
            let c = scheme();
            let st = st.h(t.popup_item_height).rounded(item_radius());
            if lit() {
                st.bg(c.selection).border_color(c.selection_border)
            } else {
                st.bg(Color::TRANSPARENT).border_color(Color::TRANSPARENT)
            }
        })
        .child(shell_icon(suggestion_icon(&s), SUGGESTION_ICON))
        .child(
            div()
                .flex_1()
                .min_w(0.0)
                .overflow_clip()
                .whitespace_nowrap()
                .dyn_style(|st| {
                    st.font_family(font())
                        .text_size(theme().font_size)
                        .text_color(scheme().text)
                })
                .child(marked_text(&s.label, &s.marks)),
        )
        .child(
            div()
                .flex_none()
                .whitespace_nowrap()
                .dyn_style(|st| {
                    st.font_family(font())
                        .text_size(theme().font_size_small)
                        .text_color(scheme().text_muted)
                })
                .child(s.kind.word()),
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
