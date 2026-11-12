//! The notifications of this session (QML `NotificationHistory`, `NotificationIcon`,
//! `NotificationPanel`), and Hokum's Do not disturb.
//!
//! Hokum draws the notifications and keeps no history. `busctl` watches the `Notify` calls
//! programs make on the session bus and prints each one as a line of JSON. Notifications that
//! only show a level (volume, brightness: the `x-canonical-private-synchronous` or
//! `x-dunst-stack-tag` hints) and the transient ones stay out of the list. The list is kept in
//! `$XDG_STATE_HOME/shojiwm-desktop/notifications.json`, so it outlives a restart of the shell
//! and a new session.

use super::{
    ERROR_RED, Panel, bar_cell, bar_picture, body_text, click_or_settings, footer, icon_or,
    icon_width, is_open, one_line, panel_link, panel_popup, small_text,
};
use crate::paths::{asset, state_dir};
use crate::theme::{Align, item_radius, scheme, theme};
use crate::ui::{
    MenuLine, aero_scroll, bar_menu_options, cp_check, font, popup_frame, popup_menu,
    section_title, shell_icon, vertical_gradient,
};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use serde_json::{Value, json};
use std::cell::Cell;
use std::time::{SystemTime, UNIX_EPOCH};

/// How many notifications the list keeps.
const KEEP: usize = 100;
/// The longest the list grows before it scrolls.
const LIST_HEIGHT: f32 = 380.0;
/// The red count on the bell: its gradient and its outline (QML).
const COUNT_TOP: Color = rgb(0xf46a5e);
const COUNT_BOTTOM: Color = rgb(0xc4271b);
const COUNT_OUTLINE: Color = rgba(0x00000080);
const COUNT_SIZE: f32 = 9.0;
/// A notification's text shows at most this many lines.
const BODY_LINES: f32 = 3.0;

/// A notification as the list keeps it.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub app: String,
    pub icon: String,
    pub summary: String,
    pub body: String,
    /// When it came, in milliseconds since 1970.
    pub time: i64,
}

impl Entry {
    fn from_json(v: &Value) -> Option<Self> {
        let s = |k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_owned();
        Some(Self {
            app: s("app"),
            icon: s("icon"),
            summary: s("summary"),
            body: s("body"),
            time: v.get("time")?.as_f64()? as i64,
        })
    }

    fn to_json(&self) -> Value {
        json!({
            "app": self.app,
            "icon": self.icon,
            "summary": self.summary,
            "body": self.body,
            "time": self.time,
        })
    }
}

/// The list, newest first; how many came since the panel was last opened; Do not disturb.
#[derive(Clone, Copy)]
struct History {
    entries: Signal<Vec<Entry>>,
    unread: Signal<u32>,
    do_not_disturb: Signal<bool>,
}

thread_local! {
    static HISTORY: Cell<Option<History>> = const { Cell::new(None) };
}

fn file() -> std::path::PathBuf {
    state_dir().join("notifications.json")
}

/// The history, read from its file the first time.
fn history() -> History {
    HISTORY.with(|cell| {
        cell.get().unwrap_or_else(|| {
            let (entries, unread) = load();
            let (h, _) = create_root(|| History {
                entries: signal(entries),
                unread: signal(unread),
                do_not_disturb: signal(false),
            });
            cell.set(Some(h));
            h
        })
    })
}

fn load() -> (Vec<Entry>, u32) {
    let Some(saved) = std::fs::read_to_string(file())
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
    else {
        return (Vec::new(), 0);
    };
    let entries = saved
        .get("entries")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(Entry::from_json)
                .take(KEEP)
                .collect()
        })
        .unwrap_or_default();
    let unread = saved.get("unread").and_then(Value::as_u64).unwrap_or(0) as u32;
    (entries, unread)
}

