//! Network and Sharing Center, from NetworkManager (`network()`): the map from this computer
//! to the Internet, the networks in use with their addresses, the network adapters with Wi-Fi's
//! switch, and the saved connections, which connect, disconnect, connect by themselves, are
//! forgotten and are edited here (`network_edit`). A network that hides its name is joined
//! from the link under them.

use super::common::{control_row, form_label, muted_small, page_column, tinted_text};
use super::network_edit::{hidden_network_form, properties};
use crate::control_panel::{heading, note, sub_heading};
use crate::paths::asset;
use crate::status::ERROR_RED;
use crate::theme::{item_radius, scheme, theme};
use crate::ui::{cp_button, cp_check, cp_link, font};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use anti_webui::shell::{
    ActivationState, ConnectionKind, DeviceKind, DeviceState, NetworkDevice, SavedConnection,
};

/// The side of a picture of the map, of an adapter, and of a connection.
const MAP_ICON: f32 = 48.0;
const ADAPTER_ICON: f32 = 40.0;
const ROW_ICON: f32 = 32.0;
/// The width of a line of the map between two pictures, and of a column of the map.
const MAP_LINE: f32 = 70.0;
const MAP_COLUMN: f32 = 130.0;
/// The width of the labels of a network's details.
const DETAIL_LABEL: f32 = 150.0;
/// The width of an adapter's tile.
const ADAPTER_WIDTH: f32 = 260.0;
/// The line of the map where the way is cut.
const CUT_RED: Color = rgb(0xd9302a);
/// A line of the map where the way is open, and the size of the badge on each line.
const OPEN_GREEN: Color = rgb(0x2e9d48);
const MAP_BADGE: f32 = 18.0;

/// What the page keeps while it shows: the connection whose properties are open and the one
/// asked to be forgotten (by UUID), and the last error.
#[derive(Clone, Copy)]
pub struct PageState {
    pub editing: Signal<String>,
    forgetting: Signal<String>,
    pub error: Signal<String>,
}

impl PageState {
    /// Shows what went wrong, while the page is open.
    pub fn report<E: std::fmt::Display>(self, done: Result<(), E>) {
        if !self.error.is_alive() {
            return;
        }
        match done {
            Ok(()) => self.error.set(String::new()),
            Err(e) => self.error.set(first_upper(&e.to_string())),
        }
    }
}

/// `text` with its first letter in capitals, as a sentence.
fn first_upper(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

/// The devices that a person thinks of as network adapters: wired and Wi-Fi.
fn adapters() -> Vec<NetworkDevice> {
    network()
        .devices()
        .into_iter()
        .filter(|d| d.kind != DeviceKind::Other)
        .collect()
}

/// The adapters on a network now.
fn connected() -> Vec<NetworkDevice> {
    adapters()
        .into_iter()
        .filter(|d| d.state == DeviceState::Connected)
        .collect()
}

/// The saved connections a person made or uses: wired, Wi-Fi and VPN. Those in use come first.
fn saved() -> Vec<SavedConnection> {
    let mut list: Vec<SavedConnection> = network()
        .connections()
        .into_iter()
        .filter(|c| c.kind != ConnectionKind::Other)
        .collect();
    list.sort_by_key(|c| c.state == ActivationState::Inactive);
    list
}

fn hostname() -> String {
    std::fs::read_to_string("/proc/sys/kernel/hostname")
        .map(|h| h.trim().to_owned())
        .unwrap_or_else(|_| "This computer".into())
}

/// The picture of a device or a connection kind.
fn kind_icon(kind: DeviceKind) -> std::path::PathBuf {
    asset(match kind {
        DeviceKind::Wifi => "icons/network-wireless.png",
        DeviceKind::Ethernet => "icons/network-wired.png",
        DeviceKind::Other => "icons/network.png",
    })
}

fn connection_icon(kind: ConnectionKind) -> std::path::PathBuf {
    match kind {
        ConnectionKind::Wifi => kind_icon(DeviceKind::Wifi),
        ConnectionKind::Ethernet => kind_icon(DeviceKind::Ethernet),
        _ => kind_icon(DeviceKind::Other),
    }
}

fn picture(path: std::path::PathBuf, side: f32) -> Img {
    img(path)
        .flex_none()
        .size(side)
        .object_fit(ObjectFit::Contain)
}

/// What the network gives access to.
fn access_text() -> &'static str {
    match network().connectivity() {
        Connectivity::Full => "Internet",
        Connectivity::Portal => "Sign-in required",
        Connectivity::Limited => "No Internet access",
        Connectivity::None => "No network access",
        Connectivity::Unknown => "Unknown",
    }
}

