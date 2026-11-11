//! The dialogs and the full-screen layers of the shell: the Run dialog (Super+R), "Shut Down
//! Windows" (Alt+F4 on the desktop, Super+Shift+E), the Start orb's menu (a right click on the
//! orb, Super+X), the titles of the compositor's Alt+Tab panel, and the Mystify screen saver.
//!
//! Which screen shows each dialog is kept here, by connector name, in shared signals: any UI
//! thread can open or close one. The layers themselves live on the thread that called
//! [`install`], one set per screen, each opened while its dialog is open on that screen.

mod admin_menu;
mod alt_tab;
mod frame;
mod run;
mod screen_saver;
mod shutdown;

pub use alt_tab::alt_tab_event;

use crate::ipc;
use crate::settings::settings;
use anti_webui::prelude::*;
use anti_webui::shell::lock_state;
use anti_webui::shell::prelude::*;
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::Duration;

static RUN_ON: OnceLock<SharedSignal<String>> = OnceLock::new();
static SHUTDOWN_ON: OnceLock<SharedSignal<String>> = OnceLock::new();
static ADMIN_MENU_ON: OnceLock<SharedSignal<String>> = OnceLock::new();
static SAVER_PREVIEW: OnceLock<SharedSignal<bool>> = OnceLock::new();

/// The screen showing a dialog, by connector name; "" when it is closed.
fn slot(cell: &'static OnceLock<SharedSignal<String>>) -> &'static SharedSignal<String> {
    cell.get_or_init(|| shared_signal(String::new()))
}

/// The screen saver shows now, for the Control Panel's Preview; not kept.
fn saver_preview() -> &'static SharedSignal<bool> {
    SAVER_PREVIEW.get_or_init(|| shared_signal(false))
}

/// `name`, or the main screen when it is "".
fn screen_or_primary(name: &str) -> String {
    if name.is_empty() {
        crate::desktop::primary_output_name()
    } else {
        name.to_owned()
    }
}

// ----- the Run dialog ----------------------------------------------------------------------------

/// Opens the Run dialog on a screen, by name ("": the main screen). The Start menu and the orb's
/// menu close.
pub fn run_open_on(output_name: &str) {
    crate::start_menu::close();
    admin_menu_close();
    slot(&RUN_ON).set(screen_or_primary(output_name));
}

pub fn run_close() {
    slot(&RUN_ON).set(String::new());
}

/// Opens the Run dialog on a screen, or closes it wherever it is open.
pub fn run_toggle_on(output_name: &str) {
    if slot(&RUN_ON).with_untracked(|on| on.is_empty()) {
        run_open_on(output_name);
    } else {
        run_close();
    }
}

// ----- Shut Down Windows -------------------------------------------------------------------------

/// Opens "Shut Down Windows" on a screen, by name ("": the main screen). The Start menu closes.
pub fn shutdown_open_on(output_name: &str) {
    crate::start_menu::close();
    slot(&SHUTDOWN_ON).set(screen_or_primary(output_name));
}

pub fn shutdown_close() {
    slot(&SHUTDOWN_ON).set(String::new());
}

// ----- the Start orb's menu ----------------------------------------------------------------------

/// Opens the orb's menu on a screen, by name ("": the main screen), or closes it when it is open
/// there. The Start menu closes.
pub fn admin_menu_toggle_on(output_name: &str) {
    let name = screen_or_primary(output_name);
    crate::start_menu::close();
    let on = slot(&ADMIN_MENU_ON);
    let open_here = on.with_untracked(|o| *o == name);
    on.set(if open_here { String::new() } else { name });
}

pub fn admin_menu_close() {
    slot(&ADMIN_MENU_ON).set(String::new());
}

// ----- the screen saver --------------------------------------------------------------------------

/// Shows the screen saver now, until the mouse moves: the Control Panel's Preview button.
pub fn preview_screen_saver() {
    saver_preview().set(true);
}

// ----- the layers --------------------------------------------------------------------------------

