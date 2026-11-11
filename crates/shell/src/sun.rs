//! Light by day and dark by night: with the setting `colorSchemeMode` at "sun", the system's
//! preference ([`set_dark`]) turns dark at sunset and light at sunrise.
//!
//! The place is the time zone's own, from tzdata's `zone1970.tab` (or `zone.tab`): no network
//! and no question to the user. The times are NOAA's sunrise equation. The choice is looked at
//! every minute, which also catches a resume from sleep. It is applied only when it changes, so
//! a choice made elsewhere (another program writing dconf) holds until the next sunrise or sunset.

use crate::settings::{change, choose_dark, dark, set_dark, settings};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::LocalTime;
use std::cell::Cell;
use std::f64::consts::PI;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::Duration;

/// How often the choice is looked at.
const CHECK: Duration = Duration::from_secs(60);
/// The first look waits for dconf to answer with the scheme in use.
const FIRST_CHECK: Duration = Duration::from_secs(2);

/// A place on Earth, in degrees: north and east are positive.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

/// The sun on one day.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SunDay {
    /// Sunrise and sunset, in seconds since 1970 (UTC).
    Times { rise: i64, set: i64 },
    /// The sun does not set.
    PolarDay,
    /// The sun does not rise.
    PolarNight,
}

/// Days since 1970-01-01 of a date of the proleptic Gregorian calendar.
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let y = i64::from(year) - i64::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = i64::from(month);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The sunrise and sunset of a date at `at`, by NOAA's equations (the general solar position:
/// the equation of time and the declination at noon, the sun's centre 0.833° below the horizon).
pub fn sun_day(at: Location, year: i32, month: u32, day: u32) -> SunDay {
    let midnight = days_from_civil(year, month, day);
    let day_of_year = (midnight - days_from_civil(year, 1, 1) + 1) as f64;
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let year_days = if leap { 366.0 } else { 365.0 };
    // The fractional year, in radians, at noon.
    let g = 2.0 * PI / year_days * (day_of_year - 1.0);
    let eqtime = 229.18
        * (0.000075 + 0.001868 * g.cos()
            - 0.032077 * g.sin()
            - 0.014615 * (2.0 * g).cos()
            - 0.040849 * (2.0 * g).sin());
    let decl = 0.006918 - 0.399912 * g.cos() + 0.070257 * g.sin() - 0.006758 * (2.0 * g).cos()
        + 0.000907 * (2.0 * g).sin()
        - 0.002697 * (3.0 * g).cos()
        + 0.00148 * (3.0 * g).sin();
    let lat = at.latitude.to_radians();
    let cos_hour_angle =
        90.833f64.to_radians().cos() / (lat.cos() * decl.cos()) - lat.tan() * decl.tan();
    if cos_hour_angle > 1.0 {
        return SunDay::PolarNight;
    }
    if cos_hour_angle < -1.0 {
        return SunDay::PolarDay;
    }
    let hour_angle = cos_hour_angle.acos().to_degrees();
    // Minutes after midnight UTC.
    let rise = 720.0 - 4.0 * (at.longitude + hour_angle) - eqtime;
    let set = 720.0 - 4.0 * (at.longitude - hour_angle) - eqtime;
    let unix = |minutes: f64| midnight * 86_400 + (minutes * 60.0).round() as i64;
    SunDay::Times {
        rise: unix(rise),
        set: unix(set),
    }
}

/// `+4852` or `+485212` (degrees, minutes, seconds) as degrees.
fn degrees(text: &str, degree_digits: usize) -> Option<f64> {
    let sign = match text.chars().next()? {
        '+' => 1.0,
        '-' => -1.0,
        _ => return None,
    };
    let digits = &text[1..];
    let part =
        |range: std::ops::Range<usize>| -> Option<f64> { digits.get(range)?.parse::<f64>().ok() };
    let d = part(0..degree_digits)?;
    let m = part(degree_digits..degree_digits + 2)?;
    let s = if digits.len() > degree_digits + 2 {
        part(degree_digits + 2..degree_digits + 4)?
    } else {
        0.0
    };
    Some(sign * (d + m / 60.0 + s / 3600.0))
}

/// The coordinates of a `zone.tab` line, `+4852+00220`, as a location.
fn parse_coordinates(text: &str) -> Option<Location> {
    let split = text[1..].find(['+', '-'])? + 1;
    Some(Location {
        latitude: degrees(&text[..split], 2)?,
        longitude: degrees(&text[split..], 3)?,
    })
}

/// The location of `zone` in a `zone.tab` file's text.
fn find_zone(table: &str, zone: &str) -> Option<Location> {
    table
        .lines()
        .filter(|l| !l.starts_with('#'))
        .map(|l| l.split('\t').collect::<Vec<_>>())
        .find(|f| f.get(2) == Some(&zone))
        .and_then(|f| parse_coordinates(f.get(1)?))
}

/// The folders tzdata may be in.
fn zone_folders() -> Vec<PathBuf> {
    let mut folders: Vec<PathBuf> = std::env::var_os("TZDIR")
        .map(PathBuf::from)
        .into_iter()
        .collect();
    folders.push("/etc/zoneinfo".into());
    folders.push("/usr/share/zoneinfo".into());
    folders
}

