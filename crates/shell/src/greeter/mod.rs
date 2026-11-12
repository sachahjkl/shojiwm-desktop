//! The log-on screen, run as `shojiwm-shell greeter` by greetd, inside a compositor that greetd
//! starts (QML `greeter.qml`, `GreeterState`, `SessionPicker`). It shows the lock screen's
//! picture, clock, account and password field on the main screen, black on the others, and the
//! session to log on to in the bottom-left corner. greetd checks the password and starts the
//! session ([`greetd`]).
//!
//! It logs one account on. The system sets (the nixconfig module that starts greetd):
//!
//! - `SHOJIWM_GREETER_USER`: the account to log on;
//! - `SHOJIWM_GREETER_SESSIONS`: the folder of the sessions' .desktop files
//!   (`share/wayland-sessions`);
//! - `SHOJIWM_GREETER_DEFAULT_SESSION`: the session chosen the first time, by file name
//!   without `.desktop`. Then the last one used is chosen (kept in `last-session` of the state
//!   folder).
//!
//! When PAM asks more than the password (a one-time code), the field shows the question and
//! what is typed next answers it.

mod greetd;

use crate::lock::{
    Account, Auth, CORNER_MARGIN, LOGON_BLUE, accounts_service_picture, full_name, logon_menu,
    logon_menu_row, logon_menu_text, logon_screen,
};
use crate::paths::{asset, state_dir};
use crate::theme::{corner, theme};
use crate::ui::shadow_text;
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use greetd::{AuthKind, Greetd, Request, Response};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

/// What the user is told when greetd refuses the password.
const INCORRECT: &str = "The user name or password is incorrect.";

/// The session picker's button: its fill and its border, at rest and under the pointer or open.
const PICKER_FILL: Color = rgba(0xffffff1f);
const PICKER_FILL_HOT: Color = rgba(0xffffff40);
const PICKER_BORDER: Color = rgba(0xffffff59);
const PICKER_BORDER_HOT: Color = rgba(0xffffff99);

/// A session of `SHOJIWM_GREETER_SESSIONS`.
#[derive(Clone, Debug, PartialEq)]
struct Session {
    /// The file name without `.desktop`.
    id: String,
    name: String,
    exec: String,
    desktop_names: Vec<String>,
}

/// What a response from greetd answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sent {
    /// `create_session` and the answers to PAM.
    Auth,
    Start,
    Cancel,
}

/// The log-on screen's state, shared by every screen.
#[derive(Clone, Copy)]
struct Greeter {
    auth: Auth,
    sessions: Signal<Vec<Session>>,
    session_id: Signal<String>,
}

thread_local! {
    static GREETER: Cell<Option<Greeter>> = const { Cell::new(None) };
    static GREETD: RefCell<Option<Greetd>> = const { RefCell::new(None) };
    /// What the responses still to come answer, in order.
    static SENT: RefCell<VecDeque<Sent>> = const { RefCell::new(VecDeque::new()) };
    /// The password typed, until PAM asks for it.
    static PENDING: RefCell<String> = const { RefCell::new(String::new()) };
}

/// greetd has started a session: the greeter leaves with success.
static LAUNCHED: AtomicBool = AtomicBool::new(false);

/// The account to log on.
fn user() -> String {
    std::env::var("SHOJIWM_GREETER_USER").unwrap_or_default()
}

/// `$XDG_STATE_HOME/shojiwm-desktop/last-session`.
fn last_session_file() -> PathBuf {
    state_dir().join("last-session")
}

/// Every .desktop file of `SHOJIWM_GREETER_SESSIONS` that is shown and has a command, by name.
fn read_sessions() -> Vec<Session> {
    let Some(dir) = std::env::var_os("SHOJIWM_GREETER_SESSIONS") else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut list: Vec<Session> = entries
        .flatten()
        .filter_map(|entry| {
            let file = entry.file_name().to_string_lossy().into_owned();
            let id = file.strip_suffix(".desktop")?.to_owned();
            let text = std::fs::read_to_string(entry.path()).ok()?;
            parse_session(id, &text)
        })
        .collect();
    list.sort_by_key(|s| s.name.to_lowercase());
    list
}

