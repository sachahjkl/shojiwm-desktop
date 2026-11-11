//! The pieces of the Windows 7 log-on screen: the account picture in its glass frame (QML
//! `UserTile`), the dark glass button (`LogonButton`), the red power button with its arrow
//! (`PowerSplitButton`), and the pale menu that opens above the power button and the session
//! picker.
//!
//! Windows 7 draws them the same in every theme, so their colours are their own, not the
//! scheme's. Their corners follow the theme (`corner`).

use crate::paths::asset;
use crate::theme::{corner, item_radius, popup_radius, theme};
use crate::ui::{dark_look, font, vertical_gradient};
use anti_webui::prelude::*;
use std::path::PathBuf;

// ----- the account picture ---------------------------------------------------------------------

/// The frame's size, measured on a Windows 7 log-on screen (116 px round a 98 px picture at
/// 800 x 600) and drawn at the size a 1080p screen shows it.
const TILE_SIZE: f32 = 136.0;
/// The glass band round the picture, and the frame's corner.
const TILE_BAND: f32 = 10.0;
const TILE_RADIUS: f32 = 8.0;
/// The shadow under the frame.
const TILE_SHADOW: Color = rgba(0x001a3338);
/// The outer line, the highlight just inside it, the light line and the dark line round the
/// picture: light, then dark (smoked glass).
const TILE_OUTER: [Color; 2] = [rgb(0x1b5f8c), rgb(0x06121c)];
const TILE_HIGHLIGHT: [Color; 2] = [rgb(0xb8ecff), rgb(0x7f9db4)];
const TILE_LIGHT_LINE: [Color; 2] = [rgb(0xa8d8f0), rgb(0x5f7d95)];
const TILE_DARK_LINE: [Color; 2] = [rgb(0x476f8a), rgb(0x0e1a24)];
/// The glass band, top to bottom, as `(offset, colour)`: nearly white at the top, cyan-blue
/// down the sides, blue at the bottom. Light, then dark.
const GLASS: [[(f32, Color); 6]; 2] = [
    [
        (0.0, rgb(0xe0f4f6)),
        (0.1, rgb(0xd2ecf0)),
        (0.35, rgb(0x8fc0d2)),
        (0.6, rgb(0x5f9fc0)),
        (0.9, rgb(0x3f8dbf)),
        (1.0, rgb(0x4a9ccf)),
    ],
    [
        (0.0, rgb(0xa9bccb)),
        (0.1, rgb(0x8ea4b6)),
        (0.35, rgb(0x4f6678)),
        (0.6, rgb(0x2f4556)),
        (0.9, rgb(0x1f3a52)),
        (1.0, rgb(0x2a5170)),
    ],
];
/// Where the glass is cut in two: the gradient has six stops, and a fill takes four.
const GLASS_SPLIT: f32 = 0.6;

/// 0 for the light frame, 1 for the smoked one.
fn shade() -> usize {
    usize::from(dark_look())
}

/// A box `inset` px inside its parent, `radius` round, in a colour of the frame.
fn tile_layer(inset: f32, radius: f32, colors: [Color; 2]) -> Div {
    div()
        .absolute()
        .inset(inset)
        .dyn_style(move |s| s.rounded(corner(radius)).bg(colors[shade()]))
}

/// The account picture in the frame the Windows 7 log-on screen puts round it, 136 px square:
/// a dark blue outer line, a highlight inside it, a band of glass, a light line, then a dark
/// line round the picture, which is square and fills its place. Each layer is a rounded box
/// inside the last, so there is one corner, not several. Smoked glass on dark colours.
pub fn user_tile(picture: impl Fn() -> PathBuf + 'static) -> Div {
    let glass_part = |from: usize, to: usize, top: bool| {
        let (start, end) = if top { (0.0, GLASS_SPLIT) } else { (GLASS_SPLIT, 1.0) };
        div()
            .flex_grow(end - start)
            .flex_basis(0.0)
            .fill(move || {
                let stops = GLASS[shade()];
                let local: Vec<(f32, Color)> = stops[from..=to]
                    .iter()
                    .map(|&(at, c)| ((at - start) / (end - start), c))
                    .collect();
                vertical_gradient(&local)
            })
            .dyn_style(move |s| {
                let r = corner(TILE_RADIUS - 2.0);
                if top { s.rounded_t(r) } else { s.rounded_b(r) }
            })
    };
    let glass = div()
        .absolute()
        .inset(2.0)
        .flex()
        .flex_col()
        .child(glass_part(0, 3, true))
        .child(glass_part(3, 5, false));
    div()
        .relative()
        .flex_none()
        .size(TILE_SIZE)
        .child(
            div()
                .absolute()
                .left(-2.0)
                .right(-2.0)
                .top(0.0)
                .bottom(-3.0)
                .bg(TILE_SHADOW)
                .dyn_style(|s| s.rounded(corner(TILE_RADIUS + 2.0))),
        )
        .child(tile_layer(0.0, TILE_RADIUS, TILE_OUTER))
        .child(tile_layer(1.0, TILE_RADIUS - 1.0, TILE_HIGHLIGHT))
        .child(glass)
        .child(tile_layer(TILE_BAND - 2.0, 2.0, TILE_LIGHT_LINE))
        .child(
            div()
                .absolute()
                .inset(TILE_BAND - 1.0)
                .dyn_style(|s| s.bg(TILE_DARK_LINE[shade()])),
        )
        .child(
            img(picture)
                .absolute()
                .inset(TILE_BAND)
                .object_fit(ObjectFit::Cover),
        )
}

