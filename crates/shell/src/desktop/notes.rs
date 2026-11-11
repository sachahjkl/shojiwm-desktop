//! The sticky notes of the desktop (QML `StickyNotes`, `StickyNote`), as Windows 7's Sticky
//! Notes: each has its screen, its place, its size and its text, kept in
//! `$XDG_STATE_HOME/shojiwm-win7/notes.json`.
//!
//! A note is a pale yellow sheet with a band at the top. Drag the band to move it, its corner to
//! size it. The band's + adds another note beside it and its × removes this one. The text is
//! kept a moment after the last key.

use crate::paths::state_dir;
use crate::theme::corner;
use crate::ui::{font, vertical_gradient};
use anti_webui::prelude::*;
use serde_json::{Value, json};
use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// The size of a new note.
const NEW_WIDTH: f32 = 220.0;
const NEW_HEIGHT: f32 = 200.0;
/// The smallest a note can be made.
const MIN_WIDTH: f32 = 140.0;
const MIN_HEIGHT: f32 = 100.0;
/// The band at the top, the handle to move the note.
const BAND: f32 = 22.0;
/// How long after the last key the text is kept.
const SAVE_AFTER: Duration = Duration::from_millis(600);

/// The colours of the sheet: Windows 7's pale yellow, not the theme's (a note is paper).
const SHADOW: Color = rgba(0x00000040);
const SHEET_TOP: Color = rgb(0xfdf8b4);
const SHEET_BOTTOM: Color = rgb(0xf8ef8e);
const SHEET_BORDER: Color = rgb(0xb8a24a);
const BAND_TOP: Color = rgb(0xf8f09a);
const BAND_BOTTOM: Color = rgb(0xefe27a);
/// The + and × at rest, and the + under the pointer (the ink of the text).
const BAND_INK: Color = rgb(0x7a6a20);
const INK: Color = rgb(0x2a2200);
/// The × under the pointer: red, as a close button.
const CLOSE_HOT: Color = rgb(0xc42b1c);
const GRIP: Color = rgb(0xc9b85a);

/// One note.
#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    pub id: String,
    /// The connector name of its screen.
    pub screen: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub text: String,
}

impl Note {
    fn from_json(v: &Value) -> Option<Self> {
        let f = |k: &str| v.get(k).and_then(Value::as_f64).map(|n| n as f32);
        let s = |k: &str| v.get(k).and_then(Value::as_str).map(String::from);
        Some(Self {
            id: s("id")?,
            screen: s("screen").unwrap_or_default(),
            x: f("x").unwrap_or(0.0),
            y: f("y").unwrap_or(0.0),
            width: f("width").unwrap_or(NEW_WIDTH),
            height: f("height").unwrap_or(NEW_HEIGHT),
            text: s("text").unwrap_or_default(),
        })
    }

    fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "screen": self.screen,
            "x": self.x.round(),
            "y": self.y.round(),
            "width": self.width.round(),
            "height": self.height.round(),
            "text": self.text,
        })
    }
}

struct Store {
    notes: Signal<Vec<Note>>,
    /// The id of the note added last, which takes the keyboard when it shows.
    last_added: Signal<String>,
    /// The id of the note being typed in, or "". The desktop layer holds the keyboard
    /// meanwhile.
    editing: Signal<String>,
}

thread_local! {
    static STORE: Cell<Option<&'static Store>> = const { Cell::new(None) };
}

fn store() -> &'static Store {
    STORE.with(|cell| {
        cell.get().unwrap_or_else(|| {
            let (store, _) = create_root(|| -> &'static Store {
                Box::leak(Box::new(Store {
                    notes: signal(load()),
                    last_added: signal(String::new()),
                    editing: signal(String::new()),
                }))
            });
            cell.set(Some(store));
            store
        })
    })
}

/// Reads the notes, from the shell's root: call it before the first desktop opens.
pub fn install() {
    store();
}

fn path() -> std::path::PathBuf {
    state_dir().join("notes.json")
}

fn load() -> Vec<Note> {
    std::fs::read_to_string(path())
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|v| {
            v.as_array()
                .map(|list| list.iter().filter_map(Note::from_json).collect())
        })
        .unwrap_or_default()
}

