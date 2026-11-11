//! The launcher (Super+Shift+D): the Start menu's search alone, near the top of the focused
//! screen (QML `Launcher`). The same search and the same rows: type, then Return starts the
//! first program, opens the task or the file, or runs what was typed. Up and Down pick a row;
//! Escape or a click outside closes it.
//!
//! Under the field, a list: before anything is typed, the programs started last, then every
//! program; while typing, the results. The sides and the bottom edge resize the panel, and the
//! size is kept (`launcher_width`, `launcher_height`: the list's height), written when the edge
//! is let go. A list of height 0 is the field alone: then the results show under it while
//! typing, only as tall as they are.
//!
//! The layer is as large as the panel ever gets on its screen, and the panel is drawn inside it,
//! so neither a key nor a drag resizes the layer; the rest of it is clear, and a press there
//! closes the launcher.

use super::search::{self, Search};
use super::{close_launcher, row_height};
use crate::paths::asset;
use crate::settings::{change, settings};
use crate::theme::{corner, field_radius, popup_radius, scheme, theme};
use crate::ui::{aero_scroll, font, vertical_gradient};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;

/// The most rows the compact panel grows to while typing.
const COMPACT_ROWS: f32 = 14.0;
/// The narrowest the panel gets.
const MIN_WIDTH: f32 = 360.0;
/// The share of the screen's width the panel can take, and of its height the panel ends at.
const MAX_WIDTH_SHARE: f32 = 0.9;
const MAX_HEIGHT_SHARE: f32 = 0.82;
/// The search field is this much taller than the Start menu's.
const FIELD_EXTRA: f32 = 8.0;
/// The thickness of the edges that resize the panel, and the size of its bottom corners.
const EDGE: f32 = 8.0;
const CORNER: f32 = 12.0;

/// The sizes of the launcher on a screen, as the theme and the settings are now.
#[derive(Clone, Copy, PartialEq)]
struct Sizes {
    /// The field and the room round it.
    header: f32,
    max_list: f32,
    max_width: f32,
    window_width: f32,
    window_height: f32,
}

fn sizes(screen: (f32, f32)) -> Sizes {
    let t = theme();
    let header = t.menu_search_height + FIELD_EXTRA + t.menu_padding * 2.0;
    let max_list = ((screen.1 * MAX_HEIGHT_SHARE).round()
        - t.taskbar_height
        - header
        - t.menu_padding
        - 8.0)
        .max(0.0);
    let max_width = (screen.0 * MAX_WIDTH_SHARE).round();
    Sizes {
        header,
        max_list,
        max_width,
        window_width: max_width,
        window_height: header + t.menu_padding + max_list.max(COMPACT_ROWS * row_height()),
    }
}

fn screen_of(output: &Output) -> (f32, f32) {
    (
        output.logical_size.width as f32,
        output.logical_size.height as f32,
    )
}

/// The size of the launcher's layer on a screen. Reading it tracks the theme and the settings.
pub(super) fn window_size(output: &Output) -> (i32, i32) {
    let s = sizes(screen_of(output));
    (s.window_width.round() as i32, s.window_height.round() as i32)
}

/// Which edges of the panel a drag moves.
#[derive(Clone, Copy)]
enum Edge {
    Left,
    Right,
    Bottom,
    BottomLeft,
    BottomRight,
}

impl Edge {
    fn moves_width(self) -> Option<bool> {
        match self {
            Self::Left | Self::BottomLeft => Some(true),
            Self::Right | Self::BottomRight => Some(false),
            Self::Bottom => None,
        }
    }

    fn moves_height(self) -> bool {
        matches!(self, Self::Bottom | Self::BottomLeft | Self::BottomRight)
    }
}