/// A speed in Mb/s, as Windows writes it.
fn speed_text(mbps: u32) -> String {
    if mbps >= 1000 {
        format!("{:.1} Gbps", f64::from(mbps) / 1000.0)
    } else {
        format!("{mbps} Mbps")
    }
}

/// Text in the theme's face, size and text colour, on one line.
fn plain(label: impl IntoTextSource) -> Text {
    tinted_text(label, || scheme().text).whitespace_nowrap()
}

pub fn page() -> Div {
    let state = PageState {
        editing: signal(String::new()),
        forgetting: signal(String::new()),
        error: signal(String::new()),
    };
    let net = network();
    page_column(10.0)
        .child(heading(
            "View your basic network information and set up connections",
        ))
        .child(show(
            move || !net.available(),
            || note("NetworkManager is not running: no network can be shown or changed."),
        ))
        .child(map())
        .child(sub_heading("View your active networks"))
        .child(active_networks())
        .child(sub_heading("Network adapters"))
        .child(adapter_tiles())
        .child(sub_heading("Saved connections"))
        .child(show(
            move || !state.error.with(String::is_empty),
            move || tinted_text(move || state.error.get(), || ERROR_RED),
        ))
        .child(div().flex().flex_col().w_full().gap(2.0).child(for_each(
            saved,
            |c: &SavedConnection| c.uuid.clone(),
            move |c: SavedConnection| connection_row(state, c.uuid),
        )))
        .child(show(
            move || saved().is_empty() && net.available(),
            || note("No connection is saved."),
        ))
        .child(show(
            || adapters().iter().any(|d| d.kind == DeviceKind::Wifi),
            move || hidden_network_form(state),
        ))
}

// ----- the map ---------------------------------------------------------------------------------

/// This computer, the network it is on and the Internet, joined by lines; a line in red, with
/// a cross, where the way is cut.
fn map() -> Div {
    let primary = memo(|| connected().into_iter().next());
    let network_name = move || {
        primary.with(|d| {
            d.as_ref()
                .map_or("Not connected".to_owned(), |d| match d.kind {
                    DeviceKind::Wifi => network()
                        .wifi()
                        .map(|ap| ap.ssid)
                        .or_else(|| d.connection.clone())
                        .unwrap_or_default(),
                    _ => d.connection.clone().unwrap_or_else(|| d.interface.clone()),
                })
        })
    };
    let on_network = move || primary.with(Option::is_some);
    let on_internet = move || on_network() && network().connectivity() == Connectivity::Full;
    let middle = img(move || {
        primary.with(|d| {
            d.as_ref()
                .map_or(kind_icon(DeviceKind::Ethernet), |d| kind_icon(d.kind))
        })
    })
    .flex_none()
    .size(MAP_ICON)
    .object_fit(ObjectFit::Contain)
    .dyn_style(move |s| s.opacity(if on_network() { 1.0 } else { 0.45 }));
    let internet = picture(asset("icons/network.png"), MAP_ICON)
        .dyn_style(move |s| s.opacity(if on_internet() { 1.0 } else { 0.45 }));
    let column = |pic: View, label: Text, under: Option<Text>| {
        div()
            .flex_none()
            .w(MAP_COLUMN)
            .flex()
            .flex_col()
            .items_center()
            .gap(4.0)
            .child(pic)
            .child(label.font_weight(FontWeight::BOLD))
            .children(under)
    };
    div()
        .flex()
        .flex_row()
        .items_start()
        .pt(4.0)
        .pb(4.0)
        .child(column(
            picture(asset("icons/computer.png"), MAP_ICON).into_view(),
            plain(hostname()),
            Some(muted_small("(This computer)")),
        ))
        .child(map_line(on_network))
        .child(column(middle.into_view(), plain(network_name), None))
        .child(map_line(on_internet))
        .child(column(internet.into_view(), plain("Internet"), None))
}

