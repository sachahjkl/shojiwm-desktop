//! The Playback and Recording pages: a card for each program that plays (or records), with its
//! icon, its name, what it plays, its volume, its mute button and the device it goes to. The
//! sounds of events (notifications, clicks) share the System Sounds card, first. A program with
//! more than one stream changes all of them from its card, and lists each under it with its own
//! volume and device.

use super::picture;
use super::{
    card_box, default_device, device_label, devices, line_text, percent_text, row, small_text,
};
use crate::control_panel::note;
use crate::theme::scheme;
use crate::ui::{VolumeControl, cp_combo, mute_button, shell_icon, volume_bar};
use anti_webui::prelude::*;
use anti_webui::shell::AudioStream;
use anti_webui::shell::prelude::*;
use std::rc::Rc;

/// The name of the card of the sounds of events.
const SYSTEM_SOUNDS: &str = "System Sounds";
/// The key of that card, which no program's name can be.
const SYSTEM_KEY: &str = "\u{0}system";
/// The size of a program's icon.
const ICON_SIZE: f32 = 32.0;
/// The width of a card's slider, and of a stream's.
const BAR_WIDTH: f32 = 260.0;
const STREAM_BAR_WIDTH: f32 = 140.0;
/// The width of the device lists.
const PICKER_WIDTH: f32 = 210.0;
const STREAM_PICKER_WIDTH: f32 = 170.0;

pub fn page(input: bool) -> Div {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(10.0)
        .child(note(if input {
            "The programs recording sound, and the device each records from."
        } else {
            "The programs playing sound, and the device each plays to."
        }))
        .child(for_each(
            move || group_keys(input),
            String::clone,
            move |key| program_card(input, key),
        ))
        .child(show(
            move || group_keys(input).is_empty(),
            move || {
                note(if input {
                    "No program is recording sound."
                } else {
                    "No program is playing sound."
                })
            },
        ))
}

// ----- what the cards control ------------------------------------------------------------------

/// A name as PipeWire gives it, without the dot and the `-wrapped` that Nix's wrappers add
/// (`.firefox-wrapped`).
fn clean(name: &str) -> String {
    let name = name.trim();
    let name = name.strip_prefix('.').unwrap_or(name);
    name.strip_suffix("-wrapped").unwrap_or(name).to_owned()
}

/// A sound of an event (a notification, a click), which the System Sounds card holds.
fn is_system_sound(stream: &AudioStream) -> bool {
    matches!(
        stream.role.to_ascii_lowercase().as_str(),
        "event" | "notification"
    )
}

/// The program a stream belongs to: its name, else its executable's, else what it plays.
fn program_name(stream: &AudioStream) -> String {
    [&stream.app_name, &stream.binary, &stream.media_name]
        .into_iter()
        .map(|n| clean(n))
        .find(|n| !n.is_empty())
        .unwrap_or_default()
}

/// The card a stream goes in.
fn group_key(stream: &AudioStream) -> String {
    if is_system_sound(stream) {
        SYSTEM_KEY.to_owned()
    } else {
        program_name(stream)
    }
}

/// The streams that play, or that record when `input`, in the card `key`.
fn streams_in(input: bool, key: &str) -> Vec<AudioStream> {
    audio()
        .streams()
        .into_iter()
        .filter(|s| s.input == input && group_key(s) == key)
        .collect()
}

/// The cards, in order: System Sounds first, then the programs in the order they started.
fn group_keys(input: bool) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    for stream in audio().streams().iter().filter(|s| s.input == input) {
        let key = group_key(stream);
        if !keys.contains(&key) {
            keys.push(key);
        }
    }
    keys.sort_by_key(|k| k != SYSTEM_KEY);
    keys
}

