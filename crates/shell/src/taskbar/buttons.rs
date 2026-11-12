//! The buttons of the taskbar (QML `TaskButton` and the row of `Taskbar.qml`): one per program,
//! with every window of it behind one button, or one per window when they are not combined.
//!
//! Left click: show the window, or put it away when it is the one in front; with several windows
//! it shows the thumbnails instead. Right click: the jump list. Middle click, Ctrl+click or
//! Shift+click: another instance. Resting on it shows the thumbnails. A pinned program with no
//! window open is its icon alone, with no plate until the pointer is on it; a click starts it.
//!
//! A button dragged along the bar goes to a new place, and a program dropped on the bar from
//! the Start menu or as a `.desktop` file is pinned where it lands. The other buttons stand
//! aside meanwhile, as Windows 7 shows it.

use super::Bar;
use super::launcher_entries::launcher_entry;
use crate::apps::{
    PROGRAM_MIME, ProgramDrag, WindowGroup, activate_group, dropped_program_id, group_is_current,
    launch, move_button, pin_dropped_at, title_of,
};
use crate::settings::{motion, settings};
use crate::theme::{Align, away_from_bar, corner, item_radius, theme};
use crate::ui::{font, shell_icon, shell_tooltip, vertical_gradient};
use anti_webui::prelude::*;
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

/// How far the pointer moves before a press is a drag along the bar.
const DRAG_START: f32 = 6.0;
/// White on glass, with a dark shadow a pixel down, as Windows 7 writes on its buttons.
const LABEL: Color = Color::WHITE;
const LABEL_SHADOW: Color = rgba(0x000000b3);
/// The label shows from this width of the plate on.
const LABEL_ROOM: f32 = 70.0;
/// The room at each side of the icon and the label on a button.
const INSIDE: f32 = 6.0;
/// A program's progress fills its button green, glossy in the upper half; an urgent program
/// turns it orange. Four stops each, at 0, 0.46, 0.5 and 1.
const PROGRESS: [Color; 4] = [rgb(0x8fe08a), rgb(0x3fbf3a), rgb(0x25a51f), rgb(0x4cc443)];
const URGENT: [Color; 4] = [rgb(0xffd08a), rgb(0xf6a43c), rgb(0xe98a12), rgb(0xf3a640)];
const PROGRESS_OPACITY: f32 = 0.85;
/// A count (unread mail, messages) in a small red badge on the icon's corner.
const BADGE_TOP: Color = rgb(0xf46a5e);
const BADGE_BOTTOM: Color = rgb(0xc4271b);
const BADGE_BORDER: Color = rgba(0x00000080);
const BADGE_HEIGHT: f32 = 14.0;
const BADGE_TEXT: f32 = 10.0;
/// How many edges of the other windows show behind a combined button.
const STACK_EDGES: usize = 2;
const STACK_LINE_INSET: f32 = 6.0;

/// Where a button is in the row, as laid out before anything moves.
#[derive(Clone, Debug, PartialEq)]
struct Slot {
    key: String,
    x: f32,
    w: f32,
}

/// The row and what is dragged over it.
#[derive(Clone, Copy)]
struct Row {
    groups: Memo<Vec<WindowGroup>>,
    slots: Memo<Vec<Slot>>,
    /// The index of each group in `groups` and `slots`, by key: a button finds its own without
    /// a pass over the row.
    positions: Memo<HashMap<String, usize>>,
    /// The index of the button being dragged.
    drag_from: Signal<Option<usize>>,
    /// The button the drop goes before (the count of buttons for the end).
    drop_before: Signal<Option<usize>>,
}

impl Row {
    /// The button the pointer at `x` is before: the first whose middle is right of it. A button
    /// dragged left goes before a button whose middle it reaches; dragged right, after it.
    fn place(&self, x: f32) -> usize {
        self.slots.with(|slots| {
            let from = self.drag_from.get_untracked().and_then(|i| slots.get(i));
            let leftward = from.is_some_and(|f| x < f.x + f.w / 2.0);
            slots
                .iter()
                .position(|s| {
                    let middle = s.x + s.w / 2.0;
                    if leftward { x <= middle } else { x < middle }
                })
                .unwrap_or(slots.len())
        })
    }

