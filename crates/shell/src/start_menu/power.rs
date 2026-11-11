//! Power: what the Start menu's power button does and what its arrow offers (Apps.qml's power
//! actions, QML `PowerMenu`), the button itself (QML `PowerPart`), and the red split button of
//! the log-on screen (QML `PowerSplitButton`).
//!
//! Restart, sleep, hibernate and shut down go through logind when it is there and allows them;
//! else they run `systemctl`, as the QML shell did. Log off ends the session with `loginctl`.
//! Lock is the shell's own lock screen.

use super::close;
use crate::paths::asset;
use crate::settings::settings;
use crate::theme::{button_radius, corner, theme};
use crate::ui::{MenuLine, font, popup_menu, vertical_gradient};
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
    PowerAction { id: "logoff", label: "Log off" },
    PowerAction { id: "lock", label: "Lock" },
    PowerAction { id: "separator", label: "" },
    PowerAction { id: "restart", label: "Restart" },
    PowerAction { id: "sleep", label: "Sleep" },
    PowerAction { id: "hibernate", label: "Hibernate" },
    PowerAction { id: "shutdown", label: "Shut down" },
];

/// The action of an id, if there is one.
pub fn power_action(id: &str) -> Option<&'static PowerAction> {
    POWER_ACTIONS.iter().find(|a| a.id == id && a.id != "separator")
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
pub(super) fn quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', r"'\''"))
}

/// The login name: `$USER`, else the home folder's name.
fn login_name() -> String {
    std::env::var("USER")
        .ok()
        .filter(|u| !u.is_empty())
        .or_else(|| {
            crate::settings::home()
                .and_then(|h| h.file_name().map(|n| n.to_string_lossy().into_owned()))
        })
        .unwrap_or_default()
}

/// Runs a power action, by id. Sleep and hibernate lock first when the Control Panel says so,
/// so that the machine wakes up locked, as Windows does.
pub fn run_power(id: &str) {
    let logind = session();
    let by_logind = |action: SessionAction| untrack(|| logind.available() && logind.can(action));
    match id {
        "lock" => crate::lock::lock(),
        "logoff" => {
            let command = match std::env::var("XDG_SESSION_ID").ok().filter(|s| !s.is_empty()) {
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
                let _ = if id == "sleep" { logind.suspend() } else { logind.hibernate() };
            } else {
                crate::programs::run(command);
            }
        }
        _ => {}
    }
}

// ----- the Start menu's power button ------------------------------------------------------------

/// The outline of the glass button (QML literal).
const GLASS_OUTLINE: Color = rgba(0x000000a6);
/// The glass's fill, top and bottom: at rest, under the pointer, pressed (QML literals).
const GLASS_REST: [Color; 2] = [rgba(0xffffff40), rgba(0xffffff1f)];
const GLASS_HOT: [Color; 2] = [rgba(0xffffff66), rgba(0xffffff38)];
const GLASS_PRESSED: [Color; 2] = [rgba(0xffffff26), rgba(0xffffff14)];
/// The light rim inside the outline: at rest, under the pointer, pressed (QML literals).
const RIM_REST: Color = rgba(0xffffff66);
const RIM_HOT: Color = rgba(0xffffff99);
const RIM_PRESSED: Color = rgba(0xffffff33);
/// The faint light line beside the seam between the two halves (QML literal).
const SEAM_LIGHT: Color = rgba(0xffffff26);
/// The label, white, set into the glass by a dark line under it (QML literals).
const LABEL: Color = Color::WHITE;
const LABEL_SHADOW: Color = rgba(0x00000099);
/// The arrow's triangle (QML literal).
const ARROW: Color = rgba(0xfffffff2);

/// The button's height, its narrowest, the room each side of its label, and the arrow's width
/// (QML).
const BUTTON_HEIGHT: f32 = 25.0;
const BUTTON_MIN_WIDTH: f32 = 56.0;
const LABEL_ROOM: f32 = 13.0;
const ARROW_WIDTH: f32 = 20.0;

