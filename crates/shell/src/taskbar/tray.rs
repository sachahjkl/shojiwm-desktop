//! The programs' icons of the notification area, as Windows 7 has them (QML `TrayArea`,
//! `TrayIcon`, `TrayOverflow`, `TrayMenu`): the icons chosen to show on the taskbar, and left of
//! them the arrow that opens the popover of the hidden ones.
//!
//! Which is which is set in the Control Panel, or by dragging an icon: anywhere onto the taskbar
//! to show it there, onto the arrow, into the popover or off the taskbar to hide it. An icon with
//! no choice made shows while there are three icons or fewer, and waits in the popover beyond.
//! While an icon is dragged over the taskbar, the icons stand aside where it would land.
//!
//! A left click activates an item, a right click opens its menu (its DBusMenu, in the theme's
//! menus). An item that offers nothing but a menu opens it on either button.

use super::Bar;
use crate::settings::{Settings, change, motion, settings};
use crate::theme::{Align, away_from_bar, item_radius, scheme, theme};
use crate::ui::{MenuLine, bar_menu_options, cp_link, popup_frame, popup_menu, shell_icon};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use anti_webui::shell::{MenuToggle, TrayMenuItem, TrayStatus};
use std::path::PathBuf;
use std::time::Duration;

/// The MIME type of a notification-area icon being dragged: its item id.
pub const TRAY_MIME: &str = "application/x-shojiwm-win7-tray-item";

/// A notification-area icon being dragged: its item id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrayDrag(pub String);

/// Up to this many items, the ones with no choice made show on the taskbar; beyond it they wait
/// in the popover.
const SHOWN_BY_DEFAULT: usize = 3;
/// The hover band of an icon on the taskbar's dark glass (in the popover it is the selection).
const HOVER_ON_GLASS: Color = rgba(0xffffff2e);
/// The arrow that opens the popover.
const ARROW_WIDTH: f32 = 18.0;
const ARROW_GLYPH_SIZE: f32 = 8.0;
/// The room between the icons.
const ROW_GAP: f32 = 2.0;
/// The room around an icon in its box, and in a cell of the popover.
const ICON_ROOM: f32 = 8.0;
const CELL_ROOM: f32 = 12.0;
/// The faint copy of a dragged icon where it would land.
const GHOST_OPACITY: f32 = 0.45;
/// The popover's columns, at most.
const COLUMNS: usize = 4;
/// The popover's padding and the room between its parts.
const OVERFLOW_PAD: f32 = 8.0;
const OVERFLOW_GAP: f32 = 6.0;

/// The programs' items, but blueman's: the shell draws its own Bluetooth icon and panel when
/// there is an adapter, and its applet keeps running only for its pairing agent.
fn items() -> Vec<TrayItem> {
    tray()
        .items()
        .into_iter()
        .filter(|i| i.id != "blueman")
        .collect()
}

/// Whether an item shows on the taskbar, or only in the popover of hidden icons: the choice made
/// for it, else shown while there are few items. Reading it tracks it.
pub fn tray_visible(id: &str) -> bool {
    let count = tray().items().len();
    settings().with(|s| {
        if s.tray_show_all || s.tray_shown.iter().any(|x| x == id) {
            true
        } else if s.tray_hidden.iter().any(|x| x == id) {
            false
        } else {
            count <= SHOWN_BY_DEFAULT
        }
    })
}

/// Shows the icon `id` at `index` among `order`, the ids shown now, left to right; the taskbar
/// keeps that order.
pub fn place_tray_icon(id: &str, index: usize, order: &[String]) {
    let mut list: Vec<String> = order.iter().filter(|o| *o != id).cloned().collect();
    list.insert(index.min(list.len()), id.to_owned());
    change(|s| {
        s.tray_shown = list;
        s.tray_hidden = Settings::with_member(&s.tray_hidden, id, false);
    });
}

/// Shows an icon on the taskbar, or hides it in the popover. Hiding one icon while every icon
/// is shown: the others stay shown, now by choice.
pub fn set_tray_visible(id: &str, on: bool) {
    let others: Vec<String> = untrack(|| tray().items())
        .into_iter()
        .map(|i| i.id)
        .filter(|other| other != id)
        .collect();
    change(|s| {
        if !on && s.tray_show_all {
            s.tray_shown = others;
            s.tray_show_all = false;
        }
        s.tray_shown = Settings::with_member(&s.tray_shown, id, on);
        s.tray_hidden = Settings::with_member(&s.tray_hidden, id, !on);
    });
}

