//! Screen Resolution (QML `DisplaySettings` and the page of `ControlPanel.qml`): a picture of the
//! screens as they are placed, to pick one, and its resolution, refresh rate and scale.
//!
//! The compositor lists the screens (`screens`, `src/display.ts`). The choices are the setting
//! `display_choices`, which the shell sends to the compositor whenever it changes
//! (`compositor::share_settings`). A change asks to be kept: as on Windows, it goes back on its
//! own after 15 seconds, so a mode the screen cannot show undoes itself. Leaving the page while
//! it asks goes back too, as closing Windows' question does.

use super::common::{FIELD_HEIGHT, control_row, form_label, page_column, tinted_text};
use crate::control_panel::{heading, note};
use crate::settings::{change, settings};
use crate::theme::{corner, scheme};
use crate::ui::{cp_button, cp_check, cp_combo, dark_look, font, vertical_gradient};
use anti_webui::prelude::*;
use serde_json::{Map, Value, json};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

/// The picture of the layout: its background in dark and in light, and its height.
const LAYOUT_DARK: Color = rgb(0x141b22);
const LAYOUT_LIGHT: Color = rgb(0xf3f6f9);
const LAYOUT_HEIGHT: f32 = 150.0;
/// A screen in the picture: its gradient, its border when not chosen, and its number.
const TILE_TOP: Color = rgb(0x5f9bd1);
const TILE_BOTTOM: Color = rgb(0x2d6aa8);
const TILE_BORDER: Color = rgb(0x5d7489);
const TILE_NUMBER: Color = Color::WHITE;
/// The question that asks to keep a change: its background and border, in dark and in light.
const CONFIRM_DARK: Color = rgb(0x2a3a1c);
const CONFIRM_LIGHT: Color = rgb(0xfff8d6);
const CONFIRM_BORDER_DARK: Color = rgb(0x6b7f3c);
const CONFIRM_BORDER_LIGHT: Color = rgb(0xe0c66b);
/// The width of the combo boxes and of the labels' column.
const COMBO_WIDTH: f32 = 300.0;
const LABEL_WIDTH: f32 = 90.0;

/// The scales offered.
const SCALES: [f64; 9] = [1.0, 1.25, 1.5, 1.75, 1.875, 2.0, 2.25, 2.5, 3.0];
/// How long a change waits to be kept.
const CONFIRM_SECONDS: i64 = 15;
/// The screens report their new modes once the compositor has set them.
const REFRESH_DELAY: Duration = Duration::from_millis(700);

/// A mode a screen can show.
#[derive(Clone, Debug, PartialEq)]
struct Mode {
    width: i64,
    height: i64,
    refresh_rate: i64,
}

/// A screen as the compositor reports it (`ScreenInfo` of `src/display.ts`).
#[derive(Clone, Debug, PartialEq)]
struct Screen {
    name: String,
    description: String,
    width: i64,
    height: i64,
    refresh_rate: i64,
    scale: f64,
    x: f64,
    y: f64,
    modes: Vec<Mode>,
}