// ----- the glass button ------------------------------------------------------------------------

/// The button's dark line, and its gradient at rest, under the pointer and pressed: the top,
/// the middle, just under it (the gloss ends there), and the bottom.
const BUTTON_LINE: Color = rgb(0x0b2238);
const BUTTON_REST: [Color; 4] = [
    rgba(0x2a5d8fcc),
    rgba(0x1d4a78cc),
    rgba(0x143b63cc),
    rgba(0x1f4f80cc),
];
const BUTTON_HOT: [Color; 4] = [
    rgba(0x3f7bb4cc),
    rgba(0x2c6aa6cc),
    rgba(0x205a93cc),
    rgba(0x2f74b5cc),
];
const BUTTON_PRESSED: [Color; 4] = [
    rgba(0x163a5ecc),
    rgba(0x12304fcc),
    rgba(0x0e2741cc),
    rgba(0x163a5ecc),
];
/// The light rim inside the dark line, at rest and under the pointer.
const BUTTON_RIM: Color = rgba(0xa8cdee7f);
const BUTTON_RIM_HOT: Color = rgba(0xd9eeffb3);

/// A button of the Windows 7 log-on screen (OK, Cancel): dark blue glass, a light rim, a gloss
/// over the top half, white text.
pub fn logon_button(label: &'static str, on_click: impl Fn() + 'static) -> Div {
    let ui = Interaction::new();
    div()
        .relative()
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .flex_none()
        .h(28.0)
        .min_w(94.0)
        .px(15.0)
        .border(1.0)
        .border_color(BUTTON_LINE)
        .select_none()
        .interaction(ui)
        .access(|a| a.role(Role::Button))
        .on_click(move |_| on_click())
        .fill(move || {
            let [top, middle, under, bottom] = if ui.pressed() {
                BUTTON_PRESSED
            } else if ui.hovered() {
                BUTTON_HOT
            } else {
                BUTTON_REST
            };
            vertical_gradient(&[(0.0, top), (0.5, middle), (0.51, under), (1.0, bottom)])
        })
        .dyn_style(|s| {
            s.rounded(corner(3.0))
                .text_color(Color::WHITE)
                .font_family(font())
                .text_size(theme().font_size + 1.0)
        })
        .child(
            div()
                .absolute()
                .inset(0.0)
                .border(1.0)
                .pointer_events_none()
                .dyn_style(move |s| {
                    s.rounded(corner(2.0))
                        .border_color(if ui.hovered() { BUTTON_RIM_HOT } else { BUTTON_RIM })
                }),
        )
        .child(text(label))
}

// ----- the power button ------------------------------------------------------------------------

/// Measured at 38 + 14 x 18 px at 800 x 600: the whole button, and where its arrow part starts.
const POWER_WIDTH: f32 = 70.0;
const POWER_HEIGHT: f32 = 24.0;
const POWER_DIVIDER: f32 = 50.0;
/// The dark outline, which shows between the two parts too.
const POWER_OUTLINE: Color = rgb(0x2a0303);
/// The light rim inside the outline.
const POWER_RIM: Color = rgba(0xffdada4d);
/// A part's gradient at rest, under the pointer and pressed: a pale red gloss over the top
/// half, a deep red below that brightens at the bottom.
const POWER_REST: [Color; 4] = [rgb(0xd98585), rgb(0xb84a4a), rgb(0x860000), rgb(0xc00808)];
const POWER_HOT: [Color; 4] = [rgb(0xeda0a0), rgb(0xcf5c5c), rgb(0xa30000), rgb(0xe01515)];
const POWER_PRESSED: [Color; 4] = [rgb(0xa65454), rgb(0x8c2c2c), rgb(0x5e0000), rgb(0x8a0505)];

