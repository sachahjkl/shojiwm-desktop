//! The taskbar (QML `Taskbar.qml` and its layer in `shell.qml`): the Start button, the pinned
//! programs and one button per running program, the notification area, the clock, and the
//! show-desktop strip at the far end. One per screen, on the theme's side of it (Aero at the
//! bottom, Kernel at the top).
//!
//! The bar owns the popups its buttons ask for (the thumbnails, the jump lists, its own menus,
//! the tray's menus and its popover), so that one of each is open at a time. The menus are
//! AntiWebUI popups, which take the pointer: a press outside closes them, so the QML shell's
//! full-screen click-catcher is not needed.
//!
//! With auto-hide, the bar slides to a 2 px line at the screen's edge and comes back when the
//! pointer reaches it; windows then have the whole screen.

mod buttons;
mod jump_list;
mod launcher_entries;
mod media;
mod orb;
mod thumbnails;
mod tray;

pub use media::media_controls;

use crate::settings::{change, motion, settings};
use crate::theme::{Align, away_from_bar, bar_at_top, parse_hex, theme};
use crate::ui::{MenuLine, popup_menu, vertical_gradient};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use serde_json::json;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

/// How long the auto-hidden bar stays up after the pointer has left it.
const STAY_UP: Duration = Duration::from_millis(700);
/// What is left of an auto-hidden bar at the screen's edge.
const HIDDEN_STRIP: f32 = 2.0;
/// How long the bar slides.
const SLIDE: f32 = 160.0;
/// How long the thumbnails stay after the pointer has left the button, so that it can move
/// into them.
const HIDE_THUMBNAILS: Duration = Duration::from_millis(180);
/// The glass of the show-desktop strip, at rest and under the pointer: three stops each.
const SHOW_DESKTOP_REST: [Color; 3] = [rgba(0xffffff40), rgba(0xffffff1a), rgba(0xffffff26)];
const SHOW_DESKTOP_HOT: [Color; 3] = [rgba(0xffffff80), rgba(0xffffff4d), rgba(0xffffff66)];
/// The share of the bar's height down to its darkest band (Windows 7's taskbar has five stops:
/// the bar is two boxes of two and three).
const BAR_MIDDLE: f32 = 0.45;

/// The thumbnails shown: whose, and where the middle of its button is, in logical pixels from
/// the screen's left.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Preview {
    pub key: String,
    pub center: f32,
}

/// What one screen's bar has open, shared by its parts.
#[derive(Clone)]
pub(crate) struct Bar {
    /// The screen's connector name.
    pub name: String,
    pub output: Output,
    pub preview: Signal<Option<Preview>>,
    /// The pointer is over the thumbnails.
    pub preview_inside: Signal<bool>,
    hide_timer: Rc<Cell<Option<Timeout>>>,
    /// The key of the button whose jump list is open.
    pub jump_list: Signal<Option<String>>,
    /// The bar's own right-click menu open ("taskbar" or "desktop"), and where.
    pub context_menu: Signal<Option<&'static str>>,
    pub context_at: Signal<Option<RectI>>,
    pub overflow_open: Signal<bool>,
    /// The key of the notification-area item whose menu is open.
    pub tray_menu: Signal<Option<String>>,
    /// A notification-area icon dragged over the bar: its id, and the shown icon it would go
    /// before.
    pub tray_drop: Signal<Option<(String, usize)>>,
    /// The row of the notification area's shown icons, to place a drop among them.
    pub tray_row: NodeRef,
    pub hovered: Signal<bool>,
    /// Up for a moment after the pointer left.
    pub staying: Signal<bool>,
    stay_timer: Rc<Cell<Option<Timeout>>>,
}

impl Bar {
    fn new(output: Output) -> Self {
        Self {
            name: output.name.clone(),
            output,
            preview: signal(None),
            preview_inside: signal(false),
            hide_timer: Rc::default(),
            jump_list: signal(None),
            context_menu: signal(None),
            context_at: signal(None),
            overflow_open: signal(false),
            tray_menu: signal(None),
            tray_drop: signal(None),
            tray_row: node_ref(),
            hovered: signal(false),
            staying: signal(false),
            stay_timer: Rc::default(),
        }
    }

    /// Closes every popup of the bar, the notification area's panels too: one at a time.
    pub fn close_popups(&self) {
        crate::status::close_popups();
        self.preview.set(None);
        self.jump_list.set(None);
        self.context_menu.set(None);
        self.overflow_open.set(false);
        self.tray_menu.set(None);
    }

    /// One of the bar's menus is open.
    fn popup_open(&self) -> bool {
        self.jump_list.with(Option::is_some)
            || self.context_menu.with(Option::is_some)
            || self.overflow_open.get()
            || self.tray_menu.with(Option::is_some)
            || crate::status::popup_open_on(&self.name)
    }