fn save() {
    let list: Vec<Value> = store()
        .notes
        .with_untracked(|n| n.iter().map(Note::to_json).collect());
    let _ = std::fs::create_dir_all(state_dir());
    let text = serde_json::to_string(&Value::Array(list)).unwrap_or_default();
    let partial = path().with_extension("json.partial");
    if std::fs::write(&partial, text + "\n").is_ok() {
        let _ = std::fs::rename(&partial, path());
    }
}

/// A new empty note at a point of a screen, which takes the keyboard.
pub fn add(screen: &str, x: f32, y: f32) {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let id = format!("{stamp:x}");
    let s = store();
    s.notes.update(|notes| {
        notes.push(Note {
            id: id.clone(),
            screen: screen.to_owned(),
            x: x.round(),
            y: y.round(),
            width: NEW_WIDTH,
            height: NEW_HEIGHT,
            text: String::new(),
        })
    });
    s.last_added.set(id.clone());
    s.editing.set(id);
    save();
}

/// Changes a note. The note's view keeps its own state, so this rebuilds nothing: the note being
/// typed in keeps its cursor.
fn update(id: &str, f: impl FnOnce(&mut Note)) {
    store().notes.update(|notes| {
        if let Some(note) = notes.iter_mut().find(|n| n.id == id) {
            f(note);
        }
    });
    save();
}

fn remove(id: &str) {
    let s = store();
    s.notes.update(|notes| notes.retain(|n| n.id != id));
    if s.editing.get_untracked() == id {
        s.editing.set(String::new());
    }
    save();
}

/// A note is being typed in. Reading it tracks it.
pub fn editing() -> bool {
    store().editing.with(|e| !e.is_empty())
}

/// The pointer left the desktop: no note keeps the keyboard.
pub fn stop_editing() {
    store().editing.set_if_changed(String::new());
}

/// The notes of one screen. `area` is the size of the desktop, to keep them on it.
pub fn view(screen: String, area: Signal<(f32, f32)>) -> impl IntoView {
    let s = store();
    for_each(
        move || {
            s.notes
                .with(|n| n.iter().filter(|n| n.screen == screen).cloned().collect())
        },
        |note: &Note| note.id.clone(),
        move |note| note_view(note, area),
    )
}

