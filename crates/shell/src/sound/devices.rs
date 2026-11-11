//! The Output Devices and Input Devices pages: a card for each device, with its name, whether it
//! is the default one (a radio button makes it so), its mute button, its slider and its volume,
//! the port it uses when its card has several (headphones, speakers, HDMI), and the balance of
//! its left and right channels. Inputs that only repeat an output (monitors) show when asked.

use super::{DEVICE_ICON, device_picture_of, picture, small_text};
use super::{card_box, default_device, device_label, devices, line_text, percent_text, row};
use crate::control_panel::note;
use crate::ui::{
    VolumeControl, balance_bar, cp_check, cp_combo, cp_radio, mute_button, volume_bar,
};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use std::rc::Rc;

/// The size of a device's picture.
const ICON_SIZE: f32 = DEVICE_ICON;
/// The width of the volume and balance sliders.
const BAR_WIDTH: f32 = 260.0;
/// The width of the label column of a card's rows (`Port`, `Balance`).
const LABEL_WIDTH: f32 = 70.0;
/// The width of the list of ports.
const PORT_WIDTH: f32 = 260.0;

pub fn page(input: bool) -> Div {
    let monitors = signal(false);
    let listed = memo(move || {
        let show_monitors = monitors.get();
        devices(input)
            .into_iter()
            .filter(|d| !input || show_monitors || !is_monitor(d))
            .map(|d| d.id)
            .collect::<Vec<_>>()
    });
    let mut page = div()
        .w_full()
        .flex()
        .flex_col()
        .gap(10.0)
        .child(note(if input {
            "The devices sound comes from. Programs record from the default one unless told \
             otherwise."
        } else {
            "The devices sound goes to. Programs play to the default one unless told otherwise."
        }));
    if input {
        page = page.child(cp_check("Show monitors of outputs", monitors));
    }
    page.child(for_each(
        move || listed.get(),
        |id: &u32| *id,
        move |id| device_card(input, id),
    ))
    .child(show(
        move || listed.get().is_empty(),
        move || {
            note(if input {
                "There is no input."
            } else {
                "There is no output."
            })
        },
    ))
}

/// An input that repeats what an output plays.
fn is_monitor(device: &AudioDevice) -> bool {
    device.name.ends_with(".monitor") || device.description.starts_with("Monitor of ")
}

/// The device has channels on the left and on the right, so a balance.
fn has_sides(device: &AudioDevice) -> bool {
    let p = &device.positions;
    p.iter().any(|p| p.ends_with('L')) && p.iter().any(|p| p.ends_with('R'))
}

/// A label of a card's row, in its column.
fn row_label(label: &'static str) -> Div {
    div().flex_none().w(LABEL_WIDTH).child(small_text(label))
}

fn device_card(input: bool, id: u32) -> Div {
    let device = memo(move || devices(input).into_iter().find(|d| d.id == id));
    let control = VolumeControl::new(
        move || device.get().map(|d| (d.volume, d.muted)),
        move |volume| {
            if let Some(d) = untrack(|| device.get()) {
                let _ = d.set_volume(volume);
            }
        },
        move |muted| {
            if let Some(d) = untrack(|| device.get()) {
                let _ = d.set_muted(muted);
            }
        },
    );
    let kind = memo(move || {
        device
            .get()
            .map_or(if input { "microphone" } else { "speaker" }, |d| {
                device_picture_of(input, &d)
            })
    });
    let icon = dyn_view(move || picture(kind.get(), ICON_SIZE).into_view());
    let default = Binding::new(
        move || default_device(input).map(|d| d.id),
        move |chosen: Option<u32>| {
            if chosen != Some(id) {
                return;
            }
            if let Some(d) = untrack(|| device.get()) {
                let _ = audio().set_default(&d);
            }
        },
    );
    let header = row(10.0)
        .w_full()
        .child(div().flex_none().size(ICON_SIZE).child(icon))
        .child(
            div()
                .flex_1()
                .min_w(0.0)
                .flex()
                .flex_col()
                .overflow_clip()
                .child(
                    line_text(move || device.get().map(|d| device_label(&d)).unwrap_or_default())
                        .dyn_style(|s| s.font_weight(FontWeight::BOLD)),
                )
                .child(small_text(move || {
                    device.get().map(|d| d.name).unwrap_or_default()
                })),
        )
        .child(cp_radio("Default device", default, Some(id)).flex_none());
    let indent = ICON_SIZE + 10.0;
    let controls = row(8.0)
        .pl(indent)
        .child(mute_button(control.clone()))
        .child(volume_bar(control.clone(), BAR_WIDTH))
        .child(percent_text(control));
    let sides = memo(move || device.get().is_some_and(|d| has_sides(&d)));
    let balance = show(
        move || sides.get(),
        move || {
            row(8.0)
                .pl(indent)
                .child(row_label("Balance"))
                .child(small_text("L"))
                .child(balance_bar(
                    move || device.get().map_or(0.0, |d| d.balance()),
                    move |b| {
                        if let Some(d) = untrack(|| device.get()) {
                            let _ = d.set_balance(b);
                        }
                    },
                    BAR_WIDTH - 2.0 * 14.0,
                ))
                .child(small_text("R"))
        },
    );
    card_box()
        .child(header)
        .child(controls)
        .child(port_row(device, indent))
        .child(balance)
}

/// The port the device uses, to choose among its card's, when there is more than one. A port
/// with nothing plugged in says so.
fn port_row(device: Memo<Option<AudioDevice>>, indent: f32) -> View {
    let ports = memo(move || {
        device
            .get()
            .map(|d| {
                d.ports
                    .iter()
                    .map(|p| {
                        let label = if p.available == Some(false) {
                            format!("{} (unplugged)", p.description)
                        } else {
                            p.description.clone()
                        };
                        (p.index, label)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    });
    dyn_view(move || {
        let list = ports.get();
        if list.len() < 2 {
            return div().into_view();
        }
        let labels = list.iter().map(|(_, label)| label.clone()).collect();
        let indexes: Rc<[u32]> = list.iter().map(|(index, _)| *index).collect();
        let read = indexes.clone();
        let binding = Binding::new(
            move || {
                device
                    .get()
                    .and_then(|d| d.port)
                    .and_then(|port| read.iter().position(|i| *i == port))
                    .unwrap_or(0)
            },
            move |chosen: usize| {
                let Some(&index) = indexes.get(chosen) else {
                    return;
                };
                if let Some(d) = untrack(|| device.get())
                    && d.port != Some(index)
                {
                    let _ = d.set_port(index);
                }
            },
        );
        row(8.0)
            .pl(indent)
            .child(row_label("Port"))
            .child(cp_combo(labels, binding).w(PORT_WIDTH))
            .into_view()
    })
}
