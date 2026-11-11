//! System: what Windows 7's System page shows, read from the system (the edition, the
//! processor, the memory, the system type, the computer's name, how long it has been on), and
//! what this desktop is made of.

use super::common::{output_of, page_column, tinted_text};
use crate::control_panel::{heading, sub_heading};
use crate::theme::scheme;
use anti_webui::prelude::*;

/// The facts, one a line: the system's name, the processor, its threads, the memory in kB, the
/// machine, the computer's name, the kernel, the seconds since it started, the login name and
/// the full name.
const FACTS_SCRIPT: &str = ". /etc/os-release 2>/dev/null; echo \"$PRETTY_NAME\"; \
     grep -m1 'model name' /proc/cpuinfo | cut -d: -f2- | sed 's/^ *//'; \
     nproc; awk '/MemTotal/ {print $2}' /proc/meminfo; uname -m; \
     cat /proc/sys/kernel/hostname; uname -r; cut -d' ' -f1 /proc/uptime; \
     echo \"$USER\"; getent passwd \"$USER\" | cut -d: -f5 | cut -d, -f1";

/// The width of the labels' column, so the values line up through every section.
const LABEL_WIDTH: f32 = 190.0;
/// The system's logo beside its edition, as Windows 7 shows its own.
const LOGO_SIZE: f32 = 96.0;

/// The facts as the script printed them, by line.
#[derive(Clone, Debug, Default, PartialEq)]
struct Facts(Vec<String>);

impl Facts {
    fn line(&self, i: usize) -> String {
        self.0
            .get(i)
            .map(|l| l.trim().to_owned())
            .unwrap_or_default()
    }

    fn os(&self) -> String {
        self.line(0)
    }

    fn kernel(&self) -> String {
        let release = self.line(6);
        if release.is_empty() {
            release
        } else {
            format!("Linux {release}")
        }
    }

    fn processor(&self) -> String {
        let threads = self.line(2);
        if threads.is_empty() {
            self.line(1)
        } else {
            format!("{}  ({threads} threads)", self.line(1))
        }
    }

    fn memory(&self) -> String {
        self.line(3)
            .parse::<f64>()
            .map(|kb| format!("{:.1} GB", kb / 1_048_576.0))
            .unwrap_or_default()
    }

    fn system_type(&self) -> String {
        let machine = self.line(4);
        if machine == "x86_64" {
            "64-bit Operating System (x86_64)".to_owned()
        } else {
            machine
        }
    }

    fn computer_name(&self) -> String {
        self.line(5)
    }

    fn uptime(&self) -> String {
        let seconds = self.line(7).parse::<f64>().unwrap_or(0.0) as u64;
        let days = seconds / 86_400;
        let hours = seconds % 86_400 / 3_600;
        let minutes = seconds % 3_600 / 60;
        let days = if days > 0 {
            format!("{days} d ")
        } else {
            String::new()
        };
        format!("{days}{hours} h {minutes} min")
    }

    /// The full name and the login name.
    fn user(&self) -> String {
        let login = self.line(8);
        let full = self.line(9);
        let name = if full.is_empty() { login.clone() } else { full };
        format!("{name} ({login})")
    }
}

pub fn page() -> Div {
    let facts = signal(Facts::default());
    task(|| {
        Facts(
            output_of("sh", &["-c", FACTS_SCRIPT])
                .lines()
                .map(String::from)
                .collect(),
        )
    })
    .then(move |read| facts.set(read));
    let fact = move |label: &'static str, value: fn(&Facts) -> String| {
        info_row(label, move || facts.with(value))
    };
    let settings_file = crate::settings::path().display().to_string();
    page_column(10.0)
        .child(heading("View basic information about your computer"))
        .child(sub_heading("Edition"))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .w_full()
                .min_h(LOGO_SIZE)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(6.0)
                        .child(fact("Operating system", Facts::os))
                        .child(fact("Kernel", Facts::kernel)),
                )
                .child(
                    img(crate::paths::asset("nixos-snowflake.svg"))
                        .w(LOGO_SIZE)
                        .h(LOGO_SIZE)
                        .mr(24.0)
                        .object_fit(ObjectFit::Contain),
                ),
        )
        .child(sub_heading("System"))
        .child(fact("Processor", Facts::processor))
        .child(fact("Installed memory (RAM)", Facts::memory))
        .child(fact("System type", Facts::system_type))
        .child(fact("Up for", Facts::uptime))
        .child(sub_heading("Computer name"))
        .child(fact("Computer name", Facts::computer_name))
        .child(sub_heading("This desktop"))
        .child(info_row("Desktop", || "shojiwm-win7".to_owned()))
        .child(info_row("Compositor", || "ShojiWM".to_owned()))
        .child(info_row("Shell", || "shojiwm-shell (AntiWebUI)".to_owned()))
        .child(info_row("Launcher", || {
            "The Start menu's search (Super+Shift+D)".to_owned()
        }))
        .child(fact("User", Facts::user))
        .child(info_row("Settings file", move || settings_file.clone()))
}

/// One line of facts: the label, muted, in a column of one width, then the value.
fn info_row(label: &'static str, value: impl Fn() -> String + 'static) -> Div {
    div()
        .flex()
        .flex_row()
        .items_start()
        .gap(12.0)
        .child(
            div()
                .flex_none()
                .w(LABEL_WIDTH)
                .child(tinted_text(format!("{label}:"), || scheme().text_muted)),
        )
        .child(tinted_text(value, || scheme().text))
}