/// Writes the list and the unseen count to the file.
fn save() {
    let h = history();
    let entries: Vec<Value> = h
        .entries
        .with_untracked(|e| e.iter().map(Entry::to_json).collect());
    let text = json!({ "unread": h.unread.get_untracked(), "entries": entries }).to_string();
    let path = file();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, text + "\n");
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64)
}

/// The text without the markup the specification allows in a body.
fn strip_markup(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_tag = false;
    for c in text.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

/// A `Notify` call as `busctl monitor --json=short` prints it: the arguments are
/// `app, replaces_id, icon, summary, body, actions, hints, timeout`.
fn add(line: &str) {
    let Ok(message) = serde_json::from_str::<Value>(line) else {
        return;
    };
    let Some(data) = message.pointer("/payload/data").and_then(Value::as_array) else {
        return;
    };
    if data.len() < 7 {
        return;
    }
    let hints = &data[6];
    if hints.pointer("/transient/data") == Some(&Value::Bool(true)) {
        return;
    }
    if hints.get("x-canonical-private-synchronous").is_some()
        || hints.get("x-dunst-stack-tag").is_some()
    {
        return;
    }
    let s = |i: usize| data[i].as_str().unwrap_or("").to_owned();
    let entry = Entry {
        app: s(0),
        icon: s(2),
        summary: s(3),
        body: strip_markup(&s(4)),
        time: now_ms(),
    };
    if entry.summary.is_empty() && entry.body.is_empty() {
        return;
    }
    let h = history();
    h.entries.update(|e| {
        e.insert(0, entry);
        e.truncate(KEEP);
    });
    h.unread.update(|n| *n += 1);
    save();
}

fn remove(index: usize) {
    history().entries.update(|e| {
        if index < e.len() {
            e.remove(index);
        }
    });
    save();
}

fn clear() {
    let h = history();
    h.entries.set(Vec::new());
    h.unread.set(0);
    save();
}

fn mark_read() {
    let h = history();
    if h.unread.get_untracked() != 0 {
        h.unread.set(0);
        save();
    }
}

fn do_not_disturb() -> bool {
    history().do_not_disturb.get()
}

fn set_do_not_disturb(on: bool) {
    crate::programs::run_helper(if on { "hokum dnd on" } else { "hokum dnd off" });
}

/// Watches the session bus for notifications, and Hokum for its Do not disturb, for as long
/// as the shell runs.
pub fn install() {
    let h = history();
    // The processes live as long as the root they are made in: the whole shell.
    let (_, _) = create_root(move || {
        let monitor = Process::new("busctl")
            .args([
                "--user",
                "monitor",
                "--json=short",
                "--match",
                "type='method_call',interface='org.freedesktop.Notifications',member='Notify'",
            ])
            .keep(1)
            .spawn();
        monitor.on_line(add);
        // Hokum prints its Do not disturb at start and at each change.
        let dnd = Process::new("hokum")
            .args(["dnd", "--watch", "--json"])
            .keep(1)
            .spawn();
        dnd.on_line(move |line| {
            if let Ok(v) = serde_json::from_str::<Value>(line) {
                h.do_not_disturb
                    .set(v.get("dnd") == Some(&Value::Bool(true)));
            }
        });
    });
}

/// "Just now", "5 min ago", the time for one of today, or the date for an older one.
fn age(time_ms: i64, now: &LocalTime) -> String {
    let minutes = (now.unix * 1000 - time_ms) / 60_000;
    if minutes < 1 {
        return "Just now".into();
    }
    if minutes < 60 {
        return format!("{minutes} min ago");
    }
    let then = LocalTime::at(time_ms / 1000);
    if (then.year, then.month, then.day) == (now.year, now.month, now.day) {
        then.format("%H:%M")
    } else {
        super::locale::plain_date(&then)
    }
}

/// The bell of the notification area: a red count shows the notifications not seen yet, a
/// slashed bell Do not disturb. A click opens the panel, a middle click switches Do not
/// disturb, a right click opens its menu.
pub fn icon(output_name: &str) -> Div {
    let name = output_name.to_owned();
    let anchor = node_ref();
    let menu = signal(false);
    let h = history();
    let active = {
        let name = name.clone();
        move || is_open(&name, Panel::Notifications)
    };
    let opener = name.clone();
    let lines_name = name.clone();
    let count = div()
        .absolute()
        .right(0.0)
        .top(2.0)
        .h(12.0)
        .min_w(12.0)
        .px(3.0)
        .border(1.0)
        .flex()
        .items_center()
        .justify_center()
        .pointer_events_none()
        .fill(|| vertical_gradient(&[(0.0, COUNT_TOP), (1.0, COUNT_BOTTOM)]))
        .dyn_style(move |s| {
            let s = s
                .rounded(6.0)
                .border_color(COUNT_OUTLINE)
                .font_family(font())
                .text_size(COUNT_SIZE)
                .font_weight(FontWeight::BOLD)
                .text_color(Color::WHITE);
            if h.unread.get() > 0 && !h.do_not_disturb.get() {
                s
            } else {
                s.hidden()
            }
        })
        .child(text(move || {
            let n = h.unread.get();
            if n > 9 {
                "9+".to_owned()
            } else {
                n.to_string()
            }
        }));
    bar_cell(icon_width, 2.0, active)
        .node_ref(anchor)
        .child(bar_picture(
            || {
                asset(if do_not_disturb() {
                    "bell-off.svg"
                } else {
                    "bell.svg"
                })
            },
            2.0,
        ))
        .child(count)
        .on_click(move |e| {
            click_or_settings(e, &opener, Panel::Notifications, || {
                crate::control_panel::open_page("notifications")
            })
        })
        .on_aux_click(move |e| match e.button {
            Some(PointerButton::Middle) => set_do_not_disturb(!untrack(do_not_disturb)),
            Some(PointerButton::Secondary) => menu.set(!menu.get_untracked()),
            _ => {}
        })
        .child(popup_menu(
            move || menu.get(),
            move || menu.set(false),
            bar_menu_options(anchor),
            move || {
                let name = lines_name.clone();
                let dnd = do_not_disturb();
                vec![
                    MenuLine::item("Open notifications", move || {
                        super::open(&name, Panel::Notifications);
                    }),
                    MenuLine::item("Clear all", clear)
                        .enabled(!history().entries.with(Vec::is_empty)),
                    MenuLine::separator(),
                    MenuLine::item("Do not disturb", move || set_do_not_disturb(!dnd)).checked(dnd),
                    MenuLine::separator(),
                    MenuLine::item("Customize notification icons", || {
                        crate::control_panel::open_page("notifications");
                    }),
                ]
            },
        ))
        .child(panel_popup(
            &name,
            Panel::Notifications,
            Align::Center,
            panel,
        ))
}

/// The notification panel: the notifications, newest first, each with its program's icon, its
/// title, its text and when it came. A cross removes one, Clear all removes them all, and the
/// check box at the bottom is Hokum's Do not disturb.
fn panel() -> Div {
    let h = history();
    untrack(mark_read);
    // The panel grows away from the bar, so a shorter list would move its far edge from under
    // the pointer. While it is open, it keeps the tallest height it had.
    let held = signal(0.0f32);
    let header = div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .h(22.0)
        .mx(10.0)
        .mt(8.0)
        .child(section_title("Notifications").dyn_style(|s| s.text_size(theme().font_size)))
        .child(panel_link("Clear all", true, clear).dyn_style(move |s| {
            if h.entries.with(Vec::is_empty) {
                s.hidden()
            } else {
                s
            }
        }));
    let list = dyn_view(move || {
        let entries = h.entries.get();
        div().flex().flex_col().gap(2.0).children(
            entries
                .into_iter()
                .enumerate()
                .map(|(i, e)| card(i, e))
                .collect::<Vec<_>>(),
        )
    });
    let empty = div()
        .flex()
        .items_center()
        .h(40.0)
        .pl(16.0)
        .dyn_style(move |s| {
            if h.entries.with(Vec::is_empty) {
                s
            } else {
                s.hidden()
            }
        })
        .child(body_text("No new notifications.", || scheme().text_muted));
    popup_frame()
        .w(340.0)
        .on_resized(move |_, height| {
            if height > held.get_untracked() {
                held.set(height);
            }
        })
        .dyn_style(move |s| s.min_h(held.get()))
        .child(header)
        .child(
            aero_scroll(list)
                .mx(6.0)
                .mt(4.0)
                .max_h(LIST_HEIGHT)
                .dyn_style(move |s| {
                    if h.entries.with(Vec::is_empty) {
                        s.hidden()
                    } else {
                        s
                    }
                }),
        )
        .child(empty)
        .child(div().flex_1().min_h(8.0))
        .child(footer(
            32.0,
            false,
            cp_check(
                "Do not disturb",
                Binding::new(do_not_disturb, set_do_not_disturb),
            ),
        ))
}

/// A notification of the panel: its icon, title, age, text and program; a cross under the
/// pointer removes it.
fn card(index: usize, entry: Entry) -> Div {
    let ui = Interaction::new();
    let cross = Interaction::new();
    let wanted = if entry.icon.is_empty() {
        entry.app.to_lowercase()
    } else {
        entry.icon.clone()
    };
    let icon = icon_or(&wanted, "dialog-information");
    let title = if entry.summary.is_empty() {
        entry.app.clone()
    } else {
        entry.summary.clone()
    };
    let time = entry.time;
    let has_app_line = !entry.summary.is_empty() && !entry.app.is_empty();
    let body = entry.body.clone();
    let words = div()
        .flex_1()
        .min_w(0.0)
        .flex()
        .flex_col()
        .gap(1.0)
        .child(
            div()
                .flex()
                .flex_row()
                .gap(6.0)
                .child(
                    one_line(body_text(title, || scheme().text).font_weight(FontWeight::BOLD))
                        .flex_1(),
                )
                .child(small_text(
                    move || age(time, &local_time(Every::Minute)),
                    || scheme().text_muted,
                )),
        )
        .children((!body.is_empty()).then(|| {
            div()
                .overflow_clip()
                .dyn_style(|s| {
                    let size = theme().font_size_small;
                    s.max_h((size * 1.4 * BODY_LINES).ceil())
                })
                .child(small_text(body, || scheme().text))
        }))
        .children(has_app_line.then(|| {
            text(entry.app.clone()).dyn_style(|s| {
                s.font_family(font())
                    .text_size(theme().font_size_small - 1.0)
                    .text_color(scheme().text_muted)
            })
        }));
    div()
        .relative()
        .flex()
        .flex_row()
        .items_start()
        .gap(8.0)
        .min_h(44.0)
        .pl(8.0)
        .pr(28.0)
        .pt(7.0)
        .pb(7.0)
        .border(1.0)
        .interaction(ui)
        .dyn_style(move |s| {
            let c = scheme();
            let s = s.rounded(item_radius());
            if ui.hovered() {
                s.bg(c.hover).border_color(c.selection_border)
            } else {
                s.bg(Color::TRANSPARENT).border_color(Color::TRANSPARENT)
            }
        })
        .child(div().pt(1.0).child(shell_icon(icon, 24.0)))
        .child(words)
        .child(
            div()
                .absolute()
                .right(8.0)
                .top(4.0)
                .px(2.0)
                .interaction(cross)
                .on_click(move |e| {
                    e.stop_propagation();
                    remove(index);
                })
                .dyn_style(move |s| {
                    let s = s.text_size(16.0).text_color(if cross.hovered() {
                        ERROR_RED
                    } else {
                        scheme().text_muted
                    });
                    if ui.hovered() || cross.hovered() {
                        s
                    } else {
                        s.hidden()
                    }
                })
                .child("\u{00D7}"),
        )
}
