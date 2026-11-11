//! The properties of a saved connection, unfolded under it on the Network and Sharing Center:
//! its name, whether it connects by itself, Wi-Fi's key, and how it gets its IPv4 and IPv6
//! addresses (automatically, or the address, gateway and DNS servers typed). And the form that
//! joins a Wi-Fi network that hides its name.

use super::common::{control_row, cp_field, field_border, muted_small, tinted_text};
use super::network::{PageState, form_row};
use crate::control_panel::note;
use crate::status::ERROR_RED;
use crate::theme::scheme;
use crate::ui::{cp_button, cp_check, cp_combo, cp_default_button, cp_link};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use anti_webui::shell::{
    ConnectionKind, ConnectionSettings, IpMethod, IpSettings, NetError, SavedConnection, Security,
};

/// The width of the labels of the form, and of its fields and combo boxes.
const LABEL_WIDTH: f32 = 150.0;
const FIELD_WIDTH: f32 = 260.0;

/// The ways of getting addresses, in the order the combo box lists them.
const METHODS: [IpMethod; 5] = [
    IpMethod::Automatic,
    IpMethod::Manual,
    IpMethod::LinkLocal,
    IpMethod::Shared,
    IpMethod::Disabled,
];

fn method_labels(v6: bool) -> Vec<String> {
    let automatic = if v6 { "Automatic" } else { "Automatic (DHCP)" };
    [
        automatic,
        "Manual",
        "Link-local only",
        "Shared to other computers",
        "Disabled",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

/// The words of a list typed in a field: separated by commas, semicolons or spaces.
fn words(text: &str) -> Vec<String> {
    text.split([',', ';', ' ', '\n'])
        .map(str::trim)
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}

/// A text field of the form, bound to `value`; a password field when `secret`.
fn field(value: Signal<String>, placeholder: &'static str, secret: bool) -> Div {
    let input = text_input()
        .value(value)
        .placeholder(placeholder)
        .purpose(if secret {
            ImePurpose::Password
        } else {
            ImePurpose::Normal
        });
    cp_field(input, FIELD_WIDTH, field_border)
}

/// What the form holds of one IP version while it is edited.
#[derive(Clone, Copy)]
struct IpForm {
    method: Signal<usize>,
    addresses: Signal<String>,
    gateway: Signal<String>,
    dns: Signal<String>,
}

impl IpForm {
    fn new() -> Self {
        IpForm {
            method: signal(0),
            addresses: signal(String::new()),
            gateway: signal(String::new()),
            dns: signal(String::new()),
        }
    }

    fn fill(self, ip: &IpSettings) {
        self.method
            .set(METHODS.iter().position(|m| *m == ip.method).unwrap_or(0));
        self.addresses.set(ip.addresses.join(", "));
        self.gateway.set(ip.gateway.clone());
        self.dns.set(ip.dns.join(", "));
    }

    fn settings(self) -> IpSettings {
        IpSettings {
            method: METHODS[self.method.get_untracked().min(METHODS.len() - 1)],
            addresses: words(&self.addresses.get_untracked()),
            gateway: self.gateway.get_untracked().trim().to_owned(),
            dns: words(&self.dns.get_untracked()),
        }
    }

    fn method(self) -> IpMethod {
        METHODS[self.method.get().min(METHODS.len() - 1)]
    }

    /// The combo box of the method; the address and the gateway when manual; the DNS servers
    /// unless disabled.
    fn view(self, title: &'static str, v6: bool) -> Div {
        let example = if v6 { "fd00::20/64" } else { "192.168.1.20/24" };
        let gateway = if v6 { "fd00::1" } else { "192.168.1.1" };
        let dns = if v6 {
            "2606:4700:4700::1111"
        } else {
            "1.1.1.1, 9.9.9.9"
        };
        div()
            .flex()
            .flex_col()
            .gap(6.0)
            .child(tinted_text(title, || scheme().text).font_weight(FontWeight::BOLD))
            .child(form_row(
                "Get addresses:",
                LABEL_WIDTH,
                cp_combo(method_labels(v6), self.method).w(FIELD_WIDTH),
            ))
            .child(show(
                move || self.method() == IpMethod::Manual,
                move || {
                    div()
                        .flex()
                        .flex_col()
                        .gap(6.0)
                        .child(form_row(
                            "Address/prefix:",
                            LABEL_WIDTH,
                            field(self.addresses, example, false),
                        ))
                        .child(form_row(
                            "Default gateway:",
                            LABEL_WIDTH,
                            field(self.gateway, gateway, false),
                        ))
                },
            ))
            .child(show(
                move || self.method() != IpMethod::Disabled,
                move || form_row("DNS servers:", LABEL_WIDTH, field(self.dns, dns, false)),
            ))
    }
}

/// The properties of `connection`, read from NetworkManager when they unfold. Save writes them
/// and folds them; Cancel folds them.
pub fn properties(state: PageState, connection: SavedConnection) -> Div {
    let loaded = signal(None::<ConnectionSettings>);
    let failed = signal(String::new());
    let name = signal(String::new());
    let autoconnect = signal(true);
    let key = signal(String::new());
    let ipv4 = IpForm::new();
    let ipv6 = IpForm::new();
    let busy = signal(false);
    network().settings(&connection).then(move |read| {
        if !loaded.is_alive() {
            return;
        }
        match read {
            Ok(s) => {
                name.set(s.name.clone());
                autoconnect.set(s.autoconnect);
                ipv4.fill(&s.ipv4);
                ipv6.fill(&s.ipv6);
                loaded.set(Some(s));
            }
            Err(e) => failed.set(format!("The settings cannot be read: {e}")),
        }
    });
    let secured = move || loaded.with(|s| s.as_ref().is_some_and(|s| s.security != Security::Open));
    let enterprise = move || {
        loaded.with(|s| {
            s.as_ref()
                .is_some_and(|s| s.security == Security::Enterprise)
        })
    };
    let is_wifi = connection.kind == ConnectionKind::Wifi;
    let saving = connection;
    let save = move || {
        let Some(read) = loaded.get_untracked() else {
            return;
        };
        let typed = key.get_untracked();
        let changed = ConnectionSettings {
            name: name.get_untracked().trim().to_owned(),
            autoconnect: autoconnect.get_untracked(),
            key: (!typed.is_empty()).then_some(typed),
            ipv4: ipv4.settings(),
            ipv6: ipv6.settings(),
            ..read
        };
        failed.set(String::new());
        busy.set(true);
        network().save_settings(&saving, changed).then(move |done| {
            if !busy.is_alive() {
                return;
            }
            busy.set(false);
            match done {
                Ok(()) => {
                    state.error.set(String::new());
                    state.editing.set(String::new());
                }
                Err(e) => failed.set(e.to_string()),
            }
        });
    };
    let general = div()
        .flex()
        .flex_col()
        .gap(6.0)
        .child(form_row(
            "Connection name:",
            LABEL_WIDTH,
            field(name, "", false),
        ))
        .child(form_row(
            "",
            LABEL_WIDTH,
            cp_check("Connect automatically when in range", autoconnect),
        ))
        .child(show(
            move || is_wifi && secured() && !enterprise(),
            move || {
                div()
                    .flex()
                    .flex_col()
                    .gap(2.0)
                    .child(form_row(
                        "Network security key:",
                        LABEL_WIDTH,
                        field(key, "Unchanged", true),
                    ))
                    .child(
                        div()
                            .pl(LABEL_WIDTH + 8.0)
                            .child(muted_small("Leave it empty to keep the saved key.")),
                    )
            },
        ))
        .child(show(
            move || is_wifi && enterprise(),
            || note("This network signs in with an account (802.1X), which is not edited here."),
        ));
    let form = div()
        .flex()
        .flex_col()
        .gap(12.0)
        .child(general)
        .child(ipv4.view("Internet Protocol Version 4 (TCP/IPv4)", false))
        .child(ipv6.view("Internet Protocol Version 6 (TCP/IPv6)", true))
        .child(
            control_row(8.0)
                .justify_end()
                .w_full()
                .child(cp_default_button(
                    move || if busy.get() { "Saving\u{2026}" } else { "Save" },
                    save,
                ))
                .child(cp_button("Cancel", move || {
                    state.editing.set(String::new())
                })),
        );
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap(8.0)
        .pt(8.0)
        .pl(44.0)
        .child(show(
            move || !failed.with(String::is_empty),
            move || tinted_text(move || failed.get(), || ERROR_RED),
        ))
        .child(show(
            move || loaded.with(Option::is_none) && failed.with(String::is_empty),
            || note("Reading the settings\u{2026}"),
        ))
        .child(form.dyn_style(move |s| {
            if loaded.with(Option::is_some) {
                s
            } else {
                s.hidden()
            }
        }))
}

/// The kinds of security a hidden network can have, in the order the combo box lists them.
const SECURITIES: [(Security, &str); 4] = [
    (Security::Wpa, "WPA2-Personal"),
    (Security::Wpa3, "WPA3-Personal"),
    (Security::Wep, "WEP"),
    (Security::Open, "No authentication (Open)"),
];

/// "Join a hidden network": a link that unfolds the network's name, its security and its key,
/// and Connect, which joins it and saves it.
pub fn hidden_network_form(state: PageState) -> Div {
    let unfolded = signal(false);
    let ssid = signal(String::new());
    let security = signal(0usize);
    let key = signal(String::new());
    let busy = signal(false);
    let failed = signal(String::new());
    let chosen = move || SECURITIES[security.get().min(SECURITIES.len() - 1)].0;
    let join = move || {
        let name = ssid.get_untracked().trim().to_owned();
        if name.is_empty() {
            failed.set("Type the network's name.".into());
            return;
        }
        let kind = SECURITIES[security.get_untracked().min(SECURITIES.len() - 1)].0;
        let typed = key.get_untracked();
        let password = (kind != Security::Open && !typed.is_empty()).then_some(typed);
        failed.set(String::new());
        busy.set(true);
        network()
            .join_hidden(&name, kind, password)
            .then(move |done| {
                if !busy.is_alive() {
                    return;
                }
                busy.set(false);
                match done {
                    Ok(()) => {
                        state.error.set(String::new());
                        unfolded.set(false);
                        ssid.set(String::new());
                        key.set(String::new());
                    }
                    Err(NetError::NeedsSecret) => {
                        failed.set("Type the network security key.".into());
                    }
                    Err(e) => failed.set(e.to_string()),
                }
            });
    };
    let form = move || {
        div()
            .flex()
            .flex_col()
            .gap(6.0)
            .pt(4.0)
            .child(form_row(
                "Network name:",
                LABEL_WIDTH,
                field(ssid, "", false),
            ))
            .child(form_row(
                "Security type:",
                LABEL_WIDTH,
                cp_combo(
                    SECURITIES.iter().map(|(_, l)| (*l).to_owned()).collect(),
                    security,
                )
                .w(FIELD_WIDTH),
            ))
            .child(show(
                move || chosen() != Security::Open,
                move || form_row("Security key:", LABEL_WIDTH, field(key, "", true)),
            ))
            .child(show(
                move || !failed.with(String::is_empty),
                move || tinted_text(move || failed.get(), || ERROR_RED),
            ))
            .child(
                control_row(8.0)
                    .pl(LABEL_WIDTH + 8.0)
                    .child(cp_default_button(
                        move || {
                            if busy.get() {
                                "Connecting\u{2026}"
                            } else {
                                "Connect"
                            }
                        },
                        join,
                    ))
                    .child(cp_button("Cancel", move || unfolded.set(false))),
            )
    };
    div()
        .flex()
        .flex_col()
        .pt(8.0)
        .child(cp_link("Join a hidden network\u{2026}", false, move || {
            failed.set(String::new());
            unfolded.set(!unfolded.get_untracked());
        }))
        .child(show(move || unfolded.get(), form))
}
