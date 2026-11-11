//! Date and Time: the clock's format, the date under the time, the additional clocks in other
//! time zones, and the system's time zone and time synchronization as systemd reports them
//! (`timedatectl`).

use super::common::{
    control_row, cp_field, field_border, form_label, output_of, page_column, tinted_text,
};
use crate::control_panel::{heading, note, sub_heading};
use crate::settings::{ExtraClock, change, settings};
use crate::theme::scheme;
use crate::ui::{cp_check, cp_link, cp_radio};
use anti_webui::prelude::*;
use std::collections::HashMap;

/// The border of a time zone the system does not know.
const UNKNOWN_ZONE: Color = rgb(0xd9302a);
/// How many additional clocks there can be.
const EXTRA_CLOCKS: usize = 2;
/// The clock a new additional clock shows.
const NEW_CLOCK_ZONE: &str = "America/New_York";
const NEW_CLOCK_LABEL: &str = "New York";
/// The zones offered for what is typed: at most this many, once this many letters are typed.
const MATCHES: usize = 6;
const MATCH_FROM: usize = 2;
const FIELD_WIDTH: f32 = 260.0;
const LABEL_WIDTH: f32 = 100.0;

/// The locale's name, as `en_US` (Qt's `Qt.locale().name`): `LC_ALL`, `LC_TIME` or `LANG`
/// without its encoding.
fn locale_name() -> String {
    let value = ["LC_ALL", "LC_TIME", "LANG"]
        .iter()
        .filter_map(|k| std::env::var(k).ok())
        .find(|v| !v.is_empty())
        .unwrap_or_else(|| "C".into());
    value
        .split(['.', '@'])
        .next()
        .unwrap_or("C")
        .to_owned()
}

pub fn page() -> Div {
    let zones = signal(Vec::<String>::new());
    task(|| {
        output_of("timedatectl", &["list-timezones"])
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect::<Vec<_>>()
    })
    .then(move |list| zones.set(list));
    let format = Binding::new(
        || settings().with(|s| s.clock_format.clone()),
        |v: String| change(|s| s.clock_format = v),
    );
    let show_date = Binding::new(
        || settings().with(|s| s.show_date),
        |on: bool| change(|s| s.show_date = on),
    );
    page_column(10.0)
        .child(heading("Date and time"))
        .child(sub_heading("Clock format"))
        .child(cp_radio(
            format!("As the region sets it ({})", locale_name()),
            format,
            "locale".to_owned(),
        ))
        .child(cp_radio("24-hour (17:45)", format, "24h".to_owned()))
        .child(cp_radio("12-hour (5:45 PM)", format, "12h".to_owned()))
        .child(cp_check("Show the date under the time", show_date))
        .child(note(
            "The language and the date format follow the system locale (LANG, \
             i18n.defaultLocale on NixOS).",
        ))
        .child(sub_heading("Additional clocks"))
        .child(note(
            "Show the time in other time zones under the calendar and when you rest the \
             pointer on the clock.",
        ))
        .children((0..EXTRA_CLOCKS).map(|i| extra_clock(i, zones)).collect::<Vec<_>>())
        .child(sub_heading("Time zone and synchronization"))
        .child(time_facts())
        .child(note(
            "Set by the system's configuration (time.timeZone on NixOS), so it is shown here \
             and changed there.",
        ))
}

/// Keeps `zone` and `label` as additional clock `index`.
fn store(index: usize, zone: String, label: String) {
    change(|s| {
        if let Some(clock) = s.extra_clocks.get_mut(index) {
            *clock = ExtraClock { zone, label };
        }
    });
}

/// Additional clock `index`: a check box to show it, then its name and time zone. A second
/// clock comes after a first.
fn extra_clock(index: usize, zones: Signal<Vec<String>>) -> Div {
    let clock = memo(move || settings().with(|s| s.extra_clocks.get(index).cloned()));
    let count = memo(|| settings().with(|s| s.extra_clocks.len()));
    let shown = Binding::new(
        move || index < count.get(),
        move |on: bool| {
            change(|s| {
                if on {
                    s.extra_clocks.push(ExtraClock {
                        zone: NEW_CLOCK_ZONE.into(),
                        label: NEW_CLOCK_LABEL.into(),
                    });
                } else if index < s.extra_clocks.len() {
                    s.extra_clocks.remove(index);
                }
            })
        },
    );
    div()
        .flex()
        .flex_col()
        .gap(6.0)
        .dyn_style(move |s| if index <= count.get() { s } else { s.hidden() })
        .child(cp_check("Show this clock", shown))
        .child(show(
            move || clock.with(Option::is_some),
            move || clock_fields(index, clock, zones),
        ))
}