/// One half of the power button: the button itself (`left`), or the arrow beside it. The two
/// halves share an edge and round only their outer corners: each is a box rounded all round,
/// wider than the half on its inner side and clipped there, so the glass, which is
/// translucent, is painted once.
fn power_part(left: bool, hot: impl Fn() -> bool + 'static, pressed: impl Fn() -> bool + 'static) -> Div {
    let hot = std::rc::Rc::new(hot);
    let pressed = std::rc::Rc::new(pressed);
    let (hot_fill, pressed_fill) = (hot.clone(), pressed.clone());
    let (hot_rim, pressed_rim) = (hot.clone(), pressed.clone());
    let overhang = move || button_radius() + 1.0;
    let shape = div()
        .absolute()
        .top(0.0)
        .bottom(0.0)
        .border(1.0)
        .border_color(GLASS_OUTLINE)
        .fill(move || {
            let f = if pressed_fill() {
                GLASS_PRESSED
            } else if hot_fill() {
                GLASS_HOT
            } else {
                GLASS_REST
            };
            vertical_gradient(&[(0.0, f[0]), (1.0, f[1])])
        })
        .dyn_style(move |s| {
            let s = s.rounded(button_radius());
            if left {
                s.left(0.0).right(-overhang())
            } else {
                s.left(-overhang()).right(0.0)
            }
        })
        // The light rim inside the outline, all round.
        .child(div().absolute().inset(0.0).border(1.0).dyn_style(move |s| {
            let rim = if pressed_rim() {
                RIM_PRESSED
            } else if hot_rim() {
                RIM_HOT
            } else {
                RIM_REST
            };
            s.rounded(corner((button_radius() - 1.0).max(0.0))).border_color(rim)
        }));
    let part = div()
        .relative()
        .flex_none()
        .h_full()
        .overflow_clip()
        .child(shape);
    if left {
        return part;
    }
    // The seam between the halves, on the arrow's left edge: the outline's line, and a faint
    // light one beside it.
    part.child(div().absolute().left(0.0).top(0.0).bottom(0.0).w(1.0).bg(GLASS_OUTLINE))
        .child(div().absolute().left(1.0).top(1.0).bottom(1.0).w(1.0).bg(SEAM_LIGHT))
}

/// The power button at the foot of the places: glass, as on Windows 7, as wide as its label and
/// a margin each side. Its arrow opens the other actions.
pub(super) fn power_button() -> Div {
    let button = Interaction::new();
    let arrow = Interaction::new();
    let menu_open = signal(false);
    let arrow_anchor = node_ref();
    let label = move || power_action(power_button_action()).map_or("", |a| a.label);
    // White, set into the glass by a dark line under it. The label alone, as Windows 7 has it;
    // Vista had the icon.
    let words = div()
        .relative()
        .whitespace_nowrap()
        .dyn_style(|s| s.font_family(font()).text_size(theme().font_size))
        .child(
            div()
                .absolute()
                .left(0.0)
                .top(1.0)
                .text_color(LABEL_SHADOW)
                .child(text(label)),
        )
        .child(div().relative().text_color(LABEL).child(text(label)));
    let main = power_part(true, move || button.hovered(), move || button.pressed())
        .min_w(BUTTON_MIN_WIDTH)
        .px(LABEL_ROOM)
        .flex()
        .items_center()
        .justify_center()
        .interaction(button)
        .on_click(|_| {
            let id = untrack(power_button_action);
            close();
            run_power(id);
        })
        .child(words);
    let side = power_part(
        false,
        move || arrow.hovered() || menu_open.get(),
        move || arrow.pressed() || menu_open.get(),
    )
    .w(ARROW_WIDTH)
    .flex()
    .items_center()
    .justify_center()
    .node_ref(arrow_anchor)
    .interaction(arrow)
    .on_click(move |_| menu_open.update(|o| *o = !*o))
    .child(
        div()
            .relative()
            .text_size(7.0)
            .text_color(ARROW)
            .child("\u{25B6}"),
    )
    .child(power_menu(menu_open, arrow_anchor));
    div()
        .absolute()
        .flex()
        .flex_row()
        .h(BUTTON_HEIGHT)
        .dyn_style(|s| {
            let pad = theme().menu_padding + 2.0;
            s.left(pad).bottom(pad)
        })
        .child(main)
        .child(side)
}

/// What the arrow beside the power button opens: every action but the one the button runs, to
/// the right of the arrow, as on Windows.
fn power_menu(open: Signal<bool>, anchor: NodeRef) -> Show {
    let options = MenuOptions::new()
        .anchor(anchor)
        .placement(Placement::RightStart)
        .gap(untrack(|| theme().popup_gap) as i32);
    popup_menu(
        move || open.get(),
        move || open.set(false),
        options,
        || {
            let button = untrack(power_button_action);
            POWER_ACTIONS
                .iter()
                .filter(|a| a.id != button)
                .map(|a| {
                    if a.id == "separator" {
                        MenuLine::separator()
                    } else {
                        let id = a.id;
                        MenuLine::item(a.label, move || {
                            close();
                            run_power(id);
                        })
                    }
                })
                .collect()
        },
    )
}

