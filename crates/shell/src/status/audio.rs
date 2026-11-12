//! The volume (QML `Audio`, `VolumeIcon`, `VolumePopup`): the default output of PipeWire, the
//! speaker of the notification area and the volume panel it opens, as Windows 7 shows it: the
//! device at the top, a vertical slider, the mute button, and the link to the Sound window.

use super::{
    Panel, bar_cell, bar_picture, click_or_settings, footer, icon_width, is_open, lit_row_style,
    one_line, panel_link, panel_popup, small_text,
};
use crate::paths::asset;
use crate::theme::{Align, away_from_bar, scheme, theme};
use crate::ui::{
    MenuLine, VolumeControl, bar_menu_options, font, ink_asset, mute_button, popup_frame,
    popup_menu, shell_tooltip, volume_slider, wheel_steps,
};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use std::cell::Cell;

/// What the speaker is called when its device has no name of its own.
const SPEAKERS: &str = "Speakers";
/// A notch of the wheel changes the volume by this many percent, as Windows 7 steps it.
const NOTCH_PERCENT: f64 = 2.0;
/// The height of the slider.
const SLIDER_HEIGHT: f32 = 150.0;
/// The width of the panel, alone and with a player's controls or the outputs.
const NARROW: f32 = 104.0;
const WIDE: f32 = 220.0;

thread_local! {
    static DEVICES_OPEN: Cell<Option<Signal<bool>>> = const { Cell::new(None) };
}

/// The outputs are listed under the device's name in the panel.
fn devices_open_signal() -> Signal<bool> {
    DEVICES_OPEN.with(|cell| {
        cell.get().unwrap_or_else(|| {
            let (s, _) = create_root(|| signal(false));
            cell.set(Some(s));
            s
        })
    })
}

pub fn set_devices_open(on: bool) {
    devices_open_signal().set(on);
}

/// The default output, when PipeWire has one.
fn sink() -> Option<AudioDevice> {
    audio().default_sink()
}

/// The default output's volume, as the slider and the mute button change it.
fn control() -> VolumeControl {
    crate::sound::default_device_control(false)
}

fn ready() -> bool {
    control().ready()
}

fn muted() -> bool {
    control().muted()
}

/// The volume as a whole percentage.
fn percent() -> i64 {
    control().percent()
}

fn device_name() -> String {
    sink()
        .map(|s| s.description)
        .filter(|d| !d.is_empty())
        .unwrap_or_else(|| SPEAKERS.to_owned())
}

/// 1 to 3 waves for the level, or "muted".
fn level() -> &'static str {
    control().level()
}

/// The outputs to choose from: PipeWire's sinks (a program's stream is not one).
fn outputs() -> Vec<AudioDevice> {
    audio().sinks()
}

/// Changes the volume by a number of percent.
fn step(percent_change: f64) {
    control().step(percent_change);
}

fn toggle_mute() {
    control().toggle_mute();
}

fn open_mixer() {
    crate::sound::open_page(crate::sound::Page::Playback);
}

/// The speaker of the notification area: its waves show the level, a red mark a muted output.
/// A click opens the volume panel, the wheel changes the volume, a middle click mutes, and a
/// right click opens its menu.
pub fn icon(output_name: &str) -> Div {
    let name = output_name.to_owned();
    let anchor = node_ref();
    let menu = signal(false);
    let rest = Cell::new(0.0f32);
    let active = {
        let name = name.clone();
        move || is_open(&name, Panel::Volume)
    };
    let opener = name.clone();
    let lines_name = name.clone();
    let wheel_name = name.clone();
    let cell = bar_cell(icon_width, 2.0, active)
        .node_ref(anchor)
        .child(bar_picture(
            || asset(&format!("speaker-{}.svg", level())),
            2.0,
        ))
        .on_click(move |e| {
            set_devices_open(false);
            click_or_settings(e, &opener, Panel::Volume, || {
                crate::sound::open_page(crate::sound::Page::Playback)
            });
        })
        .on_aux_click(move |e| match e.button {
            Some(PointerButton::Middle) => toggle_mute(),
            Some(PointerButton::Secondary) => menu.set(!menu.get_untracked()),
            _ => {}
        })
        .on_scroll(move |e| {
            // The panel is a child of the icon, so its wheel comes here too: the panel's
            // slider has its own, and its list must scroll.
            if untrack(|| is_open(&wheel_name, Panel::Volume)) {
                return;
            }
            let steps = wheel_steps(e, &rest);
            if steps != 0 {
                e.stop_propagation();
                step(steps as f64 * NOTCH_PERCENT);
            }
        })
        .child(popup_menu(
            move || menu.get(),
            move || menu.set(false),
            bar_menu_options(anchor),
            move || {
                let name = lines_name.clone();
                vec![
                    MenuLine::item("Open Volume Mixer", open_mixer),
                    MenuLine::item("Playback devices", move || {
                        super::open(&name, Panel::Volume);
                        set_devices_open(true);
                    }),
                    MenuLine::item("Mute", toggle_mute)
                        .checked(muted())
                        .enabled(ready()),
                    MenuLine::separator(),
                    MenuLine::item("Sound settings", || {
                        crate::control_panel::open_page("sound");
                    }),
                ]
            },
        ))
        .child(panel_popup(&name, Panel::Volume, Align::Center, panel));
    shell_tooltip(cell, untrack(|| away_from_bar(Align::Center)), || {
        let state = if muted() {
            "Muted".to_owned()
        } else {
            format!("{}%", percent())
        };
        format!("{}: {state}", device_name())
    })
}

