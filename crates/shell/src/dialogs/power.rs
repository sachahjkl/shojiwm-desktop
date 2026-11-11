//! The power actions (Apps.qml's `powerActions`, `runPower`): what "Shut Down Windows" and the
//! orb's menu offer, and how each runs. Shut down, restart, sleep and hibernate ask logind;
//! locking is the shell's own lock screen; logging off ends the session through `loginctl`.

use crate::settings::settings;
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;

/// A power action: its id, as the settings keep it (`power_button_action`,
/// `shutdown_dialog_action`), and what it says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PowerAction {
    pub id: &'static str,
    pub label: &'static str,
}

/// Every power action, in the order of the Start menu's power menu.
pub const POWER_ACTIONS: [PowerAction; 6] = [
    PowerAction { id: "logoff", label: "Log off" },
    PowerAction { id: "lock", label: "Lock" },
    PowerAction { id: "restart", label: "Restart" },
    PowerAction { id: "sleep", label: "Sleep" },
    PowerAction { id: "hibernate", label: "Hibernate" },
    PowerAction { id: "shutdown", label: "Shut down" },
];

/// Runs a power action by id. Before sleep and hibernate the session locks when the settings
/// ask for it, so that the computer wakes up locked, as Windows does.
pub fn run_power(id: &str) {
    match id {
        "lock" => crate::lock::lock(),
        "sleep" | "hibernate" => {
            if untrack(|| settings().with(|s| s.lock_on_suspend)) {
                crate::lock::lock();
            }
            let session = session();
            // Dropping the task does not cancel the call.
            drop(if id == "sleep" { session.suspend() } else { session.hibernate() });
        }
        "restart" => drop(session().reboot()),
        "shutdown" => drop(session().power_off()),
        "logoff" => log_off(),
        _ => {}
    }
}

/// Ends this session: `loginctl terminate-session`, or `terminate-user` when the session has no
/// id in the environment.
fn log_off() {
    let args: Vec<String> = match std::env::var("XDG_SESSION_ID").ok().filter(|s| !s.is_empty()) {
        Some(id) => vec!["terminate-session".into(), id],
        None => vec!["terminate-user".into(), login_name()],
    };
    match std::process::Command::new("loginctl").args(args).spawn() {
        Ok(mut child) => drop(std::thread::spawn(move || child.wait())),
        Err(e) => eprintln!("shojiwm-shell: cannot log off: {e}"),
    }
}

/// `$USER`, else the last part of the home folder.
pub(super) fn login_name() -> String {
    std::env::var("USER")
        .ok()
        .filter(|u| !u.is_empty())
        .or_else(|| {
            crate::settings::home()
                .and_then(|h| h.file_name().map(|n| n.to_string_lossy().into_owned()))
        })
        .unwrap_or_default()
}