    /// A menu, a panel or the thumbnails of the bar are open: an auto-hidden bar stays up.
    fn busy(&self) -> bool {
        crate::start_menu::is_open_on(&self.name)
            || self.preview.with(Option::is_some)
            || self.popup_open()
    }

    /// The bar is on the screen, not slid away.
    fn raised(&self) -> bool {
        !settings().with(|s| s.taskbar_auto_hide)
            || self.hovered.get()
            || self.staying.get()
            || self.busy()
    }

    /// Shows the thumbnails of the button `key`, whose middle is at `center`.
    pub fn show_preview(&self, key: String, center: f32) {
        if let Some(t) = self.hide_timer.take() {
            t.cancel();
        }
        self.jump_list.set(None);
        self.tray_menu.set(None);
        self.preview.set(Some(Preview { key, center }));
    }

    /// Hides the thumbnails in a moment, unless the pointer is in them by then.
    pub fn hide_preview_soon(&self) {
        if let Some(t) = self.hide_timer.take() {
            t.cancel();
        }
        let bar = self.clone();
        let timer = set_timeout(HIDE_THUMBNAILS, move || {
            bar.hide_timer.set(None);
            if bar.preview_inside.get_untracked() {
                bar.hide_preview_soon();
            } else {
                bar.preview.set(None);
            }
        });
        self.hide_timer.set(Some(timer));
    }

    /// Opens the bar's menu `key` at `at` (window physical pixels), or closes it when it is the
    /// one open.
    fn toggle_context_menu(&self, key: &'static str, at: RectI) {
        let again = self.context_menu.get_untracked() == Some(key);
        self.close_popups();
        if !again {
            self.context_at.set(Some(at));
            self.context_menu.set(Some(key));
        }
    }

    fn hover(&self, inside: bool) {
        self.hovered.set(inside);
        if let Some(t) = self.stay_timer.take() {
            t.cancel();
        }
        if !inside {
            self.staying.set(true);
            let staying = self.staying;
            self.stay_timer
                .set(Some(set_timeout(STAY_UP, move || staying.set(false))));
        }
    }
}

thread_local! {
    static INSTALLED: Cell<bool> = const { Cell::new(false) };
}

/// Starts what every bar shares: the windows and programs, the progress programs show, and the
/// `taskbar` commands (`activate N`, `launchNew N`: Super+N and Super+Shift+N). Runs once;
/// [`open`] calls it.
pub fn install() {
    if INSTALLED.with(|i| i.replace(true)) {
        return;
    }
    crate::apps::install();
    launcher_entries::install();
    crate::ipc::handle("taskbar", |function, args| {
        let number = args
            .first()
            .and_then(|a| a.trim().parse::<usize>().ok())
            .ok_or_else(|| "a button number from 1".to_owned())?;
        match function {
            "activate" => crate::apps::activate_button(number, false),
            "launchNew" => crate::apps::activate_button(number, true),
            _ => return Err(format!("no function {function}")),
        }
        Ok(String::new())
    });
}

/// The edges the bar is anchored to.
fn bar_anchor() -> Anchor {
    let edge = if bar_at_top() {
        Anchor::TOP
    } else {
        Anchor::BOTTOM
    };
    edge | Anchor::LEFT | Anchor::RIGHT
}

/// Opens the taskbar of a screen, for as long as the owner it is called under lives (the
/// screen's, in a `for_each_output`).
pub fn open(output: Output) {
    install();
    let bar = Bar::new(output.clone());
    let height = untrack(|| theme().taskbar_height as i32);
    let layer = open_layer(
        LayerOptions::new()
            .layer(Layer::Top)
            .anchor(untrack(bar_anchor))
            .height(height)
            .exclusive(Exclusive::Fixed(height))
            .keyboard(Keyboard::None)
            .output(&output)
            .namespace("shojiwm-win7-taskbar"),
        {
            let bar = bar.clone();
            move || view(bar)
        },
    );
    // The side, the height and the room kept for it, as the theme and the settings say.
    let shape = memo(|| {
        (
            bar_anchor(),
            theme().taskbar_height as i32,
            settings().with(|s| s.taskbar_auto_hide),
        )
    });
    {
        let layer = layer.clone();
        effect(move || {
            let (anchor, height, auto_hide) = shape.get();
            layer.set_anchor(anchor);
            layer.set_size(LayerSize::Stretch, LayerSize::Fixed(height));
            layer.set_exclusive(if auto_hide {
                Exclusive::None
            } else {
                Exclusive::Fixed(height)
            });
        });
    }
    // Only the part of the bar on the screen takes the pointer.
    let region = {
        let bar = bar.clone();
        memo(move || {
            if bar.raised() {
                return InputRegion::Full;
            }
            let (_, height, _) = shape.get();
            let width = bar
                .output
                .current()
                .map_or(bar.output.logical_size.width, |o| o.logical_size.width);
            let strip = HIDDEN_STRIP as i32;
            let y = if bar_at_top() { 0 } else { height - strip };
            InputRegion::Rects(vec![RectI::new(0, y, width.max(1), strip)])
        })
    };
    effect(move || layer.set_input_region(region.get()));
    thumbnails::follow(bar);
}

