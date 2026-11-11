//! The clock of the taskbar and its calendar (QML `Clock`, `CalendarPopup`, `CalendarArrow`),
//! and the additional clocks of the Date and Time settings (QML `WorldClocks`).
//!
//! The clock shows the time over the date. Its tooltip is the long date and the additional
//! clocks. A click opens the calendar, as Windows 7 shows it: the long date at the top, then a
//! month with today framed, arrows for the months around it, and the additional clocks under
//! it. Weeks start on the locale's first day.

use super::locale::{locale, long_date, month_title, short_date, short_time};
use super::{Panel, bar_cell, body_text, collect, divider, every, is_open, panel_link, panel_popup, small_text, toggle};
use crate::paths::asset;
use crate::settings::settings;
use crate::theme::{Align, away_from_bar, corner, item_radius, scheme, theme};
use crate::ui::{MenuLine, bar_menu_options, clock_font, font, popup_frame, popup_menu, shell_tooltip};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use std::cell::Cell;
use std::time::Duration;

/// The width of a day of the calendar, and the heights of a day and of a weekday's name.
const CELL: f32 = 30.0;
const DAY_HEIGHT: f32 = 24.0;
const WEEKDAY_HEIGHT: f32 = 20.0;
/// The room around the calendar's content.
const MARGIN: f32 = 12.0;
/// How often the additional clocks are read again.
const WORLD_REFRESH: Duration = Duration::from_secs(30);

// ----- the additional clocks -------------------------------------------------------------------

/// An additional clock: the time in another zone, and its day from here.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldClock {
    pub label: String,
    pub zone: String,
    pub time: String,
    /// "Today", "Tomorrow" or "Yesterday".
    pub day: &'static str,
}

thread_local! {
    static CLOCKS: Cell<Option<Signal<Vec<WorldClock>>>> = const { Cell::new(None) };
    static ZONES: Cell<Option<Signal<Vec<String>>>> = const { Cell::new(None) };
}

fn clocks_signal() -> Signal<Vec<WorldClock>> {
    CLOCKS.with(|cell| {
        cell.get().unwrap_or_else(|| {
            let (s, _) = create_root(|| signal(Vec::new()));
            cell.set(Some(s));
            s
        })
    })
}

/// The additional clocks, as last read. Reading them tracks them.
fn world_clocks() -> Vec<WorldClock> {
    clocks_signal().get()
}

/// Every time zone the system knows (`timedatectl list-timezones`), for the Date and Time
/// settings. Read the first time it is asked for. Reading it tracks it.
pub fn zones() -> Vec<String> {
    let s = ZONES.with(|cell| {
        cell.get().unwrap_or_else(|| {
            let (s, _) = create_root(|| signal(Vec::new()));
            cell.set(Some(s));
            collect("timedatectl", ["list-timezones"], move |lines| {
                s.set(lines.into_iter().filter(|l| !l.is_empty()).collect());
            });
            s
        })
    });
    s.get()
}

/// Reads the time in each zone of the settings' additional clocks. `date` reads each zone from
/// the system's zone database.
fn refresh_clocks() {
    let (list, twelve) =
        settings().with_untracked(|s| (s.extra_clocks.clone(), s.clock_format == "12h"));
    let target = clocks_signal();
    if list.is_empty() {
        target.set(Vec::new());
        return;
    }
    let format = if twelve { "%-I:%M %p" } else { "%H:%M" };
    let mut args = vec![
        "-c".to_owned(),
        "format=$1; shift; date +%Y%m%d; for zone; do TZ=$zone date +\"%Y%m%d|$format\"; done"
            .to_owned(),
        "sh".to_owned(),
        format.to_owned(),
    ];
    args.extend(list.iter().map(|c| c.zone.clone()));
    collect("sh", args, move |lines| {
        let here: u32 = lines.first().and_then(|l| l.trim().parse().ok()).unwrap_or(0);
        let out = list
            .iter()
            .enumerate()
            .map(|(i, clock)| {
                let line = lines.get(i + 1).cloned().unwrap_or_default();
                let (date, time) = line.split_once('|').unwrap_or(("", ""));
                // Dates as yyyymmdd numbers: compare their order only.
                let there: u32 = date.trim().parse().unwrap_or(0);
                WorldClock {
                    label: if clock.label.is_empty() { clock.zone.clone() } else { clock.label.clone() },
                    zone: clock.zone.clone(),
                    time: time.to_owned(),
                    day: match there.cmp(&here) {
                        std::cmp::Ordering::Equal => "Today",
                        std::cmp::Ordering::Greater => "Tomorrow",
                        std::cmp::Ordering::Less => "Yesterday",
                    },
                }
            })
            .collect();
        target.set(out);
    });
}

/// Reads the additional clocks every 30 seconds, and at once when they or the format change.
pub fn install() {
    let wanted = memo(|| settings().with(|s| (s.extra_clocks.clone(), s.clock_format.clone())));
    effect(move || {
        wanted.with(|_| ());
        untrack(refresh_clocks);
    });
    every(WORLD_REFRESH, refresh_clocks);
}

