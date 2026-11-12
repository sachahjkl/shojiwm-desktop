//! Where the Run dialog is on its screen. A drag on its caption moves it; the place is kept per
//! screen (by connector name), from the screen's top-left in logical pixels, in
//! `run-dialog.json`, and used the next time the dialog opens there. The suggestions open
//! under the field when their rows fit there, else over it.

use crate::paths::state_dir;
use serde_json::{Map, Value, json};
use std::path::PathBuf;

/// The space between the field and its suggestions.
pub const LIST_GAP: f32 = 2.0;

fn places_path() -> PathBuf {
    state_dir().join("run-dialog.json")
}

fn read_places() -> Map<String, Value> {
    std::fs::read_to_string(places_path())
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|value| match value {
            Value::Object(map) => Some(map),
            _ => None,
        })
        .unwrap_or_default()
}

/// The place kept for the screen `output`, if any. It reads the disk: call it in a task.
pub fn load(output: &str) -> Option<(f32, f32)> {
    let places = read_places();
    let at = places.get(output)?;
    let x = at.get("x")?.as_f64()?;
    let y = at.get("y")?.as_f64()?;
    Some((x as f32, y as f32))
}

/// Keeps `at` as the place of the dialog on the screen `output`. It reads and writes the disk:
/// call it on the writer thread (`crate::apps::run_in_order`), so that two saves keep their order.
pub fn save(output: &str, at: (f32, f32)) {
    let mut places = read_places();
    places.insert(
        output.to_owned(),
        json!({ "x": at.0.round(), "y": at.1.round() }),
    );
    let path = places_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, Value::Object(places).to_string());
}

/// `at`, the top-left of a box of `size`, moved so that the box is all on a screen of `screen`.
/// A box larger than the screen keeps its top-left on the screen.
pub fn clamp(at: (f32, f32), size: (f32, f32), screen: (f32, f32)) -> (f32, f32) {
    let axis = |v: f32, size: f32, room: f32| v.min(room - size).max(0.0);
    (axis(at.0, size.0, screen.0), axis(at.1, size.1, screen.1))
}

/// The height of the suggestions with `rows` rows of `row_height`: the rows, the frame's
/// padding (2 above, 2 below) and its line (1 above, 1 below).
pub fn list_height(rows: usize, row_height: f32) -> f32 {
    rows as f32 * row_height + 4.0 + 2.0
}

/// The suggestions open under the field (from `field_top` to `field_bottom` on the screen,
/// `screen_height` tall) when `list` fits between the field and the taskbar, or the bottom of
/// the screen when the bar is at the top. Else they open over the field when they fit there,
/// else on the side with more room.
pub fn list_below(
    field_top: f32,
    field_bottom: f32,
    list: f32,
    screen_height: f32,
    taskbar_height: f32,
    bar_at_top: bool,
) -> bool {
    let (top, bottom) = if bar_at_top {
        (taskbar_height, screen_height)
    } else {
        (0.0, screen_height - taskbar_height)
    };
    let room_below = bottom - field_bottom - LIST_GAP;
    let room_above = field_top - LIST_GAP - top;
    room_below >= list || (room_above < list && room_below >= room_above)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: (f32, f32) = (1920.0, 1080.0);
    const DIALOG: (f32, f32) = (420.0, 200.0);

    #[test]
    fn clamp_keeps_a_place_on_the_screen() {
        assert_eq!(clamp((100.0, 300.0), DIALOG, SCREEN), (100.0, 300.0));
    }

    #[test]
    fn clamp_moves_a_place_off_the_screen_back_onto_it() {
        assert_eq!(clamp((-50.0, -10.0), DIALOG, SCREEN), (0.0, 0.0));
        assert_eq!(clamp((1800.0, 1000.0), DIALOG, SCREEN), (1500.0, 880.0));
    }

    #[test]
    fn clamp_follows_a_smaller_resolution() {
        assert_eq!(
            clamp((1500.0, 880.0), DIALOG, (1280.0, 720.0)),
            (860.0, 520.0)
        );
    }

    #[test]
    fn clamp_keeps_the_top_left_of_a_box_larger_than_the_screen() {
        assert_eq!(clamp((30.0, 30.0), (2000.0, 1200.0), SCREEN), (0.0, 0.0));
    }

    #[test]
    fn list_height_counts_rows_padding_and_line() {
        assert_eq!(list_height(8, 26.0), 214.0);
    }

    #[test]
    fn list_opens_below_when_it_fits_there() {
        assert!(list_below(400.0, 424.0, 214.0, 1080.0, 40.0, false));
    }

    #[test]
    fn list_opens_above_near_the_bottom_bar() {
        // The field ends 100 px over the bar: 214 px do not fit under it.
        assert!(!list_below(916.0, 940.0, 214.0, 1080.0, 40.0, false));
    }

    #[test]
    fn list_counts_a_bar_at_the_top_as_taken_room() {
        // Room under the field: the screen's bottom, as the bar is at the top.
        assert!(list_below(916.0, 940.0, 120.0, 1080.0, 40.0, true));
        // Room over the field: 160 - 2 - 40 = 118 px only, and less still under it.
        assert!(!list_below(160.0, 1000.0, 214.0, 1080.0, 40.0, true));
    }

    #[test]
    fn list_takes_the_larger_side_when_neither_fits() {
        assert!(list_below(100.0, 124.0, 2000.0, 1080.0, 40.0, false));
        assert!(!list_below(900.0, 924.0, 2000.0, 1080.0, 40.0, false));
    }
}
