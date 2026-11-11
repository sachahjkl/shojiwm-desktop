//! The lock screen, as the Windows 7 log-on screen is laid out (QML `LockScreen`): the picture
//! of your choice behind, the time over it, your account picture in its glass frame, your name,
//! and the password field with the round arrow. A wrong password replaces the field with the
//! message and OK, as Windows does. The red power button and its arrow sit in the bottom right
//! corner.
//!
//! It shows on the main screen only. The lock (`lock`) checks the password with PAM; the
//! log-on screen (`greeter`) with greetd. Each gives an [`Auth`].

use super::parts::{logon_button, logon_menu, logon_menu_row, power_split_button, user_tile};
use crate::paths::asset;
use crate::power::{power_button_action, run_power};
use crate::settings::{dark, settings};
use crate::theme::{is_kernel, theme};
use crate::ui::{dark_look, font, shadow_text, vertical_gradient};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;

/// The blue behind the picture, which the main screen shows until the picture is there.
pub const LOGON_BLUE: Color = rgb(0x0e3a63);

/// The shade from the top and the bottom, so that white text reads on any picture: on light
/// colours, then on dark ones, at 0, 0.35, 0.7 and 1 of the height.
const SHADE: [[Color; 4]; 2] = [
    [
        rgba(0x00000059),
        rgba(0x00000014),
        rgba(0x00000014),
        rgba(0x00000073),
    ],
    [
        rgba(0x00000080),
        rgba(0x00000033),
        rgba(0x00000033),
        rgba(0x00000099),
    ],
];

/// The password field: its fill, its border at rest and with the keyboard, the inner shade a
/// Windows text box has along its top and its left, its text and its hint. Light, then dark.
const FIELD: [Color; 2] = [rgb(0xffffff), rgb(0x141c24)];
const FIELD_BORDER: [Color; 2] = [rgb(0x5a7f9c), rgb(0x3c5164)];
const FIELD_BORDER_FOCUS: [Color; 2] = [rgb(0x3d7bad), rgb(0x4b8fd0)];
const FIELD_SHADE_TOP: [Color; 2] = [rgb(0xc4d3de), rgb(0x0a1016)];
const FIELD_SHADE_LEFT: [Color; 2] = [rgb(0xdde6ec), rgb(0x0d141b)];
const FIELD_TEXT: [Color; 2] = [rgb(0x000000), rgb(0xe8f1f9)];
const FIELD_HINT: [Color; 2] = [rgb(0x6d6d6d), rgb(0x7f94a6)];

/// The power actions the arrow of the power button offers, by id, with their labels.
const POWER_MENU: [(&str, &str); 4] = [
    ("restart", "Restart"),
    ("sleep", "Sleep"),
    ("hibernate", "Hibernate"),
    ("shutdown", "Shut down"),
];

/// The room from the screen's edges to the power button and the session picker.
pub const CORNER_MARGIN: f32 = 26.0;

/// What checks the password, and what the screen shows of it: the lock's PAM or the log-on
/// screen's greetd. Make it under an owner that lives as long as the screens.
#[derive(Clone, Copy)]
pub struct Auth {
    /// A password is being checked.
    pub busy: Signal<bool>,
    /// What to tell the user after a failure.
    pub message: Signal<String>,
    /// Counts the failures, so that each screen clears its field and shows the message.
    pub failures: Signal<u32>,
    /// What is typed in the password field, shared by every screen: the keyboard can go to any
    /// of them.
    pub typed: Signal<String>,
    /// A question from PAM after the password (a one-time code), or "": the field asks it.
    pub prompt: Signal<String>,
    /// The answer to `prompt` shows as typed, not as dots.
    pub prompt_echo: Signal<bool>,
    /// Checks a password (or answers `prompt`).
    pub authenticate: fn(String),
}

impl Auth {
    pub fn new(authenticate: fn(String)) -> Self {
        Self {
            busy: signal(false),
            message: signal(String::new()),
            failures: signal(0),
            typed: signal(String::new()),
            prompt: signal(String::new()),
            prompt_echo: signal(false),
            authenticate,
        }
    }