/// A session from its .desktop file: `Name`, `Exec` and `DesktopNames` of `[Desktop Entry]`.
/// `None` when it is hidden or has no command.
fn parse_session(id: String, text: &str) -> Option<Session> {
    let mut session = Session {
        id,
        name: String::new(),
        exec: String::new(),
        desktop_names: Vec::new(),
    };
    let mut in_entry = false;
    let mut hidden = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_entry = line == "[Desktop Entry]";
            continue;
        }
        let Some((key, value)) = line.split_once('=').filter(|_| in_entry) else {
            continue;
        };
        match key {
            "Name" => session.name = value.to_owned(),
            "Exec" => session.exec = value.to_owned(),
            "DesktopNames" => {
                session.desktop_names = value
                    .split(';')
                    .filter(|n| !n.is_empty())
                    .map(String::from)
                    .collect();
            }
            "Hidden" | "NoDisplay" if value == "true" => hidden = true,
            _ => {}
        }
    }
    (!hidden && !session.exec.is_empty()).then_some(session)
}

impl Greeter {
    /// The state, with no session until a worker has read the sessions and the last one used.
    fn new() -> Self {
        let greeter = Self {
            auth: Auth::new(authenticate),
            sessions: signal(Vec::new()),
            session_id: signal(String::new()),
        };
        let file = last_session_file();
        task(move || {
            let chosen = std::fs::read_to_string(file)
                .map(|text| text.trim().to_owned())
                .unwrap_or_else(|_| {
                    std::env::var("SHOJIWM_GREETER_DEFAULT_SESSION").unwrap_or_default()
                });
            (read_sessions(), chosen)
        })
        .then(move |(sessions, chosen)| {
            batch(|| {
                // A session picked before the list came stays picked.
                if greeter.session_id.with_untracked(String::is_empty) {
                    greeter.session_id.set(chosen);
                }
                greeter.sessions.set(sessions);
            });
        });
        greeter
    }

    /// The session chosen, else the first. Reading it tracks it.
    fn session(self) -> Option<Session> {
        let id = self.session_id.get();
        self.sessions.with(|list| {
            list.iter()
                .find(|s| s.id == id)
                .or_else(|| list.first())
                .cloned()
        })
    }
}

fn current() -> Option<Greeter> {
    GREETER.with(Cell::get)
}

/// Sends a request to greetd, and remembers what its response answers.
fn send(sent: Sent, request: Request) {
    GREETD.with(|g| {
        if let Some(greetd) = g.borrow().as_ref() {
            SENT.with(|s| s.borrow_mut().push_back(sent));
            greetd.send(request);
        }
    });
}

fn available() -> bool {
    GREETD.with(|g| g.borrow().is_some())
}

/// Logs on with `password`, or answers PAM's question with it.
fn authenticate(password: String) {
    let Some(greeter) = current() else {
        return;
    };
    let auth = greeter.auth;
    if auth.busy.get_untracked() || password.is_empty() {
        return;
    }
    if !auth.prompt.with_untracked(String::is_empty) {
        batch(|| {
            auth.prompt.set(String::new());
            auth.typed.set(String::new());
            auth.busy.set(true);
        });
        send(Sent::Auth, Request::Respond(Some(password)));
        return;
    }
    if !available() || user().is_empty() || untrack(|| greeter.session()).is_none() {
        fail(auth, "Cannot log on: no greetd, account or session.");
        return;
    }
    PENDING.with(|p| *p.borrow_mut() = password);
    auth.busy.set(true);
    auth.message.set(String::new());
    send(Sent::Auth, Request::CreateSession { username: user() });
}

fn fail(auth: Auth, text: impl Into<String>) {
    PENDING.with(|p| p.borrow_mut().clear());
    auth.fail(text);
}

