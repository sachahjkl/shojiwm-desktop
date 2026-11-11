//! The controls of the Control Panel and the dialogs, as Windows 7 draws them (QML `CpButton`,
//! `CpCheck`, `CpRadio`, `CpCombo`, `CpLink`). Each is AntiWebUI's control of that kind (its
//! keys, focus and accessibility) with the theme's look.
//!
//! In light Aero they take Windows 7's own blues under the pointer, which `Scheme` does not
//! hold; the dark colours lighten the scheme's instead, so that the light text stays readable.

use super::gradient::vertical_gradient;
use super::{dark_look, font, ink_asset, label_text, lighter};
use crate::theme::{button_radius, corner, field_radius, is_kernel, scheme, theme};
use anti_webui::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

/// Windows 7's push button under the pointer: top, middle and bottom of its gradient.
const BUTTON_HOT: [Color; 3] = [rgb(0xeaf6fd), rgb(0xd9f0fc), rgb(0xa7d9f5)];
/// The border of a light control under the pointer or with the keyboard.
const LIGHT_HOT_BORDER: Color = rgb(0x3c7fb1);
/// The border of a light push button at rest.
const LIGHT_BUTTON_BORDER: Color = rgb(0x707070);
/// The text of a light push button under the pointer.
const LIGHT_BUTTON_HOT_TEXT: Color = rgb(0x10283c);
/// The line inside a button that has the keyboard or is the default one.
const FOCUS_LINE_DARK: Color = rgba(0xffffff99);
const FOCUS_LINE_LIGHT: Color = rgba(0x00000099);
/// The box of a check box or a radio button: its border at rest, and its gradient under the
/// pointer.
const BOX_BORDER: Color = rgb(0x8e8f8f);
const BOX_HOT: [Color; 2] = [rgb(0xdff2fd), rgb(0xb0dbf5)];
/// The tick: dark blue on the light box, light blue on the dark scheme's box.
const TICK_ON_LIGHT: Color = rgb(0x1e4f7c);
const TICK_ON_DARK: Color = rgb(0x9cd0ff);
/// The dot of a chosen radio button.
const RADIO_DOT: [Color; 2] = [rgb(0x5aa9f0), rgb(0x1c63ae)];
/// The row under the pointer in an open combo box's list, and its text.
const COMBO_LIT: Color = rgb(0x3399ff);
const COMBO_LIT_TEXT: Color = Color::WHITE;
/// A link under the pointer, in light and in dark Aero.
const LINK_HOT_LIGHT: Color = rgb(0x3399ff);
const LINK_HOT_DARK: Color = rgb(0xa8d4ff);

/// The height of a push button and of a combo box's field.
const CONTROL_HEIGHT: f32 = 24.0;
/// The height of a check box's or a radio button's row, and of a combo box's list rows.
const ROW_HEIGHT: f32 = 22.0;
/// The side of a check box's or a radio button's box.
const BOX_SIZE: f32 = 13.0;
/// The longest a combo box's list grows before it scrolls.
const COMBO_LIST_HEIGHT: f32 = 220.0;
/// A wheel on a touchpad steps a combo box's choice once per this many pixels.
const WHEEL_STEP: f32 = 40.0;

/// A push button: a pale gradient that turns blue under the pointer. `on_click` runs when it
/// is clicked, or pressed with Enter or Space while it has the keyboard.
pub fn cp_button(label: impl IntoTextSource, on_click: impl Fn() + 'static) -> Div {
    push_button(label, false, on_click)
}

/// The default push button of a dialog: a line inside it says that Enter presses it.
pub fn cp_default_button(label: impl IntoTextSource, on_click: impl Fn() + 'static) -> Div {
    push_button(label, true, on_click)
}

