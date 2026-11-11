//! Ease of Access: the system's animations switch, and the speed of the shell's and the
//! windows' animations.

use super::common::{control_row, page_column, tinted_text};
use crate::control_panel::{heading, note, sub_heading};
use crate::settings::{animations, change, set_animations, settings};
use crate::theme::{corner, scheme, theme};
use crate::ui::{cp_check, cp_link, font, vertical_gradient};
use anti_webui::prelude::*;

/// The slowest and the fastest speed, and the step between speeds.
const SPEED_LOW: f32 = 0.25;
const SPEED_HIGH: f32 = 3.0;
const SPEED_STEP: f32 = 0.25;
/// The slider's track, and the room at each end where the pointer still reaches its ends.
const TRACK_WIDTH: f32 = 260.0;
const TRACK_INSET: f32 = 6.0;
const SLIDER_HEIGHT: f32 = 24.0;
/// The slider's thumb.
const THUMB_WIDTH: f32 = 10.0;
const THUMB_HEIGHT: f32 = 20.0;

pub fn page() -> Div {
    let off = Binding::new(|| !animations(), |on: bool| set_animations(!on));
    page_column(10.0)
        .child(heading("Make the computer easier to see"))
        .child(cp_check(
            "Turn off all unnecessary animations (when possible)",
            off,
        ))
        .child(note(
            "This is the system's own setting: GTK, libadwaita and the programs that read the \
             settings portal follow it too. The shell's fades and the windows' animations stop \
             at once.",
        ))
        .child(sub_heading("Animation speed"))
        .child(note(
            "How fast windows open, close, minimise and snap, and how fast the menus fade. 1x \
             is the speed of Windows 7.",
        ))
        .child(speed_row())
}

/// "Slower", the slider, "Faster", the speed, and a link back to 1x. Dimmed and inert while
/// the animations are off.
fn speed_row() -> Div {
    let speed = Binding::new(
        || settings().with(|s| s.animation_speed) as f32,
        |v: f32| change(|s| s.animation_speed = v as f64),
    );
    let muted = || scheme().text_muted;
    control_row(12.0)
        .dyn_style(|s| s.opacity(if animations() { 1.0 } else { 0.5 }))
        .child(tinted_text("Slower", muted))
        .child(speed_slider(speed).disabled(|| !animations()))
        .child(tinted_text("Faster", muted))
        .child(
            text(|| format!("{:.2}x", settings().with(|s| s.animation_speed)))
                .dyn_style(|s| {
                    s.font_family(font())
                        .text_size(theme().font_size)
                        .text_color(scheme().text)
                        .bold()
                }),
        )
        .child(show(
            || settings().with(|s| s.animation_speed) != 1.0,
            || {
                cp_link("Reset", false, || change(|s| s.animation_speed = 1.0))
                    .disabled(|| !animations())
            },
        ))
}

/// A track from 0.25x to 3x in steps of 0.25x, with a tick at 1x and the thumb of a push
/// button, which lights up under the pointer.
fn speed_slider(speed: Binding<f32>) -> Div {
    let options = SliderOptions::new().step(SPEED_STEP).inset(TRACK_INSET);
    slider_with(speed, SPEED_LOW, SPEED_HIGH, options, |s| {
        let lit = move || s.hovered() || s.dragging();
        let track = div()
            .absolute()
            .left(TRACK_INSET)
            .top((SLIDER_HEIGHT - 4.0) / 2.0)
            .w(TRACK_WIDTH)
            .h(4.0)
            .border(1.0)
            .dyn_style(|st| {
                let c = scheme();
                st.rounded(corner(2.0)).bg(c.field).border_color(c.field_border)
            });
        let one = (1.0 - SPEED_LOW) / (SPEED_HIGH - SPEED_LOW);
        let tick = div()
            .absolute()
            .left(TRACK_INSET + one * TRACK_WIDTH - 0.5)
            .top(SLIDER_HEIGHT - 5.0)
            .w(1.0)
            .h(5.0)
            .dyn_style(|st| st.bg(scheme().field_border));
        let thumb = div()
            .absolute()
            .top((SLIDER_HEIGHT - THUMB_HEIGHT) / 2.0)
            .w(THUMB_WIDTH)
            .h(THUMB_HEIGHT)
            .border(1.0)
            .fill(move || {
                let c = scheme();
                let (top, bottom) = if lit() {
                    (c.button_hot_top, c.button_hot_bottom)
                } else {
                    (c.button_top, c.button_bottom)
                };
                vertical_gradient(&[(0.0, top), (1.0, bottom)])
            })
            .dyn_style(move |st| {
                let c = scheme();
                st.left(TRACK_INSET + s.fraction() * TRACK_WIDTH - THUMB_WIDTH / 2.0)
                    .rounded(corner(2.0))
                    .border_color(if lit() { c.field_focus } else { c.field_border })
            });
        div()
            .relative()
            .flex_none()
            .w(TRACK_WIDTH + 2.0 * TRACK_INSET)
            .h(SLIDER_HEIGHT)
            .child(track)
            .child(tick)
            .child(thumb)
    })
}