/// A line of the map, at the height of the pictures' middles: with a green check while `whole`,
/// else red and cut by a cross.
fn map_line(whole: impl Fn() -> bool + Copy + 'static) -> Div {
    let half = || div().flex_1().h(2.0);
    div()
        .flex_none()
        .w(MAP_LINE)
        .h(MAP_ICON)
        .flex()
        .flex_row()
        .items_center()
        .gap(2.0)
        .child(half().dyn_style(move |s| {
            s.bg(if whole() {
                scheme().text_muted
            } else {
                CUT_RED
            })
        }))
        // In its middle a round badge: a green check where the way is open, a red cross where
        // it is cut.
        .child(
            div()
                .flex_none()
                .size(MAP_BADGE)
                .rounded(MAP_BADGE / 2.0)
                .flex()
                .items_center()
                .justify_center()
                .dyn_style(move |s| s.bg(if whole() { OPEN_GREEN } else { CUT_RED }))
                .child(
                    text(move || if whole() { "\u{2714}" } else { "\u{2715}" })
                        .text_size(11.0)
                        .text_color(Color::WHITE),
                ),
        )
        .child(half().dyn_style(move |s| {
            s.bg(if whole() {
                scheme().text_muted
            } else {
                CUT_RED
            })
        }))
}

// ----- the active networks ---------------------------------------------------------------------

/// Each network in use: its picture, its name and access, and its addresses; then the VPNs
/// connected.
fn active_networks() -> Div {
    let vpns = || {
        saved()
            .into_iter()
            .filter(|c| c.kind == ConnectionKind::Vpn && c.state == ActivationState::Activated)
            .collect::<Vec<_>>()
    };
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(14.0)
        .child(for_each(
            connected,
            |d: &NetworkDevice| d.interface.clone(),
            |d: NetworkDevice| active_network(d.interface),
        ))
        .child(for_each(
            vpns,
            |c: &SavedConnection| c.uuid.clone(),
            |c: SavedConnection| {
                control_row(12.0)
                    .child(picture(connection_icon(c.kind), ADAPTER_ICON))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(plain(c.name).font_weight(FontWeight::BOLD))
                            .child(muted_small("VPN connection")),
                    )
            },
        ))
        .child(show(
            || connected().is_empty(),
            || note("You are currently not connected to any network."),
        ))
}

/// A network in use, by its adapter's interface.
fn active_network(interface: String) -> Div {
    let found = memo(move || connected().into_iter().find(|d| d.interface == interface));
    let name = move || {
        found.with(|d| {
            let Some(d) = d else {
                return String::new();
            };
            match (d.kind, network().wifi()) {
                (DeviceKind::Wifi, Some(ap)) => ap.ssid,
                _ => d.connection.clone().unwrap_or_else(|| d.interface.clone()),
            }
        })
    };
    let icon =
        img(move || found.with(|d| kind_icon(d.as_ref().map_or(DeviceKind::Other, |d| d.kind))))
            .flex_none()
            .size(ADAPTER_ICON)
            .object_fit(ObjectFit::Contain);
    let head = div()
        .flex()
        .flex_col()
        .child(plain(name).font_weight(FontWeight::BOLD))
        .child(muted_small(move || {
            format!("Access type: {}", access_text())
        }));
    control_row(12.0).items_start().w_full().child(icon).child(
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w(0.0)
            .gap(6.0)
            .child(head)
            .child(dyn_view(move || {
                found.with(|d| d.as_ref().map(details)).unwrap_or_else(div)
            })),
    )
}