/// One part's fill: lit under the pointer, darker while pressed.
fn power_fill(ui: Interaction) -> Div {
    div()
        .absolute()
        .top(1.0)
        .bottom(1.0)
        .fill(move || {
            let [top, gloss, deep, bottom] = if ui.pressed() {
                POWER_PRESSED
            } else if ui.hovered() {
                POWER_HOT
            } else {
                POWER_REST
            };
            vertical_gradient(&[(0.0, top), (0.47, gloss), (0.5, deep), (1.0, bottom)])
        })
        .dyn_style(|s| s.rounded(corner(2.0)))
}

/// A part that takes the clicks, over the drawing.
fn power_area(ui: Interaction, on_click: impl Fn() + 'static) -> Div {
    div()
        .absolute()
        .top(0.0)
        .bottom(0.0)
        .interaction(ui)
        .access(|a| a.role(Role::Button))
        .on_click(move |_| on_click())
}

/// The red power button of the Windows 7 log-on screen, and the arrow joined to its right that
/// opens the other choices: one rounded piece with a dark outline and a dark line between the
/// two parts, and a white ring for the power mark. `on_power` runs for the button,
/// `on_menu` for the arrow.
pub fn power_split_button(on_power: impl Fn() + 'static, on_menu: impl Fn() + 'static) -> Div {
    let main = Interaction::new();
    let arrow = Interaction::new();
    let ring = 16.0;
    div()
        .relative()
        .flex_none()
        .w(POWER_WIDTH)
        .h(POWER_HEIGHT)
        .select_none()
        .child(
            div()
                .absolute()
                .inset(0.0)
                .bg(POWER_OUTLINE)
                .dyn_style(|s| s.rounded(corner(3.0)))
                .child(power_fill(main).left(1.0).w(POWER_DIVIDER - 1.0))
                .child(power_fill(arrow).left(POWER_DIVIDER + 1.0).right(1.0)),
        )
        .child(
            div()
                .absolute()
                .inset(1.0)
                .border(1.0)
                .border_color(POWER_RIM)
                .pointer_events_none()
                .dyn_style(|s| s.rounded(corner(2.0))),
        )
        .child(
            img(asset("power-ring.svg"))
                .absolute()
                .left(((POWER_DIVIDER - ring) / 2.0).round())
                .top(((POWER_HEIGHT - ring) / 2.0).round())
                .size(ring)
                .pointer_events_none(),
        )
        .child(
            div()
                .absolute()
                .left(POWER_DIVIDER)
                .right(0.0)
                .top(0.0)
                .bottom(0.0)
                .flex()
                .items_center()
                .justify_center()
                .pointer_events_none()
                .text_color(Color::WHITE)
                .text_size(7.0)
                .child(text("\u{25B2}")),
        )
        .child(power_area(main, on_power).left(0.0).w(POWER_DIVIDER))
        .child(power_area(arrow, on_menu).left(POWER_DIVIDER).right(0.0))
}

// ----- the menu --------------------------------------------------------------------------------

/// The pale glass of the menu above the power button and the session picker, its outline, its
/// text, and a row under the pointer with its border.
const MENU_FILL: Color = rgba(0xf4f9ffe6);
const MENU_OUTLINE: Color = rgba(0x0000006e);
pub const MENU_TEXT: Color = rgb(0x10283c);
const ROW_HOT: Color = rgba(0x9ad0f766);
const ROW_HOT_BORDER: Color = rgba(0xcfe9ffb3);
/// The height of a row.
const ROW_HEIGHT: f32 = 26.0;

/// The box of the menu: pale glass with a dark outline. A column; add the rows.
pub fn logon_menu() -> Div {
    div()
        .flex()
        .flex_col()
        .p(4.0)
        .border(1.0)
        .border_color(MENU_OUTLINE)
        .bg(MENU_FILL)
        .select_none()
        .dyn_style(|s| {
            s.rounded(popup_radius())
                .font_family(font())
                .text_size(theme().font_size)
        })
}

/// A row of the menu: its label `indent` px from its left edge, light blue under the pointer.
pub fn logon_menu_row(
    label: impl IntoTextSource,
    indent: f32,
    on_click: impl Fn() + 'static,
) -> Div {
    let ui = Interaction::new();
    div()
        .relative()
        .flex()
        .flex_row()
        .items_center()
        .flex_none()
        .h(ROW_HEIGHT)
        // The border is there at rest too, unseen, so that the label does not move.
        .border(1.0)
        .pl(indent - 1.0)
        .pr(16.0)
        .whitespace_nowrap()
        .text_color(MENU_TEXT)
        .interaction(ui)
        .access(|a| a.role(Role::MenuItem))
        .on_click(move |_| on_click())
        .dyn_style(move |s| {
            let hot = ui.hovered();
            s.rounded(item_radius())
                .bg(if hot { ROW_HOT } else { Color::TRANSPARENT })
                .border_color(if hot { ROW_HOT_BORDER } else { Color::TRANSPARENT })
        })
        .child(text(label))
}
