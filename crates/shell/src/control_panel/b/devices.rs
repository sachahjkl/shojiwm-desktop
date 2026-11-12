//! Devices and Printers: the computer, the screens and the USB devices (hubs left out), then the
//! printers with the default one ticked, from `lsusb` and `lpstat`.

use super::common::{icon_or, output_of, page_column};
use crate::control_panel::{heading, note, sub_heading};
use crate::theme::{scheme, theme};
use crate::ui::{font, shell_icon};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;

/// The tick on the default printer, as Windows marks it: a green disc with a white tick.
const DEFAULT_TICK: Color = rgb(0x3fbf3a);
const DEFAULT_TICK_INK: Color = Color::WHITE;
/// A tile: its width, and the room of its icon.
const TILE_WIDTH: f32 = 120.0;
const ICON_SIZE: f32 = 48.0;
const ICON_ROOM: f32 = 52.0;

/// The USB devices, the printers and the default one, as one script prints them.
const LIST_SCRIPT: &str = "echo '#usb'; lsusb 2>/dev/null | cut -d' ' -f7- | grep -v -i 'hub'; \
     echo '#printers'; lpstat -p 2>/dev/null | awk '/^printer/ {print $2 \"\\t\" $4}'; \
     echo '#default'; lpstat -d 2>/dev/null | sed -n 's/.*: //p'";

/// The words of a device's name, and the icon of the theme that fits them.
const KINDS: &[(&str, &str)] = &[
    ("keyboard", "input-keyboard"),
    ("desktop", "input-keyboard"),
    ("mouse", "input-mouse"),
    ("dongle", "input-mouse"),
    ("webcam", "camera-web"),
    ("camera", "camera-web"),
    ("mv5", "audio-input-microphone"),
    ("microphone", "audio-input-microphone"),
    ("audio", "audio-card"),
    ("headset", "audio-headset"),
    ("monitor", "video-display"),
    ("display", "video-display"),
    ("yubikey", "dialog-password"),
    ("security", "dialog-password"),
    ("storage", "drive-removable-media"),
    ("flash", "drive-removable-media"),
    ("bluetooth", "bluetooth"),
    ("led", "preferences-desktop-color"),
];

#[derive(Clone, Debug, Default, PartialEq)]
struct Printer {
    name: String,
    state: String,
}

/// What the page shows, read by a worker: the icons are looked up in the theme there too.
#[derive(Clone, Debug, Default, PartialEq)]
struct Devices {
    /// The name of each USB device, and its icon.
    usb: Vec<(String, String)>,
    /// The icons of a screen and of a printer.
    display_icon: String,
    printer_icon: String,
    printers: Vec<Printer>,
    default_printer: String,
}

fn read_devices() -> Devices {
    let text = output_of("sh", &["-c", LIST_SCRIPT]);
    let mut devices = Devices::default();
    let mut section = "";
    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        if let Some(name) = line.strip_prefix('#') {
            section = match name {
                "usb" => "usb",
                "printers" => "printers",
                "default" => "default",
                _ => "",
            };
            continue;
        }
        match section {
            "usb" => {
                let name = line.trim().to_owned();
                let icon = icon_for(&name);
                devices.usb.push((name, icon));
            }
            "printers" => {
                let (name, state) = line.split_once('\t').unwrap_or((line, ""));
                devices.printers.push(Printer {
                    name: name.to_owned(),
                    state: state.to_owned(),
                });
            }
            "default" => devices.default_printer = line.trim().to_owned(),
            _ => {}
        }
    }
    devices.display_icon = icon_or("video-display", "computer");
    devices.printer_icon = icon_or("printer", "document-print");
    devices
}

/// An icon of the theme that fits a device's name. It looks in the theme: call it on a worker
/// thread only.
fn icon_for(name: &str) -> String {
    let lower = name.to_lowercase();
    let icon = KINDS
        .iter()
        .find(|(word, _)| lower.contains(word))
        .map_or("drive-harddisk-usb", |(_, icon)| icon);
    icon_or(icon, "drive-harddisk")
}