// ----- the clock -------------------------------------------------------------------------------

/// The clock at the right end of the taskbar: the time over the date. A click opens the
/// calendar, a right click the clock's menu.
pub fn clock_view(output_name: &str) -> Div {
    let name = output_name.to_owned();
    let anchor = node_ref();
    let menu = signal(false);
    let active = {
        let name = name.clone();
        move || is_open(&name, Panel::Calendar)
    };
    let opener = name.clone();
    let lines_name = name.clone();
    let line = |f: fn(&LocalTime) -> String| {
        div()
            .flex_none()
            .whitespace_nowrap()
            .dyn_style(|s| {
                let t = theme();
                s.font_family(clock_font())
                    .text_size(t.font_size)
                    .text_color(t.text_on_glass)
            })
            .child(text(move || f(&local_time(Every::Minute))))
    };
    let column = div()
        .relative()
        .flex()
        .flex_col()
        .items_center()
        .child(line(short_time))
        .child(
            line(short_date)
                .dyn_style(|s| if settings().with(|s| s.show_date) { s } else { s.hidden() }),
        );
    let cell = bar_cell(|| theme().clock_width, 0.0, active)
        .node_ref(anchor)
        .child(column)
        .on_click(move |_| toggle(&opener, Panel::Calendar))
        .on_aux_click(move |e| {
            if e.button == Some(PointerButton::Secondary) {
                menu.set(!menu.get_untracked());
            }
        })
        .child(popup_menu(
            move || menu.get(),
            move || menu.set(false),
            bar_menu_options(anchor),
            move || {
                let name = lines_name.clone();
                vec![
                    MenuLine::item("Open the calendar", move || {
                        super::open(&name, Panel::Calendar);
                    })
                    .icon(asset("icons/calendar.png").to_string_lossy().into_owned()),
                    MenuLine::item("Adjust date/time", || {
                        crate::control_panel::open_page("clock");
                    }),
                    MenuLine::separator(),
                    MenuLine::item("Show the desktop", crate::taskbar::show_the_desktop),
                    MenuLine::item("Start Task Manager", || {
                        crate::programs::run(&crate::programs::program("taskManager"));
                    }),
                ]
            },
        ))
        .child(panel_popup(&name, Panel::Calendar, Align::End, calendar));
    shell_tooltip(cell, untrack(|| away_from_bar(Align::Center)), || {
        let mut lines = vec![long_date(&local_time(Every::Minute))];
        lines.extend(world_clocks().into_iter().map(|c| {
            let day = if c.day == "Today" { String::new() } else { format!(" ({})", c.day.to_lowercase()) };
            format!("{}: {}{day}", c.label, c.time)
        }));
        lines.join("\n")
    })
}

// ----- the calendar ----------------------------------------------------------------------------

/// A month as one number, to count in: the year times 12, and the month from 0.
fn month_index(year: i32, month: u8) -> i32 {
    year * 12 + month as i32 - 1
}

fn from_month_index(i: i32) -> (i32, u8) {
    (i.div_euclid(12), i.rem_euclid(12) as u8 + 1)
}