/// What greetd says.
fn on_response(response: Response) {
    let Some(greeter) = current() else {
        return;
    };
    let auth = greeter.auth;
    let sent = SENT.with(|s| s.borrow_mut().pop_front());
    match (sent, response) {
        // Whatever a cancel answers, it is over.
        (Some(Sent::Cancel), _) => {}
        (_, Response::AuthMessage { kind, text }) => match kind {
            // greetd waits for an answer to every message, information too.
            AuthKind::Info | AuthKind::Error => send(Sent::Auth, Request::Respond(None)),
            // PAM asks for the password: give it the one typed.
            AuthKind::Secret if PENDING.with(|p| !p.borrow().is_empty()) => {
                let password = PENDING.with(|p| std::mem::take(&mut *p.borrow_mut()));
                send(Sent::Auth, Request::Respond(Some(password)));
            }
            // A question after it (a one-time code) goes to the field.
            _ => batch(|| {
                auth.busy.set(false);
                auth.typed.set(String::new());
                auth.prompt_echo.set(kind == AuthKind::Visible);
                auth.prompt
                    .set(text.trim().trim_end_matches(':').trim_end().to_owned());
            }),
        },
        (Some(Sent::Start), Response::Success) => {
            LAUNCHED.store(true, Ordering::SeqCst);
            quit();
        }
        (_, Response::Success) => launch(greeter),
        (
            _,
            Response::Error {
                auth: wrong,
                description,
            },
        ) => {
            send(Sent::Cancel, Request::CancelSession);
            if wrong {
                fail(auth, INCORRECT);
            } else {
                fail(auth, format!("Cannot log on: {description}"));
            }
        }
    }
}

/// The account is checked: starts the session chosen, and keeps it for next time.
fn launch(greeter: Greeter) {
    let Some(session) = untrack(|| greeter.session()) else {
        send(Sent::Cancel, Request::CancelSession);
        fail(greeter.auth, "Cannot log on: no session.");
        return;
    };
    // The file is written before the session starts: the greeter leaves once it has.
    let file = last_session_file();
    let id = session.id.clone();
    crate::background::off_thread(
        move || {
            if let Some(dir) = file.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = std::fs::write(&file, format!("{id}\n"));
        },
        move |()| start_session(&session),
    );
}

/// Asks greetd to start `session`.
fn start_session(session: &Session) {
    let desktops = if session.desktop_names.is_empty() {
        session.id.clone()
    } else {
        session.desktop_names.join(":")
    };
    // greetd joins the command's words with spaces and gives the line to a shell, after
    // /etc/profile (the system's PATH and XDG_DATA_DIRS). So Exec goes as one word: the shell
    // splits it as the .desktop file writes it. ["/bin/sh", "-c", exec] would be split again,
    // and run the first word of Exec alone.
    send(
        Sent::Start,
        Request::StartSession {
            cmd: vec![session.exec.clone()],
            env: vec![
                "XDG_SESSION_TYPE=wayland".into(),
                format!("XDG_SESSION_DESKTOP={}", session.id),
                format!("XDG_CURRENT_DESKTOP={desktops}"),
            ],
        },
    );
}

