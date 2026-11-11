//! The network (QML `Network`, `NetworkIcon`, `NetworkPopup`), from NetworkManager: the icon of
//! the notification area, and the network panel it opens, as Windows 7 shows it: the connection
//! in use and its access at the top, then the wireless networks in reach with their bars. A
//! click on a network unfolds it with its Connect button, and a field for the key of a new
//! secured network. The link at the bottom opens the Network and Sharing Center of the Control
//! Panel.

use super::{
    ERROR_RED, Panel, badge, bar_cell, bar_picture, body_text, divider, footer, icon_width,
    is_open, lit_row_style, one_line, panel_link, panel_popup, small_text, toggle,
};
use crate::paths::asset;
use crate::theme::{Align, away_from_bar, field_radius, scheme, theme};
use crate::ui::{
    MenuLine, aero_scroll, bar_menu_options, cp_button, font, ink_asset, popup_frame, popup_menu,
    section_title, shell_tooltip,
};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use anti_webui::shell::{AccessPoint, NetError, Security};

/// The longest the list of networks grows before it scrolls.
const LIST_HEIGHT: f32 = 260.0;

/// The connection in use.
#[derive(Clone, Debug, PartialEq)]
enum Primary {
    /// Wi-Fi, on this network, with this signal (0 to 100).
    Wifi { ssid: String, strength: u8 },
    /// The wired connection, on this interface.
    Wired { interface: String },
}

/// The connection in use: the wired one when it is connected, as NetworkManager puts the
/// wired device's route first, else Wi-Fi.
fn primary() -> Option<Primary> {
    let net = network();
    if let Some(w) = net.wired().filter(|w| w.connected) {
        return Some(Primary::Wired {
            interface: w.interface,
        });
    }
    net.wifi().map(|ap| Primary::Wifi {
        ssid: ap.ssid,
        strength: ap.strength,
    })
}

fn primary_name(p: &Primary) -> String {
    match p {
        Primary::Wifi { ssid, .. } => ssid.clone(),
        Primary::Wired { interface } => format!("Wired connection ({interface})"),
    }
}

/// Whether the computer has a wireless device at all, on or off: a network interface with
/// a `wireless` directory in sysfs. NetworkManager's service does not tell it.
fn has_wifi() -> bool {
    std::fs::read_dir("/sys/class/net")
        .map(|dir| {
            dir.flatten()
                .any(|entry| entry.path().join("wireless").exists())
        })
        .unwrap_or(false)
}

/// 0 to 4 bars for a signal.
fn bars_of(strength: u8) -> u8 {
    match strength {
        80.. => 4,
        60.. => 3,
        40.. => 2,
        20.. => 1,
        _ => 0,
    }
}

fn secure(ap: &AccessPoint) -> bool {
    ap.security != Security::Open
}

fn set_wifi(on: bool) {
    let _ = network().set_wifi_enabled(on);
}

/// The Network and Sharing Center, where the connections are edited.
fn open_editor() {
    crate::control_panel::open_page("network");
}

/// The picture of the connection in use: bars for Wi-Fi, a screen for the wire.
fn glyph(has_wifi: bool) -> String {
    match primary() {
        Some(Primary::Wifi { strength, .. }) => format!("network-wifi-{}", bars_of(strength)),
        Some(Primary::Wired { .. }) => "network-wired".into(),
        None if !has_wifi => "network-wired".into(),
        None => "network-wifi-0".into(),
    }
}

/// The mark on the icon: "error" with no connection, "warning" without the Internet, "" else.
fn badge_name() -> &'static str {
    if primary().is_none() {
        return "error";
    }
    match network().connectivity() {
        Connectivity::Full | Connectivity::Unknown => "",
        _ => "warning",
    }
}

/// The network icon of the notification area: a screen for a wired connection, bars for a
/// wireless one, a yellow mark when the connection has no Internet and a red one when there
/// is none. A click opens the network panel, a right click its menu.
pub fn icon(output_name: &str) -> Div {
    let name = output_name.to_owned();
    let anchor = node_ref();
    let menu = signal(false);
    let wifi_device = has_wifi();
    let active = {
        let name = name.clone();
        move || is_open(&name, Panel::Network)
    };
    let opener = name.clone();
    let lines_name = name.clone();
    let picture =
        bar_picture(move || asset(&format!("{}.svg", glyph(wifi_device))), 2.0).child(badge(
            || asset(&format!("network-badge-{}.svg", badge_name())),
            || !badge_name().is_empty(),
        ));
    let cell = bar_cell(icon_width, 2.0, active)
        .node_ref(anchor)
        .child(picture)
        .on_click(move |_| toggle(&opener, Panel::Network))
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
                let on = network().wifi_enabled();
                vec![
                    MenuLine::item("Open the network panel", move || {
                        super::open(&name, Panel::Network);
                    }),
                    MenuLine::item("Wi-Fi", move || set_wifi(!on))
                        .checked(on)
                        .enabled(has_wifi()),
                    MenuLine::separator(),
                    MenuLine::item("Network connections", open_editor),
                ]
            },
        ))
        .child(panel_popup(&name, Panel::Network, Align::Center, panel));
    shell_tooltip(
        cell,
        untrack(|| away_from_bar(Align::Center)),
        || match primary() {
            Some(p) => {
                let access = if network().connectivity() == Connectivity::Full {
                    "Internet access"
                } else {
                    "No Internet access"
                };
                format!("{}\n{access}", primary_name(&p))
            }
            None => "Not connected".into(),
        },
    )
}