/// The volume of a card: its loudest stream's; muted when all are. A change goes to all.
fn group_control(input: bool, key: Rc<str>) -> VolumeControl {
    let (read_key, volume_key) = (key.clone(), key.clone());
    VolumeControl::new(
        move || {
            let streams = streams_in(input, &read_key);
            let loudest = streams.iter().map(|s| s.volume).reduce(f64::max)?;
            Some((loudest, streams.iter().all(|s| s.muted)))
        },
        move |volume| {
            for s in untrack(|| streams_in(input, &volume_key)) {
                let _ = s.set_volume(volume);
            }
        },
        move |muted| {
            for s in untrack(|| streams_in(input, &key)) {
                let _ = s.set_muted(muted);
            }
        },
    )
}

/// The stream `id`, while it is there.
fn stream(id: u32) -> Option<AudioStream> {
    audio().streams().into_iter().find(|s| s.id == id)
}

/// The volume of one stream.
fn stream_control(id: u32) -> VolumeControl {
    VolumeControl::new(
        move || stream(id).map(|s| (s.volume, s.muted)),
        move |volume| {
            if let Some(s) = untrack(|| stream(id)) {
                let _ = s.set_volume(volume);
            }
        },
        move |muted| {
            if let Some(s) = untrack(|| stream(id)) {
                let _ = s.set_muted(muted);
            }
        },
    )
}

/// What a card shows of its program: the name to show and the icon, as the program's
/// `.desktop` file gives them when one is found.
#[derive(Clone, PartialEq)]
struct Program {
    name: String,
    icon: String,
}

/// The program of a stream: the icon it names, else its `.desktop` file's (found by its
/// executable or its name), else the generic one.
fn program_of(icon_name: &str, binary: &str, app_name: &str, fallback_name: String) -> Program {
    let entry = [clean(binary), clean(app_name)]
        .iter()
        .filter(|n| !n.is_empty())
        .find_map(|n| crate::apps::entry_for(n));
    let found = |name: &str| !name.is_empty() && anti_webui::shell::icon_path(name, 32).is_some();
    // An icon the theme does not have draws nothing: the generic one stands in.
    let icon = [
        icon_name.to_owned(),
        crate::apps::entry_icon(entry.as_ref()),
    ]
    .into_iter()
    .find(|name| found(name))
    .unwrap_or_else(|| crate::apps::FALLBACK_ICON.to_owned());
    let name = entry
        .map(|e| e.name)
        .filter(|n| !n.is_empty())
        .unwrap_or(fallback_name);
    Program { name, icon }
}

/// A list of the devices to send streams to (or take them from when `input`): `current` gives
/// the device in use, the default one when it gives none; a choice calls `choose`. The list is
/// built again when a device comes or goes.
fn device_picker(
    input: bool,
    width: f32,
    current: impl Fn() -> Option<u32> + 'static,
    choose: impl Fn(&AudioDevice) + 'static,
) -> View {
    let choices = memo(move || {
        devices(input)
            .into_iter()
            .map(|d| (d.id, device_label(&d)))
            .collect::<Vec<_>>()
    });
    let current = Rc::new(current);
    let choose = Rc::new(choose);
    dyn_view(move || {
        let list = choices.get();
        if list.is_empty() {
            return div().into_view();
        }
        let labels = list.iter().map(|(_, label)| label.clone()).collect();
        let ids: Rc<[u32]> = list.iter().map(|(id, _)| *id).collect();
        let (read_ids, current) = (ids.clone(), current.clone());
        let choose = choose.clone();
        let binding = Binding::new(
            move || {
                current()
                    .or_else(|| default_device(input).map(|d| d.id))
                    .and_then(|id| read_ids.iter().position(|i| *i == id))
                    .unwrap_or(0)
            },
            move |index: usize| {
                let Some(&id) = ids.get(index) else {
                    return;
                };
                if let Some(device) = untrack(|| devices(input)).iter().find(|d| d.id == id) {
                    choose(device);
                }
            },
        );
        cp_combo(labels, binding).w(width).into_view()
    })
}

// ----- the cards -------------------------------------------------------------------------------