/// The icons on the taskbar, left to right: in the order they were put there, then the ones
/// shown by default.
fn shown_items() -> Vec<TrayItem> {
    let order = settings().with(|s| s.tray_shown.clone());
    let mut ranked: Vec<(usize, TrayItem)> = items()
        .into_iter()
        .filter(|i| tray_visible(&i.id))
        .enumerate()
        .map(|(natural, i)| {
            (
                order
                    .iter()
                    .position(|o| *o == i.id)
                    .unwrap_or(order.len() + natural),
                i,
            )
        })
        .collect();
    ranked.sort_by_key(|(rank, _)| *rank);
    ranked.into_iter().map(|(_, i)| i).collect()
}

fn hidden_items() -> Vec<TrayItem> {
    items()
        .into_iter()
        .filter(|i| !tray_visible(&i.id))
        .collect()
}

/// The room an icon takes in the row.
fn slot() -> f32 {
    theme().tray_icon_size + ICON_ROOM + ROW_GAP
}

/// Where among the shown icons a drop at `e` goes.
fn index_at(bar: &Bar, e: &DropEvent, count: usize) -> usize {
    let x = e.position.x / window_scale().as_f32();
    let Some(row) = bar.tray_row.logical_rect() else {
        return count;
    };
    let local = (x - row.x).clamp(0.0, row.width.max(0.0));
    let slot = untrack(slot);
    let half = (slot - ROW_GAP) / 2.0;
    (0..count)
        .find(|i| local < *i as f32 * slot + half)
        .unwrap_or(count)
}

/// The bar takes a notification-area icon dropped anywhere on it: on the icons, at that place
/// among them; elsewhere, at the end. The arrow and the popover take theirs before it.
pub(super) fn take_drops(target: Div, bar: Bar) -> Div {
    let over = bar.clone();
    let left = bar.tray_drop;
    target
        .on_drag_over::<TrayDrag>(move |drag, e| {
            let count = untrack(shown_items).len();
            over.tray_drop
                .set(Some((drag.0.clone(), index_at(&over, e, count))));
        })
        .on_drag_hover(move |inside| {
            if !inside {
                left.set(None);
            }
        })
        .drop_target::<TrayDrag>(move |drag, e| {
            bar.tray_drop.set(None);
            let order: Vec<String> = untrack(shown_items).into_iter().map(|i| i.id).collect();
            let at = index_at(&bar, e, order.len());
            let id = drag.0.clone();
            // After the drag has ended: the change can remove its source.
            set_timeout(Duration::ZERO, move || place_tray_icon(&id, at, &order));
        })
}

/// Hides an icon dropped here, once the drag has ended.
fn hide_dropped(drag: &TrayDrag, _: &DropEvent) {
    let id = drag.0.clone();
    set_timeout(Duration::ZERO, move || set_tray_visible(&id, false));
}

