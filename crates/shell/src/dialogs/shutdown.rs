//! "Shut Down Windows", as Windows 7 shows it after Alt+F4 on the desktop (QML
//! `ShutdownDialog`): the screen dims (the compositor blurs what is under the layer), and a small
//! window asks what the computer should do, from a list of the power actions.
//!
//! The keys: Tab and Shift+Tab move between the list, OK and Cancel. On the list, Up and Down
//! change the choice, and Alt+Down, F4 and Space open it, as in a Windows combo box. Return does
//! what has the keyboard (or picks in the open list); Space presses a button. Escape closes the
//! list, then the dialog.

use super::frame::{button_band, dialog_frame, dialog_page};
use super::power::{POWER_ACTIONS, PowerAction, login_name, run_power};
use super::shutdown_close;
use crate::paths::asset;
use crate::settings::{home, settings};
use crate::theme::{corner, scheme, theme};
use crate::ui::{cp_button, cp_default_button, font, ink_asset, vertical_gradient};
use anti_webui::prelude::*;
use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

/// The dimmed screen round the dialog.
const DIM: Color = rgba(0x00000073);
/// The width of the dialog.
const WIDTH: f32 = 440.0;
/// The banner: its height, its deep blue glass (top, middle, bottom), and the sweep of light
/// across it, soft at both edges.
const BANNER_HEIGHT: f32 = 64.0;
const BANNER: [Color; 3] = [rgb(0x1c4f86), rgb(0x0f3768), rgb(0x0a2850)];
const SWEEP: [Color; 3] = [rgba(0x7fc0ff00), rgba(0x7fc0ff40), rgba(0xffffff00)];
/// The banner's inside width: the dialog less its frame, the page's margins and outline.
const BANNER_WIDTH: f32 = WIDTH - 2.0 - 14.0 - 2.0;
/// The user's name on the banner: white, with a darker copy a pixel under it.
const NAME_SIZE: f32 = 22.0;
const NAME_SHADOW: Color = rgba(0x00000040);
/// The field, and a row of the open list.
const FIELD_HEIGHT: f32 = 24.0;
const LIST_ROW_HEIGHT: f32 = 22.0;
const DROP_WIDTH: f32 = 18.0;
/// The choice in a focused combo box, and the lit row of its list: Windows' blue.
const PICKED: Color = rgb(0x3399ff);
const PICKED_TEXT: Color = Color::WHITE;
/// The band of buttons and the line along its top.
const BAND_DARK: Color = rgb(0x1a232c);
const BAND_LIGHT: Color = rgb(0xf0f0f0);
const BAND_LINE_DARK: Color = rgba(0xffffff33);
const BAND_LINE_LIGHT: Color = rgb(0xdfdfdf);
/// A touchpad steps the choice once per this many pixels.
const WHEEL_STEP: f32 = 40.0;

fn description(id: &str) -> &'static str {
    match id {
        "shutdown" => "Closes all open programs and turns off the computer.",
        "restart" => "Closes all open programs, turns off the computer, and then turns it on again.",
        "sleep" => "Keeps your session in memory and puts the computer in a low-power state.",
        "hibernate" => "Saves your session to the disk and turns off the computer.",
        "logoff" => "Closes all open programs and logs you off.",
        "lock" => "Locks the computer. Your programs keep running.",
        _ => "",
    }
}

/// The power actions, the one chosen in the Control Panel first (else Log off).
fn actions() -> Vec<PowerAction> {
    let wanted = untrack(|| settings().with(|s| s.shutdown_dialog_action.clone()));
    let first = if POWER_ACTIONS.iter().any(|a| a.id == wanted) { wanted } else { "logoff".into() };
    let mut list = POWER_ACTIONS.to_vec();
    list.sort_by_key(|a| a.id != first);
    list
}

/// What has the keyboard.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    List,
    Ok,
    Cancel,
}

impl Focus {
    fn step(self, by: i32) -> Self {
        let all = [Focus::List, Focus::Ok, Focus::Cancel];
        let at = all.iter().position(|f| *f == self).unwrap_or(0) as i32;
        all[(at + by).rem_euclid(3) as usize]
    }
}

#[derive(Clone, Copy)]
struct State {
    /// The index of the chosen action.
    current: Signal<usize>,
    /// The list of actions is open under the field.
    list_open: Signal<bool>,
    focus: Signal<Focus>,
    count: usize,
}

impl State {
    fn step(&self, by: i32) {
        let last = self.count.saturating_sub(1) as i32;
        self.current
            .update(|c| *c = (*c as i32 + by).clamp(0, last) as usize);
    }
}