    /// How far the button at `index` stands aside.
    fn shift_of(&self, index: usize) -> f32 {
        let Some(to) = self.drop_before.get() else {
            return 0.0;
        };
        let t = theme();
        let from = self.drag_from.get();
        let room = from
            .and_then(|f| self.slots.with(|s| s.get(f).map(|s| s.w)))
            .unwrap_or(t.pinned_button_width)
            + t.task_button_gap;
        match from {
            None if index >= to => room,
            None => 0.0,
            Some(from) if from < to && index > from && index < to => -room,
            Some(from) if to <= index && index < from => room,
            Some(_) => 0.0,
        }
    }
}

/// `x` of a drop event, in logical pixels from the left of its box.
fn local_x(e: &DropEvent) -> f32 {
    e.local.x / window_scale().as_f32()
}

/// The row of buttons, between the Start button and the notification area. A very long row is
/// cut off rather than running under the clock.
pub(super) fn buttons(bar: Bar) -> Div {
    let available = signal(0.0f32);
    let groups = memo(crate::apps::window_groups);
    let slots = memo(move || {
        let t = theme();
        let compact = settings().with(|s| s.taskbar_compact);
        let room = available.get();
        groups.with(|list| {
            let idle = list.iter().filter(|g| g.windows.is_empty()).count();
            let open = list.len() - idle;
            let gaps = list.len().saturating_sub(1) as f32 * t.task_button_gap;
            // A pinned program with no window keeps its narrow button; the others share
            // what is left.
            let width = if open == 0 {
                0.0
            } else {
                ((room - idle as f32 * t.pinned_button_width - gaps) / open as f32)
                    .max(t.task_button_min_width)
                    .min(t.task_button_max_width)
            };
            let mut x = 0.0;
            list.iter()
                .map(|g| {
                    let w = if compact {
                        t.compact_button_width
                    } else if g.windows.is_empty() {
                        t.pinned_button_width
                    } else {
                        width
                    };
                    let slot = Slot {
                        key: g.key.clone(),
                        x,
                        w,
                    };
                    x += w + t.task_button_gap;
                    slot
                })
                .collect()
        })
    });
    let positions = memo(move || {
        groups.with(|list| {
            list.iter()
                .enumerate()
                .map(|(i, g)| (g.key.clone(), i))
                .collect::<HashMap<_, _>>()
        })
    });
    let row = Row {
        groups,
        slots,
        positions,
        drag_from: signal(None),
        drop_before: signal(None),
    };
    let pin_at = move |id: String, before: usize| {
        row.drop_before.set(None);
        // Once the drop is over: the change builds the buttons again.
        set_timeout(Duration::ZERO, move || pin_dropped_at(&id, before));
    };
    let (pin_typed, pin_file, pin_data) = (pin_at, pin_at, pin_at);
    div()
        .relative()
        .flex_1()
        .min_w(0.0)
        .self_end()
        .overflow_clip()
        .dyn_style(|s| {
            let t = theme();
            s.h(t.task_button_height)
                .ml(t.task_button_gap * 2.0)
                .mb((t.taskbar_height - t.task_button_height) / 2.0)
        })
        .on_resized(move |w, _| available.set(w))
        .on_drag_over::<ProgramDrag>(move |_, e| row.drop_before.set(Some(row.place(local_x(e)))))
        .on_drag_over::<ExternalDrag>(move |drag, e| {
            if drag.has_files() {
                row.drop_before.set(Some(row.place(local_x(e))));
            }
        })
        .on_drag_hover(move |over| {
            if !over {
                row.drop_before.set(None);
            }
        })
        .drop_target::<ProgramDrag>(move |drag, e| pin_typed(drag.0.clone(), row.place(local_x(e))))
        .on_drop_files(move |files, e| {
            if let Some(id) = dropped_program_id(files) {
                pin_file(id, row.place(local_x(e)));
            }
        })
        .on_drop_data(PROGRAM_MIME, move |bytes, e| {
            let id = String::from_utf8_lossy(bytes).trim().to_owned();
            if !id.is_empty() {
                pin_data(id, row.place(local_x(e)));
            }
        })
        .child(for_each(
            move || groups.with(|g| g.iter().map(|g| g.key.clone()).collect::<Vec<_>>()),
            |key| key.clone(),
            move |key| task_button(bar.clone(), row, key),
        ))
}