/// The arrow and the shown icons.
pub(super) fn area(bar: Bar) -> Div {
    let shown = memo(shown_items);
    let hidden = memo(hidden_items);
    // The index among the shown icons of the one dragged over the bar, if it is one of them.
    let preview_from = {
        let drop = bar.tray_drop;
        memo(move || {
            let (id, _) = drop.get()?;
            shown.with(|s| s.iter().position(|i| i.id == id))
        })
    };
    let drop = bar.tray_drop;
    let shift_of = move |index: usize| -> f32 {
        let Some((_, to)) = drop.get() else {
            return 0.0;
        };
        let slot = slot();
        match preview_from.get() {
            // From the popover: the icons before the place make room leftward, so the area keeps
            // its width and the drop place does not move.
            None if index < to => -slot,
            None => 0.0,
            Some(from) if index == from => {
                let target = if to > from { to - 1 } else { to };
                (target as f32 - from as f32) * slot
            }
            Some(from) if from < to && index > from && index < to => -slot,
            Some(from) if to <= index && index < from => slot,
            Some(_) => 0.0,
        }
    };
    // The faint copy of an icon dragged in from the popover, in the gap the others leave.
    let incoming = memo(move || {
        let (id, before) = drop.get()?;
        if preview_from.get().is_some() {
            return None;
        }
        items()
            .into_iter()
            .find(|i| i.id == id)
            .map(|i| (i.key(), before))
    });
    let ghost = div()
        .absolute()
        .top(0.0)
        .bottom(0.0)
        .flex()
        .items_center()
        .justify_center()
        .opacity(GHOST_OPACITY)
        .pointer_events_none()
        .dyn_style(move |s| {
            let t = theme();
            let s = s.w(t.tray_icon_size + ICON_ROOM);
            match incoming.get() {
                Some((_, before)) => s.left(before as f32 * slot() - slot()),
                None => s.hidden(),
            }
        })
        .child(dyn_view(move || {
            let (key, _) = incoming.get()?;
            Some(picture(
                move || items().into_iter().find(|i| i.key() == key),
                untrack(|| theme().tray_icon_size),
            ))
        }));
    let icons = div()
        .relative()
        .flex()
        .flex_row()
        .items_center()
        .gap(ROW_GAP)
        .node_ref(bar.tray_row)
        .child(ghost)
        .child(for_each(move || shown.get(), |item| item.key(), {
            let bar = bar.clone();
            move |item: TrayItem| {
                let key = item.key();
                let index = memo(move || shown.with(|s| s.iter().position(|i| i.key() == key)));
                let ghosted = move || index.get().is_some_and(|i| preview_from.get() == Some(i));
                tray_icon(bar.clone(), item, true).dyn_style(move |s| {
                    let shift = index.get().map_or(0.0, shift_of);
                    s.translate_x(shift)
                        .opacity(if ghosted() { GHOST_OPACITY } else { 1.0 })
                        .transition(Transition::ms(motion(theme().hover_in)).ease_out())
                })
            }
        }));
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(ROW_GAP)
        .child(arrow(bar.clone(), hidden))
        .child(icons)
}

/// "Show hidden icons": the arrow, and its popover.
fn arrow(bar: Bar, hidden: Memo<Vec<TrayItem>>) -> Div {
    let ui = Interaction::new();
    let over = signal(false);
    let dragging = memo(|| dragged::<TrayDrag>().is_some());
    let open = bar.overflow_open;
    let click = bar.clone();
    div()
        .relative()
        .flex_none()
        .w(ARROW_WIDTH)
        .flex()
        .items_center()
        .justify_center()
        .interaction(ui)
        .dyn_style(move |s| {
            let s = s.h(theme().tray_icon_size + ICON_ROOM);
            // Kept while an icon is dragged, so that there is somewhere to drop the last one.
            if hidden.with(Vec::is_empty) && !dragging.get() && !over.get() {
                s.hidden()
            } else {
                s
            }
        })
        .child(
            div()
                .absolute()
                .left(0.0)
                .top(0.0)
                .right(0.0)
                .bottom(0.0)
                .bg(HOVER_ON_GLASS)
                .dyn_style(move |s| {
                    let lit = ui.hovered() || open.get() || over.get();
                    s.rounded(item_radius())
                        .opacity(if lit { 1.0 } else { 0.0 })
                        .transition(Transition::ms(motion(theme().fade)))
                }),
        )
        .child(
            div()
                .relative()
                .text_size(ARROW_GLYPH_SIZE)
                .dyn_style(|s| s.text_color(theme().text_on_glass))
                .child(text(
                    move || if open.get() { "\u{25BC}" } else { "\u{25B2}" },
                )),
        )
        .on_click(move |_| {
            let now = !click.overflow_open.get_untracked();
            click.close_popups();
            click.overflow_open.set(now);
        })
        // An icon dropped on the arrow goes to the popover.
        .drop_target::<TrayDrag>(hide_dropped)
        .on_drag_hover(move |inside| over.set(inside))
        .child(overflow(bar, hidden))
}

