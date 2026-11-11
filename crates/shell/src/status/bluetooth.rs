//! Bluetooth (QML `BluetoothIcon`, `BluetoothPopup`), through BlueZ: the icon of the
//! notification area, shown while there is an adapter, and the panel it opens, in the style of
//! the network panel: the adapter and its switch at the top, then the paired devices (the
//! connected ones first, with their battery), then the devices in reach, found while the panel
//! is open. A click on a device unfolds it with Connect or Disconnect, Pair, and Remove. The
//! link at the bottom opens the Bluetooth settings (blueman-manager, whose agent asks for PINs).

use super::{
    Panel, bar_cell, bar_picture, body_text, divider, footer, icon_or, icon_width, is_open,
    lit_row_style, one_line, panel_link, panel_popup, small_text, toggle,
};
use crate::paths::asset;
use crate::theme::{Align, scheme};
use crate::ui::{
    MenuLine, aero_scroll, bar_menu_options, cp_button, popup_frame, popup_menu, section_title,
    shell_icon,
};
use anti_webui::prelude::*;
use anti_webui::shell::BluetoothDevice;
use anti_webui::shell::prelude::*;
use std::collections::HashMap;

/// The Bluetooth settings, whose agent asks for the PINs of new devices.
const SETTINGS: &str = "blueman-manager";
/// The longest the list of devices grows before it scrolls.
const LIST_HEIGHT: f32 = 300.0;

fn powered() -> bool {
    bluetooth().adapter().is_some_and(|a| a.powered)
}

fn open_settings() {
    crate::programs::run(SETTINGS);
}

/// The Bluetooth icon of the notification area: a rune on blue glass, grey while Bluetooth is
/// off. It shows only when there is an adapter. A click opens the panel, a right click its menu.
pub fn icon(output_name: &str) -> Div {
    let name = output_name.to_owned();
    let anchor = node_ref();
    let menu = signal(false);
    let present = memo(|| bluetooth().adapter().is_some());
    let active = {
        let name = name.clone();
        move || is_open(&name, Panel::Bluetooth)
    };
    let opener = name.clone();
    let lines_name = name.clone();
    bar_cell(icon_width, 2.0, active)
        .node_ref(anchor)
        .dyn_style(move |s| if present.get() { s } else { s.hidden() })
        .child(bar_picture(
            || asset(if powered() { "bluetooth.svg" } else { "bluetooth-off.svg" }),
            0.0,
        ))
        .on_click(move |_| toggle(&opener, Panel::Bluetooth))
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
                    MenuLine::item("Open the Bluetooth panel", move || {
                        super::open(&name, Panel::Bluetooth);
                    }),
                    MenuLine::item("Bluetooth settings", open_settings),
                ]
            },
        ))
        .child(panel_popup(&name, Panel::Bluetooth, Align::Center, panel))
}

/// The adapter's devices: the paired ones (connected first, then by name), then the others
/// found nearby, by name.
fn devices() -> Vec<BluetoothDevice> {
    let by_name = |a: &BluetoothDevice, b: &BluetoothDevice| {
        a.name.to_lowercase().cmp(&b.name.to_lowercase())
    };
    let (mut paired, mut nearby): (Vec<_>, Vec<_>) =
        bluetooth().devices().into_iter().partition(|d| d.paired);
    paired.sort_by(|a, b| b.connected.cmp(&a.connected).then_with(|| by_name(a, b)));
    nearby.sort_by(by_name);
    paired.extend(nearby);
    paired
}

/// What the panel keeps while it is open: the unfolded device, and what is being done to each
/// device, by address ("Pairing…", "Connecting…", "Disconnecting…").
#[derive(Clone, Copy)]
struct PanelState {
    unfolded: Signal<String>,
    working: Signal<HashMap<String, &'static str>>,
}

impl PanelState {
    /// Runs `task` on `device`, showing `doing` until it ends.
    fn work(
        self,
        device: &BluetoothDevice,
        doing: &'static str,
        task: Task<Result<(), ServiceError>>,
    ) {
        let address = device.address.clone();
        self.working.update(|w| {
            w.insert(address.clone(), doing);
        });
        task.then(move |_| {
            if self.working.is_alive() {
                self.working.update(|w| {
                    w.remove(&address);
                });
            }
        });
    }
}