/// The volume panel.
fn panel() -> Div {
    // The list of outputs is folded again when the panel closes.
    on_cleanup(|| set_devices_open(false));
    let has_player = memo(|| media().current().is_some());
    let devices_open = devices_open_signal();
    popup_frame()
        .dyn_style(move |s| {
            s.w(if has_player.get() || devices_open.get() {
                WIDE
            } else {
                NARROW
            })
        })
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(8.0)
                .pt(10.0)
                .pb(10.0)
                .child(
                    div()
                        .self_stretch()
                        .px(10.0)
                        .dyn_style(move |s| if has_player.get() { s } else { s.hidden() })
                        .child(crate::taskbar::media_controls()),
                )
                .child(
                    img(|| ink_asset("speaker-3.svg"))
                        .size(32.0)
                        .object_fit(ObjectFit::Contain),
                )
                .child(device_line())
                .child(show(move || devices_open.get(), device_list))
                .child(volume_slider(control(), SLIDER_HEIGHT))
                .child(small_text(|| percent().to_string(), || scheme().text_muted))
                .child(mute_button(control())),
        )
        .child(footer(
            28.0,
            true,
            panel_link("Mixer", false, || {
                open_mixer();
                super::close(Panel::Volume);
            }),
        ))
}

/// The device's name; with more than one output, a click lists them to choose another.
fn device_line() -> Div {
    let ui = Interaction::new();
    let several = memo(|| outputs().len() > 1);
    let devices_open = devices_open_signal();
    div()
        .self_stretch()
        .mx(6.0)
        .flex()
        .justify_center()
        .overflow_clip()
        .whitespace_nowrap()
        .interaction(ui)
        .on_click(move |_| {
            if untrack(|| several.get()) {
                devices_open.set(!devices_open.get_untracked());
            }
        })
        .dyn_style(move |s| {
            let lit = ui.hovered() && several.get();
            let s = s
                .font_family(font())
                .text_size(theme().font_size_small)
                .text_color(if ui.hovered() {
                    scheme().link
                } else {
                    scheme().text
                });
            let s = if several.get() {
                s.cursor(CursorIcon::Pointer)
            } else {
                s
            };
            if lit { s.underline() } else { s }
        })
        .child(text(move || {
            let arrow = match (several.get(), devices_open.get()) {
                (false, _) => "",
                (true, true) => "  \u{25B4}",
                (true, false) => "  \u{25BE}",
            };
            format!("{}{arrow}", device_name())
        }))
}

/// The outputs, the one in use chosen; a click makes another the default.
fn device_list() -> Div {
    div()
        .self_stretch()
        .mx(6.0)
        .flex()
        .flex_col()
        .gap(1.0)
        .child(for_each(
            outputs,
            |d: &AudioDevice| d.id,
            |device: AudioDevice| {
                let ui = Interaction::new();
                let id = device.id;
                let current = memo(move || sink().is_some_and(|s| s.id == id));
                let label = device.description.clone();
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .h(24.0)
                    .px(6.0)
                    .interaction(ui)
                    .on_click(move |_| {
                        let _ = audio().set_default(&device);
                        set_devices_open(false);
                    })
                    .dyn_style(move |s| lit_row_style(s, current.get(), ui))
                    .child(one_line(small_text(label, || scheme().text).dyn_style(
                        move |s| {
                            if current.get() {
                                s.font_weight(FontWeight::BOLD)
                            } else {
                                s
                            }
                        },
                    )))
            },
        ))
}