/// One flat layer of a Kernel button: `colour` over the whole button, shown while `shown` says
/// so, fading in quickly and out slowly as the glass plates do.
fn flat_layer(colour: fn() -> Color, shown: impl Fn() -> bool + 'static) -> Div {
    div().absolute().inset(0.0).dyn_style(move |s| {
        let t = theme();
        let on = shown();
        s.rounded(corner(item_radius()))
            .bg(colour())
            .opacity(if on { 1.0 } else { 0.0 })
            .transition(Transition::ms(motion(if on {
                t.hover_in
            } else {
                t.hover_out
            })))
    })
}

/// The looks of a button's plate, as Windows 7 paints them in layers: the resting glass, the
/// active plate over it, the pointer's glow over that, and the pressed plate on top.
#[derive(Clone, Copy)]
enum Plate {
    Rest,
    Active,
    Hover,
    Pressed,
}

impl Plate {
    /// The gradient's colours, at 0, 0.46, 0.5 and 1, and the light line inside the outline.
    fn colours(self) -> ([Color; 4], Color) {
        let t = theme();
        match self {
            Plate::Rest => (
                [
                    t.button_rest_top,
                    t.button_rest_upper,
                    t.button_rest_lower,
                    t.button_rest_bottom,
                ],
                t.button_rest_border,
            ),
            Plate::Active => (
                [
                    t.button_active_top,
                    t.button_active_upper,
                    t.button_active_lower,
                    t.button_active_bottom,
                ],
                t.button_active_border,
            ),
            Plate::Hover => (
                [
                    t.button_hover_top,
                    t.button_hover_upper,
                    t.button_hover_lower,
                    t.button_hover_bottom,
                ],
                t.button_hover_border,
            ),
            Plate::Pressed => (
                [
                    t.button_pressed_top,
                    t.button_pressed_upper,
                    t.button_pressed_lower,
                    t.button_pressed_bottom,
                ],
                t.button_rest_border,
            ),
        }
    }
}

/// A gradient of four stops at 0, 0.46, 0.5 and 1: lighter in the upper half.
fn glossy(c: [Color; 4]) -> ShaderPaint {
    vertical_gradient(&[(0.0, c[0]), (0.46, c[1]), (0.5, c[2]), (1.0, c[3])])
}

/// One look of the plate, shown while `shown` says so. It fades in quickly and out slowly, so
/// that the glow is added to an active button rather than replacing it.
fn plate(kind: Plate, shown: impl Fn() -> bool + 'static) -> Div {
    let shown = Rc::new(shown);
    let style_shown = shown.clone();
    div()
        .absolute()
        .left(1.0)
        .top(1.0)
        .right(1.0)
        .bottom(1.0)
        .border(1.0)
        .fill(move || glossy(kind.colours().0))
        .dyn_style(move |s| {
            let t = theme();
            let on = style_shown() && !t.task_flat;
            s.rounded(corner((item_radius() - 1.0).max(0.0)))
                .border_color(kind.colours().1)
                .opacity(if on { 1.0 } else { 0.0 })
                .transition(Transition::ms(motion(if on {
                    t.hover_in
                } else {
                    t.hover_out
                })))
        })
}