/// The display name and the time zone of a clock. A click on a zone offered for what is typed
/// picks it, and its city as the name.
fn clock_fields(index: usize, clock: Memo<Option<ExtraClock>>, zones: Signal<Vec<String>>) -> Div {
    // The name: kept when Enter is pressed or the field is left.
    let label = signal(String::new());
    effect(move || label.set(clock.with(|c| c.as_ref().map(|c| c.label.clone()).unwrap_or_default())));
    let finish_label = move || {
        let typed = label.get_untracked();
        if let Some(c) = untrack(|| clock.get())
            && typed != c.label
        {
            store(index, c.zone, typed);
        }
    };
    let label_field = cp_field(
        text_input()
            .value(label)
            .on_submit(move |_| finish_label())
            .on_focus_change(move |on| {
                if !on {
                    finish_label()
                }
            }),
        FIELD_WIDTH,
        field_border,
    );

    // The zone: only a zone the system knows is kept.
    let zone = signal(String::new());
    effect(move || zone.set(clock.with(|c| c.as_ref().map(|c| c.zone.clone()).unwrap_or_default())));
    let typing = signal(false);
    // The pointer is on the zones offered: a click there must not lose them first.
    let pointing = signal(false);
    let known = move |name: &str| zones.with(|z| z.iter().any(|x| x == name));
    let finish_zone = move || {
        let typed = zone.get_untracked();
        if let Some(c) = untrack(|| clock.get())
            && typed != c.zone
            && untrack(|| known(&typed))
        {
            store(index, typed, c.label);
        }
    };
    let zone_field = cp_field(
        text_input()
            .value(zone)
            .on_submit(move |_| finish_zone())
            .on_focus_change(move |on| {
                typing.set(on);
                if !on {
                    finish_zone()
                }
            }),
        FIELD_WIDTH,
        move |focused| {
            let c = scheme();
            let typed = zone.get();
            if focused {
                c.field_focus
            } else if zones.with(Vec::is_empty) || known(&typed) {
                c.field_border
            } else {
                UNKNOWN_ZONE
            }
        },
    );
    let matches = move || {
        if !typing.get() && !pointing.get() {
            return Vec::new();
        }
        let text = zone.get();
        let typed = text.to_lowercase().replace(' ', "_");
        if typed.chars().count() < MATCH_FROM {
            return Vec::new();
        }
        zones.with(|z| {
            z.iter()
                .filter(|name| name.to_lowercase().contains(&typed) && **name != text)
                .take(MATCHES)
                .cloned()
                .collect::<Vec<_>>()
        })
    };
    let offered = div()
        .flex()
        .flex_col()
        .gap(2.0)
        .on_hover(move |on| pointing.set(on))
        .child(for_each(matches, String::clone, move |name: String| {
            let shown = name.replace('_', " ");
            cp_link(shown, false, move || {
                let city = name.rsplit('/').next().unwrap_or(&name).replace('_', " ");
                store(index, name.clone(), city);
                typing.set(false);
                pointing.set(false);
            })
        }));
    div()
        .flex()
        .flex_col()
        .gap(6.0)
        .pl(24.0)
        .child(
            control_row(12.0)
                .child(form_label("Display name:", LABEL_WIDTH))
                .child(label_field),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_start()
                .gap(12.0)
                .child(form_label("Time zone:", LABEL_WIDTH))
                .child(div().flex().flex_col().gap(2.0).child(zone_field).child(offered)),
        )
}

/// The time zone and the synchronization, as `timedatectl show` reports them.
fn time_facts() -> Div {
    let values = signal(HashMap::<String, String>::new());
    task(|| {
        output_of("timedatectl", &["show"])
            .lines()
            .filter_map(|l| l.split_once('='))
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect::<HashMap<_, _>>()
    })
    .then(move |read| values.set(read));
    let value = move |key: &str| values.with(|v| v.get(key).cloned());
    let yes_no = move |key: &str, yes: &str, no: &str| match value(key).as_deref() {
        Some("yes") => yes.to_owned(),
        Some("no") => no.to_owned(),
        _ => "unknown".to_owned(),
    };
    let ink = || scheme().text;
    div()
        .flex()
        .flex_col()
        .gap(10.0)
        .child(tinted_text(
            move || format!("Time zone:  {}", value("Timezone").unwrap_or_else(|| "unknown".into())),
            ink,
        ))
        .child(tinted_text(
            move || {
                format!(
                    "Synchronized with an Internet time server:  {}",
                    yes_no("NTPSynchronized", "Yes", "No")
                )
            },
            ink,
        ))
        .child(tinted_text(
            move || format!("Network time:  {}", yes_no("NTP", "On", "Off")),
            ink,
        ))
}
