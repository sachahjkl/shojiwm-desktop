//! The Action Center (QML `Maintenance`, `ActionCenterIcon`, `ActionCenterPanel`): what
//! Windows 7's Action Center reports, for this system: the system's and the user's services
//! that failed (systemd), and the disks almost full. It looks again every minute and whenever
//! its panel opens.

use super::{
    Panel, badge, bar_cell, bar_picture, body_text, collect, every, icon_width, is_open, one_line,
    panel_popup, shell_quote, small_text, toggle,
};
use crate::paths::asset;
use crate::theme::{Align, item_radius, scheme, theme};
use crate::ui::{
    MenuLine, bar_menu_options, cp_link, dark_look, popup_frame, popup_menu, section_title,
};
use anti_webui::prelude::*;
use std::cell::Cell;
use std::time::Duration;

/// A disk this full or more is reported, and this full or more is an error.
const FULL_PERCENT: u32 = 90;
const ERROR_PERCENT: u32 = 97;
/// How often the checks run.
const REFRESH: Duration = Duration::from_secs(60);
/// How long after a message is cleared the checks run again.
const AFTER_RESET: Duration = Duration::from_millis(800);
/// The tint and the outline of a message: red for an error, yellow for a warning, stronger on
/// dark colours (QML).
const ERROR_TINT_DARK: Color = rgba(0xd9302a33);
const ERROR_TINT_LIGHT: Color = rgba(0xd9302a1f);
const WARNING_TINT_DARK: Color = rgba(0xf2c33533);
const WARNING_TINT_LIGHT: Color = rgba(0xf2c33526);
const ERROR_OUTLINE: Color = rgba(0xd9302a99);
const WARNING_OUTLINE: Color = rgba(0xc9a02099);

/// The failed services of the system and of the user, then the use of each disk.
const CHECKS: &str = "echo '#system'; systemctl --failed --no-legend --plain 2>/dev/null | awk '{print $1}'; \
echo '#user'; systemctl --user --failed --no-legend --plain 2>/dev/null | awk '{print $1}'; \
echo '#disks'; df -P -x tmpfs -x devtmpfs -x efivarfs -x overlay -x squashfs 2>/dev/null \
| awk 'NR > 1 {sub(\"%\", \"\", $5); print $5, $6}'";

/// A problem the system has now.
#[derive(Clone, Debug, PartialEq)]
struct Issue {
    error: bool,
    title: String,
    detail: String,
    /// The failed unit, or "".
    unit: String,
    /// The unit is the user's.
    user: bool,
    /// The full disk's mount point, or "".
    mount: String,
}

thread_local! {
    static ISSUES: Cell<Option<Signal<Vec<Issue>>>> = const { Cell::new(None) };
}

/// The problems, errors first. Reading them tracks them.
fn issues_signal() -> Signal<Vec<Issue>> {
    ISSUES.with(|cell| {
        cell.get().unwrap_or_else(|| {
            let (s, _) = create_root(|| signal(Vec::new()));
            cell.set(Some(s));
            s
        })
    })
}

fn has_errors() -> bool {
    issues_signal().with(|list| list.iter().any(|i| i.error))
}

fn parse(lines: &[String]) -> Vec<Issue> {
    let mut section = "";
    let mut list = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        if let Some(name) = line.strip_prefix('#') {
            section = match name {
                "system" => "system",
                "user" => "user",
                "disks" => "disks",
                _ => "",
            };
            continue;
        }
        match section {
            "system" | "user" => {
                let user = section == "user";
                let whose = if user {
                    "A service of yours"
                } else {
                    "A system service"
                };
                list.push(Issue {
                    error: true,
                    title: format!("{} failed", line.trim_end_matches(".service")),
                    detail: format!("{whose} stopped with an error ({line})."),
                    unit: line.clone(),
                    user,
                    mount: String::new(),
                });
            }
            "disks" => {
                let Some((used, mount)) = line.split_once(' ') else {
                    continue;
                };
                let Ok(used) = used.parse::<u32>() else {
                    continue;
                };
                if used >= FULL_PERCENT {
                    list.push(Issue {
                        error: used >= ERROR_PERCENT,
                        title: format!("Disk almost full: {mount}"),
                        detail: format!("{used} % of its space is used."),
                        unit: String::new(),
                        user: false,
                        mount: mount.to_owned(),
                    });
                }
            }
            _ => {}
        }
    }
    // Stable: errors first, each kind in the order it came.
    list.sort_by_key(|i| !i.error);
    list
}

/// Runs the checks again.
fn refresh() {
    let issues = issues_signal();
    collect("sh", ["-c", CHECKS], move |lines| issues.set(parse(&lines)));
}

/// Clears a failed service of the user, so that it is not reported again.
fn reset_unit(issue: &Issue) {
    let scope = if issue.user { "--user " } else { "" };
    crate::programs::run_helper(&format!(
        "systemctl {scope}reset-failed {}",
        shell_quote(&issue.unit)
    ));
    set_timeout(AFTER_RESET, refresh);
}