/// The dialog over its dimmed screen. A press on the dim closes it.
pub fn view() -> Div {
    let actions = Rc::new(actions());
    let state = State {
        current: signal(0usize),
        list_open: signal(false),
        focus: signal(Focus::List),
        count: actions.len(),
    };
    let accept = {
        let actions = actions.clone();
        Rc::new(move || {
            let chosen = actions.get(state.current.get_untracked()).map(|a| a.id);
            shutdown_close();
            if let Some(id) = chosen {
                run_power(id);
            }
        })
    };
    let confirm = {
        let accept = accept.clone();
        move || {
            if state.list_open.get_untracked() {
                state.list_open.set(false);
            } else if state.focus.get_untracked() == Focus::Cancel {
                shutdown_close();
            } else {
                accept();
            }
        }
    };
    let keys = node_ref();
    request_frame(move |_| keys.focus());

    let page = dialog_page()
        .child(banner())
        .child(content(state, actions.clone(), keys))
        .child(button_band(
            (BAND_DARK, BAND_LIGHT),
            Some((BAND_LINE_DARK, BAND_LINE_LIGHT)),
            buttons(state, accept),
        ));
    let frame = dialog_frame("Shut Down Windows", WIDTH, page).on_pointer_down(move |e| {
        e.stop_propagation();
        state.list_open.set(false);
    });

    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(DIM)
        .node_ref(keys)
        .focusable()
        .on_pointer_down(|_| shutdown_close())
        .on_key_down(move |e| {
            let on_list = state.focus.get_untracked() == Focus::List;
            match e.key {
                Key::Named(NamedKey::Tab) => {
                    state.list_open.set(false);
                    state.focus.update(|f| *f = f.step(if e.modifiers.shift { -1 } else { 1 }));
                }
                Key::Named(NamedKey::ArrowUp) if on_list => state.step(-1),
                Key::Named(NamedKey::ArrowDown) if on_list => {
                    if e.modifiers.alt {
                        state.list_open.update(|o| *o = !*o);
                    } else {
                        state.step(1);
                    }
                }
                Key::Named(NamedKey::Enter) => confirm(),
                Key::Named(NamedKey::Space) => {
                    if on_list {
                        state.list_open.update(|o| *o = !*o);
                    } else {
                        confirm();
                    }
                }
                Key::Named(NamedKey::F(4)) if on_list && !e.modifiers.alt => {
                    state.list_open.update(|o| *o = !*o);
                }
                Key::Named(NamedKey::Escape) => {
                    if state.list_open.get_untracked() {
                        state.list_open.set(false);
                    } else {
                        shutdown_close();
                    }
                }
                _ => return,
            }
            e.prevent_default();
            e.stop_propagation();
        })
        .child(frame)
}

/// OK and Cancel; the one with the keyboard has the line inside it.
fn buttons(state: State, accept: Rc<dyn Fn()>) -> View {
    dyn_view(move || {
        let focus = state.focus.get();
        let ok = {
            let accept = accept.clone();
            move || accept()
        };
        let ok = if focus == Focus::Ok {
            cp_default_button("OK", ok)
        } else {
            cp_button("OK", ok)
        };
        let cancel = if focus == Focus::Cancel {
            cp_default_button("Cancel", shutdown_close)
        } else {
            cp_button("Cancel", shutdown_close)
        };
        div().flex().flex_row().gap(8.0).child(ok).child(cancel)
    })
}

/// The banner: deep blue glass with a light sweep, the account picture and the user's name.
fn banner() -> Div {
    let full_name = full_name();
    let sweep = div()
        .absolute()
        .left(-40.0)
        .top(30.0)
        .w(BANNER_WIDTH * 1.4)
        .h(90.0)
        .rounded(45.0)
        .rotate(-6.0)
        .fill(|| vertical_gradient(&[(0.0, SWEEP[0]), (0.12, SWEEP[1]), (0.55, SWEEP[2])]));
    let name = move || {
        full_name
            .get()
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| "Shut Down".into())
    };
    let name = Rc::new(name);
    let shadow_name = name.clone();
    div()
        .relative()
        .flex()
        .flex_row()
        .items_center()
        .flex_none()
        .h(BANNER_HEIGHT)
        .overflow_clip()
        .fill(|| {
            vertical_gradient(&[(0.0, BANNER[0]), (0.55, BANNER[1]), (1.0, BANNER[2])])
        })
        .child(sweep)
        .child(user_tile(48.0).ml(16.0))
        .child(
            div()
                .relative()
                .ml(12.0)
                .whitespace_nowrap()
                .text_size(NAME_SIZE)
                .font_weight(FontWeight::LIGHT)
                .dyn_style(|s| s.font_family(font()))
                .child(
                    div()
                        .absolute()
                        .left(0.0)
                        .top(1.0)
                        .text_color(NAME_SHADOW)
                        .child(text(move || shadow_name())),
                )
                .child(
                    div()
                        .relative()
                        .text_color(Color::WHITE)
                        .child(text(move || name())),
                ),
        )
}