/// A program's card: its icon, its name and what it plays, the device it goes to; its mute
/// button, its slider and its volume; and its streams when it has more than one.
fn program_card(input: bool, key: String) -> Div {
    let key: Rc<str> = key.into();
    let control = group_control(input, key.clone());
    let system = &*key == SYSTEM_KEY;
    // What the program says of itself, which changes less often than its volume.
    let said = {
        let key = key.clone();
        memo(move || {
            streams_in(input, &key).first().map(|s| {
                (
                    s.icon_name.clone(),
                    s.binary.clone(),
                    s.app_name.clone(),
                    program_name(s),
                )
            })
        })
    };
    let program = memo(move || {
        if system {
            return Program {
                name: SYSTEM_SOUNDS.to_owned(),
                icon: String::new(),
            };
        }
        said.get()
            .map(|(icon, binary, app, name)| program_of(&icon, &binary, &app, name))
            .unwrap_or(Program {
                name: String::new(),
                icon: String::new(),
            })
    });
    let ids = {
        let key = key.clone();
        memo(move || {
            streams_in(input, &key)
                .iter()
                .map(|s| s.id)
                .collect::<Vec<_>>()
        })
    };
    // What it plays: the title of its one stream, else how many there are.
    let detail = {
        let key = key.clone();
        memo(move || {
            let streams = streams_in(input, &key);
            match streams.as_slice() {
                [one] => one.media_name.clone(),
                many => format!("{} streams", many.len()),
            }
        })
    };
    let icon = if system {
        picture("sound", ICON_SIZE).into_view()
    } else {
        shell_icon(move || program.get().icon, ICON_SIZE).into_view()
    };
    let (target_key, move_key) = (key.clone(), key.clone());
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
                    line_text(move || program.get().name)
                        .dyn_style(|s| s.font_weight(FontWeight::BOLD)),
                )
                .child(small_text(move || detail.get())),
        )
        .child(device_picker(
            input,
            PICKER_WIDTH,
            move || {
                streams_in(input, &target_key)
                    .first()
                    .and_then(|s| s.target)
            },
            move |device| {
                for s in untrack(|| streams_in(input, &move_key)) {
                    let _ = s.move_to(device);
                }
            },
        ));
    let controls = row(8.0)
        .pl(ICON_SIZE + 10.0)
        .child(mute_button(control.clone()))
        .child(volume_bar(control.clone(), BAR_WIDTH))
        .child(percent_text(control));
    card_box().child(header).child(controls).child(show(
        move || ids.get().len() > 1,
        move || {
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(2.0)
                .pt(4.0)
                .pl(ICON_SIZE + 10.0)
                .border_t(1.0)
                .dyn_style(|s| s.border_color(scheme().menu_divider))
                .child(for_each(
                    move || ids.get(),
                    |id: &u32| *id,
                    move |id| stream_row(input, id),
                ))
        },
    ))
}

/// A stream of a program with several: what it plays, its mute button, its slider, its volume
/// and its device.
fn stream_row(input: bool, id: u32) -> Div {
    let control = stream_control(id);
    let title = memo(move || {
        stream(id)
            .map(|s| {
                if s.media_name.is_empty() {
                    format!("Stream {id}")
                } else {
                    s.media_name
                }
            })
            .unwrap_or_default()
    });
    row(6.0)
        .w_full()
        .child(
            div()
                .flex_1()
                .min_w(0.0)
                .overflow_clip()
                .child(small_text(move || title.get())),
        )
        .child(mute_button(control.clone()))
        .child(volume_bar(control.clone(), STREAM_BAR_WIDTH))
        .child(percent_text(control))
        .child(device_picker(
            input,
            STREAM_PICKER_WIDTH,
            move || stream(id).and_then(|s| s.target),
            move |device| {
                if let Some(s) = untrack(|| stream(id)) {
                    let _ = s.move_to(device);
                }
            },
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nix_wrappers_are_taken_off_names() {
        assert_eq!(clean(".firefox-wrapped"), "firefox");
        assert_eq!(clean("Chromium"), "Chromium");
        assert_eq!(clean(" mpv "), "mpv");
    }
}