/// What the panel keeps while it is open: the unfolded network, the key typed for it, the
/// network being joined, and the last error.
#[derive(Clone, Copy)]
struct PanelState {
    unfolded: Signal<String>,
    key: Signal<String>,
    busy: Signal<bool>,
    error: Signal<String>,
}

impl PanelState {
    /// Joins a network: a saved one with its connection, a new one with the key typed.
    fn connect(self, ap: AccessPoint) {
        let key = self.key.get_untracked();
        let key = (secure(&ap) && !ap.known && !key.is_empty()).then_some(key);
        self.error.set(String::new());
        self.busy.set(true);
        network().connect(&ap, key).then(move |done| {
            if !self.busy.is_alive() {
                return;
            }
            self.busy.set(false);
            match done {
                Ok(()) => self.key.set(String::new()),
                Err(NetError::NeedsSecret) => {
                    self.error.set("Type the network security key.".into());
                }
                Err(e) => self.error.set(e.to_string()),
            }
        });
    }

    fn disconnect(self) {
        self.error.set(String::new());
        self.busy.set(true);
        network().disconnect().then(move |done| {
            if !self.busy.is_alive() {
                return;
            }
            self.busy.set(false);
            if let Err(e) = done {
                self.error.set(e.to_string());
            }
        });
    }
}

/// The network panel.
fn panel() -> Div {
    let state = PanelState {
        unfolded: signal(String::new()),
        key: signal(String::new()),
        busy: signal(false),
        error: signal(String::new()),
    };
    let wifi_device = has_wifi();
    if untrack(|| network().wifi_enabled()) {
        let _ = network().scan();
    }
    let wifi_on = memo(|| network().wifi_enabled());
    let current = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(10.0)
        .child(
            img(move || {
                let picture = match primary() {
                    Some(Primary::Wifi { strength, .. }) => {
                        format!("network-wifi-{}.svg", bars_of(strength))
                    }
                    _ => "network-wired.svg".into(),
                };
                ink_asset(&picture)
            })
            .size(28.0)
            .object_fit(ObjectFit::Contain),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .child(
                    one_line(
                        body_text(
                            || primary().map_or("Not connected".into(), |p| primary_name(&p)),
                            || scheme().text,
                        )
                        .font_weight(FontWeight::BOLD),
                    )
                    .w(210.0),
                )
                .child(small_text(
                    || match (primary(), network().connectivity()) {
                        (None, _) => "No connections are available",
                        (_, Connectivity::Full) => "Internet access",
                        (_, Connectivity::Portal) => "Sign-in required",
                        (_, Connectivity::Unknown) => "Connected",
                        _ => "No Internet access",
                    },
                    || scheme().text_muted,
                )),
        );
    let wireless_heading = div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .h(18.0)
        .child(section_title("Wireless Network Connection"))
        .child(panel_link(
            move || if wifi_on.get() { "Turn off" } else { "Turn on" },
            true,
            move || set_wifi(!untrack(|| wifi_on.get())),
        ));
    let list = div().flex().flex_col().gap(1.0).child(for_each(
        || network().access_points(),
        |ap: &AccessPoint| ap.ssid.clone(),
        move |ap: AccessPoint| network_row(state, ap.ssid),
    ));
    let wireless = div()
        .flex()
        .flex_col()
        .gap(8.0)
        .child(divider())
        .child(wireless_heading)
        .child(
            aero_scroll(list)
                .max_h(LIST_HEIGHT)
                .dyn_style(move |s| if wifi_on.get() { s } else { s.hidden() }),
        )
        .child(
            small_text("Wireless is turned off.", || scheme().text_muted)
                .dyn_style(move |s| if wifi_on.get() { s.hidden() } else { s }),
        );
    popup_frame()
        .w(290.0)
        .child(
            div()
                .flex()
                .flex_col()
                .gap(8.0)
                .p(10.0)
                .pb(14.0)
                .child(section_title("Currently connected to:"))
                .child(current)
                .children(wifi_device.then_some(wireless)),
        )
        .child(footer(
            28.0,
            true,
            panel_link("Open Network Connections", false, || {
                open_editor();
                super::close(Panel::Network);
            }),
        ))
}