/// The question, the combo box of actions with its list, and what the chosen one does.
fn content(state: State, actions: Rc<Vec<PowerAction>>, keys: NodeRef) -> Div {
    let labels = actions.clone();
    let described = actions.clone();
    div()
        .flex()
        .flex_col()
        .gap(10.0)
        .pt(16.0)
        .px(20.0)
        .pb(12.0)
        .child(
            div()
                .dyn_style(|s| {
                    s.text_color(scheme().page_heading)
                        .text_size(theme().font_size + 2.0)
                })
                .child("What do you want the computer to do?"),
        )
        .child(
            div()
                .relative()
                .flex_none()
                .h(FIELD_HEIGHT)
                .z_index(10)
                .child(field(state, labels, keys))
                .child(list(state, actions, keys)),
        )
        .child(div().min_h(34.0).child(text(move || {
            described
                .get(state.current.get())
                .map(|a| description(a.id))
                .unwrap_or("")
                .to_owned()
        })))
}

/// The combo box: the choice in a field, selected while the list has the keyboard, and the drop
/// button at its right end.
fn field(state: State, actions: Rc<Vec<PowerAction>>, keys: NodeRef) -> Div {
    let ui = Interaction::new();
    let wheel = Rc::new(Cell::new(0.0f32));
    let selected = move || !state.list_open.get() && state.focus.get() == Focus::List;
    let choice = div()
        .flex_1()
        .min_w(0.0)
        .self_stretch()
        .m(2.0)
        .pl(4.0)
        .flex()
        .items_center()
        .overflow_clip()
        .whitespace_nowrap()
        .dyn_style(move |s| {
            let s = s.rounded(corner(1.0));
            if selected() {
                s.bg(PICKED).text_color(PICKED_TEXT)
            } else {
                s.bg(Color::TRANSPARENT).text_color(scheme().text)
            }
        })
        .child(text(move || {
            actions
                .get(state.current.get())
                .map(|a| a.label)
                .unwrap_or("")
                .to_owned()
        }));
    let drop_button = div()
        .flex_none()
        .w(DROP_WIDTH)
        .self_stretch()
        .flex()
        .items_center()
        .justify_center()
        .fill(move || {
            let c = scheme();
            let [top, bottom] = if state.list_open.get() {
                [c.button_pressed_top, c.button_pressed_bottom]
            } else if ui.hovered() {
                [c.button_hot_top, c.button_hot_bottom]
            } else {
                [c.field, c.field]
            };
            vertical_gradient(&[(0.0, top), (1.0, bottom)])
        })
        .border(1.0)
        .dyn_style(move |s| {
            s.border_color(if ui.hovered() || state.list_open.get() {
                scheme().selection_border
            } else {
                Color::TRANSPARENT
            })
        })
        .child(
            img(|| ink_asset("combo-arrow.svg"))
                .w(8.0)
                .h(5.0)
                .object_fit(ObjectFit::Contain),
        );
    div()
        .flex()
        .flex_row()
        .size_full()
        .border(1.0)
        .interaction(ui)
        .dyn_style(move |s| {
            let c = scheme();
            s.rounded(corner(2.0)).bg(c.field).border_color(
                if ui.hovered() || state.list_open.get() { c.field_focus } else { c.field_border },
            )
        })
        // On the press: the frame closes the list on it.
        .on_pointer_down(move |e| {
            e.stop_propagation();
            state.list_open.update(|o| *o = !*o);
            keys.focus();
        })
        .on_scroll(move |e| {
            let steps = if e.source == ScrollSource::Wheel {
                e.delta_y.signum() as i32
            } else {
                let total = wheel.get() + e.delta_y;
                let steps = (total / WHEEL_STEP).trunc();
                wheel.set(total - steps * WHEEL_STEP);
                steps as i32
            };
            if steps != 0 {
                e.stop_propagation();
                state.step(steps);
            }
        })
        .child(choice)
        .child(drop_button)
}

/// The open list, over the description and the band.
fn list(state: State, actions: Rc<Vec<PowerAction>>, keys: NodeRef) -> Show {
    show(
        move || state.list_open.get(),
        move || {
            let hovered = signal(false);
            let rows: Vec<Div> = actions
                .iter()
                .enumerate()
                .map(|(i, action)| list_row(state, i, action.label, hovered, keys))
                .collect();
            div()
                .absolute()
                .left(0.0)
                .right(0.0)
                .top(FIELD_HEIGHT)
                .flex()
                .flex_col()
                .p(1.0)
                .border(1.0)
                .on_hover(move |on| hovered.set(on))
                .dyn_style(|s| {
                    let c = scheme();
                    s.bg(c.field).border_color(c.field_focus)
                })
                .children(rows)
        },
    )
}