fn days_in(year: i32, month: u8) -> i32 {
    match month {
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// The day of the week of a date: 0 is Sunday (Sakamoto's method).
fn weekday(year: i32, month: u8, day: i32) -> i32 {
    const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if month < 3 { year - 1 } else { year };
    (y + y / 4 - y / 100 + y / 400 + T[month as usize - 1] + day).rem_euclid(7)
}

/// A day of the grid: its month (as a [`month_index`]) and its day of that month.
#[derive(Clone, Copy, PartialEq)]
struct Day {
    month: i32,
    day: i32,
}

/// The 42 days of the grid: the month, padded with its neighbours, from the locale's first
/// day of the week.
fn days_shown(month: i32, first_weekday: u8) -> Vec<Day> {
    let (year, m) = from_month_index(month);
    let (py, pm) = from_month_index(month - 1);
    let (count, before) = (days_in(year, m), days_in(py, pm));
    let lead = (weekday(year, m, 1) - first_weekday as i32).rem_euclid(7);
    (0..42)
        .map(|n| {
            let d = n - lead + 1;
            if d < 1 {
                Day { month: month - 1, day: before + d }
            } else if d > count {
                Day { month: month + 1, day: d - count }
            } else {
                Day { month, day: d }
            }
        })
        .collect()
}

/// The calendar.
fn calendar() -> Div {
    let today = memo(|| {
        let t = local_time(Every::Minute);
        (month_index(t.year, t.month), t.day as i32)
    });
    // Opening always starts from this month.
    let shown = signal(untrack(|| today.get().0));
    let header = div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .h(26.0)
        .child(arrow("\u{25C2}", move || shown.update(|m| *m -= 1)))
        .child(body_text(
            move || {
                let (y, m) = from_month_index(shown.get());
                month_title(y, m)
            },
            || scheme().text,
        )
        .font_weight(FontWeight::BOLD)
        .dyn_style(|s| s.text_size(theme().font_size + 1.0)))
        .child(arrow("\u{25B8}", move || shown.update(|m| *m += 1)));
    let weekdays = dyn_view(move || {
        let l = locale();
        div().flex().flex_row().children(
            (0..7)
                .map(|i| {
                    let name = l
                        .short_day_names
                        .get((l.first_weekday as usize + i) % 7)
                        .cloned()
                        .unwrap_or_default();
                    div()
                        .flex()
                        .justify_center()
                        .items_center()
                        .w(CELL)
                        .h(WEEKDAY_HEIGHT)
                        .child(small_text(name, || scheme().menu_heading))
                })
                .collect::<Vec<_>>(),
        )
    });
    let grid = dyn_view(move || {
        let month = shown.get();
        let first = locale().first_weekday;
        div()
            .flex()
            .flex_row()
            .flex_wrap()
            .w(CELL * 7.0)
            .children(
                days_shown(month, first)
                    .into_iter()
                    .map(|d| day_cell(d, month, today, shown))
                    .collect::<Vec<_>>(),
            )
    });
    let back_to_today = div()
        .flex()
        .justify_center()
        .dyn_style(move |s| if shown.get() == today.get().0 { s.hidden() } else { s })
        .child(panel_link("Today", false, move || shown.set(untrack(|| today.get().0))));
    let others = dyn_view(move || {
        let list = world_clocks();
        div()
            .flex()
            .flex_col()
            .gap(6.0)
            .children((!list.is_empty()).then(divider))
            .children(list.into_iter().map(world_clock_row).collect::<Vec<_>>())
    });
    popup_frame().w(CELL * 7.0 + MARGIN * 2.0).child(
        div()
            .flex()
            .flex_col()
            .gap(6.0)
            .p(MARGIN)
            .child(
                div()
                    .flex()
                    .justify_center()
                    .overflow_clip()
                    .whitespace_nowrap()
                    .child(body_text(
                        || long_date(&local_time(Every::Minute)),
                        || scheme().link,
                    )),
            )
            .child(divider())
            .child(header)
            .child(div().flex().flex_col().child(weekdays).child(grid))
            .child(back_to_today)
            .child(others),
    )
}

/// A month arrow of the calendar.
fn arrow(glyph: &'static str, on_click: impl Fn() + 'static) -> Div {
    let ui = Interaction::new();
    div()
        .flex()
        .items_center()
        .justify_center()
        .w(24.0)
        .h(22.0)
        .border(1.0)
        .interaction(ui)
        .on_click(move |_| on_click())
        .dyn_style(move |s| {
            let c = scheme();
            let s = s
                .rounded(item_radius())
                .font_family(font())
                .text_size(theme().font_size)
                .text_color(c.text);
            if ui.hovered() {
                s.bg(c.hover).border_color(c.selection_border)
            } else {
                s.bg(Color::TRANSPARENT).border_color(Color::TRANSPARENT)
            }
        })
        .child(glyph)
}

/// A day of the grid: today framed; a day of the month around turns to it when clicked.
fn day_cell(d: Day, month: i32, today: Memo<(i32, i32)>, shown: Signal<i32>) -> Div {
    let ui = Interaction::new();
    let in_month = d.month == month;
    let is_today = memo(move || today.get() == (d.month, d.day));
    div()
        .flex_none()
        .w(CELL)
        .h(DAY_HEIGHT)
        .p(1.0)
        .on_click(move |_| {
            if !in_month {
                shown.set(d.month);
            }
        })
        .interaction(ui)
        .child(
            div()
                .w_full()
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .border(1.0)
                .dyn_style(move |s| {
                    let c = scheme();
                    let s = s
                        .rounded(corner(2.0))
                        .font_family(font())
                        .text_size(theme().font_size)
                        .text_color(if in_month { c.text } else { c.other_month });
                    if is_today.get() {
                        s.bg(c.selection).border_color(c.today).font_weight(FontWeight::BOLD)
                    } else if ui.hovered() {
                        s.bg(c.hover).border_color(c.selection_border)
                    } else {
                        s.bg(Color::TRANSPARENT).border_color(Color::TRANSPARENT)
                    }
                })
                .child(text(d.day.to_string())),
        )
}

/// An additional clock under the month: its name and its day, and its time, large.
fn world_clock_row(clock: WorldClock) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .h(34.0)
        .child(
            div()
                .flex()
                .flex_col()
                .child(body_text(clock.label.clone(), || scheme().text))
                .child(small_text(clock.day, || scheme().text_muted)),
        )
        .child(
            body_text(clock.time.clone(), || scheme().text)
                .font_weight(FontWeight::LIGHT)
                .dyn_style(|s| s.text_size(theme().font_size + 4.0)),
        )
}