/// The session to log on to, in the bottom-left corner: its name and an arrow; a click opens
/// the list of sessions above it, styled as the power button's menu.
fn session_picker(greeter: Greeter) -> Div {
    let open = signal(false);
    let ui = Interaction::new();
    let list = move || {
        logon_menu()
            .absolute()
            .left(0.0)
            .bottom(32.0)
            .min_w(Length::Percent(100.0))
            .child(for_each(
                move || greeter.sessions.get(),
                |s| s.id.clone(),
                move |s| {
                    let id = s.id.clone();
                    let chosen = id.clone();
                    logon_menu_row(s.name.clone(), 24.0, move || {
                        greeter.session_id.set(id.clone());
                        open.set(false);
                    })
                    // The chosen session, with a dot, as a radio item.
                    .child(
                        div()
                            .absolute()
                            .left(7.0)
                            .top(0.0)
                            .bottom(0.0)
                            .flex()
                            .items_center()
                            .dyn_style(move |st| {
                                let st = st
                                    .text_size(theme().font_size - 3.0)
                                    .text_color(logon_menu_text());
                                if greeter.session().is_some_and(|s| s.id == chosen) {
                                    st
                                } else {
                                    st.hidden()
                                }
                            })
                            .child(text("\u{25CF}")),
                    )
                },
            ))
    };
    div()
        .relative()
        .flex_none()
        .child(show(move || open.get(), list))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .h(28.0)
                .pl(11.0)
                .pr(9.0)
                .gap(12.0)
                .border(1.0)
                .select_none()
                .interaction(ui)
                .access(|a| a.role(Role::Button))
                .on_click(move |_| open.update(|o| *o = !*o))
                .dyn_style(move |s| {
                    let hot = ui.hovered() || open.get();
                    s.rounded(corner(3.0))
                        .bg(if hot { PICKER_FILL_HOT } else { PICKER_FILL })
                        .border_color(if hot {
                            PICKER_BORDER_HOT
                        } else {
                            PICKER_BORDER
                        })
                })
                .child(
                    shadow_text(move || {
                        greeter
                            .session()
                            .map_or_else(|| "No session".to_owned(), |s| s.name)
                    })
                    .dyn_style(|s| s.text_size(theme().font_size + 1.0)),
                )
                // The arrow, pointing up to the list.
                .child(
                    text("\u{25B4}")
                        .text_color(Color::WHITE)
                        .dyn_style(|s| s.text_size(theme().font_size)),
                ),
        )
}

/// One output's surface: the log-on screen and the session picker on the main screen, black
/// elsewhere.
fn surface(primary: Memo<bool>, greeter: Greeter, account: Account) -> Div {
    div()
        .size_full()
        .dyn_style(move |s| {
            s.bg(if primary.get() {
                LOGON_BLUE
            } else {
                Color::BLACK
            })
        })
        .child(dyn_view(move || {
            let primary = primary.get();
            let account = account.clone();
            untrack(move || {
                primary.then(|| {
                    div()
                        .relative()
                        .size_full()
                        .child(logon_screen(greeter.auth, account))
                        .child(
                            session_picker(greeter)
                                .absolute()
                                .left(CORNER_MARGIN)
                                .bottom(CORNER_MARGIN),
                        )
                        .into_view()
                })
            })
        }))
}

/// The log-on screen, until greetd has started a session: `Ok` then, an error when it left
/// without (greetd's command then offers its text log-on).
pub fn run() -> Result<(), String> {
    // The greeter's account has no settings of its own: the defaults, never written.
    crate::settings::never_save();
    let result = Shell::new()
        .namespace("shojiwm-desktop-greeter")
        .app_id("shojiwm-shell-greeter")
        .run(|| {
            let greeter = Greeter::new();
            GREETER.with(|g| g.set(Some(greeter)));
            match Greetd::connect(on_response) {
                Ok(greetd) => GREETD.with(|g| *g.borrow_mut() = Some(greetd)),
                Err(e) => eprintln!("shojiwm-shell greeter: {e}"),
            }
            let login = user();
            let name = full_name(&login);
            let picture = accounts_service_picture(&login);
            let account = Account {
                name: Rc::new(move || name.get()),
                picture: Rc::new(move || picture.get().unwrap_or_else(|| asset("user.svg"))),
            };
            for_each_output(move |output| {
                let output_name = output.name.clone();
                let primary = memo(move || crate::desktop::primary_output_name() == output_name);
                let account = account.clone();
                let layer = open_layer(
                    LayerOptions::new()
                        .layer(Layer::Overlay)
                        .anchor(Anchor::ALL)
                        .exclusive(Exclusive::Ignore)
                        .namespace("shojiwm-desktop-greeter"),
                    move || surface(primary, greeter, account),
                );
                // The keyboard goes to the main screen's password field alone.
                effect(move || {
                    layer.set_keyboard(if primary.get() {
                        Keyboard::Exclusive
                    } else {
                        Keyboard::None
                    });
                });
            });
        });
    result.map_err(|e| e.to_string())?;
    if LAUNCHED.load(Ordering::SeqCst) {
        Ok(())
    } else {
        Err("the log-on screen left without starting a session".into())
    }
}