    /// Checks what is typed, unless it is empty or a check is going on.
    pub fn submit(self) {
        if self.busy.get_untracked() || self.typed.with_untracked(String::is_empty) {
            return;
        }
        (self.authenticate)(self.typed.get_untracked());
    }

    /// Counts a failure and says why: each screen then clears its field and shows `text`.
    pub fn fail(self, text: impl Into<String>) {
        batch(|| {
            self.busy.set(false);
            self.prompt.set(String::new());
            self.message.set(text.into());
            self.failures.update(|n| *n += 1);
        });
    }
}

/// The account the screen shows: its name and its picture, read in reactive closures.
#[derive(Clone)]
pub struct Account {
    pub name: Rc<dyn Fn() -> String>,
    pub picture: Rc<dyn Fn() -> PathBuf>,
}

/// The built-in wallpaper of the theme in use (Settings.qml `builtInWallpaper`).
fn built_in_wallpaper() -> PathBuf {
    if is_kernel() {
        asset("kernel/wallpaper.svg")
    } else if dark() {
        asset("wallpaper-dark.svg")
    } else {
        asset("wallpaper.svg")
    }
}

/// The lock screen's picture: its own ("builtin" for the built-in one), else the desktop's,
/// else the built-in one (Settings.qml `lockSource`). Reading it tracks the settings.
pub fn lock_picture() -> PathBuf {
    let (lock, wallpaper) = settings().with(|s| (s.lock_background.clone(), s.wallpaper.clone()));
    match (lock.as_str(), wallpaper.is_empty()) {
        ("builtin", _) | ("", true) => built_in_wallpaper(),
        ("", false) => PathBuf::from(wallpaper),
        (own, _) => PathBuf::from(own),
    }
}

/// The time in the Control Panel's format: 24-hour, 12-hour, or the locale's.
fn time_text() -> String {
    let format = match settings().with(|s| s.clock_format.clone()).as_str() {
        "12h" => "%-I:%M %p",
        _ => "%H:%M",
    };
    local_time(Every::Minute).format(format)
}