/// The popover of hidden icons, over the arrow, as Windows 7 shows it: the icons in a grid, and
/// "Customize..." under them, which opens the Control Panel's page for them.
fn overflow(bar: Bar, hidden: Memo<Vec<TrayItem>>) -> Show {
    let open = bar.overflow_open;
    show(
        move || open.get() && !hidden.with(Vec::is_empty),
        move || {
            let bar = bar.clone();
            let customize = bar.clone();
            let dismiss = bar.clone();
            let (placement, gap) =
                untrack(|| (away_from_bar(Align::Center), theme().popup_gap as i32 + 2));
            popup()
                .placement(placement)
                .gap(gap)
                .on_dismiss(move || dismiss.overflow_open.set(false))
                .child(
                    popup_frame()
                        .p(OVERFLOW_PAD)
                        .gap(OVERFLOW_GAP)
                        .items_center()
                        // An icon dragged here from the taskbar is hidden.
                        .drop_target::<TrayDrag>(hide_dropped)
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .flex_wrap()
                                .dyn_style(move |s| {
                                    let columns = hidden.with(Vec::len).clamp(1, COLUMNS) as f32;
                                    s.w(columns * (theme().tray_icon_size + CELL_ROOM))
                                })
                                .child(for_each(
                                    move || hidden.get(),
                                    |item| item.key(),
                                    move |item| {
                                        div()
                                            .flex_none()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .dyn_style(|s| {
                                                s.size(theme().tray_icon_size + CELL_ROOM)
                                            })
                                            .child(tray_icon(bar.clone(), item, false))
                                    },
                                )),
                        )
                        .child(
                            div()
                                .self_stretch()
                                .h(1.0)
                                .dyn_style(|s| s.bg(scheme().menu_divider)),
                        )
                        .child(cp_link("Customize...", false, move || {
                            customize.overflow_open.set(false);
                            crate::control_panel::open_page("notifications");
                        })),
                )
        },
    )
}

/// The file of an icon named `name`, in the program's own folder or in the icon theme.
fn named_icon(name: &str, theme_path: Option<&str>, pixels: u32) -> Option<PathBuf> {
    if let Some(dir) = theme_path.filter(|d| !d.is_empty()) {
        for extension in ["svg", "png"] {
            let file = PathBuf::from(dir).join(format!("{name}.{extension}"));
            if file.is_file() {
                return Some(file);
            }
        }
    }
    anti_webui::shell::icon_path(name, pixels)
}

/// An item's picture, `size` logical pixels square: its attention icon while it asks to be
/// looked at, a name of the icon theme, or the pictures it sends.
fn picture(item: impl Fn() -> Option<TrayItem> + 'static, size: f32) -> Div {
    div().flex_none().size(size).child(dyn_view(move || {
        let item = item()?;
        let icon = match (&item.status, &item.attention_icon) {
            (TrayStatus::NeedsAttention, Some(attention)) => attention.clone(),
            _ => item.icon.clone(),
        };
        let pixels = (size * window_scale().as_f32()).ceil().max(1.0) as u32;
        let named = icon
            .name
            .as_deref()
            .filter(|n| !n.is_empty())
            .and_then(|n| named_icon(n, icon.theme_path.as_deref(), pixels));
        Some(match (named, icon.best(pixels)) {
            (Some(file), _) => img(file)
                .size(size)
                .object_fit(ObjectFit::Contain)
                .into_view(),
            (None, Some(bitmap)) => bitmap_view(move || Some(bitmap.clone()))
                .size(size)
                .object_fit(ObjectFit::Contain)
                .into_view(),
            (None, None) => shell_icon(icon.name.clone().unwrap_or_default(), size).into_view(),
        })
    }))
}

/// The lines of an item's menu, as deep as it goes.
fn tray_lines(item: &TrayItem, entries: &[TrayMenuItem]) -> Vec<MenuLine> {
    entries
        .iter()
        .filter(|m| m.visible)
        .map(|m| {
            if m.separator {
                return MenuLine::separator();
            }
            let line = if m.children.is_empty() {
                let (item, id) = (item.clone(), m.id);
                let line = MenuLine::item(m.label.clone(), move || {
                    let _ = item.menu_clicked(id);
                });
                match m.toggle {
                    Some(MenuToggle::Check(on)) => line.checked(on),
                    Some(MenuToggle::Radio(on)) => line.radio(on),
                    None => line,
                }
            } else {
                MenuLine::submenu(m.label.clone(), tray_lines(item, &m.children))
            };
            line.icon(m.icon_name.clone().unwrap_or_default())
                .enabled(m.enabled)
        })
        .collect()
}