fn push_button(label: impl IntoTextSource, default: bool, on_click: impl Fn() + 'static) -> Div {
    let ui = Interaction::new();
    let focus_line = div()
        .absolute()
        .inset(2.0)
        .border(1.0)
        .pointer_events_none()
        .dyn_style(move |s| {
            let s = s.rounded(corner(1.0)).border_color(if dark_look() {
                FOCUS_LINE_DARK
            } else {
                FOCUS_LINE_LIGHT
            });
            if default || ui.focus_visible() { s } else { s.hidden() }
        });
    div()
        .relative()
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .flex_none()
        .h(CONTROL_HEIGHT)
        .min_w(86.0)
        .px(12.0)
        .border(1.0)
        .select_none()
        .focusable()
        .interaction(ui)
        .access(|a| a.role(Role::Button))
        .on_click(move |_| on_click())
        .fill(move || {
            let c = scheme();
            let [top, middle, bottom] = if ui.pressed() {
                [c.button_pressed_top, c.button_pressed_bottom, c.button_pressed_bottom]
            } else if ui.hovered() && !dark_look() {
                BUTTON_HOT
            } else if ui.hovered() {
                [c.button_hot_top, c.button_hot_bottom, c.button_hot_bottom]
            } else {
                [c.button_top, c.button_bottom, c.button_bottom]
            };
            vertical_gradient(&[(0.0, top), (0.5, middle), (1.0, bottom)])
        })
        .dyn_style(move |s| {
            let c = scheme();
            let dark = dark_look();
            let hot = ui.hovered() || ui.focus_visible();
            let border = match (hot, dark) {
                (true, true) => c.selection_border,
                (true, false) => LIGHT_HOT_BORDER,
                (false, true) => c.field_border,
                (false, false) => LIGHT_BUTTON_BORDER,
            };
            let text = if ui.hovered() && !dark { LIGHT_BUTTON_HOT_TEXT } else { c.text };
            s.rounded(button_radius())
                .border_color(border)
                .text_color(text)
                .font_family(font())
                .text_size(theme().font_size)
        })
        .child(text(label))
        .child(focus_line)
}

/// The box of a check box or a radio button, 13 px square: the field's colour, blue under the
/// pointer. `round` for a radio button.
fn choice_box(round: bool, hovered: impl Fn() -> bool + Copy + 'static) -> Div {
    div()
        .flex_none()
        .w(BOX_SIZE)
        .h(BOX_SIZE)
        .border(1.0)
        .flex()
        .items_center()
        .justify_center()
        .fill(move || {
            let field = scheme().field;
            let [top, bottom] = if hovered() { BOX_HOT } else { [field, field] };
            vertical_gradient(&[(0.0, top), (1.0, bottom)])
        })
        .dyn_style(move |s| {
            let s = s.border_color(if hovered() { LIGHT_HOT_BORDER } else { BOX_BORDER });
            if round { s.rounded(BOX_SIZE / 2.0) } else { s }
        })
}

/// A choice's row: its box, then its label.
fn choice_row(indicator: Div, label: impl IntoTextSource) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(7.0)
        .h(ROW_HEIGHT)
        .select_none()
        .child(indicator)
        .child(label_text(label).whitespace_nowrap())
}

/// A check box with its label, bound to `checked`: a signal, or
/// `Binding::new(get, set)` for a setting.
pub fn cp_check(label: impl IntoTextSource, checked: impl Into<Binding<bool>>) -> Div {
    checkbox_with(checked, move |s| {
        let hovered = move || s.hovered();
        let tick = text(move || if s.checked() { "\u{2714}" } else { "" })
            .text_size(11.0)
            .line_height(LineHeight::Relative(1.0))
            .dyn_style(move |st| {
                st.text_color(if s.hovered() || !dark_look() {
                    TICK_ON_LIGHT
                } else {
                    TICK_ON_DARK
                })
            });
        choice_row(choice_box(false, hovered).child(tick), label)
    })
}

/// A radio button with its label: it sets `selected` to `value` when clicked, and is chosen
/// while `selected` is `value`.
pub fn cp_radio<T: Clone + PartialEq + 'static>(
    label: impl IntoTextSource,
    selected: impl Into<Binding<T>>,
    value: T,
) -> Div {
    radio_with(selected, value, move |s| {
        let hovered = move || s.hovered();
        let dot = div()
            .w(7.0)
            .h(7.0)
            .rounded(3.5)
            .fill(|| vertical_gradient(&[(0.0, RADIO_DOT[0]), (1.0, RADIO_DOT[1])]))
            .dyn_style(move |st| if s.selected() { st } else { st.hidden() });
        choice_row(choice_box(true, hovered).child(dot), label)
    })
}