/// Opens the full disk in the file manager.
fn open_mount(issue: &Issue) {
    let manager = crate::programs::program("fileManager");
    crate::programs::run(&format!("{manager} {}", shell_quote(&issue.mount)));
}

/// Runs the checks now and every minute.
pub fn install() {
    refresh();
    every(REFRESH, refresh);
}

/// The flag of the notification area: a red mark when something failed, a yellow one for a
/// warning. A click opens the panel, a right click its menu.
pub fn icon(output_name: &str) -> Div {
    let name = output_name.to_owned();
    let anchor = node_ref();
    let menu = signal(false);
    let issues = issues_signal();
    let active = {
        let name = name.clone();
        move || is_open(&name, Panel::ActionCenter)
    };
    let opener = name.clone();
    let lines_name = name.clone();
    let picture = bar_picture(|| asset("action-center.svg"), 2.0).child(badge(
        || {
            asset(if has_errors() {
                "network-badge-error.svg"
            } else {
                "network-badge-warning.svg"
            })
        },
        move || issues.with(|l| !l.is_empty()),
    ));
    bar_cell(icon_width, 2.0, active)
        .node_ref(anchor)
        .child(picture)
        .on_click(move |_| toggle(&opener, Panel::ActionCenter))
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
                    MenuLine::item("Open Action Center", move || {
                        super::open(&name, Panel::ActionCenter);
                    }),
                    MenuLine::separator(),
                    MenuLine::item("Control Panel", || crate::control_panel::open_page("home")),
                    MenuLine::item("Customize notification icons", || {
                        crate::control_panel::open_page("notifications");
                    }),
                ]
            },
        ))
        .child(panel_popup(
            &name,
            Panel::ActionCenter,
            Align::Center,
            panel,
        ))
}

/// The Action Center panel: the problems the system has now, each with what can be done about
/// it, or a word that all is well.
fn panel() -> Div {
    untrack(refresh);
    let issues = issues_signal();
    let title = section_title(move || {
        let n = issues.with(Vec::len);
        match n {
            0 => "Action Center".to_owned(),
            1 => "1 message".to_owned(),
            n => format!("{n} messages"),
        }
    })
    .dyn_style(|s| s.text_size(theme().font_size));
    let all_well = small_text(
        "No current issues detected: every service runs, and the disks have room.",
        || scheme().text,
    )
    .dyn_style(move |s| {
        if issues.with(Vec::is_empty) {
            s
        } else {
            s.hidden()
        }
    });
    let list = dyn_view(move || {
        div()
            .flex()
            .flex_col()
            .gap(8.0)
            .children(issues.get().into_iter().map(issue_card).collect::<Vec<_>>())
    });
    popup_frame().w(320.0).child(
        div()
            .flex()
            .flex_col()
            .gap(8.0)
            .p(10.0)
            .child(title)
            .child(all_well)
            .child(list),
    )
}

/// A problem: its mark, its title, what happened, and what can be done.
fn issue_card(issue: Issue) -> Div {
    let error = issue.error;
    let mark = if error {
        "network-badge-error.svg"
    } else {
        "network-badge-warning.svg"
    };
    let mut actions = div().flex().flex_row().items_center().gap(12.0);
    if !issue.unit.is_empty() && issue.user {
        let cleared = issue.clone();
        actions = actions.child(cp_link("Clear this message", false, move || {
            reset_unit(&cleared)
        }));
    }
    if !issue.unit.is_empty() && !issue.user {
        actions = actions.child(
            text(format!("journalctl -u {}", issue.unit)).dyn_style(|s| {
                s.font_family(FontFamily("monospace".into()))
                    .text_size(theme().font_size_small)
                    .text_color(scheme().text_muted)
            }),
        );
    }
    if !issue.mount.is_empty() {
        let disk = issue.clone();
        actions = actions.child(cp_link("Open the disk", false, move || open_mount(&disk)));
    }
    div()
        .flex()
        .flex_row()
        .items_start()
        .gap(8.0)
        .pl(8.0)
        .pr(8.0)
        .pt(7.0)
        .pb(7.0)
        .border(1.0)
        .dyn_style(move |s| {
            let dark = dark_look();
            let tint = match (error, dark) {
                (true, true) => ERROR_TINT_DARK,
                (true, false) => ERROR_TINT_LIGHT,
                (false, true) => WARNING_TINT_DARK,
                (false, false) => WARNING_TINT_LIGHT,
            };
            s.rounded(item_radius()).bg(tint).border_color(if error {
                ERROR_OUTLINE
            } else {
                WARNING_OUTLINE
            })
        })
        .child(
            img(asset(mark))
                .flex_none()
                .size(16.0)
                .mt(1.0)
                .object_fit(ObjectFit::Contain),
        )
        .child(
            div()
                .flex_1()
                .min_w(0.0)
                .flex()
                .flex_col()
                .gap(3.0)
                .child(one_line(
                    body_text(issue.title.clone(), || scheme().text).font_weight(FontWeight::BOLD),
                ))
                .child(small_text(issue.detail.clone(), || scheme().text_muted))
                .child(actions),
        )
}