/// The launcher on a screen, as it is when it opens.
pub(super) fn launcher(output: Output) -> Div {
    let screen = screen_of(&output);
    let sz = memo(move || sizes(screen));
    let search = Search::new();
    let pick = signal(0usize);
    // While an edge is dragged, the size it gives: the settings are written once, when it is
    // let go.
    let drag_width = signal(None::<f32>);
    let drag_list = signal(None::<f32>);
    let list_height = memo(move || {
        let wanted = drag_list
            .get()
            .unwrap_or_else(|| settings().with(|s| s.launcher_height) as f32);
        wanted.min(sz.get().max_list).max(0.0)
    });
    let compact = memo(move || list_height.get() == 0.0);
    let panel_width = memo(move || {
        let wanted = drag_width
            .get()
            .unwrap_or_else(|| settings().with(|s| s.launcher_width) as f32);
        wanted.min(sz.get().max_width).max(MIN_WIDTH)
    });
    // The compact panel shows its rows only while typing, as tall as they are.
    let compact_rows = memo(move || {
        if !search.active() {
            return 0.0;
        }
        let rows = search.rows.with(Vec::len) + usize::from(search.pick_count.get() == 0);
        (rows as f32).min(COMPACT_ROWS)
    });
    let panel_height = memo(move || {
        let t = theme();
        let s = sz.get();
        if compact.get() {
            let rows = compact_rows.get();
            s.header + if rows > 0.0 { rows * row_height() + t.menu_padding } else { 0.0 }
        } else {
            s.header + list_height.get() + t.menu_padding
        }
    });
    // Each key starts the pick again at the top.
    effect(move || {
        search.query.with(|_| ());
        pick.set(0);
    });
    let move_pick = move |by: isize| {
        let count = untrack(|| search.pick_count.get()) as isize;
        let now = pick.get_untracked() as isize;
        pick.set((now + by).clamp(0, (count - 1).max(0)) as usize);
    };
    // Return: the row picked. Before anything is typed that is a program of the list; the
    // search's own `accept` does nothing then, since the Start menu's empty box does nothing.
    let accept = move || {
        if !untrack(|| search.active()) {
            if let Some(row) = search.row_at(pick.get_untracked()) {
                search::choose(&row);
                close_launcher();
            }
            return;
        }
        if search.accept(pick.get_untracked()) {
            close_launcher();
        }
    };
    let input = node_ref();
    request_frame(move |_| input.focus());
    let focused = signal(false);
    let query = search.query;
    let field = text_input()
        .value(query)
        .node_ref(input)
        .w_full()
        .on_focus_change(move |on| focused.set(on))
        .on_submit(move |_| accept())
        .on_key_down(move |e| {
            let page = {
                let shown = if untrack(|| compact.get()) {
                    untrack(|| compact_rows.get()) * row_height()
                } else {
                    untrack(|| list_height.get())
                };
                ((shown / untrack(row_height)).floor() as isize - 1).max(1)
            };
            match &e.key {
                Key::Named(NamedKey::Escape) => {
                    e.prevent_default();
                    if query.with_untracked(String::is_empty) {
                        close_launcher();
                    } else {
                        query.set(String::new());
                    }
                }
                Key::Named(NamedKey::ArrowUp) => {
                    e.prevent_default();
                    move_pick(-1);
                }
                Key::Named(NamedKey::ArrowDown) => {
                    e.prevent_default();
                    move_pick(1);
                }
                Key::Named(NamedKey::PageUp) => {
                    e.prevent_default();
                    move_pick(-page);
                }
                Key::Named(NamedKey::PageDown) => {
                    e.prevent_default();
                    move_pick(page);
                }
                _ => {}
            }
        })
        .dyn_style(|s| {
            let c = scheme();
            s.font_family(font())
                .text_size(theme().font_size + 2.0)
                .text_color(c.text)
                .selection_color(c.selection)
        });
    let search_box = div()
        .absolute()
        .flex()
        .flex_row()
        .items_center()
        .border(1.0)
        .dyn_style(move |s| {
            let t = theme();
            let c = scheme();
            s.left(t.menu_padding)
                .right(t.menu_padding)
                .top(t.menu_padding)
                .h(t.menu_search_height + FIELD_EXTRA)
                .rounded(field_radius())
                .bg(c.field)
                .border_color(if focused.get() { c.field_focus } else { c.field_border })
        })
        .child(
            img(asset("search.svg"))
                .size(16.0)
                .ml(8.0)
                .flex_none()
                .object_fit(ObjectFit::Contain),
        )
        .child(
            div()
                .relative()
                .flex_1()
                .min_w(0.0)
                .mx(8.0)
                .flex()
                .items_center()
                .overflow_clip()
                .child(field)
                .child(search::placeholder(
                    query,
                    "Search programs, settings and files, or type a command",
                    || theme().font_size + 2.0,
                )),
        );
    let results = div()
        .absolute()
        .flex()
        .flex_col()
        .dyn_style(move |s| {
            let t = theme();
            let s = s
                .left(t.menu_padding)
                .right(t.menu_padding)
                .top(sz.get().header)
                .bottom(t.menu_padding);
            if compact.get() && !search.active() { s.hidden() } else { s }
        })
        .child(aero_scroll(search::results(search, pick, close_launcher, false)).flex_1());
    // The panel is centred, with its top fixed: a side sets the width to twice the pointer's
    // distance from the middle, the bottom edge sets the list's height from the pointer's
    // height, a bottom corner both. The edge stays under the pointer.
    let edge = move |which: Edge| {
        let grabbed = move |e: &GrabEvent| {
            let scale = window_scale().as_f32();
            let (x, y) = (e.position.x / scale, e.position.y / scale);
            let s = untrack(|| sz.get());
            match e.phase {
                GrabPhase::Start | GrabPhase::Move => {
                    if let Some(left) = which.moves_width() {
                        let from_middle = if left { s.window_width / 2.0 - x } else { x - s.window_width / 2.0 };
                        drag_width.set(Some((2.0 * from_middle).round()));
                    }
                    if which.moves_height() {
                        let t = untrack(theme);
                        let height = (y - s.header - t.menu_padding).round();
                        drag_list.set(Some(height.clamp(0.0, s.max_list)));
                    }
                }
                GrabPhase::End | GrabPhase::Cancel => {
                    let width = drag_width.get_untracked().map(|_| untrack(|| panel_width.get()));
                    let height = drag_list.get_untracked().map(|_| untrack(|| list_height.get()));
                    if width.is_some() || height.is_some() {
                        change(|s| {
                            if let Some(w) = width {
                                s.launcher_width = w.round() as i64;
                            }
                            if let Some(h) = height {
                                s.launcher_height = h.round() as i64;
                            }
                        });
                    }
                    drag_width.set(None);
                    drag_list.set(None);
                }
            }
        };
        let el = div().absolute().grab_threshold(0.0).on_grab(grabbed);
        match which {
            Edge::Left => el.left(0.0).top(0.0).bottom(0.0).w(EDGE).cursor(CursorIcon::EwResize),
            Edge::Right => el.right(0.0).top(0.0).bottom(0.0).w(EDGE).cursor(CursorIcon::EwResize),
            Edge::Bottom => el
                .left(EDGE)
                .right(EDGE)
                .bottom(0.0)
                .h(EDGE)
                .cursor(CursorIcon::NsResize),
            Edge::BottomLeft => el.left(0.0).bottom(0.0).size(CORNER).cursor(CursorIcon::Move),
            Edge::BottomRight => el.right(0.0).bottom(0.0).size(CORNER).cursor(CursorIcon::Move),
        }
    };
    let panel = div()
        .relative()
        .flex_none()
        .dyn_style(move |s| {
            s.w(panel_width.get())
                .h(panel_height.get())
                .rounded(corner(popup_radius() + 1.0))
                .bg(scheme().start_glass_outline)
        })
        // A press on the panel itself stays here.
        .on_pointer_down(|e| e.stop_propagation())
        .child(
            div()
                .absolute()
                .inset(1.0)
                .fill(|| {
                    let c = scheme();
                    vertical_gradient(&[(0.0, c.menu_top), (1.0, c.menu_bottom)])
                })
                .dyn_style(|s| s.rounded(popup_radius())),
        )
        .child(search_box)
        .child(results)
        .child(edge(Edge::Left))
        .child(edge(Edge::Right))
        .child(edge(Edge::Bottom))
        .child(edge(Edge::BottomLeft))
        .child(edge(Edge::BottomRight));
    div()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .select_none()
        .dyn_style(|s| s.font_family(font()).text_size(theme().font_size).text_color(scheme().text))
        .on_pointer_down(|_| close_launcher())
        .child(panel)
}
