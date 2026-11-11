//! The shell's own icons of the notification area (Action Center, notifications, Bluetooth,
//! network, volume), the clock, and the panels they open, with the services behind them.
//!
//! At most one panel is open on the whole desktop, on one output: opening one closes the
//! others, as the QML taskbar's `closePopups` did. Each panel is a popup of its icon, placed
//! away from the bar; a press outside of it closes it.

mod action_center;
mod audio;
mod bluetooth;
mod calendar;
mod locale;
mod network;
mod notifications;

use crate::settings::motion;
use crate::theme::{Align, away_from_bar, corner, item_radius, popup_radius, scheme, theme};
use crate::ui::{dark_look, font};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use std::cell::{Cell, RefCell};
use std::time::Duration;

/// The panels of the notification area and of the clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Panel {
    ActionCenter,
    Notifications,
    Bluetooth,
    Network,
    Volume,
    Calendar,
}

/// The panel that is open, and the screen it is on.
type Open = Option<(String, Panel)>;

thread_local! {
    static OPEN: Cell<Option<Signal<Open>>> = const { Cell::new(None) };
}

/// The panel open now, and the output it is open on.
fn open_state() -> Signal<Option<(String, Panel)>> {
    OPEN.with(|cell| {
        cell.get().unwrap_or_else(|| {
            let (s, _) = create_root(|| signal(None));
            cell.set(Some(s));
            s
        })
    })
}

/// `panel` is open on the output `output_name`. Reading it tracks it.
pub fn is_open(output_name: &str, panel: Panel) -> bool {
    open_state().with(|s| {
        s.as_ref()
            .is_some_and(|(o, p)| o == output_name && *p == panel)
    })
}

/// Opens `panel` on `output_name`, and closes any other.
pub fn open(output_name: &str, panel: Panel) {
    open_state().set(Some((output_name.to_owned(), panel)));
}

/// Opens `panel` on `output_name`, or closes it when it is the one open there.
pub fn toggle(output_name: &str, panel: Panel) {
    if untrack(|| is_open(output_name, panel)) {
        open_state().set(None);
    } else {
        open(output_name, panel);
    }
}

/// Closes `panel`, wherever it is open.
pub fn close(panel: Panel) {
    if untrack(|| open_state().with(|s| s.as_ref().is_some_and(|(_, p)| *p == panel))) {
        open_state().set(None);
    }
}

/// Closes the panel that is open, if any. For the taskbar, when it opens a popup of its own.
pub fn close_popups() {
    if untrack(|| open_state().with(Option::is_some)) {
        open_state().set(None);
    }
}

/// A panel of this module is open on `output_name`: an auto-hidden bar stays up meanwhile.
/// Reading it tracks it.
pub fn popup_open_on(output_name: &str) -> bool {
    open_state().with(|s| s.as_ref().is_some_and(|(o, _)| o == output_name))
}

/// The shell's own icons of the notification area, in the taskbar's order: Action Center,
/// notifications, Bluetooth, network, volume.
pub fn area(output: Output) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .flex_none()
        .dyn_style(|s| s.gap(theme().tray_gap))
        .child(action_center::icon(&output.name))
        .child(notifications::icon(&output.name))
        .child(bluetooth::icon(&output.name))
        .child(network::icon(&output.name))
        .child(audio::icon(&output.name))
}

/// The clock of the taskbar, with its calendar.
pub fn clock(output: Output) -> Div {
    calendar::clock_view(&output.name)
}

/// Starts the services (the notification history, the Action Center's checks, the additional
/// clocks, the locale) and answers the IPC targets `volume`, `bluetooth` and `actioncenter`.
/// Call once, from the root: the timers that refresh the services belong to the window they
/// are set in.
pub fn install() {
    locale::install();
    notifications::install();
    action_center::install();
    calendar::install();
    crate::ipc::handle("volume", |function, _| match function {
        "open" => {
            open(&crate::desktop::primary_output_name(), Panel::Volume);
            audio::set_devices_open(false);
            Ok(String::new())
        }
        "openDevices" => {
            open(&crate::desktop::primary_output_name(), Panel::Volume);
            audio::set_devices_open(true);
            Ok(String::new())
        }
        "close" => {
            close(Panel::Volume);
            Ok(String::new())
        }
        _ => Err(format!("no function {function}")),
    });
    crate::ipc::handle("bluetooth", |function, args| match function {
        "open" => {
            open(&crate::desktop::primary_output_name(), Panel::Bluetooth);
            Ok(String::new())
        }
        "openOn" => {
            let screen = args.first().cloned().unwrap_or_default();
            let screen = if screen.is_empty() {
                crate::desktop::primary_output_name()
            } else {
                screen
            };
            open(&screen, Panel::Bluetooth);
            Ok(String::new())
        }
        "close" => {
            close(Panel::Bluetooth);
            Ok(String::new())
        }
        _ => Err(format!("no function {function}")),
    });
    crate::ipc::handle("actioncenter", |function, _| match function {
        "open" => {
            open(&crate::desktop::primary_output_name(), Panel::ActionCenter);
            Ok(String::new())
        }
        "close" => {
            close(Panel::ActionCenter);
            Ok(String::new())
        }
        _ => Err(format!("no function {function}")),
    });
}