/// Opens a layer over all of `output` while `open` says so, with `view` in it.
fn layer_while<V: IntoView>(
    output: &Output,
    open: impl Fn() -> bool + 'static,
    options: impl Fn() -> LayerOptions + 'static,
    view: impl Fn(Output) -> V + 'static,
) {
    let shown = memo(open);
    let output = output.clone();
    let view = Rc::new(view);
    // Opened in the effect, the layer closes when the effect runs again, as the dialog closes.
    effect(move || {
        if !shown.get() {
            return;
        }
        let inside = output.clone();
        let view = view.clone();
        let options = untrack(&options).output(&output);
        open_layer(options, move || view(inside));
    });
}

/// A layer over the whole screen, over everything, for a dialog that takes the keyboard.
fn dialog_layer(namespace: &'static str) -> LayerOptions {
    LayerOptions::new()
        .layer(Layer::Overlay)
        .anchor(Anchor::ALL)
        .exclusive(Exclusive::Ignore)
        .keyboard(Keyboard::Exclusive)
        .namespace(namespace)
}

/// A layer over the whole screen, over everything, that takes no keys.
fn overlay_layer() -> LayerOptions {
    LayerOptions::new()
        .layer(Layer::Overlay)
        .anchor(Anchor::ALL)
        .exclusive(Exclusive::Ignore)
        .keyboard(Keyboard::None)
        .namespace("no_blur")
}

/// Opens the dialogs' layers on every screen as they are asked for, follows the idle time for the
/// screen saver, and answers the IPC targets `run`, `shutdown` and `adminmenu`. Call once, from
/// the root.
pub fn install() {
    install_ipc();
    let idle_now = follow_idle();
    for_each_output(move |output| {
        let name = output.name.clone();

        let here = name.clone();
        layer_while(
            &output,
            move || slot(&RUN_ON).with(|o| *o == here),
            || dialog_layer("shojiwm-desktop-run"),
            |_| run::view(),
        );

        let here = name.clone();
        layer_while(
            &output,
            move || slot(&SHUTDOWN_ON).with(|o| *o == here),
            || dialog_layer("shojiwm-desktop-shutdown"),
            |_| shutdown::view(),
        );

        let here = name.clone();
        layer_while(
            &output,
            move || slot(&ADMIN_MENU_ON).with(|o| *o == here),
            || dialog_layer("shojiwm-desktop-adminmenu"),
            |output| admin_menu::view(output.name),
        );

        let here = name.clone();
        layer_while(
            &output,
            move || alt_tab::shown_on(&here),
            // The panel's grab has the keyboard and the pointer.
            || overlay_layer().input_region(InputRegion::Empty),
            |output| alt_tab::view(output.name),
        );

        layer_while(
            &output,
            move || {
                let chosen = settings().with(|s| s.screensaver != "none");
                let unlocked = matches!(lock_state(), LockState::Unlocked | LockState::Refused);
                chosen && (idle_now.get() || saver_preview().get()) && unlocked
            },
            overlay_layer,
            screen_saver::view,
        );
    });
}

/// Whether nothing has been touched for the screen saver's minutes, while a screen saver is
/// chosen. The watch is made again when the minutes change.
fn follow_idle() -> Signal<bool> {
    let idle_now = signal(false);
    effect(move || {
        let (chosen, minutes) =
            settings().with(|s| (s.screensaver != "none", s.screensaver_minutes));
        if !chosen {
            idle_now.set(false);
            return;
        }
        let away = idle(Duration::from_secs(minutes.max(1) as u64 * 60));
        effect(move || idle_now.set(away.is_idle()));
    });
    idle_now
}

fn first(args: &[String]) -> &str {
    args.first().map(String::as_str).unwrap_or("")
}

fn install_ipc() {
    ipc::handle("run", |function, args| {
        match function {
            "open" => run_open_on(""),
            "openOn" => run_open_on(first(args)),
            "toggleOn" => run_toggle_on(first(args)),
            "close" => run_close(),
            _ => return Err(format!("no function {function}")),
        }
        Ok(String::new())
    });
    ipc::handle("shutdown", |function, args| {
        match function {
            "openOn" => shutdown_open_on(first(args)),
            "close" => shutdown_close(),
            _ => return Err(format!("no function {function}")),
        }
        Ok(String::new())
    });
    ipc::handle("adminmenu", |function, args| {
        match function {
            "toggle" => admin_menu_toggle_on(""),
            "toggleOn" => admin_menu_toggle_on(first(args)),
            "close" => admin_menu_close(),
            _ => return Err(format!("no function {function}")),
        }
        Ok(String::new())
    });
}