impl Screen {
    fn from_json(v: &Value) -> Option<Self> {
        let int =
            |v: &Value, k: &str| v.get(k).and_then(Value::as_f64).unwrap_or(0.0).round() as i64;
        let num = |k: &str| v.get(k).and_then(Value::as_f64).unwrap_or(0.0);
        let modes = v
            .get("modes")
            .and_then(Value::as_array)
            .map(|list| {
                list.iter()
                    .map(|m| Mode {
                        width: int(m, "width"),
                        height: int(m, "height"),
                        refresh_rate: int(m, "refreshRate"),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let scale = num("scale");
        Some(Self {
            name: v.get("name")?.as_str()?.to_owned(),
            description: v
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned(),
            width: int(v, "width"),
            height: int(v, "height"),
            refresh_rate: int(v, "refreshRate"),
            scale: if scale > 0.0 { scale } else { 1.0 },
            x: num("x"),
            y: num("y"),
            modes,
        })
    }

    /// Its size on the desktop, in logical pixels.
    fn logical_width(&self) -> f64 {
        self.width as f64 / self.scale
    }
    fn logical_height(&self) -> f64 {
        self.height as f64 / self.scale
    }
}

fn size_label(width: i64, height: i64) -> String {
    format!("{width} × {height}")
}

fn scale_label(scale: f64) -> String {
    let percent = (scale * 1000.0).round() / 10.0;
    let note = if scale == 1.0 {
        " (smaller)"
    } else if scale >= 2.0 {
        " (larger)"
    } else {
        ""
    };
    format!("{percent} %{note}")
}

/// The page's state: the screens, the one being set and what is chosen for it, and the
/// change waiting to be kept.
#[derive(Clone, Copy)]
struct Display {
    screens: Signal<Vec<Screen>>,
    /// The screen being set, by connector name; "" for the first.
    selected: Signal<String>,
    screen: Memo<Option<Screen>>,
    /// The screen's resolutions, largest first, and the refresh rates of the one shown.
    sizes: Memo<Vec<(i64, i64)>>,
    rates: Memo<Vec<i64>>,
    size_index: Signal<usize>,
    rate_index: Signal<usize>,
    scale_index: Signal<usize>,
    /// The choices before the change waiting to be kept.
    previous: Signal<Option<Map<String, Value>>>,
    seconds_left: Signal<i64>,
    countdown: Signal<Option<Timeout>>,
    /// The page's owner: the timers and the compositor's answers belong to it, whatever
    /// handler starts them, so they stop when the page goes.
    owner: Option<Owner>,
}

impl Display {
    fn new() -> Self {
        let screens = signal(Vec::<Screen>::new());
        let selected = signal(String::new());
        let screen = memo(move || {
            screens.with(|list| {
                selected
                    .with(|name| list.iter().find(|s| &s.name == name).cloned())
                    .or_else(|| list.first().cloned())
            })
        });
        let sizes = memo(move || {
            let mut out: Vec<(i64, i64)> = Vec::new();
            screen.with(|s| {
                for m in s.iter().flat_map(|s| &s.modes) {
                    if !out.contains(&(m.width, m.height)) {
                        out.push((m.width, m.height));
                    }
                }
            });
            out
        });
        let size_index = signal(0usize);
        let rates = memo(move || {
            let size = sizes.with(|l| l.get(size_index.get()).copied());
            let mut out: Vec<i64> = Vec::new();
            screen.with(|s| {
                for m in s.iter().flat_map(|s| &s.modes) {
                    if Some((m.width, m.height)) == size && !out.contains(&m.refresh_rate) {
                        out.push(m.refresh_rate);
                    }
                }
            });
            out
        });
        Self {
            screens,
            selected,
            screen,
            sizes,
            rates,
            size_index,
            rate_index: signal(0),
            scale_index: signal(0),
            previous: signal(None),
            seconds_left: signal(0),
            countdown: signal(None),
            owner: Owner::current(),
        }
    }

    /// Runs `f` under the page's owner.
    fn within<R>(self, f: impl FnOnce() -> R) -> R {
        match self.owner {
            Some(owner) => owner.with(f),
            None => f(),
        }
    }

    fn confirming(self) -> bool {
        self.previous.with(Option::is_some)
    }

    fn refresh(self) {
        self.within(|| {
            crate::compositor::request("screens", json!({})).then(move |answer| {
                if let Some(list) = answer.as_ref().and_then(Value::as_array) {
                    self.screens
                        .set(list.iter().filter_map(Screen::from_json).collect());
                }
            });
        });
    }

    fn refresh_later(self) {
        self.within(|| {
            set_timeout(REFRESH_DELAY, move || self.refresh());
        });
    }

    /// Shows the selected screen's current mode and scale.
    fn load(self) {
        untrack(|| {
            let Some(screen) = self.screen.get() else {
                return;
            };
            let size = (screen.width, screen.height);
            let size_at = self.sizes.with(|l| l.iter().position(|s| *s == size));
            self.size_index.set(size_at.unwrap_or(0));
            let rate_at = self
                .rates
                .with(|l| l.iter().position(|r| *r == screen.refresh_rate));
            self.rate_index.set(rate_at.unwrap_or(0));
            let scale_at = SCALES.iter().position(|s| (s - screen.scale).abs() < 1e-6);
            self.scale_index.set(scale_at.unwrap_or(0));
        });
    }

    /// The choices with the selected screen's new mode and scale, applied.
    fn apply(self) {
        untrack(|| {
            let Some(screen) = self.screen.get() else {
                return;
            };
            let Some((width, height)) = self.sizes.with(|l| l.get(self.size_index.get()).copied())
            else {
                return;
            };
            let rate = self
                .rates
                .with(|l| l.get(self.rate_index.get()).copied())
                .unwrap_or(0);
            let scale = SCALES[self.scale_index.get().min(SCALES.len() - 1)];
            let mut choices = settings().with(|s| s.display_choices.clone());
            choices.insert(
                screen.name,
                json!({ "width": width, "height": height, "refreshRate": rate, "scale": scale }),
            );
            self.change_to(choices);
        });
    }

    /// Applies new choices for every screen, and waits for them to be kept.
    fn change_to(self, choices: Map<String, Value>) {
        if !untrack(|| self.confirming()) {
            let now = settings().with_untracked(|s| s.display_choices.clone());
            self.previous.set(Some(now));
        }
        change(|s| s.display_choices = choices);
        self.seconds_left.set(CONFIRM_SECONDS);
        self.stop_countdown();
        self.tick();
        self.refresh_later();
    }

    /// One second of the countdown; at its end the change goes back.
    fn tick(self) {
        let timeout = self.within(|| {
            set_timeout(Duration::from_secs(1), move || {
                self.countdown.set(None);
                let left = self.seconds_left.get_untracked() - 1;
                self.seconds_left.set(left);
                if left <= 0 {
                    self.revert();
                } else {
                    self.tick();
                }
            })
        });
        self.countdown.set(Some(timeout));
    }

    fn stop_countdown(self) {
        if let Some(timeout) = self.countdown.get_untracked() {
            timeout.cancel();
        }
        self.countdown.set(None);
    }

    fn keep(self) {
        self.stop_countdown();
        self.previous.set(None);
    }

    fn revert(self) {
        self.stop_countdown();
        if let Some(previous) = self.previous.get_untracked() {
            change(|s| s.display_choices = previous);
        }
        self.previous.set(None);
        self.refresh_later();
    }

    /// Back to the session's own layout.
    fn reset(self) {
        self.change_to(Map::new());
    }
}

pub fn page() -> Div {
    let d = Display::new();
    d.refresh();
    effect(move || {
        d.screen.with(|_| ());
        d.load();
    });
    // Leaving the page while a change waits to be kept takes it back. The page's signals are
    // gone when its cleanups run, so the choices to go back to are kept apart.
    let waiting: Rc<RefCell<Option<Map<String, Value>>>> = Rc::default();
    {
        let waiting = waiting.clone();
        effect(move || *waiting.borrow_mut() = d.previous.get());
    }
    on_cleanup(move || {
        if let Some(previous) = waiting.borrow_mut().take() {
            change(|s| s.display_choices = previous);
        }
    });
    page_column(12.0)
        .child(heading("Change the appearance of your displays"))
        .child(layout_picture(d))
        .child(show(
            move || d.screens.with(Vec::is_empty),
            || {
                note(
                    "The compositor did not list the screens. This page needs the shell \
                     started by the Windows 7 configuration.",
                )
            },
        ))
        .child(show(
            move || d.screen.with(Option::is_some),
            move || choices(d),
        ))
        .child(cp_check(
            "Make this my main display",
            Binding::new(
                move || {
                    d.screen.with(|s| {
                        s.as_ref()
                            .is_some_and(|s| crate::desktop::primary_output_name() == s.name)
                    })
                },
                move |on: bool| {
                    if let Some(screen) = untrack(|| d.screen.get()) {
                        change(|s| s.primary_screen = if on { screen.name } else { String::new() });
                    }
                },
            ),
        ))
        .child(show(
            move || !d.confirming(),
            move || {
                control_row(10.0)
                    .pt(4.0)
                    .child(cp_button("Apply", move || d.apply()))
                    .child(show(
                        || settings().with(|s| !s.display_choices.is_empty()),
                        move || cp_button("Use the session's layout", move || d.reset()),
                    ))
            },
        ))
        .child(show(move || d.confirming(), move || confirm_box(d)))
        .child(note(
            "Your choices replace the session's layout (SHOJIWM_WIN7_OUTPUTS) for the screens \
             you change; the screens are then placed side by side, left to right.",
        ))
}

/// The screens as they are placed, scaled to fit, each a numbered tile: a click picks one.
fn layout_picture(d: Display) -> Div {
    let frame_width = signal(0.0f32);
    div()
        .relative()
        .w_full()
        .h(LAYOUT_HEIGHT)
        .border(1.0)
        .overflow_clip()
        .on_resized(move |w, _| frame_width.set(w))
        .dyn_style(|s| {
            let c = scheme();
            s.rounded(corner(3.0))
                .bg(if dark_look() {
                    LAYOUT_DARK
                } else {
                    LAYOUT_LIGHT
                })
                .border_color(c.field_border)
        })
        .child(dyn_view(move || {
            let list = d.screens.get();
            let width = frame_width.get() as f64;
            let right = list
                .iter()
                .map(|s| s.x + s.logical_width())
                .fold(1.0, f64::max);
            let bottom = list
                .iter()
                .map(|s| s.y + s.logical_height())
                .fold(1.0, f64::max);
            let k = ((width - 40.0) / right)
                .min((LAYOUT_HEIGHT as f64 - 30.0) / bottom)
                .max(0.0);
            let left = (width - right * k) / 2.0;
            let top = (LAYOUT_HEIGHT as f64 - bottom * k) / 2.0;
            div().children(
                list.into_iter()
                    .enumerate()
                    .map(|(index, screen)| {
                        let tile_height = (screen.logical_height() * k - 4.0) as f32;
                        screen_tile(
                            d,
                            index,
                            screen.name.clone(),
                            (left + screen.x * k + 2.0) as f32,
                            (top + screen.y * k + 2.0) as f32,
                            (screen.logical_width() * k - 4.0) as f32,
                            tile_height,
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        }))
}

fn screen_tile(d: Display, index: usize, name: String, x: f32, y: f32, w: f32, h: f32) -> Div {
    let chosen = {
        let name = name.clone();
        memo(move || {
            d.screen
                .with(|s| s.as_ref().is_some_and(|s| s.name == name))
        })
    };
    div()
        .absolute()
        .left(x)
        .top(y)
        .w(w.max(0.0))
        .h(h.max(0.0))
        .flex()
        .items_center()
        .justify_center()
        .select_none()
        .fill(|| vertical_gradient(&[(0.0, TILE_TOP), (1.0, TILE_BOTTOM)]))
        .dyn_style(move |s| {
            let on = chosen.get();
            s.rounded(corner(2.0))
                .border(if on { 2.0 } else { 1.0 })
                .border_color(if on {
                    scheme().field_focus
                } else {
                    TILE_BORDER
                })
        })
        .on_click(move |_| {
            d.selected.set(name.clone());
            d.load();
        })
        .child(text((index + 1).to_string()).dyn_style(move |s| {
            s.font_family(font())
                .text_size((h * 0.5).clamp(1.0, 36.0))
                .text_color(TILE_NUMBER)
                .bold()
        }))
}

/// The screen, its resolution, refresh rate and scale, each a combo box.
fn choices(d: Display) -> Div {
    let row = |label: &'static str, combo: View| {
        control_row(12.0)
            .child(form_label(label, LABEL_WIDTH))
            .child(combo)
    };
    let screen_combo = dyn_view(move || {
        let labels: Vec<String> = d.screens.with(|list| {
            list.iter()
                .enumerate()
                .map(|(i, s)| format!("{}. {} ({})", i + 1, s.description, s.name))
                .collect()
        });
        let current = Binding::new(
            move || {
                let name = d.screen.with(|s| s.as_ref().map(|s| s.name.clone()));
                d.screens
                    .with(|l| l.iter().position(|s| Some(&s.name) == name.as_ref()))
                    .unwrap_or(0)
            },
            move |i: usize| {
                if let Some(name) = d
                    .screens
                    .with_untracked(|l| l.get(i).map(|s| s.name.clone()))
                {
                    d.selected.set(name);
                    d.load();
                }
            },
        );
        untrack(|| cp_combo(labels, current).w(COMBO_WIDTH))
    });
    let size_combo = dyn_view(move || {
        let labels = d
            .sizes
            .with(|l| l.iter().map(|(w, h)| size_label(*w, *h)).collect());
        let current = Binding::new(
            move || d.size_index.get(),
            move |i: usize| {
                d.size_index.set(i);
                d.rate_index.set(0);
            },
        );
        untrack(|| cp_combo(labels, current).w(COMBO_WIDTH))
    });
    let rate_combo = dyn_view(move || {
        let labels = d
            .rates
            .with(|l| l.iter().map(|r| format!("{r} Hz")).collect());
        untrack(|| cp_combo(labels, d.rate_index).w(COMBO_WIDTH))
    });
    let scale_labels = SCALES.iter().map(|s| scale_label(*s)).collect();
    let scale_combo = cp_combo(scale_labels, d.scale_index).w(COMBO_WIDTH);
    div()
        .flex()
        .flex_col()
        .gap(10.0)
        .child(row("Display:", screen_combo))
        .child(row("Resolution:", size_combo))
        .child(row("Refresh rate:", rate_combo))
        .child(row("Scale:", scale_combo.into_view()))
}

/// "Keep these display settings?", the seconds left, and the two answers.
fn confirm_box(d: Display) -> Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(12.0)
        .w_full()
        .min_h(FIELD_HEIGHT + 20.0)
        .px(12.0)
        .py(10.0)
        .border(1.0)
        .dyn_style(|s| {
            let dark = dark_look();
            s.rounded(corner(3.0))
                .bg(if dark { CONFIRM_DARK } else { CONFIRM_LIGHT })
                .border_color(if dark {
                    CONFIRM_BORDER_DARK
                } else {
                    CONFIRM_BORDER_LIGHT
                })
        })
        .child(tinted_text(
            move || {
                format!(
                    "Keep these display settings? Reverting in {} s.",
                    d.seconds_left.get()
                )
            },
            || scheme().text,
        ))
        .child(cp_button("Keep changes", move || d.keep()))
        .child(cp_button("Revert", move || d.revert()))
}