/// A combo box: the label of the choice `current` (an index into `labels`) in a field, the
/// drop button at its end, and under it the list, blue under the pointer, which scrolls past
/// 220 px. The wheel on the field steps the choice. 220 px wide unless given a width.
pub fn cp_combo(labels: Vec<String>, current: impl Into<Binding<usize>>) -> Div {
    let current: Binding<usize> = current.into();
    let count = labels.len();
    let labels = Rc::new(labels);
    let hovered = signal(false);
    let wheel = Rc::new(Cell::new(0.0f32));
    let field = select_with(current, (0..count).collect(), move |s| {
        let shown = labels.clone();
        let drop_button = div()
            .flex_none()
            .w(18.0)
            .self_stretch()
            .border(1.0)
            .flex()
            .items_center()
            .justify_center()
            .fill(move || {
                let c = scheme();
                let [top, bottom] = if s.is_open() {
                    [c.button_pressed_top, c.button_pressed_bottom]
                } else if hovered.get() {
                    [c.button_hot_top, c.button_hot_bottom]
                } else {
                    [c.field, c.field]
                };
                vertical_gradient(&[(0.0, top), (1.0, bottom)])
            })
            .dyn_style(move |st| {
                st.border_color(if hovered.get() || s.is_open() {
                    scheme().selection_border
                } else {
                    Color::TRANSPARENT
                })
            })
            .child(
                img(|| ink_asset("combo-arrow.svg"))
                    .w(8.0)
                    .h(5.0)
                    .object_fit(ObjectFit::Contain),
            );
        let field = div()
            .flex()
            .flex_row()
            .items_center()
            .w(220.0)
            .h(CONTROL_HEIGHT)
            .border(1.0)
            .on_hover(move |on| hovered.set(on))
            .dyn_style(move |st| {
                let c = scheme();
                st.rounded(field_radius())
                    .bg(c.field)
                    .border_color(if hovered.get() || s.is_open() {
                        c.field_focus
                    } else {
                        c.field_border
                    })
                    .font_family(font())
                    .text_size(theme().font_size)
                    .text_color(c.text)
            })
            .child(
                div()
                    .flex_1()
                    .min_w(0.0)
                    .pl(5.0)
                    .pr(4.0)
                    .overflow_clip()
                    .whitespace_nowrap()
                    .child(text(move || {
                        s.selected()
                            .and_then(|i| shown.get(i).cloned())
                            .unwrap_or_default()
                    })),
            )
            .child(drop_button);
        let listed = labels.clone();
        let list = move || {
            div()
                .flex()
                .flex_col()
                .max_h(COMBO_LIST_HEIGHT)
                .overflow_y_scroll()
                .border(1.0)
                .p(0.0)
                .dyn_style(|st| {
                    let c = scheme();
                    st.bg(c.field)
                        .border_color(c.field_focus)
                        .font_family(font())
                        .text_size(theme().font_size)
                })
                .children(listed.iter().enumerate().map(|(i, name)| {
                    let row = div()
                        .flex_none()
                        .flex()
                        .items_center()
                        .h(ROW_HEIGHT)
                        .pl(6.0)
                        .whitespace_nowrap()
                        .dyn_style(move |st| {
                            if s.is_lit(i) {
                                st.bg(COMBO_LIT).text_color(COMBO_LIT_TEXT)
                            } else {
                                st.bg(Color::TRANSPARENT).text_color(scheme().text)
                            }
                        })
                        .child(text(name.clone()));
                    s.option(i, row)
                }))
        };
        (field, list)
    });
    field.on_scroll(move |e| {
        if count == 0 {
            return;
        }
        let steps = if e.source == ScrollSource::Wheel {
            e.delta_y.signum() as i32
        } else {
            let total = wheel.get() + e.delta_y;
            let steps = (total / WHEEL_STEP).trunc();
            wheel.set(total - steps * WHEEL_STEP);
            steps as i32
        };
        if steps != 0 {
            e.stop_propagation();
            let at = current.get_untracked() as i32 + steps;
            current.set(at.clamp(0, count as i32 - 1) as usize);
        }
    })
}

/// A link of the Control Panel in `link2`, underlined and lighter under the pointer. `large`
/// for a page's main links.
pub fn cp_link(label: impl IntoTextSource, large: bool, on_click: impl Fn() + 'static) -> Div {
    cp_link_tinted(label, large, || scheme().link2, on_click)
}

/// A link in a colour of its own, as a Kernel category gives its links.
pub fn cp_link_tinted(
    label: impl IntoTextSource,
    large: bool,
    tint: impl Fn() -> Color + 'static,
    on_click: impl Fn() + 'static,
) -> Div {
    let ui = Interaction::new();
    div()
        .flex_none()
        .whitespace_nowrap()
        .select_none()
        .cursor(CursorIcon::Pointer)
        .focusable()
        .interaction(ui)
        .access(|a| a.role(Role::Link))
        .on_click(move |_| on_click())
        .dyn_style(move |s| {
            let t = theme();
            let rest = tint();
            let colour = if !ui.hovered() {
                rest
            } else if is_kernel() {
                lighter(rest, 1.35)
            } else if dark_look() {
                LINK_HOT_DARK
            } else {
                LINK_HOT_LIGHT
            };
            let s = s
                .font_family(font())
                .text_size(if large { t.font_size + 3.0 } else { t.font_size })
                .text_color(colour);
            if ui.hovered() { s.underline() } else { s }
        })
        .child(text(label))
}