// ----- the log-on screen's power button ---------------------------------------------------------

/// The split button's size, and where its two parts meet (QML).
const SPLIT_WIDTH: f32 = 70.0;
const SPLIT_HEIGHT: f32 = 24.0;
const SPLIT_DIVIDER: f32 = 50.0;
/// The outline, and the light rim inside it (QML literals).
const SPLIT_OUTLINE: Color = rgb(0x2a0303);
const SPLIT_RIM: Color = rgba(0xffdada4d);
/// A part's fill: a pale red gloss over the top half, a deep red below that brightens at the
/// bottom; lit under the pointer, darker while pressed (QML literals). Stops at 0, 0.47, 0.5, 1.
const SPLIT_REST: [Color; 4] = [rgb(0xd98585), rgb(0xb84a4a), rgb(0x860000), rgb(0xc00808)];
const SPLIT_HOT: [Color; 4] = [rgb(0xeda0a0), rgb(0xcf5c5c), rgb(0xa30000), rgb(0xe01515)];
const SPLIT_DOWN: [Color; 4] = [rgb(0xa65454), rgb(0x8c2c2c), rgb(0x5e0000), rgb(0x8a0505)];

/// One part of the split button's fill.
fn split_fill(ui: Interaction) -> ShaderPaint {
    let f = if ui.pressed() {
        SPLIT_DOWN
    } else if ui.hovered() {
        SPLIT_HOT
    } else {
        SPLIT_REST
    };
    vertical_gradient(&[(0.0, f[0]), (0.47, f[1]), (0.5, f[2]), (1.0, f[3])])
}

/// The red power button of the Windows 7 log-on screen, and the arrow joined to its right that
/// opens the other choices: `on_power` and `on_menu` run on their clicks. One rounded piece
/// with a dark outline and a dark line between the two parts, and a white ring for the power
/// mark. To hang a menu from it, give what comes back a `node_ref`.
pub fn power_split_button(on_power: impl Fn() + 'static, on_menu: impl Fn() + 'static) -> Div {
    let main = Interaction::new();
    let arrow = Interaction::new();
    div()
        .relative()
        .flex_none()
        .w(SPLIT_WIDTH)
        .h(SPLIT_HEIGHT)
        .bg(SPLIT_OUTLINE)
        .overflow_clip()
        .select_none()
        .dyn_style(|s| s.rounded(corner(3.0)))
        .child(
            div()
                .absolute()
                .left(1.0)
                .top(1.0)
                .bottom(1.0)
                .w(SPLIT_DIVIDER - 1.0)
                .fill(move || split_fill(main))
                .dyn_style(|s| s.rounded(corner(2.0))),
        )
        .child(
            div()
                .absolute()
                .left(SPLIT_DIVIDER + 1.0)
                .right(1.0)
                .top(1.0)
                .bottom(1.0)
                .fill(move || split_fill(arrow))
                .dyn_style(|s| s.rounded(corner(2.0))),
        )
        // The light rim inside the outline.
        .child(
            div()
                .absolute()
                .inset(1.0)
                .border(1.0)
                .border_color(SPLIT_RIM)
                .pointer_events_none()
                .dyn_style(|s| s.rounded(corner(2.0))),
        )
        .child(
            div()
                .absolute()
                .left(0.0)
                .top(0.0)
                .bottom(0.0)
                .w(SPLIT_DIVIDER)
                .flex()
                .items_center()
                .justify_center()
                .interaction(main)
                .on_click(move |_| on_power())
                .child(
                    img(asset("power-ring.svg"))
                        .size(16.0)
                        .object_fit(ObjectFit::Contain),
                ),
        )
        .child(
            div()
                .absolute()
                .left(SPLIT_DIVIDER)
                .right(0.0)
                .top(0.0)
                .bottom(0.0)
                .flex()
                .items_center()
                .justify_center()
                .text_size(7.0)
                .text_color(Color::WHITE)
                .interaction(arrow)
                .on_click(move |_| on_menu())
                .child("\u{25B2}"),
        )
}