/// One notification-area icon: on the taskbar (`in_taskbar`), or in the popover.
fn tray_icon(bar: Bar, item: TrayItem, in_taskbar: bool) -> Div {
    let key = item.key();
    let id = item.id.clone();
    let now = {
        let key = key.clone();
        memo(move || tray().items().into_iter().find(|i| i.key() == key))
    };
    // Its menu is read the first time it is asked for: ask now, so that it is there on the
    // first right click.
    untrack(|| {
        let _ = item.menu();
    });
    let ui = Interaction::new();
    let anchor = node_ref();
    let menu_open = {
        let key = key.clone();
        let menus = bar.tray_menu;
        memo(move || menus.with(|m| m.as_deref() == Some(key.as_str())))
    };
    // Programs fill some menus only when told they are about to be shown.
    effect(move || {
        if menu_open.get()
            && let Some(item) = untrack(|| now.get())
        {
            let _ = item.menu_about_to_show(0);
        }
    });
    let toggle_menu = {
        let bar = bar.clone();
        let key = key.clone();
        move || {
            let again = bar.tray_menu.get_untracked().as_deref() == Some(key.as_str());
            bar.preview.set(None);
            bar.jump_list.set(None);
            bar.tray_menu
                .set(if again { None } else { Some(key.clone()) });
        }
    };
    // The item was used: a popover holding it closes.
    let used = {
        let open = bar.overflow_open;
        move || {
            if !in_taskbar {
                open.set(false);
            }
        }
    };
    let press_menu = toggle_menu;
    let (click_used, aux_used) = (used, used);
    let drag_id = id.clone();
    let menu_bar = bar.clone();
    let size = untrack(|| theme().tray_icon_size);
    div()
        .relative()
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .node_ref(anchor)
        .interaction(ui)
        .dyn_style(|s| s.size(theme().tray_icon_size + ICON_ROOM))
        .child(
            div()
                .absolute()
                .left(0.0)
                .top(0.0)
                .right(0.0)
                .bottom(0.0)
                .dyn_style(move |s| {
                    let band = if in_taskbar {
                        HOVER_ON_GLASS
                    } else {
                        scheme().selection
                    };
                    s.rounded(item_radius())
                        .bg(band)
                        .opacity(if ui.hovered() { 1.0 } else { 0.0 })
                        .transition(Transition::ms(motion(theme().fade)))
                }),
        )
        .child(picture(move || now.get(), size).relative())
        // A platform drag, so that it can leave the taskbar: dropped where nothing takes it, an
        // icon of the taskbar goes to the popover.
        .draggable(move || {
            DragData::new(TrayDrag(drag_id.clone())).data(TRAY_MIME, drag_id.as_bytes().to_vec())
        })
        .on_drag_end(move |result| {
            if in_taskbar && result == DragResult::Cancelled {
                let id = id.clone();
                set_timeout(Duration::ZERO, move || set_tray_visible(&id, false));
            }
        })
        .on_pointer_down(move |e| {
            let Some(item) = untrack(|| now.get()) else {
                return;
            };
            match e.button {
                Some(PointerButton::Secondary) => {
                    e.stop_propagation();
                    if item.has_menu {
                        press_menu();
                    }
                }
                Some(PointerButton::Primary) if item.item_is_menu && item.has_menu => press_menu(),
                _ => {}
            }
        })
        .on_click(move |_| {
            if let Some(item) = untrack(|| now.get())
                && !item.item_is_menu
            {
                let _ = item.activate(0, 0);
                click_used();
            }
        })
        .on_aux_click(move |e| {
            if e.button == Some(PointerButton::Secondary)
                && let Some(item) = untrack(|| now.get())
                && !item.has_menu
                && !item.item_is_menu
            {
                let _ = item.secondary_activate(0, 0);
                aux_used();
            }
        })
        .child(dyn_view(move || {
            let item = untrack(|| now.get())?;
            // Built again when the menu changes, as it is read or as the program changes it.
            let entries = item.menu();
            let close = menu_bar.clone();
            let lines_item = item.clone();
            Some(popup_menu(
                move || menu_open.get(),
                move || {
                    close.tray_menu.set(None);
                    // A choice made in the menu of a hidden icon closes the popover too.
                    if !in_taskbar {
                        close.overflow_open.set(false);
                    }
                },
                untrack(|| bar_menu_options(anchor)),
                move || tray_lines(&lines_item, &entries),
            ))
        }))
}