// ----- the pieces the icons and panels share ---------------------------------------------------

/// The white veil over an icon of the bar: stronger while its panel is open than under the
/// pointer (QML: a white rectangle at these opacities).
const VEIL_OPEN: f32 = 0.22;
const VEIL_HOVER: f32 = 0.14;

/// The band at the bottom of a panel, under its link: a shade on dark colours, a light veil on
/// light ones (QML `#33000000`, `#80ffffff`).
const FOOTER_DARK: Color = rgba(0x00000033);
const FOOTER_LIGHT: Color = rgba(0xffffff80);

/// The red of an error's text and of a notification's cross under the pointer.
pub(crate) const ERROR_RED: Color = rgb(0xd9302a);

/// The box of an icon or of the clock on the bar: `width` wide, as tall as the bar's buttons,
/// with the white veil, `inset` from its top and bottom, that lights it under the pointer and
/// while `active` (its panel is open). Its content is centred.
fn bar_cell(
    width: impl Fn() -> f32 + 'static,
    inset: f32,
    active: impl Fn() -> bool + 'static,
) -> Div {
    let ui = Interaction::new();
    let length = untrack(|| motion(theme().fade));
    let veil = follow(
        move || {
            if active() {
                VEIL_OPEN
            } else if ui.hovered() {
                VEIL_HOVER
            } else {
                0.0
            }
        },
        Transition::ms(length).ease_out(),
    );
    div()
        .relative()
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .select_none()
        .interaction(ui)
        .dyn_style(move |s| {
            let t = theme();
            s.w(width()).h(t.taskbar_height - t.taskbar_padding * 2.0)
        })
        .child(
            div()
                .absolute()
                .left(0.0)
                .right(0.0)
                .top(inset)
                .bottom(inset)
                .pointer_events_none()
                .dyn_style(move |s| {
                    s.rounded(item_radius())
                        .bg(Color::WHITE.with_alpha(veil.get().clamp(0.0, 1.0)))
                }),
        )
}

/// The room of a tray icon on the bar (QML `trayIconSize + 8`).
fn icon_width() -> f32 {
    theme().tray_icon_size + 8.0
}

/// A picture of the bar's icons, `grow` pixels over the theme's tray icon size.
fn bar_picture(path: impl Fn() -> std::path::PathBuf + 'static, grow: f32) -> Div {
    div()
        .relative()
        .flex_none()
        .dyn_style(move |s| s.size(theme().tray_icon_size + grow))
        .child(img(path).w_full().h_full().object_fit(ObjectFit::Contain))
}

/// The badge at the bottom right of an icon (a red or yellow mark), shown while `shown`.
fn badge(
    path: impl Fn() -> std::path::PathBuf + 'static,
    shown: impl Fn() -> bool + 'static,
) -> Div {
    div()
        .absolute()
        .right(-2.0)
        .bottom(-1.0)
        .size(10.0)
        .pointer_events_none()
        .dyn_style(move |s| if shown() { s } else { s.hidden() })
        .child(img(path).size(10.0).object_fit(ObjectFit::Contain))
}

/// A panel of `output_name`, shown while it is open there: `content` in a popup of the element
/// this is a child of, away from the bar and lined up by `align`.
fn panel_popup(
    output_name: &str,
    panel: Panel,
    align: Align,
    content: impl Fn() -> Div + 'static,
) -> Show {
    let name = output_name.to_owned();
    show(
        move || is_open(&name, panel),
        move || {
            let t = untrack(theme);
            popup()
                .placement(untrack(|| away_from_bar(align)))
                .gap(t.popup_gap as i32 + 2)
                .on_dismiss(move || close(panel))
                // The icon it is declared in has nothing to do with what is pressed here.
                .child(
                    content()
                        .on_pointer_down(|e| e.stop_propagation())
                        .on_click(|e| e.stop_propagation())
                        .on_aux_click(|e| e.stop_propagation()),
                )
        },
    )
}

/// The steps of a wheel or a touchpad, up positive: one per notch, or per 40 px of a touchpad.
fn wheel_steps(e: &ScrollEvent, rest: &Cell<f32>) -> i32 {
    const TOUCHPAD_STEP: f32 = 40.0;
    if e.source == ScrollSource::Wheel {
        return -(e.delta_y.signum() as i32);
    }
    let total = rest.get() + e.delta_y;
    let steps = (total / TOUCHPAD_STEP).trunc();
    rest.set(total - steps * TOUCHPAD_STEP);
    -(steps as i32)
}

