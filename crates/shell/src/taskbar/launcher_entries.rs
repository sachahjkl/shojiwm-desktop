//! The progress and counts programs show on their taskbar button, as Windows 7 lets them: a
//! download in the browser fills its button green (QML `LauncherEntries.qml`).
//!
//! Programs announce them with the `com.canonical.Unity.LauncherEntry` `Update` signal on the
//! session bus (Chromium, Firefox, KDE and GTK programs do), naming their `.desktop` file.
//! AntiWebUI keeps its D-Bus connection to itself, so `busctl` watches that signal and prints
//! each one as a line of JSON, as the QML shell did.

use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use serde_json::Value;
use std::cell::Cell;
use std::collections::HashMap;

/// What a program shows on its button.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LauncherEntry {
    /// From 0 to 1, or `None` for no progress.
    pub progress: Option<f64>,
    /// A count to show on the icon, or `None`.
    pub count: Option<i64>,
    /// The program asks to be looked at.
    pub urgent: bool,
}

impl Default for LauncherEntry {
    fn default() -> Self {
        Self { progress: None, count: None, urgent: false }
    }
}

thread_local! {
    static ENTRIES: Cell<Option<Signal<HashMap<String, LauncherEntry>>>> = const { Cell::new(None) };
}

fn entries() -> Signal<HashMap<String, LauncherEntry>> {
    ENTRIES.with(|cell| {
        cell.get().unwrap_or_else(|| {
            let (entries, _) = create_root(|| {
                let entries = signal(HashMap::new());
                let watch = Process::new("busctl")
                    .args([
                        "--user",
                        "monitor",
                        "--json=short",
                        "--match",
                        "type='signal',interface='com.canonical.Unity.LauncherEntry'",
                    ])
                    .keep(1)
                    .spawn();
                watch.on_line(move |line| update(entries, line));
                entries
            });
            cell.set(Some(entries));
            entries
        })
    })
}

/// Starts watching, outside of any surface. Call once, from the root.
pub fn install() {
    entries();
}

/// What the program of a `.desktop` file name (`firefox.desktop`) shows, if anything. Reading it
/// tracks it.
pub fn launcher_entry(file_id: &str) -> Option<LauncherEntry> {
    entries().with(|e| e.get(file_id).copied())
}

/// Reads one signal: `(application://<id>, { progress, progress-visible, count, count-visible,
/// urgent })`, each property given as `{ type, data }`.
fn update(entries: Signal<HashMap<String, LauncherEntry>>, line: &str) {
    let Ok(message) = serde_json::from_str::<Value>(line) else {
        return;
    };
    let Some(data) = message.pointer("/payload/data").and_then(Value::as_array) else {
        return;
    };
    let (Some(app), Some(props)) = (data.first().and_then(Value::as_str), data.get(1)) else {
        return;
    };
    let id = app.trim_start_matches("application://").to_owned();
    let value = |key: &str| props.get(key).and_then(|p| p.get("data"));
    let old = entries.with_untracked(|e| e.get(&id).copied()).unwrap_or_default();
    let mut next = old;
    let progress_visible = value("progress-visible").and_then(Value::as_bool);
    let progress = value("progress").and_then(Value::as_f64);
    if progress_visible.is_some() || progress.is_some() {
        let shown = progress_visible.unwrap_or(old.progress.is_some());
        let at = progress.or(old.progress).unwrap_or(0.0);
        next.progress = shown.then(|| at.clamp(0.0, 1.0));
    }
    let count_visible = value("count-visible").and_then(Value::as_bool);
    let count = value("count").and_then(Value::as_i64);
    if count_visible.is_some() || count.is_some() {
        let counted = count_visible.unwrap_or(old.count.is_some());
        let n = count.or(old.count).unwrap_or(0);
        next.count = counted.then_some(n);
    }
    if let Some(urgent) = value("urgent") {
        next.urgent = urgent.as_bool() == Some(true);
    }
    entries.update(|e| {
        e.insert(id, next);
    });
}
