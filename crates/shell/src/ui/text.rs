//! Text in the theme's look: plain labels, section titles, and the shadowed white text of the
//! lock screen.

use super::{font, heading_font, text_fn};
use crate::theme::{scheme, theme};
use anti_webui::prelude::*;

/// The colours of the shadow under [`shadow_text`]: a dark copy a pixel down, and a fainter
/// one wider (QML `ShadowText`). Text has no blur of its own.
/// The log-on screen's shadow under white text, and how soft it is.
const SHADOW: Color = rgba(0x000000b3);
const SHADOW_BLUR: f32 = 2.0;

/// Text in the theme's face and size, in the scheme's text colour.
pub fn label_text(label: impl IntoTextSource) -> Text {
    text(label).dyn_style(|s| {
        s.font_family(font())
            .text_size(theme().font_size)
            .text_color(scheme().text)
    })
}

/// The title of a part of a popup or a page, in `menu_heading`; in Kernel, after a bar in the
/// accent colour. See [`section_title_with`].
pub fn section_title(label: impl IntoTextSource) -> Div {
    section_title_with(label, || scheme().menu_heading, || theme().calendar_today)
}

/// The title of a part of a popup or a page, as the theme draws it. Aero: the words alone.
/// Kernel: a short bar in `accent`, then the words in spaced, monospaced capitals. The words
/// are `color`, in the small size; to change the size, add a later `dyn_style` that sets
/// `text_size`.
pub fn section_title_with(
    label: impl IntoTextSource,
    color: impl Fn() -> Color + 'static,
    accent: impl Fn() -> Color + 'static,
) -> Div {
    let label = text_fn(label);
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(6.0)
        .select_none()
        .dyn_style(move |s| {
            let t = theme();
            s.font_family(heading_font())
                .text_size(t.font_size_small)
                .letter_spacing(t.heading_spacing)
                .text_color(color())
        })
        .child(div().flex_none().dyn_style(move |s| {
            let t = theme();
            if t.heading_bar > 0.0 {
                s.w(t.heading_bar)
                    .h((t.font_size_small * 0.95).round())
                    .bg(accent())
            } else {
                s.hidden()
            }
        }))
        .child(
            div()
                .flex_1()
                .min_w(0.0)
                .overflow_clip()
                .whitespace_nowrap()
                .child(text(move || {
                    let words = label();
                    if theme().heading_caps {
                        words.to_uppercase()
                    } else {
                        words
                    }
                })),
        )
}

/// White text with the soft dark shadow Windows 7 puts under the text of the log-on screen, so
/// that it reads on any picture: a blurred copy under it, a pixel lower. Set the size with
/// `.text_size(..)`.
pub fn shadow_text(label: impl IntoTextSource) -> Div {
    soft_shadowed(label, Color::WHITE, SHADOW, 1.0, SHADOW_BLUR)
        .dyn_style(|s| s.font_family(font()))
}

/// `label` in `color` over a blurred copy of itself in `shadow`, `drop` pixels lower: a shadow,
/// not an outline. The copy is blurred by `blur` (a standard deviation, in logical pixels).
pub fn soft_shadowed(
    label: impl IntoTextSource,
    color: Color,
    shadow: Color,
    drop: f32,
    blur: f32,
) -> Div {
    let label = text_fn(label);
    let under = label.clone();
    div()
        .relative()
        .flex_none()
        .whitespace_nowrap()
        .select_none()
        // The shadow at z 0, the text at z 1: otherwise the positioned copy is drawn over the
        // text, and dark over white reads as grey, silvery.
        .child(
            div()
                .absolute()
                .left(0.0)
                .top(drop)
                .z_index(0)
                .whitespace_nowrap()
                .text_color(shadow)
                .blur(blur)
                .pointer_events_none()
                .child(text(move || under())),
        )
        .child(
            div()
                .relative()
                .z_index(1)
                .text_color(color)
                .child(text(move || label())),
        )
}
