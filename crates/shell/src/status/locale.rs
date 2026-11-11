//! The dates and times of the clock, the calendar and the notifications in the user's locale.
//!
//! Qt gave the QML shell its locale (`Qt.locale()`). Here the C library's time functions run in
//! the "C" locale, so the locale's names and formats are read once from `locale -k LC_TIME`,
//! and the numbers are put in them by `LocalTime::format`.

use super::collect;
use crate::settings::settings;
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use std::cell::Cell;

/// The names and formats of the locale's time category.
#[derive(Clone, Debug, PartialEq)]
pub struct Locale {
    /// The "C" or "POSIX" locale (no language chosen): the date is day first, as on Windows.
    pub c: bool,
    /// From Sunday.
    pub day_names: Vec<String>,
    pub short_day_names: Vec<String>,
    /// From January.
    pub month_names: Vec<String>,
    /// The short date, as `strftime` takes it (`%m/%d/%Y`).
    pub date_format: String,
    /// The locale writes the time on a 12-hour clock.
    pub twelve_hour: bool,
    /// The first day of the week: 0 is Sunday.
    pub first_weekday: u8,
}

const DAYS: [&str; 7] = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
const SHORT_DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September",
    "October", "November", "December",
];

/// Whether the environment chose no language (`LC_ALL`, `LC_TIME`, then `LANG`).
fn chose_c() -> bool {
    let chosen = ["LC_ALL", "LC_TIME", "LANG"]
        .iter()
        .filter_map(|v| std::env::var(v).ok())
        .find(|v| !v.is_empty());
    match chosen {
        None => true,
        Some(v) => v == "C" || v == "POSIX" || v.starts_with("C."),
    }
}

impl Default for Locale {
    /// English names, day first, weeks from Monday (Qt's "C" locale), until `locale` answers.
    fn default() -> Self {
        Self {
            c: chose_c(),
            day_names: DAYS.map(String::from).to_vec(),
            short_day_names: SHORT_DAYS.map(String::from).to_vec(),
            month_names: MONTHS.map(String::from).to_vec(),
            date_format: "%d/%m/%Y".into(),
            twelve_hour: false,
            first_weekday: 1,
        }
    }
}

thread_local! {
    static LOCALE: Cell<Option<Signal<Locale>>> = const { Cell::new(None) };
}

fn locale_signal() -> Signal<Locale> {
    LOCALE.with(|cell| {
        cell.get().unwrap_or_else(|| {
            let (s, _) = create_root(|| signal(Locale::default()));
            cell.set(Some(s));
            s
        })
    })
}

/// The locale. Reading it tracks it.
pub fn locale() -> Locale {
    locale_signal().get()
}

/// Reads the locale's time category once.
pub fn install() {
    let s = locale_signal();
    collect("locale", ["-k", "LC_TIME"], move |lines| {
        let mut l = Locale::default();
        let list = |v: &str| -> Vec<String> { v.split(';').map(String::from).collect() };
        for line in &lines {
            let Some((key, value)) = line.split_once('=') else { continue };
            let value = value.trim().trim_matches('"');
            match key.trim() {
                "day" if list(value).len() == 7 => l.day_names = list(value),
                "abday" if list(value).len() == 7 => l.short_day_names = list(value),
                "mon" if list(value).len() == 12 => l.month_names = list(value),
                "d_fmt" if !value.is_empty() => l.date_format = value.to_owned(),
                "t_fmt" => {
                    l.twelve_hour = ["%r", "%I", "%l", "%p"].iter().any(|f| value.contains(f));
                }
                // 1 is the day `week-1stday` names, which glibc sets to a Sunday.
                "first_weekday" => {
                    if let Ok(n) = value.parse::<u8>() {
                        l.first_weekday = n.saturating_sub(1) % 7;
                    }
                }
                _ => {}
            }
        }
        if l.c {
            // Qt's "C" locale: day first, weeks from Monday.
            l.date_format = "%d/%m/%Y".into();
            l.first_weekday = 1;
        }
        s.set(l);
    });
}

/// The time as the clock shows it: the setting's 24- or 12-hour clock, or the locale's.
pub fn short_time(t: &LocalTime) -> String {
    let format = settings().with(|s| s.clock_format.clone());
    let twelve = match format.as_str() {
        "24h" => false,
        "12h" => true,
        _ => locale().twelve_hour,
    };
    t.format(if twelve { "%-I:%M %p" } else { "%H:%M" })
}

/// The date in full, day first and with the year in four digits, as Windows 7 shows it under
/// the time: the locale's short form with its numbers padded and its spaces made slashes.
pub fn short_date(t: &LocalTime) -> String {
    let l = locale();
    let format = l
        .date_format
        .replace("%y", "%Y")
        .replace("%e", "%d")
        .replace("%-d", "%d")
        .replace("%-m", "%m")
        .replace(' ', "/");
    t.format(&format)
}

/// The locale's own short date, as the notifications show an older one's day.
pub fn plain_date(t: &LocalTime) -> String {
    t.format(&locale().date_format)
}

/// The long date: the day's name, the day, the month's name and the year, in the order of the
/// locale's short date (month first, or day first).
pub fn long_date(t: &LocalTime) -> String {
    let l = locale();
    let day = l.day_names.get(t.weekday as usize).cloned().unwrap_or_default();
    let month = l
        .month_names
        .get(t.month.saturating_sub(1) as usize)
        .cloned()
        .unwrap_or_default();
    if l.date_format.trim_start().starts_with("%m") {
        format!("{day}, {month} {}, {}", t.day, t.year)
    } else {
        format!("{day} {} {month} {}", t.day, t.year)
    }
}

/// A month and its year, as the calendar titles it.
pub fn month_title(year: i32, month: u8) -> String {
    let name = locale()
        .month_names
        .get(month.saturating_sub(1) as usize)
        .cloned()
        .unwrap_or_default();
    format!("{name} {year}")
}