/// The name of the time zone, as `Europe/Paris`: where `/etc/localtime` points, else systemd's.
fn zone_name() -> Option<String> {
    if let Ok(target) = std::fs::read_link("/etc/localtime") {
        let target = target.to_string_lossy().into_owned();
        if let Some((_, name)) = target.split_once("zoneinfo/") {
            return Some(name.to_owned());
        }
    }
    let out = std::process::Command::new("timedatectl")
        .args(["show", "-p", "Timezone", "--value"])
        .output()
        .ok()?;
    let name = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    (!name.is_empty()).then_some(name)
}

/// The place of the time zone; `None` when the zone or tzdata's tables are not found.
pub fn location() -> Option<Location> {
    static LOCATION: OnceLock<Option<Location>> = OnceLock::new();
    *LOCATION.get_or_init(|| {
        let zone = zone_name()?;
        zone_folders().iter().find_map(|folder| {
            ["zone1970.tab", "zone.tab"].iter().find_map(|table| {
                let text = std::fs::read_to_string(folder.join(table)).ok()?;
                find_zone(&text, &zone)
            })
        })
    })
}

/// Today's sun at the time zone's place; `None` when the place is not known.
pub fn today() -> Option<SunDay> {
    let now = LocalTime::now();
    Some(sun_day(
        location()?,
        now.year,
        now.month.into(),
        now.day.into(),
    ))
}

/// Whether it is night at `unix` on that day.
fn night(day: SunDay, unix: i64) -> bool {
    match day {
        SunDay::Times { rise, set } => unix < rise || unix >= set,
        SunDay::PolarDay => false,
        SunDay::PolarNight => true,
    }
}

/// The user's choice: "light", "dark" or "sun".
pub fn choose_mode(mode: &str) {
    match mode {
        "sun" => change(|s| s.color_scheme_mode = "sun".into()),
        _ => choose_dark(mode == "dark"),
    }
}

/// Follows the sun while the setting says "sun". Call it once, from the shell's root.
pub fn install() {
    // The last choice applied: a new one is applied only when it differs.
    let applied: Rc<Cell<Option<bool>>> = Rc::default();
    let check: Rc<dyn Fn()> = {
        let applied = applied.clone();
        Rc::new(move || {
            if settings().with_untracked(|s| s.color_scheme_mode != "sun") {
                return;
            }
            let Some(day) = today() else {
                return;
            };
            let want = night(day, LocalTime::now().unix);
            if applied.replace(Some(want)) != Some(want) && untrack(dark) != want {
                set_dark(want);
            }
        })
    };
    // A change of mode starts afresh: choosing "sun" applies the sun's choice at once. The
    // first run waits for `FIRST_CHECK` instead, for dconf to answer with the scheme in use.
    let mode = memo(|| settings().with(|s| s.color_scheme_mode.clone()));
    let started = Cell::new(false);
    {
        let check = check.clone();
        effect(move || {
            mode.with(|_| ());
            applied.set(None);
            if started.replace(true) {
                untrack(|| check());
            }
        });
    }
    {
        let check = check.clone();
        set_timeout(FIRST_CHECK, move || check());
    }
    tick(check);
}

fn tick(check: Rc<dyn Fn()>) {
    set_timeout(CHECK, move || {
        check();
        tick(check);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const PARIS: Location = Location {
        latitude: 48.0 + 52.0 / 60.0,
        longitude: 2.0 + 20.0 / 60.0,
    };

    /// Minutes after local midnight, at a fixed UTC offset in hours.
    fn local_minutes(unix: i64, offset_hours: i64) -> i64 {
        (unix + offset_hours * 3600).rem_euclid(86_400) / 60
    }

    #[test]
    fn paris_summer_solstice() {
        let SunDay::Times { rise, set } = sun_day(PARIS, 2026, 6, 21) else {
            panic!("Paris has a sunrise in June");
        };
        // 05:47 and 21:58 in summer time (UTC+2).
        assert!((local_minutes(rise, 2) - (5 * 60 + 47)).abs() <= 2);
        assert!((local_minutes(set, 2) - (21 * 60 + 58)).abs() <= 2);
    }

    #[test]
    fn polar() {
        let north = Location {
            latitude: 78.2,
            longitude: 15.6,
        };
        assert_eq!(sun_day(north, 2026, 6, 21), SunDay::PolarDay);
        assert_eq!(sun_day(north, 2026, 12, 21), SunDay::PolarNight);
    }

    #[test]
    fn zone_table() {
        let table = "# comment\nFR\t+4852+00220\tEurope/Paris\nUS\t+404251-0740023\tAmerica/New_York\tEastern\n";
        assert_eq!(find_zone(table, "Europe/Paris"), Some(PARIS));
        let ny = find_zone(table, "America/New_York").unwrap();
        assert!((ny.latitude - 40.7142).abs() < 0.001);
        assert!((ny.longitude + 74.0064).abs() < 0.001);
    }

    #[test]
    fn days() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2000, 3, 1), 11_017);
    }
}