fn note_view(note: Note, area: Signal<(f32, f32)>) -> Div {
    let id: Rc<str> = note.id.clone().into();
    let place = signal((note.x, note.y));
    let size = signal((note.width, note.height));
    let text_value = signal(note.text.clone());
    let editor = node_ref();
    // A new note takes the keyboard once it shows.
    if untrack(|| store().last_added.get()) == note.id {
        set_timeout(Duration::ZERO, move || editor.focus());
    }
    // Typing is kept a moment after the last key.
    let pending: Rc<Cell<Option<Timeout>>> = Rc::default();
    let typed = {
        let id = id.clone();
        move |now: &str| {
            if let Some(t) = pending.take() {
                t.cancel();
            }
            let id = id.clone();
            let now = now.to_owned();
            pending.set(Some(set_timeout(SAVE_AFTER, move || {
                update(&id, |n| n.text = now);
            })));
        }
    };

    let moved_from = Rc::new(Cell::new((0.0f32, 0.0f32)));
    let band_id = id.clone();
    let band = div()
        .absolute()
        .left(1.0)
        .top(1.0)
        .right(1.0)
        .h(BAND)
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .px(6.0)
        .select_none()
        .cursor(CursorIcon::Move)
        .fill(|| vertical_gradient(&[(0.0, BAND_TOP), (1.0, BAND_BOTTOM)]))
        .on_grab(move |g| match g.phase {
            GrabPhase::Start => moved_from.set(place.get_untracked()),
            GrabPhase::Move => {
                let (x, y) = moved_from.get();
                let (w, h) = size.get_untracked();
                let (aw, ah) = area.get_untracked();
                place.set((
                    (x + g.total.x).min(aw - w).max(0.0),
                    (y + g.total.y).min(ah - h).max(0.0),
                ));
            }
            GrabPhase::End => {
                let (x, y) = place.get_untracked();
                update(&band_id, |n| {
                    n.x = x.round();
                    n.y = y.round();
                });
            }
            GrabPhase::Cancel => place.set(moved_from.get()),
        })
        .child(band_button("+", BAND_INK, INK, {
            let screen = note.screen.clone();
            move || {
                let (x, y) = place.get_untracked();
                let (aw, _) = area.get_untracked();
                add(&screen, (x + 24.0).min(aw - NEW_WIDTH), y + 24.0);
            }
        }))
        .child(band_button("\u{00D7}", BAND_INK, CLOSE_HOT, {
            let id = id.clone();
            move || remove(&id)
        }));

    let focus_id = id.clone();
    let field = text_area()
        .value(text_value)
        .node_ref(editor)
        .w_full()
        .h_full()
        .bg(Color::TRANSPARENT)
        .border(0.0)
        .p(0.0)
        .text_size(14.0)
        .text_color(INK)
        .dyn_style(|s| {
            let face = format!("Segoe Print, Comic Neue, {}", font().0);
            s.font_family(FontFamily(face.into()))
        })
        .on_input(typed)
        .on_focus_change(move |focused| {
            if focused {
                store().editing.set_if_changed(focus_id.to_string());
            }
        })
        .on_key_down(move |e| {
            if e.key == Key::Named(NamedKey::Escape) {
                editor.blur();
                stop_editing();
            }
        });

    let sized_from = Rc::new(Cell::new((0.0f32, 0.0f32)));
    let corner_id = id.clone();
    let grip = div()
        .absolute()
        .right(0.0)
        .bottom(0.0)
        .w(14.0)
        .h(14.0)
        .flex()
        .items_end()
        .justify_end()
        .p(1.0)
        .cursor(CursorIcon::Move)
        .text_size(9.0)
        .text_color(GRIP)
        .on_grab(move |g| match g.phase {
            GrabPhase::Start => sized_from.set(size.get_untracked()),
            GrabPhase::Move => {
                let (w, h) = sized_from.get();
                size.set((
                    (w + g.total.x).max(MIN_WIDTH),
                    (h + g.total.y).max(MIN_HEIGHT),
                ));
            }
            GrabPhase::End => {
                let (w, h) = size.get_untracked();
                update(&corner_id, |n| {
                    n.width = w.round();
                    n.height = h.round();
                });
            }
            GrabPhase::Cancel => size.set(sized_from.get()),
        })
        .child("\u{25E2}");

    div()
        .absolute()
        .dyn_style(move |s| {
            let (x, y) = place.get();
            let (w, h) = size.get();
            s.left(x).top(y).w(w).h(h)
        })
        // A soft shadow under the sheet.
        .child(
            div()
                .absolute()
                .left(3.0)
                .top(3.0)
                .right(-3.0)
                .bottom(-4.0)
                .bg(SHADOW)
                .dyn_style(|s| s.rounded(corner(2.0))),
        )
        .child(
            div()
                .absolute()
                .inset(0.0)
                .border(1.0)
                .border_color(SHEET_BORDER)
                .fill(|| vertical_gradient(&[(0.0, SHEET_TOP), (1.0, SHEET_BOTTOM)]))
                .dyn_style(|s| s.rounded(corner(1.0))),
        )
        .child(band)
        .child(
            div()
                .absolute()
                .left(8.0)
                .top(BAND + 4.0)
                .right(8.0)
                .bottom(10.0)
                .overflow_y_scroll()
                .child(field),
        )
        .child(grip)
}

/// The + or × of the band: `rest` at rest, `hot` under the pointer.
fn band_button(label: &'static str, rest: Color, hot: Color, run: impl Fn() + 'static) -> Div {
    let ui = Interaction::new();
    div()
        .flex_none()
        .px(4.0)
        .text_size(17.0)
        .cursor(CursorIcon::Pointer)
        .interaction(ui)
        .dyn_style(move |s| s.text_color(if ui.hovered() { hot } else { rest }))
        .on_click(move |e| {
            e.stop_propagation();
            run();
        })
        .child(label)
}