/// A wireless network of the list, by its name: its name and bars; unfolded, what it is and
/// its button.
fn network_row(state: PanelState, ssid: String) -> Div {
    let ui = Interaction::new();
    let found = {
        let ssid = ssid.clone();
        memo(move || {
            network()
                .access_points()
                .into_iter()
                .find(|ap| ap.ssid == ssid)
        })
    };
    let unfolded = {
        let ssid = ssid.clone();
        memo(move || state.unfolded.with(|u| *u == ssid))
    };
    let asks_key = memo(move || {
        found.with(|ap| {
            ap.as_ref()
                .is_some_and(|ap| secure(ap) && !ap.known && !ap.active)
        })
    });
    let active = memo(move || found.with(|ap| ap.as_ref().is_some_and(|ap| ap.active)));
    let fold = ssid.clone();
    let head = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(6.0)
        .h(30.0)
        .px(8.0)
        .child(
            one_line(
                body_text(ssid.clone(), || scheme().text).dyn_style(move |s| {
                    if active.get() {
                        s.font_weight(FontWeight::BOLD)
                    } else {
                        s
                    }
                }),
            )
            .flex_1(),
        )
        .child(text("\u{1F512}").text_size(10.0).dyn_style(move |s| {
            let s = s.text_color(scheme().text_muted);
            if found.with(|ap| ap.as_ref().is_some_and(secure)) {
                s
            } else {
                s.hidden()
            }
        }))
        .child(
            img(move || {
                let strength = found.with(|ap| ap.as_ref().map_or(0, |ap| ap.strength));
                ink_asset(&format!("network-wifi-{}.svg", bars_of(strength)))
            })
            .size(16.0)
            .object_fit(ObjectFit::Contain),
        );
    let what = small_text(
        move || {
            found.with(|ap| match ap {
                Some(ap) if ap.active => "Connected",
                Some(ap) if ap.known => "Saved network",
                Some(ap) if secure(ap) => "Secured",
                _ => "Open network",
            })
        },
        || scheme().text_muted,
    );
    let key_field = div()
        .flex()
        .flex_row()
        .items_center()
        .h(24.0)
        .px(4.0)
        .border(1.0)
        .dyn_style(move |s| {
            let c = scheme();
            let s = s
                .rounded(field_radius())
                .bg(c.field)
                .border_color(c.field_border)
                .font_family(font())
                .text_size(theme().font_size)
                .text_color(c.text);
            if asks_key.get() { s } else { s.hidden() }
        })
        .on_click(|e| e.stop_propagation())
        .child(
            text_input()
                .value(state.key)
                .placeholder("Security key")
                .purpose(ImePurpose::Password)
                .on_submit(move |_| {
                    if let Some(ap) = untrack(|| found.get()) {
                        state.connect(ap);
                    }
                })
                .flex_1(),
        );
    let error = small_text(move || state.error.get(), || ERROR_RED).dyn_style(move |s| {
        if state.error.with(String::is_empty) {
            s.hidden()
        } else {
            s
        }
    });
    let button = div()
        .flex()
        .flex_row()
        .justify_end()
        .on_click(|e| e.stop_propagation())
        .child(cp_button(
            move || {
                if state.busy.get() {
                    "Connecting\u{2026}"
                } else if active.get() {
                    "Disconnect"
                } else {
                    "Connect"
                }
            },
            move || {
                if untrack(|| active.get()) {
                    state.disconnect();
                } else if let Some(ap) = untrack(|| found.get()) {
                    state.connect(ap);
                }
            },
        ));
    let details = div()
        .flex()
        .flex_col()
        .gap(6.0)
        .px(8.0)
        .pb(8.0)
        .dyn_style(move |s| if unfolded.get() { s } else { s.hidden() })
        .child(what)
        .child(key_field)
        .child(error)
        .child(button);
    div()
        .flex()
        .flex_col()
        .interaction(ui)
        .on_click(move |_| {
            let open = state.unfolded.with_untracked(|u| *u == fold);
            state.error.set(String::new());
            state.key.set(String::new());
            state
                .unfolded
                .set(if open { String::new() } else { fold.clone() });
        })
        .dyn_style(move |s| lit_row_style(s, unfolded.get(), ui))
        .child(head)
        .child(details)
}