/// What Windows' "Network Connection Details" lists: the adapter, the signal, the speed, the
/// addresses, the gateways, the DNS servers and the physical address. Empty ones are left out.
fn details(d: &NetworkDevice) -> Div {
    let mut lines: Vec<(&str, String)> = Vec::new();
    let kind = match d.kind {
        DeviceKind::Wifi => "Wi-Fi",
        DeviceKind::Ethernet => "Ethernet",
        DeviceKind::Other => "Other",
    };
    lines.push(("Connection", format!("{kind} ({})", d.interface)));
    if d.kind == DeviceKind::Wifi
        && let Some(ap) = network().wifi()
    {
        let band = if ap.frequency >= 5000 {
            "5 GHz"
        } else {
            "2.4 GHz"
        };
        lines.push(("Signal quality", format!("{} % ({band})", ap.strength)));
    }
    if d.speed > 0 {
        lines.push(("Speed", speed_text(d.speed)));
    }
    let joined = |list: &[String]| list.join("\n");
    lines.push(("IPv4 address", joined(&d.ipv4.addresses)));
    lines.push(("IPv4 default gateway", d.ipv4.gateway.clone()));
    lines.push(("IPv4 DNS server", joined(&d.ipv4.dns)));
    lines.push(("IPv6 address", joined(&d.ipv6.addresses)));
    lines.push(("IPv6 default gateway", d.ipv6.gateway.clone()));
    lines.push(("IPv6 DNS server", joined(&d.ipv6.dns)));
    lines.push(("Physical address", d.hardware_address.clone()));
    div().flex().flex_col().gap(2.0).children(
        lines
            .into_iter()
            .filter(|(_, value)| !value.is_empty())
            .map(|(label, value)| {
                div()
                    .flex()
                    .flex_row()
                    .items_start()
                    .child(
                        div()
                            .flex_none()
                            .w(DETAIL_LABEL)
                            .child(muted_small(format!("{label}:"))),
                    )
                    .child(div().flex_1().min_w(0.0).child(text(value).dyn_style(|s| {
                        s.font_family(font())
                            .text_size(theme().font_size_small)
                            .text_color(scheme().text)
                    })))
            }),
    )
}

// ----- the adapters ----------------------------------------------------------------------------

/// The wired and Wi-Fi adapters as tiles, as Windows' Network Connections folder shows them.
fn adapter_tiles() -> Div {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .w_full()
        .gap(12.0)
        .child(for_each(
            adapters,
            |d: &NetworkDevice| d.interface.clone(),
            |d: NetworkDevice| adapter_tile(d.interface),
        ))
        .child(show(
            || adapters().is_empty(),
            || note("NetworkManager lists no wired or Wi-Fi adapter."),
        ))
}

/// What an adapter is doing, in words.
fn adapter_status(d: &NetworkDevice) -> String {
    let wifi_off = d.kind == DeviceKind::Wifi && !network().wifi_enabled();
    match d.state {
        _ if wifi_off => "Wireless is turned off".into(),
        DeviceState::Connected => d.connection.clone().unwrap_or_else(|| "Connected".into()),
        DeviceState::Connecting => "Connecting\u{2026}".into(),
        DeviceState::Disconnecting => "Disconnecting\u{2026}".into(),
        DeviceState::Unavailable if d.kind == DeviceKind::Ethernet => {
            "Network cable unplugged".into()
        }
        DeviceState::Unavailable => "Not available".into(),
        DeviceState::Unmanaged => "Not managed by NetworkManager".into(),
        DeviceState::Failed => "Failed".into(),
        DeviceState::Disconnected if !d.autoconnect => "Disabled".into(),
        _ => "Disconnected".into(),
    }
}

