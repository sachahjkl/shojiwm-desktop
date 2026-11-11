//! The progress and counts programs show on their taskbar button, as Windows 7 lets them: a
//! download in the browser fills its button green (QML `LauncherEntries`).
//!
//! Programs announce them with the `com.canonical.Unity.LauncherEntry` `Update` signal on the
//! session bus (Chromium, Firefox, KDE and GTK programs do), naming their .desktop file.
//! `busctl` watches that signal and prints each one as a line of JSON.

use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use serde_json::Value;
use std::cell::Cell;
use std::collections::HashMap;

/// What a program shows on its button.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LauncherEntry {
    /// From 0 to 1, or `None` for no progress bar.
    pub progress: Option<f64>,
    /// The count on the button, or `None` for none.
    pub count: Option<i64>,
    /// It asks for attention.
    pub urgent: bool,
}

type Entries = Signal<HashMap<String, LauncherEntry>>;

thread_local! {
    static ENTRIES: Cell<Option<Entries>> = const { Cell::new(None) };
}

fn entries() -> Entries {
    ENTRIES.with(|cell| {
        cell.get().unwrap_or_else(|| {
            let (e, _) = create_root(|| {
                let e = signal(HashMap::new());
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
                // It lives as long as the root it was made in: the whole shell.
                watch.on_line(move |line| update(e, line));
                e
            });
            cell.set(Some(e));
            e
        })
    })
}

/// What the program of a .desktop file name (`chromium.desktop`) shows, if anything. Reading it
/// tracks it.
pub fn of(file_id: &str) -> Option<LauncherEntry> {
    entries().with(|e| e.get(file_id).copied())
}

/// Reads one signal.
fn update(entries: Entries, line: &str) {
    let Ok(message) = serde_json::from_str::<Value>(line) else {
        return;
    };
    let Some(data) = message
        .pointer("/payload/data")
        .and_then(Value::as_array)
        .filter(|d| d.len() >= 2)
    else {
        return;
    };
    let id = data[0]
        .as_str()
        .unwrap_or_default()
        .trim_start_matches("application://")
        .to_owned();
    let props = &data[1];
    let value = |key: &str| props.get(key).and_then(|p| p.get("data"));
    let old = entries.with_untracked(|e| e.get(&id).copied()).unwrap_or(LauncherEntry {
        progress: None,
        count: None,
        urgent: false,
    });
    let mut next = old;
    if value("progress-visible").is_some() || value("progress").is_some() {
        let shown = value("progress-visible")
            .and_then(Value::as_bool)
            .unwrap_or(old.progress.is_some());
        let progress = value("progress")
            .and_then(Value::as_f64)
            .unwrap_or_else(|| old.progress.unwrap_or(0.0));
        next.progress = shown.then(|| progress.clamp(0.0, 1.0));
    }
    if value("count-visible").is_some() || value("count").is_some() {
        let counted = value("count-visible")
            .and_then(Value::as_bool)
            .unwrap_or(old.count.is_some());
        let count = value("count")
            .and_then(Value::as_i64)
            .unwrap_or_else(|| old.count.unwrap_or(0));
        next.count = counted.then_some(count);
    }
    if let Some(urgent) = value("urgent") {
        next.urgent = urgent.as_bool() == Some(true);
    }
    if next != old || entries.with_untracked(|e| !e.contains_key(&id)) {
        entries.update(|e| {
            e.insert(id, next);
        });
    }
}