/// What a device is doing or is: pairing, connected with its battery, paired, …
fn status_of(device: &BluetoothDevice, working: Option<&'static str>) -> String {
    if let Some(doing) = working {
        return doing.to_owned();
    }
    if device.connected {
        return match device.battery {
            Some(level) => format!("Connected \u{00B7} battery {level} %"),
            None => "Connected".into(),
        };
    }
    if device.paired { "Paired".into() } else { "Not paired".into() }
}

/// The Bluetooth panel. It looks for devices from when it opens to when it closes, as
/// Windows does.
fn panel() -> Div {
    let state = PanelState {
        unfolded: signal(String::new()),
        working: signal(HashMap::new()),
    };
    if untrack(powered) {
        let _ = bluetooth().set_discovering(true);
    }
    on_cleanup(|| {
        if untrack(powered) {
            let _ = bluetooth().set_discovering(false);
        }
    });
    let on = memo(powered);
    let listed = memo(devices);
    let paired_count = memo(move || listed.with(|d| d.iter().filter(|d| d.paired).count()));
    let adapter = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(10.0)
        .h(34.0)
        .child(
            img(move || asset(if on.get() { "bluetooth.svg" } else { "bluetooth-off.svg" }))
                .size(28.0)
                .object_fit(ObjectFit::Contain),
        )
        .child(
            div()
                .flex_1()
                .min_w(0.0)
                .flex()
                .flex_col()
                .child(
                    one_line(
                        body_text(
                            || bluetooth().adapter().map_or("Bluetooth".into(), |a| a.name),
                            || scheme().text,
                        )
                        .font_weight(FontWeight::BOLD),
                    )
                    .max_w(150.0),
                )
                .child(small_text(
                    move || {
                        if !on.get() {
                            "Bluetooth is off"
                        } else if bluetooth().adapter().is_some_and(|a| a.discovering) {
                            "Looking for devices\u{2026}"
                        } else {
                            "On"
                        }
                    },
                    || scheme().text_muted,
                )),
        )
        .child(
            panel_link(
                move || if on.get() { "Turn off" } else { "Turn on" },
                true,
                move || {
                    let turn_on = !untrack(|| on.get());
                    bluetooth().set_powered(turn_on).then(move |done| {
                        if turn_on && done.is_ok() {
                            let _ = bluetooth().set_discovering(true);
                        }
                    });
                },
            )
            .dyn_style(|s| if bluetooth().adapter().is_some() { s } else { s.hidden() }),
        );
    let heading = section_title(move || {
        if listed.with(Vec::is_empty) {
            "No devices found yet."
        } else if paired_count.get() > 0 {
            "Devices"
        } else {
            "Devices in reach"
        }
    })
    .dyn_style(move |s| if on.get() { s } else { s.hidden() });
    let list = div().flex().flex_col().gap(1.0).child(for_each(
        move || listed.get(),
        |d: &BluetoothDevice| d.address.clone(),
        move |d: BluetoothDevice| device_row(state, paired_count, d.address),
    ));
    popup_frame()
        .w(290.0)
        .child(
            div()
                .flex()
                .flex_col()
                .gap(8.0)
                .p(10.0)
                .pb(14.0)
                .child(adapter)
                .child(divider().dyn_style(move |s| if on.get() { s } else { s.hidden() }))
                .child(heading)
                .child(aero_scroll(list).max_h(LIST_HEIGHT).dyn_style(move |s| {
                    if on.get() && !listed.with(Vec::is_empty) { s } else { s.hidden() }
                }))
                .child(
                    small_text("No Bluetooth adapter.", || scheme().text_muted).dyn_style(|s| {
                        if bluetooth().adapter().is_some() { s.hidden() } else { s }
                    }),
                ),
        )
        .child(footer(
            28.0,
            true,
            panel_link("Open Bluetooth Settings", false, || {
                open_settings();
                super::close(Panel::Bluetooth);
            }),
        ))
}