fn list_row(
    state: State,
    index: usize,
    label: &'static str,
    list_hovered: Signal<bool>,
    keys: NodeRef,
) -> Div {
    let ui = Interaction::new();
    let lit = move || ui.hovered() || (!list_hovered.get() && state.current.get() == index);
    div()
        .flex_none()
        .flex()
        .items_center()
        .h(LIST_ROW_HEIGHT)
        .pl(6.0)
        .whitespace_nowrap()
        .interaction(ui)
        .on_pointer_down(move |e| {
            e.stop_propagation();
            state.current.set(index);
            state.list_open.set(false);
            keys.focus();
        })
        .dyn_style(move |s| {
            if lit() {
                s.bg(PICKED).text_color(PICKED_TEXT)
            } else {
                s.bg(Color::TRANSPARENT).text_color(scheme().text)
            }
        })
        .child(label)
}

// ----- the account picture -----------------------------------------------------------------------

/// The frame of the account picture on smoked glass (QML `UserTile` with `dark`), drawn at
/// `size`: an outer line, a highlight, a band of glass, a light then a dark line, and the
/// picture.
const TILE_SHADOW: Color = rgba(0x001a3338);
const TILE_OUTER: Color = rgb(0x06121c);
const TILE_HIGHLIGHT: Color = rgb(0x7f9db4);
/// The smoked glass, top to bottom. The QML's six stops, in the four a gradient here takes.
const TILE_GLASS: [(f32, Color); 4] = [
    (0.0, rgb(0xa9bccb)),
    (0.35, rgb(0x4f6678)),
    (0.9, rgb(0x1f3a52)),
    (1.0, rgb(0x2a5170)),
];
const TILE_LIGHT_LINE: Color = rgb(0x5f7d95);
const TILE_DARK_LINE: Color = rgb(0x0e1a24);

fn user_tile(size: f32) -> Div {
    let k = size / 136.0;
    let band = (10.0 * k).round().max(3.0);
    let r = (8.0 * k).max(3.0);
    let layer = |inset: f32| div().absolute().inset(inset);
    let picture = memo(user_picture);
    div()
        .relative()
        .flex_none()
        .size(size)
        .child(
            div()
                .absolute()
                .left(-2.0)
                .right(-2.0)
                .top(0.0)
                .bottom(-(3.0 * k).round().max(1.0))
                .bg(TILE_SHADOW)
                .dyn_style(move |s| s.rounded(corner(r + 2.0))),
        )
        .child(
            layer(0.0)
                .bg(TILE_OUTER)
                .dyn_style(move |s| s.rounded(corner(r))),
        )
        .child(
            layer(1.0)
                .bg(TILE_HIGHLIGHT)
                .dyn_style(move |s| s.rounded(corner((r - 1.0).max(0.0)))),
        )
        .child(
            layer(2.0)
                .fill(|| vertical_gradient(&TILE_GLASS))
                .dyn_style(move |s| s.rounded(corner((r - 2.0).max(0.0)))),
        )
        .child(
            layer(band - 2.0)
                .bg(TILE_LIGHT_LINE)
                .dyn_style(move |s| s.rounded(corner((2.0 * k).max(1.0)))),
        )
        .child(layer(band - 1.0).bg(TILE_DARK_LINE))
        .child(
            layer(band)
                .overflow_clip()
                .child(img(move || picture.get()).size_full().object_fit(ObjectFit::Cover)),
        )
}

/// The account picture: the one chosen in the Control Panel, else the system's (AccountsService,
/// `~/.face`, `~/.face.icon`), else the built-in one.
fn user_picture() -> PathBuf {
    let chosen = settings().with(|s| s.user_picture.clone());
    if !chosen.is_empty() {
        return PathBuf::from(chosen);
    }
    let mut places = vec![PathBuf::from("/var/lib/AccountsService/icons").join(login_name())];
    if let Some(home) = home() {
        places.push(home.join(".face"));
        places.push(home.join(".face.icon"));
    }
    places
        .into_iter()
        .find(|p| p.is_file())
        .unwrap_or_else(|| asset("user.svg"))
}

/// The full name from the password database, else the login name.
fn full_name() -> Task<String> {
    let login = login_name();
    task(move || {
        let full = std::process::Command::new("getent")
            .args(["passwd", &login])
            .output()
            .ok()
            .and_then(|out| {
                let line = String::from_utf8_lossy(&out.stdout).trim().to_owned();
                // The GECOS field: the full name first, then extras nobody fills in.
                let gecos = line.split(':').nth(4)?.split(',').next()?.trim().to_owned();
                (!gecos.is_empty()).then_some(gecos)
            });
        full.unwrap_or(login)
    })
}