pub fn page() -> Div {
    let devices = signal(Devices::default());
    task(read_devices).then(move |found| devices.set(found));
    page_column(10.0)
        .child(heading("Devices and Printers"))
        .child(dyn_view(move || {
            let count = devices.with(|d| d.usb.len()) + outputs().len();
            sub_heading(format!("Devices ({count})"))
        }))
        .child(dyn_view(move || {
            let computer = crate::paths::asset("icons/computer.png")
                .display()
                .to_string();
            let mut tiles = vec![tile("Computer".into(), String::new(), computer, false)];
            for output in outputs() {
                // Some screens do not say their model.
                let label = if output.model.is_empty() || output.model == "Unknown" {
                    "Display".to_owned()
                } else {
                    output.model.clone()
                };
                let detail = format!(
                    "{} · {} × {}",
                    output.name, output.logical_size.width, output.logical_size.height
                );
                tiles.push(tile(
                    label,
                    detail,
                    devices.with(|d| d.display_icon.clone()),
                    false,
                ));
            }
            for (name, icon) in devices.with(|d| d.usb.clone()) {
                tiles.push(tile(name, "USB".into(), icon, false));
            }
            flow(tiles)
        }))
        .child(dyn_view(move || {
            let count = devices.with(|d| d.printers.len());
            sub_heading(format!("Printers ({count})"))
        }))
        .child(dyn_view(move || {
            let (printers, default, icon) = devices.with(|d| {
                (
                    d.printers.clone(),
                    d.default_printer.clone(),
                    d.printer_icon.clone(),
                )
            });
            flow(
                printers
                    .into_iter()
                    .map(|p| {
                        let marked = p.name == default;
                        tile(p.name.replace('_', " "), p.state, icon.clone(), marked)
                    })
                    .collect(),
            )
        }))
        .child(show(
            move || devices.with(|d| d.printers.is_empty()),
            || note("No printer is set up (CUPS)."),
        ))
}

/// Tiles in rows that wrap.
fn flow(tiles: Vec<Div>) -> Div {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .gap(12.0)
        .w_full()
        .children(tiles)
}

/// A device: its icon, its name under it, and a detail under that. `marked` ticks the default
/// one.
fn tile(label: String, detail: String, icon: String, marked: bool) -> Div {
    let tick = div()
        .absolute()
        .left(TILE_WIDTH / 2.0 + 10.0)
        .top(30.0)
        .w(18.0)
        .h(18.0)
        .rounded(9.0)
        .border(1.0)
        .border_color(DEFAULT_TICK_INK)
        .bg(DEFAULT_TICK)
        .flex()
        .items_center()
        .justify_center()
        .child(
            text("\u{2714}")
                .text_size(11.0)
                .text_color(DEFAULT_TICK_INK),
        );
    let mut icon_room = div()
        .relative()
        .w_full()
        .h(ICON_ROOM)
        .flex()
        .items_center()
        .justify_center()
        .child(shell_icon(icon, ICON_SIZE));
    if marked {
        icon_room = icon_room.child(tick);
    }
    let shown_detail = !detail.is_empty();
    let mut column = div()
        .flex()
        .flex_col()
        .items_center()
        .flex_none()
        .w(TILE_WIDTH)
        .gap(4.0)
        .child(icon_room)
        .child(
            text(label)
                .w_full()
                .text_align(TextAlign::Center)
                .dyn_style(|s| {
                    s.font_family(font())
                        .text_size(theme().font_size_small)
                        .text_color(scheme().text)
                }),
        );
    if shown_detail {
        column = column.child(
            text(detail)
                .w_full()
                .text_align(TextAlign::Center)
                .whitespace_nowrap()
                .dyn_style(|s| {
                    s.font_family(font())
                        .text_size(theme().font_size_small - 1.0)
                        .text_color(scheme().text_muted)
                }),
        );
    }
    column
}
