//! The frame of the popups (the calendar, the network, the volume, Bluetooth, the
//! notifications, the Action Center, the menus) and of the tooltips.

use super::gradient::vertical_gradient;
use super::{clock_font, font};
use crate::theme::{is_kernel, item_radius, popup_radius, scheme, theme};
use anti_webui::prelude::*;
use std::rc::Rc;
use std::time::Duration;

/// The drop shadow under a popup. The QML shell had none (Quickshell's popups could not);
/// this is Windows 7's soft one.
const POPUP_SHADOW: Color = rgba(0x00000047);

/// How long the pointer rests on something before its tooltip shows (the QML shell's).
const TOOLTIP_WAIT: Duration = Duration::from_millis(600);

/// The light line just inside the outline that every Windows 7 menu has, and the shadow.
fn frame_shadows(inner_line: Color) -> Vec<BoxShadow> {
    vec![
        BoxShadow {
            spread: 1.0,
            color: inner_line,
            inset: true,
            ..Default::default()
        },
        BoxShadow {
            offset_y: 2.0,
            blur: 8.0,
            color: POPUP_SHADOW,
            ..Default::default()
        },
    ]
}

/// The box of a popup: a gradient from `popup_top` to `popup_bottom`, a 1 px `popup_outline`,
/// the `popup_inner_line` inside it, `popup_radius` corners and a drop shadow. A column; the
/// text in it has the theme's face, size and colour. The caller gives it its padding and
/// size, and puts it in a `popup()`.
pub fn popup_frame() -> Div {
    div()
        .flex()
        .flex_col()
        .border(1.0)
        .select_none()
        .fill(|| {
            let c = scheme();
            vertical_gradient(&[(0.0, c.popup_top), (1.0, c.popup_bottom)])
        })
        .dyn_style(|s| {
            let c = scheme();
            s.rounded(popup_radius())
                .border_color(c.popup_outline)
                .shadows(frame_shadows(c.popup_inner_line))
                .font_family(font())
                .text_size(theme().font_size)
                .text_color(c.text)
        })
}

/// The box of a tooltip: the popup's gradient with `item_radius` corners, outlined with the
/// accent in Kernel and with `popup_outline` in Aero. Small text in the clock's face, lines
/// kept as they are. For a tooltip with more than a line of text in it; see
/// [`shell_tooltip`] for one of text.
pub fn tooltip_frame() -> Div {
    div()
        .flex()
        .flex_col()
        .border(1.0)
        .px(8.0)
        .py(5.0)
        .whitespace_nowrap()
        .select_none()
        .fill(|| {
            let c = scheme();
            vertical_gradient(&[(0.0, c.popup_top), (1.0, c.popup_bottom)])
        })
        .dyn_style(|s| {
            let c = scheme();
            let outline = if is_kernel() { c.selection_border } else { c.popup_outline };
            s.rounded(item_radius())
                .border_color(outline)
                .shadow(BoxShadow {
                    offset_y: 1.0,
                    blur: 4.0,
                    color: POPUP_SHADOW,
                    ..Default::default()
                })
                .font_family(clock_font())
                .text_size(theme().font_size_small)
                .text_color(c.text)
        })
}

/// `target` with a tooltip of `label` (lines split by `\n`): it shows once the pointer has
/// rested on `target` for 600 ms, `placement` from it, and goes when the pointer leaves or
/// presses. For something on the bar, give `away_from_bar(Align::Center)`. The text is read
/// each time it shows. `target` is what comes back.
pub fn shell_tooltip(
    target: Div,
    placement: Placement,
    label: impl Fn() -> String + 'static,
) -> Div {
    let label: Rc<dyn Fn() -> String> = Rc::new(label);
    let gap = untrack(|| theme().popup_gap) as i32 + 2;
    let options = TooltipOptions::new()
        .show_after(TOOLTIP_WAIT)
        .on_focus(false)
        .placement(placement)
        .gap(gap);
    tooltip_with(target, options, move |_| {
        move || {
            let label = label.clone();
            tooltip_frame().child(text(move || label()))
        }
    })
}