/// A device of the list, by its address: its icon, name and state; unfolded, its buttons. The
/// first device in reach after the paired ones has a heading above it.
fn device_row(state: PanelState, paired_count: Memo<usize>, address: String) -> Div {
    let ui = Interaction::new();
    let found = {
        let address = address.clone();
        memo(move || bluetooth().devices().into_iter().find(|d| d.address == address))
    };
    let index = {
        let address = address.clone();
        memo(move || devices().iter().position(|d| d.address == address))
    };
    let first_nearby = memo(move || {
        let paired = paired_count.get();
        paired > 0
            && found.with(|d| d.as_ref().is_some_and(|d| !d.paired))
            && index.get() == Some(paired)
    });
    let unfolded = {
        let address = address.clone();
        memo(move || state.unfolded.with(|u| *u == address))
    };
    let working = {
        let address = address.clone();
        memo(move || state.working.with(|w| w.get(&address).copied()))
    };
    let flag = move |f: fn(&BluetoothDevice) -> bool| {
        memo(move || found.with(|d| d.as_ref().is_some_and(f)))
    };
    let paired = flag(|d| d.paired);
    let connected = flag(|d| d.connected);
    let pairing = memo(move || working.get() == Some("Pairing\u{2026}"));
    let shown = move |on: Memo<bool>| move |s: Style| if on.get() { s } else { s.hidden() };
    let device_icon = memo(move || {
        found.with(|d| icon_or(d.as_ref().map_or("", |d| d.icon.as_str()), "bluetooth"))
    });
    let card_head = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(8.0)
        .h(38.0)
        .pl(6.0)
        .pr(8.0)
        .child(shell_icon(move || device_icon.get(), 28.0))
        .child(
            div()
                .flex_1()
                .min_w(0.0)
                .flex()
                .flex_col()
                .child(one_line(
                    body_text(
                        move || found.with(|d| d.as_ref().map_or(String::new(), |d| d.name.clone())),
                        || scheme().text,
                    )
                    .dyn_style(move |s| {
                        if connected.get() { s.font_weight(FontWeight::BOLD) } else { s }
                    }),
                ))
                .child(small_text(
                    move || {
                        found.with(|d| {
                            d.as_ref().map_or(String::new(), |d| status_of(d, working.get()))
                        })
                    },
                    || scheme().text_muted,
                )),
        );
    let act = move |doing: &'static str, f: fn(&BluetoothDevice) -> Task<Result<(), ServiceError>>| {
        move || {
            if let Some(d) = untrack(|| found.get()) {
                state.work(&d, doing, f(&d));
            }
        }
    };
    let remove = move || {
        state.unfolded.set(String::new());
        if let Some(d) = untrack(|| found.get()) {
            let _ = d.forget();
        }
    };
    let pair = move || {
        let Some(d) = untrack(|| found.get()) else { return };
        if untrack(|| pairing.get()) {
            // BlueZ's CancelPairing, which the service does not offer.
            crate::programs::run(&format!(
                "bluetoothctl cancel-pairing {}",
                super::shell_quote(&d.address)
            ));
        } else {
            // Trusted, so that it connects again by itself.
            let _ = d.set_trusted(true);
            state.work(&d, "Pairing\u{2026}", d.pair());
        }
    };
    let connect = move || {
        if untrack(|| connected.get()) {
            act("Disconnecting\u{2026}", |d| d.disconnect())();
        } else {
            act("Connecting\u{2026}", |d| d.connect())();
        }
    };
    let buttons = div()
        .flex()
        .flex_row()
        .justify_end()
        .gap(6.0)
        .pr(8.0)
        .pb(8.0)
        .on_click(|e| e.stop_propagation())
        .dyn_style(shown(unfolded))
        .child(cp_button("Remove", remove).dyn_style(shown(paired)))
        .child(
            cp_button(move || if pairing.get() { "Cancel" } else { "Pair" }, pair)
                .dyn_style(move |s| if paired.get() { s.hidden() } else { s }),
        )
        .child(
            cp_button(move || if connected.get() { "Disconnect" } else { "Connect" }, connect)
                .dyn_style(shown(paired)),
        );
    let fold = address.clone();
    let card = div()
        .flex()
        .flex_col()
        .interaction(ui)
        .on_click(move |_| {
            let open = state.unfolded.with_untracked(|u| *u == fold);
            state.unfolded.set(if open { String::new() } else { fold.clone() });
        })
        .dyn_style(move |s| lit_row_style(s, unfolded.get(), ui))
        .child(card_head)
        .child(buttons);
    div()
        .flex()
        .flex_col()
        .child(
            div()
                .flex()
                .items_center()
                .h(22.0)
                .pt(4.0)
                .dyn_style(shown(first_nearby))
                .child(section_title("Devices in reach")),
        )
        .child(card)
}
