//! Whether the session is locked, and everything that locks it (QML `LockState` and the
//! `WlSessionLock` of `shell.qml`):
//!
//! - the Lock entry of the Start menu's power button, and Super+L (`shojiwm-shell msg lock
//!   lock`);
//! - `loginctl lock-session`, and anything else that asks logind, such as an idle daemon;
//! - going to sleep, when the Control Panel says so (`lock_on_suspend`);
//! - the pointer and the keyboard left alone for the Control Panel's delay
//!   (`lock_after_minutes`). A program that keeps the screen on, such as a video player, holds
//!   it off.
//!
//! While locked, the compositor shows the lock screens (`ext_session_lock_v1`) and nothing
//! else, and sends them every key. The log-on screen ([`screen`]) shows on the main screen; the
//! others stay black. Only a right password unlocks, checked with PAM ([`pam`]). If the shell
//! dies, the session stays locked.

mod pam;
mod parts;
mod screen;

pub(crate) use crate::account::{accounts_service_picture, full_name, login_name, user_picture};
pub(crate) use parts::{MENU_TEXT, logon_menu, logon_menu_row};
pub(crate) use screen::{Account, Auth, CORNER_MARGIN, LOGON_BLUE, logon_screen};

use crate::settings::settings;
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use pam::Refused;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

/// What the user is told when PAM refuses the password, and when it cannot be asked.
const INCORRECT: &str = "The user name or password is incorrect.";
const NOT_CHECKED: &str = "The password could not be checked.";

thread_local! {
    /// The password's state, shared by every lock screen. Made by `install`.
    static AUTH: Cell<Option<Auth>> = const { Cell::new(None) };
    /// The user's full name.
    static NAME: Cell<Option<Signal<String>>> = const { Cell::new(None) };
    /// The lock there is, and the owner of what watches it.
    static LOCK: RefCell<Option<(SessionLock, Owner)>> = const { RefCell::new(None) };
    /// Where PAM's answers come. Made once, by `install`, outside every window: one made while
    /// a panel's button is handled would answer into the panel, which is gone by then.
    static ANSWER: RefCell<Option<UiSender<Result<(), Refused>>>> = const { RefCell::new(None) };
}

/// Whether the session is locked, or locking.
pub fn is_locked() -> bool {
    LOCK.with(|l| l.borrow().is_some())
}

/// Locks the session, if it is not. Call it after `install`.
pub fn lock() {
    if is_locked() {
        return;
    }
    let Some(auth) = AUTH.with(Cell::get) else {
        return;
    };
    batch(|| {
        auth.typed.set(String::new());
        auth.message.set(String::new());
        auth.busy.set(false);
    });
    // In a root of its own: a lock asked for in an effect (the idle timer) outlives the effect.
    let (session, owner) = create_root(move || {
        let session = lock_session(move |output| surface(output, auth));
        // The compositor will not lock, as when something else has: nothing to keep. A refusal
        // left over from before does not count, until the state has been something else.
        let watched = session.clone();
        let stale = Cell::new(untrack(|| watched.state()) == LockState::Refused);
        effect(move || match watched.state() {
            LockState::Refused if !stale.get() => {
                eprintln!("shojiwm-shell: the compositor would not lock the session");
                set_timeout(Duration::ZERO, finish);
            }
            LockState::Refused => {}
            _ => stale.set(false),
        });
        session
    });
    LOCK.with(|l| *l.borrow_mut() = Some((session, owner)));
}

/// Unlocks, once the user has shown who they are.
fn finish() {
    if let Some((session, owner)) = LOCK.with(|l| l.borrow_mut().take()) {
        session.unlock();
        owner.dispose();
    }
}

/// One output's lock surface: the log-on screen on the main screen, black elsewhere.
fn surface(output: Output, auth: Auth) -> Div {
    let output_name = output.name.clone();
    let primary = memo(move || crate::desktop::primary_output_name() == output_name);
    let name = NAME.with(Cell::get);
    let account = Account {
        name: Rc::new(move || name.map(|n| n.get()).unwrap_or_default()),
        picture: Rc::new(user_picture),
    };
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
                if primary {
                    logon_screen(auth, account).into_view()
                } else {
                    screen::blank_screen(auth).into_view()
                }
            })
        }))
}

/// Checks a password with PAM, as the "login" service of `/etc/pam.d` does. Only its success
/// unlocks.
fn authenticate(password: String) {
    let Some(auth) = AUTH.with(Cell::get) else {
        return;
    };
    if auth.busy.get_untracked() || !is_locked() || password.is_empty() {
        return;
    }
    let Some(to_ui) = ANSWER.with(|a| a.borrow().clone()) else {
        return;
    };
    auth.busy.set(true);
    auth.message.set(String::new());
    let user = login_name();
    let started = std::thread::Builder::new()
        .name("shojiwm-shell pam".into())
        .spawn(move || to_ui.send(pam::authenticate(&user, password)));
    if started.is_err() {
        auth.fail(NOT_CHECKED);
    }
}

/// PAM's answer.
fn answered(result: Result<(), Refused>) {
    let Some(auth) = AUTH.with(Cell::get) else {
        return;
    };
    auth.busy.set(false);
    if !is_locked() {
        return;
    }
    match result {
        Ok(()) => {
            auth.message.set(String::new());
            auth.typed.set(String::new());
            finish();
        }
        Err(Refused::WrongPassword) => auth.fail(INCORRECT),
        // Such as faillock's: the account is locked for a while.
        Err(Refused::Told(why)) => auth.fail(why),
        Err(Refused::Other(why)) => {
            eprintln!("shojiwm-shell: PAM: {why}");
            auth.fail(NOT_CHECKED);
        }
    }
}

/// Locks after `lock_after_minutes` without input (0 for never). The timer is made again when
/// the delay changes.
fn watch_idle() {
    let minutes = memo(|| settings().with(|s| s.lock_after_minutes));
    let timer: Signal<Option<Idle>> = signal(None);
    effect(move || {
        let minutes = minutes.get();
        timer.set((minutes > 0).then(|| idle(Duration::from_secs(minutes.max(1) as u64 * 60))));
    });
    effect(move || {
        if timer.with(|t| t.as_ref().is_some_and(Idle::is_idle)) {
            untrack(lock);
        }
    });
}

/// Makes the lock's state, answers `lock` (`lock`, `isLocked`), and locks when logind asks,
/// before the computer sleeps (when the Control Panel says so) and after the idle delay. Call
/// once, from the root.
pub fn install() {
    AUTH.with(|a| a.set(Some(Auth::new(authenticate))));
    NAME.with(|n| n.set(Some(full_name(&login_name()))));
    let to_ui = ui_channel(answered);
    ANSWER.with(|a| *a.borrow_mut() = Some(to_ui));

    crate::ipc::handle("lock", |function, _| match function {
        "lock" => {
            lock();
            Ok(String::new())
        }
        "isLocked" => Ok(is_locked().to_string()),
        _ => Err(format!("no function {function}")),
    });

    // `loginctl lock-session` makes logind send Lock to the session; going to sleep comes from
    // the lid or a key as much as from the Start menu.
    let logind = session();
    logind.on_lock(lock);
    logind.on_prepare_for_sleep(|going| {
        if going && settings().with_untracked(|s| s.lock_on_suspend) {
            lock();
        }
    });

    watch_idle();
}