/// Shows the desktop: through the compositor, which brings the windows back on a second
/// click; without it, every window is put away.
pub fn show_the_desktop() {
    if crate::compositor::available() {
        crate::compositor::send("show-desktop", json!({}));
    } else {
        crate::apps::minimize_all();
    }
}

/// The lines of the bar's own right-click menus, as Windows 7 has them.
fn context_lines(key: &str) -> Vec<MenuLine> {
    let show_desktop = || MenuLine::item("Show the desktop", show_the_desktop);
    match key {
        "desktop" => vec![
            show_desktop(),
            MenuLine::item("Peek at desktop", || change(|s| s.peek = !s.peek))
                .checked(settings().get_untracked().peek),
        ],
        _ => vec![
            show_desktop(),
            MenuLine::item("Start Task Manager", || {
                crate::programs::run(&crate::programs::program("taskManager"))
            }),
            MenuLine::separator(),
            MenuLine::item("Properties", || crate::control_panel::open_page("taskbar")),
        ],
    }
}

/// Keeps a right press from reaching the bar, whose own menu would open too.
fn keep_right_press(target: Div) -> Div {
    target.on_pointer_down(|e| {
        if e.button == Some(PointerButton::Secondary) {
            e.stop_propagation();
        }
    })
}

fn view(bar: Bar) -> Div {
    let raised = {
        let bar = bar.clone();
        memo(move || bar.raised())
    };
    let hover = bar.clone();
    let menu_bar = bar.clone();
    let closed = bar.clone();
    let shown = bar.clone();
    let slid = div()
        .absolute()
        .left(0.0)
        .right(0.0)
        .top(0.0)
        .bottom(0.0)
        .dyn_style(move |s| {
            let height = theme().taskbar_height;
            let away = height - HIDDEN_STRIP;
            let y = match (raised.get(), bar_at_top()) {
                (true, _) => 0.0,
                (false, true) => -away,
                (false, false) => away,
            };
            s.translate_y(y)
                .transition(Transition::ms(motion(SLIDE)).ease_out())
        })
        .on_pointer_down(move |e| {
            if e.button == Some(PointerButton::Secondary) {
                let at = RectI::new(e.position.x as i32, e.rect.y, 1, e.rect.height);
                menu_bar.toggle_context_menu("taskbar", at);
            }
        })
        .children(glass())
        .child(contents(bar.clone()))
        .child(popup_menu(
            move || shown.context_menu.with(Option::is_some),
            move || closed.context_menu.set(None),
            untrack(|| {
                MenuOptions::new()
                    .at(bar.context_at)
                    .placement(away_from_bar(Align::Start))
                    .gap(theme().popup_gap as i32)
            }),
            {
                let bar = bar.clone();
                move || {
                    bar.context_menu
                        .get_untracked()
                        .map(context_lines)
                        .unwrap_or_default()
                }
            },
        ));
    let slid = tray::take_drops(slid, bar.clone());
    div()
        .relative()
        .size_full()
        .overflow_clip()
        .select_none()
        .on_hover(move |inside| hover.hover(inside))
        .child(slid)
}

