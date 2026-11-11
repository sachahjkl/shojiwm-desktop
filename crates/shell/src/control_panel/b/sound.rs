//! Sound: the playback and recording devices from PipeWire, the default of each marked, a
//! button to make another one the default, and links to the pages of the Sound window.

use super::common::{muted_small, page_column};
use crate::control_panel::{heading, note, sub_heading};
use crate::sound::Page;
use crate::theme::{item_radius, scheme, theme};
use crate::ui::{cp_button, cp_link, font, ink_asset};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;

/// "Default Device" under the default one, in Windows' green.
const DEFAULT_GREEN: Color = rgb(0x3fbf3a);
const ROW_HEIGHT: f32 = 46.0;

pub fn page() -> Div {
    let sound = audio();
    page_column(10.0)
        .child(heading("Sound"))
        .child(sub_heading("Playback"))
        .child(for_each(
            move || sound.sinks(),
            |d| (d.id, d.name.clone()),
            move |d| device_row(d, true),
        ))
        .child(sub_heading("Recording"))
        .child(for_each(
            move || sound.sources(),
            |d| (d.id, d.name.clone()),
            move |d| device_row(d, false),
        ))
        .child(show(
            move || sound.sinks().is_empty() && sound.sources().is_empty(),
            || note("PipeWire lists no sound device."),
        ))
        .child(sub_heading("More settings"))
        .child(sound_link("Volume of each program", Page::Playback))
        .child(sound_link("Programs recording sound", Page::Recording))
        .child(sound_link(
            "Output devices: ports, balance and volume",
            Page::Outputs,
        ))
        .child(sound_link(
            "Input devices: ports, balance and volume",
            Page::Inputs,
        ))
        .child(sound_link(
            "Sound cards: profiles (HDMI, analog, off)",
            Page::Configuration,
        ))
}

/// A link that opens the Sound window on `page`.
fn sound_link(label: &'static str, page: Page) -> Div {
    cp_link(label, false, move || crate::sound::open_page(page))
}

/// A device: its icon, its name, "Default Device" or its description, and "Set Default".
/// The default one is drawn selected.
fn device_row(device: AudioDevice, playback: bool) -> Div {
    let sound = audio();
    let id = device.id;
    let is_default = memo(move || {
        let default = if playback {
            sound.default_sink()
        } else {
            sound.default_source()
        };
        default.is_some_and(|d| d.id == id)
    });
    let icon = if playback {
        img(|| ink_asset("speaker-3.svg"))
            .w(28.0)
            .h(28.0)
            .object_fit(ObjectFit::Contain)
            .into_view()
    } else {
        text("\u{1f399}").text_size(22.0).into_view()
    };
    let name = if device.description.is_empty() {
        device.name.clone()
    } else {
        device.description.clone()
    };
    let description = device.description.clone();
    let words = div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w(0.0)
        .overflow_clip()
        .child(text(name).whitespace_nowrap().dyn_style(|s| {
            s.font_family(font())
                .text_size(theme().font_size)
                .text_color(scheme().text)
        }))
        .child(
            muted_small(move || {
                if is_default.get() {
                    "Default Device".to_owned()
                } else {
                    description.clone()
                }
            })
            .whitespace_nowrap()
            .dyn_style(move |s| {
                if is_default.get() {
                    s.text_color(DEFAULT_GREEN)
                } else {
                    s
                }
            }),
        );
    let chosen = device.clone();
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(12.0)
        .w_full()
        .h(ROW_HEIGHT)
        .pl(10.0)
        .pr(10.0)
        .border(1.0)
        .dyn_style(move |s| {
            let c = scheme();
            let s = s.rounded(item_radius());
            if is_default.get() {
                s.bg(c.selection).border_color(c.selection_border)
            } else {
                s.bg(Color::TRANSPARENT).border_color(Color::TRANSPARENT)
            }
        })
        .child(
            div()
                .flex_none()
                .w(28.0)
                .flex()
                .justify_center()
                .child(icon),
        )
        .child(words)
        .child(show(
            move || !is_default.get(),
            move || {
                let chosen = chosen.clone();
                cp_button("Set Default", move || {
                    let _ = audio().set_default(&chosen);
                })
            },
        ))
}