/// An adapter, by its interface: its picture, its name, what it is doing, and the button that
/// enables or disables it. Wi-Fi's tile has the radio's switch too.
fn adapter_tile(interface: String) -> Div {
    let found = memo(move || adapters().into_iter().find(|d| d.interface == interface));
    let kind = move || found.with(|d| d.as_ref().map_or(DeviceKind::Other, |d| d.kind));
    let busy = move || {
        found.with(|d| {
            d.as_ref().is_some_and(|d| {
                matches!(d.state, DeviceState::Connected | DeviceState::Connecting)
            })
        })
    };
    let wifi = Binding::new(
        || network().wifi_enabled(),
        |on: bool| {
            let _ = network().set_wifi_enabled(on);
        },
    );
    let button = cp_button(
        move || if busy() { "Disable" } else { "Enable" },
        move || {
            let Some(d) = untrack(|| found.get()) else {
                return;
            };
            let on = !untrack(busy);
            let _ = network().set_device_enabled(&d, on);
        },
    );
    let words = div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w(0.0)
        .gap(1.0)
        .overflow_clip()
        .child(
            plain(move || {
                found.with(|d| d.as_ref().map(|d| d.interface.clone()).unwrap_or_default())
            })
            .font_weight(FontWeight::BOLD),
        )
        .child(
            muted_small(move || found.with(|d| d.as_ref().map(adapter_status).unwrap_or_default()))
                .whitespace_nowrap(),
        )
        .child(
            muted_small(move || match kind() {
                DeviceKind::Wifi => "Wireless network adapter",
                _ => "Ethernet adapter",
            })
            .whitespace_nowrap(),
        );
    div()
        .flex()
        .flex_col()
        .flex_none()
        .w(ADAPTER_WIDTH)
        .gap(6.0)
        .p(8.0)
        .border(1.0)
        .dyn_style(|s| {
            let c = scheme();
            s.rounded(item_radius()).border_color(c.field_border)
        })
        .child(
            control_row(10.0)
                .items_start()
                .child(
                    img(move || kind_icon(kind()))
                        .flex_none()
                        .size(ADAPTER_ICON)
                        .object_fit(ObjectFit::Contain),
                )
                .child(words),
        )
        .child(
            control_row(10.0)
                .justify_between()
                .child(show(
                    move || kind() == DeviceKind::Wifi,
                    move || cp_check("Wireless on", wifi),
                ))
                .child(div().flex_1())
                .child(button.dyn_style(move |s| {
                    if kind() == DeviceKind::Wifi && !network().wifi_enabled() {
                        s.hidden()
                    } else {
                        s
                    }
                })),
        )
}

// ----- the saved connections -------------------------------------------------------------------

/// What a saved connection is and does, in words.
fn connection_status(c: &SavedConnection) -> String {
    let kind = match c.kind {
        ConnectionKind::Wifi => "Wi-Fi",
        ConnectionKind::Ethernet => "Wired",
        ConnectionKind::Vpn => "VPN",
        ConnectionKind::Other => "Other",
    };
    let state = match c.state {
        ActivationState::Activated => "Connected",
        ActivationState::Activating => "Connecting\u{2026}",
        ActivationState::Deactivating => "Disconnecting\u{2026}",
        ActivationState::Inactive => "Not connected",
    };
    let bound = if c.interface.is_empty() {
        String::new()
    } else {
        format!(" \u{b7} {}", c.interface)
    };
    format!("{kind}{bound} \u{b7} {state}")
}

