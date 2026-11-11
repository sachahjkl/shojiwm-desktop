//! The Start menu's power button and its arrow (QML `PowerPart`, `PowerMenu`), and the red split
//! button of the log-on screen (QML `PowerSplitButton`). The actions are `crate::power`'s.

use super::close;
pub use crate::power::{POWER_ACTIONS, power_action, power_button_action, run_power};
use crate::theme::{button_radius, corner, theme};
use crate::ui::{MenuLine, font, popup_menu, vertical_gradient};
use anti_webui::prelude::*;

// ----- the Start menu's power button ------------------------------------------------------------

/// The outline of the glass button (QML literal).
const GLASS_OUTLINE: Color = rgba(0x000000a6);
/// The glass's fill, top and bottom: at rest, under the pointer, pressed (QML literals).
const GLASS_REST: [Color; 2] = [rgba(0xffffff40), rgba(0xffffff1f)];
const GLASS_HOT: [Color; 2] = [rgba(0xffffff66), rgba(0xffffff38)];
const GLASS_PRESSED: [Color; 2] = [rgba(0xffffff26), rgba(0xffffff14)];
/// The light rim inside the outline: at rest, under the pointer, pressed (QML literals).
const RIM_REST: Color = rgba(0xffffff66);
const RIM_HOT: Color = rgba(0xffffff99);
const RIM_PRESSED: Color = rgba(0xffffff33);
/// The faint light line beside the seam between the two halves (QML literal).
const SEAM_LIGHT: Color = rgba(0xffffff26);
/// The label, white, set into the glass by a dark line under it (QML literals).
const LABEL: Color = Color::WHITE;
const LABEL_SHADOW: Color = rgba(0x00000099);
/// The arrow's triangle (QML literal).
const ARROW: Color = rgba(0xfffffff2);

/// The button's height, its narrowest, the room each side of its label, and the arrow's width
/// (QML).
const BUTTON_HEIGHT: f32 = 25.0;
const BUTTON_MIN_WIDTH: f32 = 56.0;
const LABEL_ROOM: f32 = 13.0;
const ARROW_WIDTH: f32 = 20.0;

/// One half of the power button: the button itself (`left`), or the arrow beside it. The two
/// halves share an edge and round only their outer corners: each is a box rounded all round,
/// wider than the half on its inner side and clipped there, so the glass, which is
/// translucent, is painted once.
fn power_part(
    left: bool,
    hot: impl Fn() -> bool + 'static,
    pressed: impl Fn() -> bool + 'static,
) -> Div {
    let hot = std::rc::Rc::new(hot);
    let pressed = std::rc::Rc::new(pressed);
    let (hot_fill, pressed_fill) = (hot.clone(), pressed.clone());
    let (hot_rim, pressed_rim) = (hot.clone(), pressed.clone());
    let overhang = move || button_radius() + 1.0;
    let shape = div()
        .absolute()
        .top(0.0)
        .bottom(0.0)
        .border(1.0)
        .border_color(GLASS_OUTLINE)
        .fill(move || {
            let f = if pressed_fill() {
                GLASS_PRESSED
            } else if hot_fill() {
                GLASS_HOT
            } else {
                GLASS_REST
            };
            vertical_gradient(&[(0.0, f[0]), (1.0, f[1])])
        })
        .dyn_style(move |s| {
            let s = s.rounded(button_radius());
            if left {
                s.left(0.0).right(-overhang())
            } else {
                s.left(-overhang()).right(0.0)
            }
        })
        // The light rim inside the outline, all round.
        .child(div().absolute().inset(0.0).border(1.0).dyn_style(move |s| {
            let rim = if pressed_rim() {
                RIM_PRESSED
            } else if hot_rim() {
                RIM_HOT
            } else {
                RIM_REST
            };
            s.rounded(corner((button_radius() - 1.0).max(0.0)))
                .border_color(rim)
        }));
    let part = div()
        .relative()
        .flex_none()
        .h_full()
        .overflow_clip()
        .child(shape);
    if left {
        return part;
    }
    // The seam between the halves, on the arrow's left edge: the outline's line, and a faint
    // light one beside it.
    part.child(
        div()
            .absolute()
            .left(0.0)
            .top(0.0)
            .bottom(0.0)
            .w(1.0)
            .bg(GLASS_OUTLINE),
    )
    .child(
        div()
            .absolute()
            .left(1.0)
            .top(1.0)
            .bottom(1.0)
            .w(1.0)
            .bg(SEAM_LIGHT),
    )
}

/// The power button at the foot of the places: glass, as on Windows 7, as wide as its label and
/// a margin each side. Its arrow opens the other actions.
pub(super) fn power_button() -> Div {
    let button = Interaction::new();
    let arrow = Interaction::new();
    let menu_open = signal(false);
    let arrow_anchor = node_ref();
    let label = move || power_action(power_button_action()).map_or("", |a| a.label);
    // White, set into the glass by a dark line under it. The label alone, as Windows 7 has it;
    // Vista had the icon.
    let words = div()
        .relative()
        .whitespace_nowrap()
        .dyn_style(|s| s.font_family(font()).text_size(theme().font_size))
        .child(
            div()
                .absolute()
                .left(0.0)
                .top(1.0)
                .text_color(LABEL_SHADOW)
                .child(text(label)),
        )
        .child(div().relative().text_color(LABEL).child(text(label)));
    let main = power_part(true, move || button.hovered(), move || button.pressed())
        .min_w(BUTTON_MIN_WIDTH)
        .px(LABEL_ROOM)
        .flex()
        .items_center()
        .justify_center()
        .interaction(button)
        .on_click(|_| {
            let id = untrack(power_button_action);
            close();
            run_power(id);
        })
        .child(words);
    let side = power_part(
        false,
        move || arrow.hovered() || menu_open.get(),
        move || arrow.pressed() || menu_open.get(),
    )
    .w(ARROW_WIDTH)
    .flex()
    .items_center()
    .justify_center()
    .node_ref(arrow_anchor)
    .interaction(arrow)
    .on_click(move |_| menu_open.update(|o| *o = !*o))
    .child(
        div()
            .relative()
            .text_size(7.0)
            .text_color(ARROW)
            .child("\u{25B6}"),
    )
    .child(power_menu(menu_open, arrow_anchor));
    div()
        .absolute()
        .flex()
        .flex_row()
        .h(BUTTON_HEIGHT)
        .dyn_style(|s| {
            let pad = theme().menu_padding + 2.0;
            s.left(pad).bottom(pad)
        })
        .child(main)
        .child(side)
}

/// What the arrow beside the power button opens: every action but the one the button runs, to
/// the right of the arrow, as on Windows.
fn power_menu(open: Signal<bool>, anchor: NodeRef) -> Show {
    let options = MenuOptions::new()
        .anchor(anchor)
        .placement(Placement::RightStart)
        .gap(untrack(|| theme().popup_gap) as i32);
    popup_menu(
        move || open.get(),
        move || open.set(false),
        options,
        || {
            let button = untrack(power_button_action);
            POWER_ACTIONS
                .iter()
                .filter(|a| a.id != button)
                .map(|a| {
                    if a.id == "separator" {
                        MenuLine::separator()
                    } else {
                        let id = a.id;
                        MenuLine::item(a.label, move || {
                            close();
                            run_power(id);
                        })
                    }
                })
                .collect()
        },
    )
}

// ----- the log-on screen's power button ---------------------------------------------------------