fn task_button(bar: Bar, row: Row, key: String) -> Div {
    let index = {
        let key = key.clone();
        memo(move || row.positions.with(|p| p.get(&key).copied()))
    };
    let group = {
        let key = key.clone();
        memo(move || {
            let i = index.get()?;
            row.groups.with(|g| {
                g.get(i)
                    .filter(|g| g.key == key)
                    // An index not yet updated: a pass over the row finds it.
                    .or_else(|| g.iter().find(|g| g.key == key))
                    .cloned()
            })
        })
    };
    let slot = memo(move || {
        index
            .get()
            .and_then(|i| row.slots.with(|s| s.get(i).cloned()))
    });
    let ui = Interaction::new();
    let anchor = node_ref();
    let reordering = signal(false);
    let drag_offset = signal(0.0f32);
    let rest: Rc<Cell<Option<Timeout>>> = Rc::default();
    let count = memo(move || group.with(|g| g.as_ref().map_or(0, |g| g.windows.len())));
    let idle = move || count.get() == 0;
    let compact = memo(|| settings().with(|s| s.taskbar_compact));
    let icon_only = move || idle() || compact.get();
    let current = memo(move || group.with(|g| g.as_ref().is_some_and(group_is_current)));
    let shows = memo(move || {
        let id = group.with(|g| {
            g.as_ref()
                .and_then(|g| g.entry.as_ref().map(|e| e.id.clone()))
        })?;
        launcher_entry(&id)
    });
    let body_w = move || slot.get().map_or(0.0, |s| s.w) - theme().stack_room;
    let icon = memo(move || group.with(|g| g.as_ref().map(|g| g.icon.clone()).unwrap_or_default()));
    let label = memo(move || {
        group.with(|g| match g {
            Some(g) if g.windows.len() > 1 => g.name.clone(),
            Some(g) => g
                .windows
                .first()
                .map(title_of)
                .unwrap_or_else(|| g.name.clone()),
            None => String::new(),
        })
    });
    let hovered = move || ui.hovered() && !ui.pressed();

    let stack = (0..STACK_EDGES).map(move |i| {
        div()
            .absolute()
            .top(0.0)
            .border(1.0)
            .z_index(-1 - i as i32)
            .dyn_style(move |s| {
                let t = theme();
                let s = s
                    .left((i + 1) as f32 * 2.0)
                    .w(body_w())
                    .h(t.task_button_height)
                    .rounded(item_radius())
                    .bg(t.button_stack)
                    .border_color(t.button_stack_border);
                if count.get() > i + 1 && !t.task_flat {
                    s
                } else {
                    s.hidden()
                }
            })
    });
    // Flat: a thin line right of the button for each other window, each shorter than the last.
    let stack_lines = (0..STACK_EDGES).map(move |i| {
        div().absolute().w(1.0).dyn_style(move |s| {
            let t = theme();
            let inset = STACK_LINE_INSET + i as f32 * 2.0;
            let s = s
                .left(body_w() + 1.0 + i as f32 * 2.0)
                .bottom(inset)
                .h((t.task_button_height - inset * 2.0).max(0.0))
                .bg(t.task_stack)
                .opacity(1.0 - i as f32 * 0.4);
            if count.get() > i + 1 && t.task_flat {
                s
            } else {
                s.hidden()
            }
        })
    });
    let body = div()
        .absolute()
        .left(0.0)
        .bottom(0.0)
        .border(1.0)
        .dyn_style(move |s| {
            let t = theme();
            let gone = idle() && !ui.hovered() && !ui.pressed();
            s.w(body_w())
                .h(t.task_button_height)
                .rounded(item_radius())
                .border_color(if t.task_flat {
                    Color::TRANSPARENT
                } else {
                    t.button_outline
                })
                .opacity(if gone { 0.0 } else { 1.0 })
                .transition(Transition::ms(motion(t.fade)))
        })
        .child(plate(Plate::Rest, || true))
        .child(plate(Plate::Active, move || current.get()))
        .child(plate(Plate::Hover, hovered))
        .child(plate(Plate::Pressed, move || ui.pressed()));
    // Kernel's flat button: nothing at rest, a lighter layer under the pointer and for the
    // active window, a darker one while pressed. Each fills the whole height of the bar.
    let flat = div()
        .absolute()
        .left(0.0)
        .bottom(0.0)
        .dyn_style(move |s| {
            let t = theme();
            let s = s.w(body_w()).h(t.task_button_height);
            if t.task_flat { s } else { s.hidden() }
        })
        .child(flat_layer(|| theme().task_active, move || current.get()))
        .child(flat_layer(|| theme().task_hover, hovered))
        .child(flat_layer(|| theme().task_pressed, move || ui.pressed()));
    let progress = div()
        .absolute()
        .left(2.0)
        .top(2.0)
        .fill(move || {
            let urgent = shows.get().is_some_and(|e| e.urgent);
            glossy(if urgent { URGENT } else { PROGRESS })
        })
        .dyn_style(move |s| {
            let t = theme();
            let entry = shows.get();
            let urgent = entry.is_some_and(|e| e.urgent);
            let share = if urgent {
                Some(1.0)
            } else {
                entry.and_then(|e| e.progress)
            };
            let s = s
                .h(t.task_button_height - 4.0)
                .rounded(corner((item_radius() - 1.0).max(0.0)))
                .opacity(PROGRESS_OPACITY)
                .transition(Transition::ms(motion(t.fade * 2.0)));
            match share {
                Some(share) => s.w(((body_w() - 4.0) * share as f32).max(0.0)),
                None => s.w(0.0).hidden(),
            }
        });
    let alone = div()
        .absolute()
        .left(0.0)
        .top(0.0)
        .bottom(0.0)
        .flex()
        .items_center()
        .justify_center()
        .dyn_style(move |s| {
            let s = s
                .w(body_w())
                .translate_y(if ui.pressed() { 1.0 } else { 0.0 });
            if icon_only() { s } else { s.hidden() }
        })
        .child(shell_icon(
            move || icon.get(),
            untrack(|| theme().task_icon_size),
        ));
    let words = move |color: Color, dy: f32| {
        div()
            .absolute()
            .left(0.0)
            .right(0.0)
            .top(dy)
            .overflow_clip()
            .whitespace_nowrap()
            .text_color(color)
            .child(text(move || label.get()))
    };
    let labelled = div()
        .absolute()
        .left(0.0)
        .top(0.0)
        .bottom(0.0)
        .px(INSIDE)
        .flex()
        .flex_row()
        .items_center()
        .gap(INSIDE)
        .dyn_style(move |s| {
            let s = s.w(body_w());
            if icon_only() { s.hidden() } else { s }
        })
        .child(shell_icon(
            move || icon.get(),
            untrack(|| theme().task_icon_size),
        ))
        .child(
            div()
                .relative()
                .flex_1()
                .min_w(0.0)
                .dyn_style(move |s| {
                    let t = theme();
                    let s = s
                        .font_family(font())
                        .text_size(t.font_size)
                        .h(t.font_size * 1.4);
                    if body_w() > LABEL_ROOM { s } else { s.hidden() }
                })
                .child(words(LABEL_SHADOW, 1.0))
                .child(words(LABEL, 0.0)),
        );
    let badge = div()
        .absolute()
        .top(2.0)
        .z_index(2)
        .h(BADGE_HEIGHT)
        .min_w(BADGE_HEIGHT)
        .px(4.0)
        .border(1.0)
        .border_color(BADGE_BORDER)
        .flex()
        .items_center()
        .justify_center()
        .text_color(Color::WHITE)
        .text_size(BADGE_TEXT)
        .font_weight(FontWeight::BOLD)
        .fill(|| vertical_gradient(&[(0.0, BADGE_TOP), (1.0, BADGE_BOTTOM)]))
        .dyn_style(move |s| {
            let t = theme();
            let left = if icon_only() {
                body_w() / 2.0 + 2.0
            } else {
                INSIDE + t.task_icon_size - 8.0
            };
            let s = s
                .left(left)
                .rounded(corner(BADGE_HEIGHT / 2.0))
                .font_family(font());
            if shows.get().and_then(|e| e.count).is_some_and(|n| n > 0) {
                s
            } else {
                s.hidden()
            }
        })
        .child(text(move || match shows.get().and_then(|e| e.count) {
            Some(n) if n > 99 => "99+".to_owned(),
            Some(n) => n.to_string(),
            None => String::new(),
        }));

    let cancel_rest = {
        let rest = rest.clone();
        move || {
            if let Some(t) = rest.take() {
                t.cancel();
            }
        }
    };
    let center = move || anchor.logical_rect().map_or(0.0, |r| r.x + r.width / 2.0);
    let hover_bar = bar.clone();
    let hover_key = key.clone();
    let hover_cancel = cancel_rest.clone();
    let on_hover = move |inside: bool| {
        hover_cancel();
        if inside && untrack(|| count.get()) > 0 {
            let bar = hover_bar.clone();
            let key = hover_key.clone();
            let wait = Duration::from_millis(untrack(|| theme().thumb_delay) as u64);
            rest.set(Some(set_timeout(wait, move || {
                bar.show_preview(key, center())
            })));
        } else if !inside {
            hover_bar.hide_preview_soon();
        }
    };
    let grab_bar = bar.clone();
    let grab_cancel = cancel_rest.clone();
    let grab_key = key.clone();
    let on_grab = move |g: &GrabEvent| {
        let Some(at) = untrack(|| index.get()) else {
            return;
        };
        let (x, w, last_end) = row.slots.with(|s| {
            let own = &s[at.min(s.len() - 1)];
            let end = s.last().map_or(0.0, |l| l.x + l.w);
            (own.x, own.w, end)
        });
        match g.phase {
            GrabPhase::Start => {
                grab_cancel();
                grab_bar.hide_preview_soon();
                reordering.set(true);
            }
            GrabPhase::Move => {
                let offset = g.total.x.clamp(-x, (last_end - x - w).max(-x));
                drag_offset.set(offset);
                row.drag_from.set(Some(at));
                row.drop_before.set(Some(row.place(x + w / 2.0 + offset)));
            }
            GrabPhase::End => {
                let before = row.drop_before.get_untracked();
                row.drag_from.set(None);
                row.drop_before.set(None);
                reordering.set(false);
                drag_offset.set(0.0);
                if let Some(before) = before {
                    let key = grab_key.clone();
                    // Once this button is done with the release: the change builds every
                    // button again.
                    set_timeout(Duration::ZERO, move || move_button(&key, before));
                }
            }
            GrabPhase::Cancel => {
                row.drag_from.set(None);
                row.drop_before.set(None);
                reordering.set(false);
                drag_offset.set(0.0);
            }
        }
    };
    let click_bar = bar.clone();
    let click_key = key.clone();
    let click_cancel = cancel_rest;
    let menu_bar = bar.clone();
    let menu_key = key.clone();

    // A program pinned but not running: its name under the pointer (a running one shows its
    // windows' thumbnails instead).
    let pinned_name = move || {
        group.with(|g| match g {
            Some(g) if g.windows.is_empty() => g.name.clone(),
            _ => String::new(),
        })
    };
    let button = div()
        .absolute()
        .top(0.0)
        .bottom(0.0)
        .node_ref(anchor)
        .interaction(ui)
        .grab_threshold(DRAG_START)
        .dyn_style(move |s| {
            let Some(at) = slot.get() else {
                return s.hidden();
            };
            let s = s.left(at.x).w(at.w);
            if reordering.get() {
                s.translate_x(drag_offset.get()).z_index(10)
            } else {
                let shift = index.get().map_or(0.0, |i| row.shift_of(i));
                s.translate_x(shift)
                    .z_index(0)
                    .transition(Transition::ms(motion(theme().hover_in)).ease_out())
            }
        })
        .children(stack)
        .children(stack_lines)
        .child(body)
        .child(flat)
        .child(progress)
        .child(alone)
        .child(labelled)
        .child(badge)
        .child(super::jump_list::jump_list(
            bar.clone(),
            key.clone(),
            group,
            anchor,
        ))
        .on_hover(on_hover)
        .on_grab(on_grab)
        .on_pointer_down(move |e| {
            if e.button == Some(PointerButton::Secondary) {
                e.stop_propagation();
                menu_bar.preview.set(None);
                menu_bar.tray_menu.set(None);
                let again =
                    menu_bar.jump_list.get_untracked().as_deref() == Some(menu_key.as_str());
                menu_bar
                    .jump_list
                    .set(if again { None } else { Some(menu_key.clone()) });
            }
        })
        .on_click(move |e| {
            click_cancel();
            let Some(g) = untrack(|| group.get()) else {
                return;
            };
            if e.modifiers.ctrl || e.modifiers.shift {
                if let Some(entry) = &g.entry {
                    launch(entry);
                }
            } else if g.windows.len() > 1 {
                click_bar.show_preview(click_key.clone(), center());
            } else {
                activate_group(&g);
            }
        })
        .on_aux_click(move |e| {
            if e.button == Some(PointerButton::Middle)
                && let Some(entry) = untrack(|| group.get()).and_then(|g| g.entry)
            {
                launch(&entry);
            }
        });
    shell_tooltip(button, away_from_bar(Align::Center), pinned_name)
}