/// The glass of the bar: light at the top, darkest a little under half way, a slight lift in
/// the lower third; the glass colour chosen in the Control Panel over it; and the edges. The
/// edge towards the desktop is a dark hairline with a lit one inside it; the edge at the
/// screen's side has a dark one. A bar at the top has them the other way up.
fn glass() -> Vec<Div> {
    let band = |top: bool| {
        div()
            .absolute()
            .left(0.0)
            .right(0.0)
            .dyn_style(move |s| {
                let height = theme().taskbar_height;
                let middle = (height * BAR_MIDDLE).round();
                if top {
                    s.top(0.0).h(middle)
                } else {
                    s.top(middle).bottom(0.0)
                }
            })
            .fill(move || {
                let t = theme();
                if top {
                    vertical_gradient(&[
                        (0.0, t.bar_top),
                        (0.1 / BAR_MIDDLE, t.bar_upper),
                        (1.0, t.bar_middle),
                    ])
                } else {
                    let lower = (0.78 - BAR_MIDDLE) / (1.0 - BAR_MIDDLE);
                    vertical_gradient(&[
                        (0.0, t.bar_middle),
                        (lower, t.bar_lower),
                        (1.0, t.bar_bottom),
                    ])
                }
            })
    };
    let line = |at: fn(bool, f32) -> f32, color: fn() -> Color| {
        div()
            .absolute()
            .left(0.0)
            .right(0.0)
            .h(1.0)
            .dyn_style(move |s| s.top(at(bar_at_top(), theme().taskbar_height)).bg(color()))
    };
    vec![
        band(true),
        band(false),
        div()
            .absolute()
            .left(0.0)
            .right(0.0)
            .top(0.0)
            .bottom(0.0)
            .dyn_style(|s| {
                let (colour, strength) =
                    settings().with(|s| (s.glass_color.clone(), s.glass_color_strength));
                match parse_hex(&colour) {
                    Some(c) if !colour.is_empty() => {
                        s.bg(c).opacity(strength.clamp(0.0, 1.0) as f32)
                    }
                    _ => s.hidden(),
                }
            }),
        line(
            |top, h| if top { h - 1.0 } else { 0.0 },
            || theme().bar_edge,
        ),
        line(
            |top, h| if top { h - 2.0 } else { 1.0 },
            || theme().bar_highlight,
        ),
        line(
            |top, h| if top { 0.0 } else { h - 1.0 },
            || theme().bar_bottom_edge,
        ),
    ]
}

/// The Start button, the buttons, and the right hand end.
fn contents(bar: Bar) -> Div {
    let status = div()
        .flex_none()
        .flex()
        .flex_row()
        .items_center()
        .dyn_style(|s| s.gap(theme().tray_gap))
        .child(tray::area(bar.clone()))
        .child(keep_right_press(
            div()
                .flex()
                .flex_row()
                .items_center()
                .dyn_style(|s| s.gap(theme().tray_gap))
                .child(crate::status::area(bar.output.clone()))
                .child(crate::status::clock(bar.output.clone())),
        ))
        .child(show_desktop(bar.clone()));
    div()
        .absolute()
        .left(0.0)
        .right(0.0)
        .top(0.0)
        .bottom(0.0)
        .flex()
        .flex_row()
        .items_center()
        .child(
            div()
                .flex_none()
                .flex()
                .items_center()
                .dyn_style(|s| s.ml(theme().orb_left))
                .child(orb::start_button(bar.clone())),
        )
        .child(buttons::buttons(bar))
        .child(status)
}

/// Show desktop: the glass strip Windows puts past the clock, a dark line and a light one at
/// its left, lit under the pointer. Resting on it turns every window to glass (Aero Peek,
/// through the compositor); a click puts them away, and a second one brings them back.
fn show_desktop(bar: Bar) -> Div {
    let ui = Interaction::new();
    let peek: Rc<Cell<Option<Timeout>>> = Rc::default();
    let stop_peek = {
        let peek = peek.clone();
        move || {
            if let Some(t) = peek.take() {
                t.cancel();
            }
        }
    };
    let hover_stop = stop_peek.clone();
    let click_stop = stop_peek;
    let menu_bar = bar.clone();
    div()
        .relative()
        .flex_none()
        .interaction(ui)
        .dyn_style(|s| {
            let t = theme();
            s.w(t.show_desktop_width).h(t.taskbar_height)
        })
        .child(
            div()
                .absolute()
                .left(2.0)
                .top(2.0)
                .right(0.0)
                .bottom(0.0)
                .fill(move || {
                    let c = if ui.hovered() {
                        SHOW_DESKTOP_HOT
                    } else {
                        SHOW_DESKTOP_REST
                    };
                    vertical_gradient(&[(0.0, c[0]), (0.5, c[1]), (1.0, c[2])])
                }),
        )
        .child(
            div()
                .absolute()
                .left(0.0)
                .top(0.0)
                .bottom(0.0)
                .w(1.0)
                .dyn_style(|s| s.bg(theme().button_outline)),
        )
        .child(
            div()
                .absolute()
                .left(1.0)
                .top(0.0)
                .bottom(0.0)
                .w(1.0)
                .dyn_style(|s| s.bg(theme().button_rest_border)),
        )
        .on_hover(move |inside| {
            hover_stop();
            if inside {
                if settings().get_untracked().peek {
                    let wait = Duration::from_millis(untrack(|| theme().peek_delay) as u64);
                    peek.set(Some(set_timeout(wait, || {
                        crate::compositor::send("peek", json!({ "on": true }));
                    })));
                }
            } else {
                crate::compositor::send("peek", json!({ "on": false }));
            }
        })
        .on_pointer_down(move |e| {
            if e.button == Some(PointerButton::Secondary) {
                e.stop_propagation();
                menu_bar.toggle_context_menu("desktop", e.rect);
            }
        })
        .on_click(move |_| {
            click_stop();
            bar.close_popups();
            show_the_desktop();
        })
}
