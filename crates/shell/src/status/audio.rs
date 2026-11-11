//! The volume (QML `Audio`, `VolumeIcon`, `VolumePopup`): the default output of PipeWire, the
//! speaker of the notification area and the volume panel it opens, as Windows 7 shows it: the
//! device at the top, a vertical slider, the mute button, and the link to the mixer.

use super::{
    Panel, bar_cell, bar_picture, footer, icon_width, is_open, lit_row_style, one_line, panel_link,
    panel_popup, small_text, toggle, wheel_steps,
};
use crate::paths::asset;
use crate::theme::{Align, away_from_bar, button_radius, corner, scheme, theme};
use crate::ui::{
    MenuLine, bar_menu_options, dark_look, font, ink_asset, popup_frame, popup_menu, shell_tooltip,
    vertical_gradient,
};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use std::cell::Cell;

/// The program that lists every stream, as the volume mixer of Windows.
const MIXER: &str = "pwvucontrol";
/// What the speaker is called when its device has no name of its own.
const SPEAKERS: &str = "Speakers";
/// A notch of the wheel changes the volume by this many percent, as Windows 7 steps it.
const NOTCH_PERCENT: f64 = 2.0;
/// The height of the slider, and the room above and below its track.
const SLIDER_HEIGHT: f32 = 150.0;
const TRACK_MARGIN: f32 = 5.0;
/// The length the level runs over: the track less its margins and its border.
const TRACK_SPAN: f32 = SLIDER_HEIGHT - TRACK_MARGIN * 2.0 - 2.0;
/// The level's green, as Windows 7 fills its volume slider.
const LEVEL_GREEN: Color = rgb(0x2fb13f);
/// The sunken track of the slider, on dark and on light colours.
const TRACK_DARK: Color = rgb(0x1a232c);
const TRACK_LIGHT: Color = rgb(0xe8edf2);
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

fn ready() -> bool {
    sink().is_some()
}

/// The volume, 1 for 100 %.
fn volume() -> f64 {
    sink().map_or(0.0, |s| s.volume)
}

fn muted() -> bool {
    sink().is_some_and(|s| s.muted)
}

/// The volume as a whole percentage.
fn percent() -> i64 {
    (volume() * 100.0).round() as i64
}

fn device_name() -> String {
    sink()
        .map(|s| s.description)
        .filter(|d| !d.is_empty())
        .unwrap_or_else(|| SPEAKERS.to_owned())
}

/// 1 to 3 waves for the level, or "muted".
fn level() -> &'static str {
    let p = percent();
    if !ready() || muted() || p == 0 {
        "muted"
    } else if p < 34 {
        "1"
    } else if p < 67 {
        "2"
    } else {
        "3"
    }
}

/// The outputs to choose from: PipeWire's sinks (a program's stream is not one).
fn outputs() -> Vec<AudioDevice> {
    audio().sinks()
}

fn set_volume(value: f64) {
    let Some(s) = untrack(sink) else { return };
    let value = value.clamp(0.0, 1.0);
    let _ = s.set_volume(value);
    if s.muted && value > 0.0 {
        let _ = s.set_muted(false);
    }
}

/// Changes the volume by a number of percent.
fn step(percent_change: f64) {
    let now = untrack(percent) as f64;
    set_volume((now + percent_change).round() / 100.0);
}

fn toggle_mute() {
    if let Some(s) = untrack(sink) {
        let _ = s.set_muted(!s.muted);
    }
}

fn open_mixer() {
    crate::programs::run(MIXER);
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
        .on_click(move |_| {
            set_devices_open(false);
            toggle(&opener, Panel::Volume);
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
                .child(slider())
                .child(small_text(|| percent().to_string(), || scheme().text_muted))
                .child(mute_button()),
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

/// The slider: a sunken track, the level filled in green, and a thumb. A press or a drag on it
/// sets the volume there; the wheel steps it.
fn slider() -> Div {
    let ui = Interaction::new();
    let rest = Cell::new(0.0f32);
    let fraction = memo(|| volume().clamp(0.0, 1.0) as f32);
    // `local` and `rect` are in the same pixels: the ratio gives the logical place.
    let set_from = |local_y: f32, height: i32| {
        if height <= 0 {
            return;
        }
        let y = local_y * SLIDER_HEIGHT / height as f32;
        set_volume(1.0 - ((y - TRACK_MARGIN - 1.0) / TRACK_SPAN) as f64);
    };
    let track = div()
        .absolute()
        .top(TRACK_MARGIN)
        .bottom(TRACK_MARGIN)
        .left(12.5)
        .w(5.0)
        .border(1.0)
        .dyn_style(|s| {
            let c = scheme();
            s.rounded(corner(2.0))
                .bg(if dark_look() { TRACK_DARK } else { TRACK_LIGHT })
                .border_color(c.field_border)
        })
        .child(
            div()
                .absolute()
                .left(0.0)
                .right(0.0)
                .bottom(0.0)
                .dyn_style(move |s| {
                    s.h((TRACK_SPAN * fraction.get()).max(0.0))
                        .rounded(corner(1.0))
                        .bg(if muted() {
                            scheme().placeholder
                        } else {
                            LEVEL_GREEN
                        })
                }),
        );
    let thumb = div()
        .absolute()
        .left(4.0)
        .w(22.0)
        .h(10.0)
        .border(1.0)
        .pointer_events_none()
        .fill(move || {
            let c = scheme();
            let (top, bottom) = if ui.hovered() {
                (c.button_hot_top, c.button_hot_bottom)
            } else {
                (c.button_top, c.button_bottom)
            };
            vertical_gradient(&[(0.0, top), (1.0, bottom)])
        })
        .dyn_style(move |s| {
            let c = scheme();
            s.top(1.0 + TRACK_SPAN * (1.0 - fraction.get()))
                .rounded(corner(2.0))
                .border_color(if ui.hovered() || ui.pressed() {
                    c.field_focus
                } else {
                    c.field_border
                })
        });
    div()
        .relative()
        .flex_none()
        .w(30.0)
        .h(SLIDER_HEIGHT)
        .interaction(ui)
        .grab_threshold(0.0)
        .on_grab(move |e| set_from(e.local.y, e.rect.height))
        .on_scroll(move |e| {
            let steps = wheel_steps(e, &rest);
            if steps != 0 {
                e.stop_propagation();
                step(steps as f64 * NOTCH_PERCENT);
            }
        })
        .child(track)
        .child(thumb)
}

/// The mute button: the speaker, with the red mark when muted.
fn mute_button() -> Div {
    let ui = Interaction::new();
    div()
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .w(30.0)
        .h(26.0)
        .border(1.0)
        .interaction(ui)
        .on_click(|_| toggle_mute())
        .dyn_style(move |s| {
            let c = scheme();
            let s = s.rounded(button_radius());
            if ui.hovered() {
                s.bg(c.hover).border_color(c.selection_border)
            } else {
                s.bg(Color::TRANSPARENT).border_color(Color::TRANSPARENT)
            }
        })
        .child(
            img(|| ink_asset(&format!("speaker-{}.svg", level())))
                .size(18.0)
                .object_fit(ObjectFit::Contain),
        )
}
