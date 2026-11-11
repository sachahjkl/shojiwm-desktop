//! The programs the desktop starts, by key: the one chosen in Default Programs
//! (`settings().programs`), else the session's (`$SHOJIWM_DESKTOP_*`), else a default.

use crate::settings::settings;
use serde_json::{Value, json};

/// The keys, the variable of the session's choice, and the default.
const SESSION: &[(&str, &str, &str)] = &[
    ("terminal", "SHOJIWM_DESKTOP_TERMINAL", "kitty"),
    ("fileManager", "SHOJIWM_DESKTOP_FILE_MANAGER", "thunar"),
    ("taskManager", "SHOJIWM_DESKTOP_TASK_MANAGER", "inspecteur"),
    (
        "systemMonitor",
        "SHOJIWM_DESKTOP_SYSTEM_MONITOR",
        "inspecteur --section performance",
    ),
    ("clipboard", "SHOJIWM_DESKTOP_CLIPBOARD", ""),
    (
        "screenshot",
        "SHOJIWM_DESKTOP_SCREENSHOT",
        "grim - | wl-copy --type image/png",
    ),
];

/// The session's command for a key, before any choice in Default Programs.
pub fn session_program(key: &str) -> String {
    SESSION
        .iter()
        .find(|(k, _, _)| *k == key)
        .map(|(_, var, default)| {
            std::env::var(var)
                .ok()
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| (*default).to_owned())
        })
        .unwrap_or_default()
}

/// The command of a program now, by its key. Reading it tracks the settings.
pub fn program(key: &str) -> String {
    let chosen = settings().with(|s| {
        s.programs
            .get(key)
            .and_then(Value::as_str)
            .map(String::from)
            .unwrap_or_default()
    });
    if chosen.is_empty() {
        session_program(key)
    } else {
        chosen
    }
}

/// What the compositor's keys start (`src/programs.ts`).
pub fn compositor_programs() -> Value {
    json!({
        "terminal": program("terminal"),
        "fileManager": program("fileManager"),
        "taskManager": program("taskManager"),
        "clipboard": program("clipboard"),
        "screenshot": program("screenshot"),
    })
}

/// Starts a program the user works in from a command line, in a scope of its own
/// (`crate::spawn`).
pub fn run(command: &str) {
    crate::spawn::start_command_line(command, None);
}

/// Runs a short helper of the shell from a command line (`systemctl`, `loginctl`, `hokum`, ...),
/// detached from this process, in the shell's own unit.
pub fn run_helper(command: &str) {
    if command.trim().is_empty() {
        return;
    }
    let _ = std::process::Command::new("setsid")
        .args(["-f", "sh", "-c", command])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}