/// The log-on screen, filling its surface: see the module's comment.
pub fn logon_screen(auth: Auth, account: Account) -> Div {
    // A wrong password is on show, until OK.
    let showing_error = signal(false);
    let power_menu_open = signal(false);
    let height = signal(0.0f32);
    let root = node_ref();
    let field = node_ref();
    request_frame(move |_| field.focus());

    // A failure: the field is cleared, and the message shows instead of it.
    let seen = std::cell::Cell::new(auth.failures.get_untracked());
    effect(move || {
        let failures = auth.failures.get();
        if failures != seen.replace(failures) {
            untrack(|| {
                auth.typed.set(String::new());
                showing_error.set(true);
                root.focus();
            });
        }
    });

    // The field cannot be typed in while a password is checked: it takes the keyboard back
    // after, unless the message shows.
    effect(move || {
        if !auth.busy.get() {
            request_frame(move |_| {
                if !showing_error.get_untracked() {
                    field.focus();
                }
            });
        }
    });

    let dismiss = move || {
        showing_error.set(false);
        auth.typed.set(String::new());
        field.focus();
    };

    let picture = memo(lock_picture);
    let account_picture = {
        let picture = account.picture.clone();
        memo(move || picture())
    };
    let name = account.name.clone();

    // ----- the time --------------------------------------------------------------------------
    let clock = div()
        .flex()
        .flex_col()
        .items_center()
        .gap(2.0)
        .dyn_style(move |s| s.mb((height.get() * 0.06).round()))
        .child(
            shadow_text(time_text).dyn_style(|s| s.text_size(76.0).font_weight(FontWeight::LIGHT)),
        )
        .child(shadow_text(|| local_time(Every::Minute).format("%A, %B %-d, %Y")).text_size(20.0));

    // ----- the password field and its round arrow ----------------------------------------------
    let entry = div()
        .mt(12.0)
        .h(28.0)
        .flex()
        .flex_row()
        .items_center()
        .gap(4.0)
        .dyn_style(move |s| if showing_error.get() { s.hidden() } else { s })
        .child(password_field(auth, field))
        .child(go_button(auth));

    // "Welcome", while the password is checked.
    let welcome = shadow_text("Welcome").mt(14.0).dyn_style(move |s| {
        let s = s.text_size(theme().font_size + 4.0);
        if auth.busy.get() { s } else { s.hidden() }
    });

    // A wrong password: the field gives way to the message and OK.
    let error = div()
        .mt(14.0)
        .flex()
        .flex_col()
        .items_center()
        .gap(16.0)
        .dyn_style(move |s| if showing_error.get() { s } else { s.hidden() })
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(10.0)
                .child(img(asset("error-shield.svg")).flex_none().size(24.0))
                .child(
                    shadow_text(move || auth.message.get())
                        .dyn_style(|s| s.text_size(theme().font_size + 4.0)),
                ),
        )
        .child(logon_button("OK", dismiss));

    // The tile's middle stands 50 px above the screen's: the upper and lower parts share what
    // is left once the tile and 100 px under the lower part are taken.
    let layout = div()
        .absolute()
        .inset(0.0)
        .flex()
        .flex_col()
        .items_center()
        .child(
            div()
                .flex_1()
                .w_full()
                .flex()
                .flex_col()
                .justify_end()
                .items_center()
                .child(clock),
        )
        .child(user_tile(move || account_picture.get()))
        .child(
            div()
                .flex_1()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .child(shadow_text(move || name()).mt(14.0).text_size(22.0))
                .child(entry)
                .child(welcome)
                .child(error),
        )
        .child(div().flex_none().h(100.0));

    div()
        .node_ref(root)
        .focusable()
        .relative()
        .size_full()
        .overflow_hidden()
        .select_none()
        .bg(LOGON_BLUE)
        .on_resized(move |_, h| height.set(h))
        // Return, Escape or Space on the message is OK.
        .on_key_down(move |e| {
            if showing_error.get_untracked()
                && matches!(
                    e.key,
                    Key::Named(NamedKey::Enter | NamedKey::Escape | NamedKey::Space)
                )
            {
                e.prevent_default();
                dismiss();
            }
        })
        .child(
            img(move || picture.get())
                .absolute()
                .inset(0.0)
                .size_full()
                .object_fit(ObjectFit::Cover),
        )
        .child(div().absolute().inset(0.0).pointer_events_none().fill(|| {
            let [top, upper, lower, bottom] = SHADE[usize::from(dark_look())];
            vertical_gradient(&[(0.0, top), (0.35, upper), (0.7, lower), (1.0, bottom)])
        }))
        .child(layout)
        .child(
            power_split_button(
                || run_power(power_button_action()),
                move || power_menu_open.update(|open| *open = !*open),
            )
            .absolute()
            .right(CORNER_MARGIN)
            .bottom(CORNER_MARGIN),
        )
        .child(show(
            move || power_menu_open.get(),
            move || power_menu(power_menu_open),
        ))
}

/// Restart, sleep, hibernate, shut down, above the power button.
fn power_menu(open: Signal<bool>) -> Div {
    logon_menu()
        .absolute()
        .right(CORNER_MARGIN)
        .bottom(CORNER_MARGIN * 2.0)
        .w(150.0)
        .children(POWER_MENU.map(|(id, label)| {
            logon_menu_row(label, 10.0, move || {
                open.set(false);
                run_power(id);
            })
        }))
}

