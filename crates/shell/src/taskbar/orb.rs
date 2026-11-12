//! The Start button at the left end of the bar, in the theme's own look: Aero's orb (QML
//! `StartOrb`) or Kernel's launcher plate (QML `KernelLauncher`). A click toggles the Start
//! menu; a right click the power user's menu (`AdminMenu`).

use super::Bar;
use crate::paths::asset;
use crate::settings::motion;
use crate::theme::{is_kernel, theme};
use crate::ui::{lighter, vertical_gradient};
use anti_webui::prelude::*;

/// The orb's pale glow behind the snowflake: faint at rest, wider and brighter under the
/// pointer, brightest while the menu is open (the QML's opacity and scale, by state).
const GLOW_REST: (f32, f32) = (0.35, 1.05);
const GLOW_HOT: (f32, f32) = (0.9, 1.2);
const GLOW_OPEN: (f32, f32) = (1.0, 1.3);
/// How soft the glow is, in logical pixels.
const GLOW_BLUR: f32 = 4.0;
/// The share of the orb the snowflake takes.
const LOGO_SHARE: f32 = 0.6;
/// The share of the Kernel plate's height the snowflake takes.
const PLATE_LOGO_SHARE: f32 = 0.56;
/// The shade over a pressed orb, as if pushed in.
const PRESSED_SHADE: Color = rgba(0x0000002e);
/// How much lighter Kernel's plate is under the pointer.
const PLATE_HOT: f32 = 1.15;

/// The Start button of the theme in use.
pub(super) fn start_button(bar: Bar) -> View {
    let kernel = memo(is_kernel);
    dyn_view(move || {
        let bar = bar.clone();
        let kernel = kernel.get();
        // Built again for a change of theme only, not for every setting it reads.
        untrack(move || {
            if kernel {
                kernel_plate(bar)
            } else {
                aero_orb(bar)
            }
        })
    })
}

/// What a click and a right click on the Start button do.
fn with_clicks(button: Div, bar: Bar) -> Div {
    let left = bar.clone();
    button
        // Its own presses: its own menu, not the bar's, and its click opens or closes the Start
        // menu, which a press reaching the bar would close first.
        .on_pointer_down(|e| e.stop_propagation())
        .on_click(move |_| {
            left.close_popups();
            crate::start_menu::toggle_on(&left.name);
        })
        .on_aux_click(move |e| {
            if e.button == Some(PointerButton::Secondary) {
                bar.close_popups();
                crate::dialogs::admin_menu_toggle_on(&bar.name);
            }
        })
}

/// A picture of `shell/assets` over the whole of a box `size` square.
fn layer(name: &str, size: f32) -> Img {
    img(asset(name))
        .absolute()
        .left(0.0)
        .top(0.0)
        .size(size)
        .object_fit(ObjectFit::Contain)
}

/// Aero's orb, in layers as on Windows 7: the orb, its lit copy, the snowflake's glow, the
/// snowflake, then the glass over them. A press pushes it in.
fn aero_orb(bar: Bar) -> Div {
    let ui = Interaction::new();
    let name = bar.name.clone();
    let open = memo(move || crate::start_menu::is_open_on(&name));
    let hot = move || ui.hovered() || open.get();
    let size = theme().orb_size;
    let logo = (size * LOGO_SHARE).round();
    let inset = (size - logo) / 2.0;
    let fade = move || Transition::ms(motion(theme().fade) * 2);
    let orb = div()
        .relative()
        .flex_none()
        .size(size)
        .select_none()
        .interaction(ui)
        .child(layer("start-orb.svg", size))
        .child(
            layer("start-orb-hot.svg", size)
                .dyn_style(move |s| s.opacity(if hot() { 1.0 } else { 0.0 }).transition(fade())),
        )
        .child(
            img(asset("start-orb-logo.svg"))
                .absolute()
                .left(inset)
                .top(inset)
                .size(logo)
                .blur(GLOW_BLUR)
                .pointer_events_none()
                .dyn_style(move |s| {
                    let (opacity, scale) = if open.get() {
                        GLOW_OPEN
                    } else if ui.hovered() {
                        GLOW_HOT
                    } else {
                        GLOW_REST
                    };
                    s.opacity(opacity).scale(scale).transition(fade())
                }),
        )
        .child(
            img(asset("start-orb-logo.svg"))
                .absolute()
                .left(inset)
                .top(inset)
                .size(logo)
                .dyn_style(move |s| {
                    s.scale(if ui.pressed() { 0.94 } else { 1.0 })
                        .opacity(if hot() { 1.0 } else { 0.92 })
                        .transition(Transition::ms(motion(theme().fade)))
                }),
        )
        .child(layer("start-orb-gloss.svg", size))
        .child(
            div()
                .absolute()
                .left(size * 0.1)
                .top(size * 0.1)
                .size(size * 0.8)
                .rounded(size * 0.4)
                .bg(PRESSED_SHADE)
                .dyn_style(move |s| s.opacity(if ui.pressed() { 1.0 } else { 0.0 })),
        );
    with_clicks(orb, bar)
}

/// Kernel's Start button: a plate in the steel blue of the window captions, as tall as the bar,
/// with the snowflake in white. Lighter under the pointer, darker while the menu is open or the
/// button held.
fn kernel_plate(bar: Bar) -> Div {
    let ui = Interaction::new();
    let name = bar.name.clone();
    let open = memo(move || crate::start_menu::is_open_on(&name));
    let sunk = move || ui.pressed() || open.get();
    let t = theme();
    let logo = (t.taskbar_height * PLATE_LOGO_SHARE).round();
    let plate = div()
        .relative()
        .flex_none()
        .w(t.orb_size)
        .h(t.taskbar_height)
        .flex()
        .items_center()
        .justify_center()
        .select_none()
        .interaction(ui)
        .fill(move || {
            let t = theme();
            let (top, bottom) = if sunk() {
                (t.button_pressed_top, t.button_pressed_bottom)
            } else if ui.hovered() {
                (
                    lighter(t.button_active_top, PLATE_HOT),
                    lighter(t.button_active_bottom, PLATE_HOT),
                )
            } else {
                (t.button_active_top, t.button_active_bottom)
            };
            vertical_gradient(&[(0.0, top), (1.0, bottom)])
        })
        // The line of light along its top (none while sunk) and the dark seam on its right.
        .child(
            div()
                .absolute()
                .left(0.0)
                .right(0.0)
                .top(0.0)
                .h(1.0)
                .dyn_style(move |s| {
                    let s = s.bg(theme().bar_highlight);
                    if sunk() { s.hidden() } else { s }
                }),
        )
        .child(
            div()
                .absolute()
                .top(0.0)
                .bottom(0.0)
                .right(0.0)
                .w(1.0)
                .dyn_style(|s| s.bg(theme().bar_edge)),
        )
        .child(
            img(asset("start-orb-logo.svg"))
                .size(logo)
                .object_fit(ObjectFit::Contain)
                .dyn_style(move |s| s.opacity(if sunk() { 0.85 } else { 1.0 })),
        );
    with_clicks(plate, bar)
}
