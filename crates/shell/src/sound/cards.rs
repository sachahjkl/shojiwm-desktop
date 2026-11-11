//! The Configuration page: a card for each sound card, with its name, the devices it gives now,
//! and its profile: what it is set up to do (analog stereo, HDMI, both ways or one, off). A
//! profile that cannot be used now (nothing plugged in) says so.

use super::{DEVICE_ICON, card_box, device_label, line_text, picture, row, small_text};
use crate::control_panel::note;
use crate::ui::cp_combo;
use anti_webui::prelude::*;
use anti_webui::shell::AudioCard;
use anti_webui::shell::prelude::*;
use std::rc::Rc;

/// The size of a card's picture.
const ICON_SIZE: f32 = DEVICE_ICON;
/// The width of the list of profiles.
const PROFILE_WIDTH: f32 = 360.0;

pub fn page() -> Div {
    let ids = memo(|| audio().cards().iter().map(|c| c.id).collect::<Vec<_>>());
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(10.0)
        .child(note(
            "A card's profile chooses which of its outputs and inputs can be used. Devices come \
             and go with it.",
        ))
        .child(for_each(move || ids.get(), |id: &u32| *id, card_view))
        .child(show(
            move || ids.get().is_empty(),
            || note("PipeWire lists no sound card."),
        ))
}

fn card_view(id: u32) -> Div {
    let card = memo(move || audio().cards().into_iter().find(|c| c.id == id));
    // A webcam's card shows the webcam, any other card the sound card.
    let icon = memo(move || {
        if card.get().is_some_and(|c| c.icon_name.contains("camera")) {
            "webcam"
        } else {
            "sound-card"
        }
    });
    // The devices it gives now, outputs then inputs.
    let gives = memo(move || {
        let sound = audio();
        let names: Vec<String> = sound
            .sinks()
            .iter()
            .chain(sound.sources().iter())
            .filter(|d| d.card == Some(id))
            .map(device_label)
            .collect();
        if names.is_empty() {
            "No device in this profile.".to_owned()
        } else {
            names.join(", ")
        }
    });
    let header = row(10.0)
        .w_full()
        .child(dyn_view(move || picture(icon.get(), ICON_SIZE).into_view()))
        .child(
            div()
                .flex_1()
                .min_w(0.0)
                .flex()
                .flex_col()
                .overflow_clip()
                .child(
                    line_text(move || card.get().map(|c| c.description).unwrap_or_default())
                        .dyn_style(|s| s.font_weight(FontWeight::BOLD)),
                )
                .child(small_text(move || {
                    card.get().map(|c| c.name).unwrap_or_default()
                })),
        );
    card_box()
        .child(header)
        .child(profile_row(card, ICON_SIZE + 10.0))
        .child(
            div()
                .pl(ICON_SIZE + 10.0)
                .overflow_clip()
                .child(small_text(move || gives.get())),
        )
}

/// The profile in use, to choose another.
fn profile_row(card: Memo<Option<AudioCard>>, indent: f32) -> View {
    let profiles = memo(move || {
        card.get()
            .map(|c| {
                c.profiles
                    .iter()
                    .map(|p| {
                        let label = if p.available == Some(false) {
                            format!("{} (unavailable)", p.description)
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
        let list = profiles.get();
        if list.is_empty() {
            return row(8.0)
                .pl(indent)
                .child(small_text("Reading its profiles…"))
                .into_view();
        }
        let labels = list.iter().map(|(_, label)| label.clone()).collect();
        let indexes: Rc<[u32]> = list.iter().map(|(index, _)| *index).collect();
        let read = indexes.clone();
        let binding = Binding::new(
            move || {
                card.get()
                    .and_then(|c| c.profile)
                    .and_then(|profile| read.iter().position(|i| *i == profile))
                    .unwrap_or(0)
            },
            move |chosen: usize| {
                let Some(&index) = indexes.get(chosen) else {
                    return;
                };
                if let Some(c) = untrack(|| card.get())
                    && c.profile != Some(index)
                {
                    let _ = c.set_profile(index);
                }
            },
        );
        row(8.0)
            .pl(indent)
            .child(div().flex_none().w(70.0).child(small_text("Profile")))
            .child(cp_combo(labels, binding).w(PROFILE_WIDTH))
            .into_view()
    })
}