/// Small text in the theme's face, in `color`.
fn small_text(label: impl IntoTextSource, color: impl Fn() -> Color + 'static) -> Text {
    text(label).dyn_style(move |s| {
        s.font_family(font())
            .text_size(theme().font_size_small)
            .text_color(color())
    })
}

/// Text in the theme's face and size, in `color`.
fn body_text(label: impl IntoTextSource, color: impl Fn() -> Color + 'static) -> Text {
    text(label).dyn_style(move |s| {
        s.font_family(font())
            .text_size(theme().font_size)
            .text_color(color())
    })
}

/// A line of text cut at the end of its box rather than wrapped.
fn one_line(content: impl IntoView) -> Div {
    div()
        .min_w(0.0)
        .overflow_clip()
        .whitespace_nowrap()
        .child(content)
}

/// A link of a panel: in `link`, underlined under the pointer. `small` in the small size.
fn panel_link(label: impl IntoTextSource, small: bool, on_click: impl Fn() + 'static) -> Div {
    let ui = Interaction::new();
    div()
        .flex_none()
        .whitespace_nowrap()
        .select_none()
        .cursor(CursorIcon::Pointer)
        .interaction(ui)
        .on_click(move |e| {
            e.stop_propagation();
            on_click();
        })
        .dyn_style(move |s| {
            let t = theme();
            let s = s
                .font_family(font())
                .text_size(if small {
                    t.font_size_small
                } else {
                    t.font_size
                })
                .text_color(scheme().link);
            if ui.hovered() { s.underline() } else { s }
        })
        .child(text(label))
}

/// A line across a panel, between its parts.
fn divider() -> Div {
    div()
        .flex_none()
        .h(1.0)
        .self_stretch()
        .dyn_style(|s| s.bg(scheme().menu_divider))
}

/// The band at the bottom of a panel, `height` tall, with a line at its top and `content`
/// centred in it (`centred`) or at its left.
fn footer(height: f32, centred: bool, content: impl IntoView) -> Div {
    div()
        .relative()
        .flex_none()
        .flex()
        .flex_row()
        .items_center()
        .h(height)
        .dyn_style(move |s| {
            let s = s
                .rounded_b(corner((popup_radius() - 1.0).max(0.0)))
                .bg(if dark_look() {
                    FOOTER_DARK
                } else {
                    FOOTER_LIGHT
                });
            if centred {
                s.justify_center()
            } else {
                s.justify_start().pl(10.0)
            }
        })
        .child(
            div()
                .absolute()
                .top(0.0)
                .left(0.0)
                .right(0.0)
                .h(1.0)
                .dyn_style(|s| s.bg(scheme().menu_divider)),
        )
        .child(content)
}

/// A row of a panel's list as Windows 7 lights it: the selection's colours while `chosen`, the
/// hover's under the pointer, nothing else. `ui` is the row's interaction.
fn lit_row_style(s: Style, chosen: bool, ui: Interaction) -> Style {
    let c = scheme();
    let s = s.rounded(item_radius()).border(1.0);
    if chosen {
        s.bg(c.selection).border_color(c.selection_border)
    } else if ui.hovered() {
        s.bg(c.hover).border_color(c.selection_border)
    } else {
        s.bg(Color::TRANSPARENT).border_color(Color::TRANSPARENT)
    }
}

/// An icon of the icon theme or a file, or `fallback` when the theme has no `name`
/// (Quickshell's `iconPath(name, fallback)`).
fn icon_or(name: &str, fallback: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        return fallback.to_owned();
    }
    if name.starts_with('/') || name.starts_with("file://") || name.starts_with("image://") {
        return name.to_owned();
    }
    if anti_webui::shell::icon_path(name, 48).is_some() {
        name.to_owned()
    } else {
        fallback.to_owned()
    }
}

/// Runs a command to its end and gives `then` the lines it printed. The command's signals
/// live in a root of their own, which goes once it has ended, so that a command run every
/// minute keeps nothing behind.
fn collect<I, S>(program: &str, args: I, then: impl FnOnce(Vec<String>) + 'static)
where
    I: IntoIterator<Item = S>,
    S: Into<std::ffi::OsString>,
{
    let process = Process::new(program).args(args);
    let (running, owner) = create_root(move || process.spawn());
    let then = RefCell::new(Some(then));
    running.on_exit(move |_| {
        let lines = untrack(|| running.lines());
        if let Some(then) = then.borrow_mut().take() {
            then(lines);
        }
        owner.dispose();
    });
}

/// Runs `f` every `every`, from now on, for as long as the shell runs.
fn every(every: Duration, f: impl Fn() + 'static) {
    fn again(every: Duration, f: std::rc::Rc<dyn Fn()>) {
        set_timeout(every, move || {
            f();
            again(every, f);
        });
    }
    again(every, std::rc::Rc::new(f));
}

/// A string as one word of `sh`.
fn shell_quote(word: &str) -> String {
    format!("'{}'", word.replace('\'', "'\\''"))
}
