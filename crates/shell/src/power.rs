//! The power actions: log off, lock, restart, sleep, hibernate, shut down. The Start menu's
//! power button and its arrow, the Shut Down dialog, the log-on screen and the Control Panel
//! all offer them from here.
//!
//! Restart, sleep, hibernate and shut down go through logind when it is there and allows them;
//! else they run `systemctl`. Log off ends the session with `loginctl`. Lock is the shell's own
//! lock screen.

use crate::settings::settings;
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;

/// Something the power button can do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PowerAction {
    /// "logoff", "lock", "restart", "sleep", "hibernate", "shutdown"; "separator" for a line
    /// between groups in a menu.
    pub id: &'static str,
    pub label: &'static str,
}

/// What the power button and its arrow offer, in the menu's order.
pub const POWER_ACTIONS: &[PowerAction] = &[
    PowerAction {
        id: "logoff",
        label: "Log off",
    },
    PowerAction {
        id: "lock",
        label: "Lock",
    },
    PowerAction {
        id: "separator",
        label: "",
    },
    PowerAction {
        id: "restart",
        label: "Restart",
    },
    PowerAction {
        id: "sleep",
        label: "Sleep",
    },
    PowerAction {
        id: "hibernate",
        label: "Hibernate",
    },
    PowerAction {
        id: "shutdown",
        label: "Shut down",
    },
];

/// The action of an id, if there is one.
pub fn power_action(id: &str) -> Option<&'static PowerAction> {
    POWER_ACTIONS
        .iter()
        .find(|a| a.id == id && a.id != "separator")
}

/// What the power button itself runs, by id: the Control Panel's choice, else "shutdown".
/// Reading it tracks the settings.
pub fn power_button_action() -> &'static str {
    let chosen = settings().with(|s| s.power_button_action.clone());
    power_action(&chosen).map_or("shutdown", |a| a.id)
}

/// What the Shut Down dialog selects, by id: the Control Panel's choice, else "logoff".
/// Reading it tracks the settings.
pub fn shutdown_dialog_action() -> &'static str {
    let chosen = settings().with(|s| s.shutdown_dialog_action.clone());
    power_action(&chosen).map_or("logoff", |a| a.id)
}

/// `text` quoted for `sh`.
pub fn quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', r"'\''"))
}

pub use crate::account::login_name;

/// Runs a power action, by id. Sleep and hibernate lock first when the Control Panel says so,
/// so that the machine wakes up locked, as Windows does.
pub fn run_power(id: &str) {
    let logind = session();
    let by_logind = |action: SessionAction| untrack(|| logind.available() && logind.can(action));
    match id {
        "lock" => crate::lock::lock(),
        "logoff" => {
            let command = match std::env::var("XDG_SESSION_ID")
                .ok()
                .filter(|s| !s.is_empty())
            {
                Some(id) => format!("loginctl terminate-session {}", quoted(&id)),
                None => format!("loginctl terminate-user {}", quoted(&login_name())),
            };
            crate::programs::run(&command);
        }
        "restart" => {
            if by_logind(SessionAction::Reboot) {
                let _ = logind.reboot();
            } else {
                crate::programs::run("systemctl reboot");
            }
        }
        "shutdown" => {
            if by_logind(SessionAction::PowerOff) {
                let _ = logind.power_off();
            } else {
                crate::programs::run("systemctl poweroff");
            }
        }
        "sleep" | "hibernate" => {
            if untrack(|| settings().with(|s| s.lock_on_suspend)) {
                crate::lock::lock();
            }
            let (action, command) = if id == "sleep" {
                (SessionAction::Suspend, "systemctl suspend")
            } else {
                (SessionAction::Hibernate, "systemctl hibernate")
            };
            if by_logind(action) {
                let _ = if id == "sleep" {
                    logind.suspend()
                } else {
                    logind.hibernate()
                };
            } else {
                crate::programs::run(command);
            }
        }
        _ => {}
    }
}