/// The field's box: its fill, its border, the inner shade, the hint and the text input.
fn password_field(auth: Auth, field: NodeRef) -> Div {
    let ui = Interaction::new();
    let echo = memo(move || !auth.prompt.with(String::is_empty) && auth.prompt_echo.get());
    let shade = || usize::from(dark_look());
    div()
        .relative()
        .flex_none()
        .w(225.0)
        .h(26.0)
        .border(1.0)
        .dyn_style(move |s| {
            let d = shade();
            let border = if ui.focused() {
                FIELD_BORDER_FOCUS[d]
            } else {
                FIELD_BORDER[d]
            };
            s.bg(FIELD[d]).border_color(border)
        })
        .child(
            div()
                .absolute()
                .left(0.0)
                .right(0.0)
                .top(0.0)
                .h(1.0)
                .dyn_style(move |s| s.bg(FIELD_SHADE_TOP[shade()])),
        )
        .child(
            div()
                .absolute()
                .left(0.0)
                .top(0.0)
                .bottom(0.0)
                .w(1.0)
                .dyn_style(move |s| s.bg(FIELD_SHADE_LEFT[shade()])),
        )
        // The hint, or PAM's question, while nothing is typed.
        .child(
            div()
                .absolute()
                .inset(0.0)
                .px(6.0)
                .flex()
                .items_center()
                .whitespace_nowrap()
                .overflow_clip()
                .pointer_events_none()
                .dyn_style(move |s| {
                    let s = s
                        .font_family(font())
                        .text_size(theme().font_size)
                        .text_color(FIELD_HINT[shade()]);
                    if auth.typed.with(String::is_empty) {
                        s
                    } else {
                        s.hidden()
                    }
                })
                .child(text(move || {
                    let prompt = auth.prompt.get();
                    if prompt.is_empty() {
                        "Password".to_owned()
                    } else {
                        prompt
                    }
                })),
        )
        .child(
            div()
                .absolute()
                .inset(0.0)
                .px(6.0)
                .flex()
                .items_center()
                // The answer to a question PAM wants to see is typed in the clear: the input
                // is made again when that changes.
                .child(dyn_view(move || {
                    let echo = echo.get();
                    untrack(|| password_input(auth, field, ui, echo).into_view())
                })),
        )
}

/// The text input itself, bound to what is typed. Enter checks it; Escape clears it.
fn password_input(auth: Auth, field: NodeRef, ui: Interaction, echo: bool) -> TextInput {
    request_frame(move |_| field.focus());
    text_input()
        .purpose(if echo {
            ImePurpose::Normal
        } else {
            ImePurpose::Password
        })
        .value(auth.typed)
        .node_ref(field)
        .interaction(ui)
        .w_full()
        .p(0.0)
        .border(0.0)
        .bg(Color::TRANSPARENT)
        .disabled(move || auth.busy.get())
        .on_submit(move |_| auth.submit())
        .on_key_down(move |e| {
            if matches!(e.key, Key::Named(NamedKey::Escape)) {
                auth.typed.set(String::new());
            }
        })
        .dyn_style(move |s| {
            let color = FIELD_TEXT[usize::from(dark_look())];
            s.font_family(font())
                .text_size(theme().font_size)
                .text_color(color)
                .caret_color(color)
        })
}

/// The round arrow: a bead of glass (`arrow-bead.svg`), brighter under the pointer, dimmed
/// while pressed, as if pushed in.
fn go_button(auth: Auth) -> Div {
    let ui = Interaction::new();
    div()
        .flex_none()
        .size(32.0)
        .interaction(ui)
        .access(|a| a.role(Role::Button))
        .on_click(move |_| auth.submit())
        .child(
            img(move || {
                let dark = if dark_look() { "-dark" } else { "" };
                let hot = if ui.hovered() && !ui.pressed() {
                    "-hot"
                } else {
                    ""
                };
                asset(&format!("arrow-bead{dark}{hot}.svg"))
            })
            .size(32.0)
            .dyn_style(move |s| s.opacity(if ui.pressed() { 0.8 } else { 1.0 })),
        )
}

/// A screen of the lock other than the main one: black, with an unseen password field that
/// shares what is typed with the main screen's, in case the compositor gives it the keyboard.
pub fn blank_screen(auth: Auth) -> Div {
    let field = node_ref();
    request_frame(move |_| field.focus());
    div().size_full().bg(Color::BLACK).child(
        text_input()
            .purpose(ImePurpose::Password)
            .value(auth.typed)
            .node_ref(field)
            .w(1.0)
            .h(1.0)
            .opacity(0.0)
            .on_submit(move |_| auth.submit())
            .on_key_down(move |e| {
                if matches!(e.key, Key::Named(NamedKey::Escape)) {
                    auth.typed.set(String::new());
                }
            }),
    )
}