/// A saved connection, by its UUID: its picture, name and state; "Connect automatically",
/// Properties, Forget and Connect or Disconnect. Its properties unfold under it.
fn connection_row(state: PageState, uuid: String) -> Div {
    let found = {
        let uuid = uuid.clone();
        memo(move || network().connections().into_iter().find(|c| c.uuid == uuid))
    };
    let open = {
        let uuid = uuid.clone();
        memo(move || state.editing.with(|e| *e == uuid))
    };
    let asking = {
        let uuid = uuid.clone();
        memo(move || state.forgetting.with(|f| *f == uuid))
    };
    let in_use = move || {
        found.with(|c| {
            c.as_ref()
                .is_some_and(|c| c.state != ActivationState::Inactive)
        })
    };
    let autoconnect = Binding::new(
        move || found.with(|c| c.as_ref().is_some_and(|c| c.autoconnect)),
        move |on: bool| {
            if let Some(c) = untrack(|| found.get()) {
                network()
                    .set_autoconnect(&c, on)
                    .then(move |done| state.report(done));
            }
        },
    );
    let connect = cp_button(
        move || if in_use() { "Disconnect" } else { "Connect" },
        move || {
            let Some(c) = untrack(|| found.get()) else {
                return;
            };
            state.error.set(String::new());
            if untrack(in_use) {
                network()
                    .deactivate(&c)
                    .then(move |done| state.report(done));
            } else {
                network().activate(&c).then(move |done| state.report(done));
            }
        },
    );
    let edit_uuid = uuid.clone();
    let forget_uuid = uuid.clone();
    let words = div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w(0.0)
        .overflow_clip()
        .child(
            plain(move || found.with(|c| c.as_ref().map(|c| c.name.clone()).unwrap_or_default()))
                .dyn_style(move |s| {
                    if in_use() {
                        s.font_weight(FontWeight::BOLD)
                    } else {
                        s
                    }
                }),
        )
        .child(
            muted_small(move || {
                found.with(|c| c.as_ref().map(connection_status).unwrap_or_default())
            })
            .whitespace_nowrap(),
        );
    let head = control_row(12.0)
        .w_full()
        .child(
            img(move || {
                connection_icon(
                    found.with(|c| c.as_ref().map_or(ConnectionKind::Other, |c| c.kind)),
                )
            })
            .flex_none()
            .size(ROW_ICON)
            .object_fit(ObjectFit::Contain),
        )
        .child(words)
        .child(connect);
    let actions = control_row(14.0)
        .w_full()
        .pl(ROW_ICON + 12.0)
        .child(cp_check("Connect automatically", autoconnect))
        .child(div().flex_1())
        .child(cp_link(
            move || {
                if open.get() {
                    "Close properties"
                } else {
                    "Properties"
                }
            },
            false,
            move || {
                let shut = state.editing.with_untracked(|e| *e == edit_uuid);
                state.forgetting.set(String::new());
                state.editing.set(if shut {
                    String::new()
                } else {
                    edit_uuid.clone()
                });
            },
        ))
        .child(cp_link("Forget", false, move || {
            state.editing.set(String::new());
            state.forgetting.set(forget_uuid.clone());
        }));
    let confirm = move || {
        control_row(10.0)
            .w_full()
            .pl(ROW_ICON + 12.0)
            .child(div().flex_1().min_w(0.0).child(tinted_text(
                move || {
                    let name = found.with(|c| c.as_ref().map(|c| c.name.clone()));
                    format!(
                        "Forget \u{201c}{}\u{201d}? Its settings and key are deleted.",
                        name.unwrap_or_default()
                    )
                },
                || scheme().text,
            )))
            .child(cp_button("Forget", move || {
                state.forgetting.set(String::new());
                if let Some(c) = untrack(|| found.get()) {
                    network().forget(&c).then(move |done| state.report(done));
                }
            }))
            .child(cp_button("Cancel", move || {
                state.forgetting.set(String::new())
            }))
    };
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(4.0)
        .p(8.0)
        .border(1.0)
        .dyn_style(move |s| {
            let c = scheme();
            let s = s.rounded(item_radius());
            if open.get() || asking.get() {
                s.bg(c.selection).border_color(c.selection_border)
            } else {
                s.bg(Color::TRANSPARENT).border_color(Color::TRANSPARENT)
            }
        })
        .child(head)
        .child(actions)
        .child(show(move || asking.get(), confirm))
        .child(show(
            move || open.get(),
            move || match untrack(|| found.get()) {
                Some(c) => properties(state, c).into_view(),
                None => div().into_view(),
            },
        ))
}

/// A row of a form: its label in a column of `width`, then its control.
pub fn form_row(label: &'static str, width: f32, control: impl IntoView) -> Div {
    control_row(8.0)
        .child(form_label(label, width))
        .child(control)
}
