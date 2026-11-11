//! The volume controls of the volume panel and the Sound window, as Windows 7 draws them: a
//! vertical slider (a sunken track, the level filled in green, a thumb), its horizontal form for
//! lists, the balance slider, and the mute button (the speaker, with the red mark when muted).
//! Each works on a [`VolumeControl`]: a device, a program, or a group of streams.

use super::{dark_look, ink_asset, vertical_gradient};
use crate::theme::{button_radius, corner, scheme};
use anti_webui::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

/// A notch of the wheel changes the volume by this many percent, as Windows 7 steps it.
const NOTCH_PERCENT: f64 = 2.0;
/// A touchpad moves this far for one notch.
const TOUCHPAD_STEP: f32 = 40.0;
/// The room above and below the track.
const TRACK_MARGIN: f32 = 5.0;
/// The level's green, as Windows 7 fills its volume slider.
const LEVEL_GREEN: Color = rgb(0x2fb13f);
/// The sunken track of the slider, on dark and on light colours.
const TRACK_DARK: Color = rgb(0x1a232c);
const TRACK_LIGHT: Color = rgb(0xe8edf2);
/// The width of the slider and of the mute button.
const CONTROL_WIDTH: f32 = 30.0;

/// What a volume control shows and changes. `read` gives the volume (1 for 100 %) and whether
/// it is muted, or None when there is nothing to control; reading tracks.
#[derive(Clone)]
pub struct VolumeControl {
    read: Rc<dyn Fn() -> Option<(f64, bool)>>,
    set_volume: Rc<dyn Fn(f64)>,
    set_muted: Rc<dyn Fn(bool)>,
}

impl VolumeControl {
    pub fn new(
        read: impl Fn() -> Option<(f64, bool)> + 'static,
        set_volume: impl Fn(f64) + 'static,
        set_muted: impl Fn(bool) + 'static,
    ) -> Self {
        Self {
            read: Rc::new(read),
            set_volume: Rc::new(set_volume),
            set_muted: Rc::new(set_muted),
        }
    }

    /// There is something to control.
    pub fn ready(&self) -> bool {
        (self.read)().is_some()
    }

    /// The volume, 1 for 100 %; 0 when there is nothing to control.
    pub fn volume(&self) -> f64 {
        (self.read)().map_or(0.0, |(v, _)| v)
    }

    pub fn muted(&self) -> bool {
        (self.read)().is_some_and(|(_, m)| m)
    }

    /// The volume as a whole percentage.
    pub fn percent(&self) -> i64 {
        (self.volume() * 100.0).round() as i64
    }

    /// Sets the volume, between 0 and 1. A volume above 0 ends the mute.
    pub fn set(&self, value: f64) {
        let Some((_, muted)) = untrack(|| (self.read)()) else {
            return;
        };
        let value = value.clamp(0.0, 1.0);
        (self.set_volume)(value);
        if muted && value > 0.0 {
            (self.set_muted)(false);
        }
    }

    /// Changes the volume by a number of percent.
    pub fn step(&self, percent_change: f64) {
        let now = untrack(|| self.percent()) as f64;
        self.set((now + percent_change).round() / 100.0);
    }

    pub fn toggle_mute(&self) {
        if let Some((_, muted)) = untrack(|| (self.read)()) {
            (self.set_muted)(!muted);
        }
    }

    /// 1 to 3 waves for the level, or "muted": the end of the speaker's picture name.
    pub fn level(&self) -> &'static str {
        let p = self.percent();
        if !self.ready() || self.muted() || p == 0 {
            "muted"
        } else if p < 34 {
            "1"
        } else if p < 67 {
            "2"
        } else {
            "3"
        }
    }
}

/// The notches of the wheel in a scroll event: one per notch of a mouse wheel, one per
/// [`TOUCHPAD_STEP`] of a touchpad, the rest kept in `rest`. Up is positive.
pub fn wheel_steps(e: &ScrollEvent, rest: &Cell<f32>) -> i32 {
    if e.source == ScrollSource::Wheel {
        return -(e.delta_y.signum() as i32);
    }
    let total = rest.get() + e.delta_y;
    let steps = (total / TOUCHPAD_STEP).trunc();
    rest.set(total - steps * TOUCHPAD_STEP);
    -(steps as i32)
}

/// The slider, `height` high. A press or a drag on it sets the volume there; the wheel steps
/// it.
pub fn volume_slider(control: VolumeControl, height: f32) -> Div {
    let ui = Interaction::new();
    let rest = Cell::new(0.0f32);
    // The length the level runs over: the track less its margins and its border.
    let span = height - TRACK_MARGIN * 2.0 - 2.0;
    let fraction = {
        let control = control.clone();
        memo(move || control.volume().clamp(0.0, 1.0) as f32)
    };
    let muted = {
        let control = control.clone();
        memo(move || control.muted())
    };
    let grab_control = control.clone();
    // `local` and `rect` are in the same pixels: the ratio gives the logical place.
    let set_from = move |local_y: f32, rect_height: i32| {
        if rect_height <= 0 {
            return;
        }
        let y = local_y * height / rect_height as f32;
        grab_control.set(1.0 - ((y - TRACK_MARGIN - 1.0) / span) as f64);
    };
    let track = div()
        .absolute()
        .top(TRACK_MARGIN)
        .bottom(TRACK_MARGIN)
        .left(12.5)
        .w(5.0)
        .border(1.0)
        .dyn_style(|s| {
            let c = scheme();
            s.rounded(corner(2.0))
                .bg(if dark_look() { TRACK_DARK } else { TRACK_LIGHT })
                .border_color(c.field_border)
        })
        .child(
            div()
                .absolute()
                .left(0.0)
                .right(0.0)
                .bottom(0.0)
                .dyn_style(move |s| {
                    s.h((span * fraction.get()).max(0.0))
                        .rounded(corner(1.0))
                        .bg(if muted.get() {
                            scheme().placeholder
                        } else {
                            LEVEL_GREEN
                        })
                }),
        );
    let thumb = div()
        .absolute()
        .left(4.0)
        .w(22.0)
        .h(10.0)
        .border(1.0)
        .pointer_events_none()
        .fill(move || {
            let c = scheme();
            let (top, bottom) = if ui.hovered() {
                (c.button_hot_top, c.button_hot_bottom)
            } else {
                (c.button_top, c.button_bottom)
            };
            vertical_gradient(&[(0.0, top), (1.0, bottom)])
        })
        .dyn_style(move |s| {
            let c = scheme();
            s.top(1.0 + span * (1.0 - fraction.get()))
                .rounded(corner(2.0))
                .border_color(if ui.hovered() || ui.pressed() {
                    c.field_focus
                } else {
                    c.field_border
                })
        });
    div()
        .relative()
        .flex_none()
        .w(CONTROL_WIDTH)
        .h(height)
        .interaction(ui)
        .grab_threshold(0.0)
        .access(|a| a.role(Role::Slider).label("Volume"))
        .on_grab(move |e| set_from(e.local.y, e.rect.height))
        .on_scroll(move |e| {
            let steps = wheel_steps(e, &rest);
            if steps != 0 {
                e.stop_propagation();
                control.step(steps as f64 * NOTCH_PERCENT);
            }
        })
        .child(track)
        .child(thumb)
}

/// The slider lying down, `width` wide: the level fills from the left. A press or a drag sets the
/// volume there; the wheel steps it.
pub fn volume_bar(control: VolumeControl, width: f32) -> Div {
    let fraction = {
        let control = control.clone();
        memo(move || control.volume().clamp(0.0, 1.0) as f32)
    };
    let muted = {
        let control = control.clone();
        memo(move || control.muted())
    };
    let rest = Cell::new(0.0f32);
    let wheel_control = control.clone();
    horizontal_slider(
        width,
        "Volume",
        move || fraction.get(),
        move |f| control.set(f as f64),
        Some(Rc::new(move || muted.get())),
    )
    .on_scroll(move |e| {
        let steps = wheel_steps(e, &rest);
        if steps != 0 {
            e.stop_propagation();
            wheel_control.step(steps as f64 * NOTCH_PERCENT);
        }
    })
}

/// The balance slider, `width` wide: the thumb in the middle when both sides are as loud, at the
/// left end for the left side alone. `read` gives the balance from -1 to 1 (tracking); `set`
/// takes a new one. A mark shows the middle; the wheel steps it.
pub fn balance_bar(
    read: impl Fn() -> f64 + 'static,
    set: impl Fn(f64) + 'static,
    width: f32,
) -> Div {
    let read = Rc::new(read);
    let set = Rc::new(set);
    let rest = Cell::new(0.0f32);
    let (wheel_read, wheel_set) = (read.clone(), set.clone());
    let grab_set = set.clone();
    horizontal_slider(
        width,
        "Balance",
        move || ((read() + 1.0) / 2.0).clamp(0.0, 1.0) as f32,
        move |f| {
            // The middle holds the thumb a little, so that it can be put back even.
            let b = f as f64 * 2.0 - 1.0;
            grab_set(if b.abs() < 0.04 { 0.0 } else { b });
        },
        None,
    )
    .child(
        div()
            .absolute()
            .left(width / 2.0 - 0.5)
            .top(1.0)
            .w(1.0)
            .h(4.0)
            .pointer_events_none()
            .dyn_style(|s| s.bg(scheme().field_border)),
    )
    .on_scroll(move |e| {
        let steps = wheel_steps(e, &rest);
        if steps != 0 {
            e.stop_propagation();
            let now = untrack(|| wheel_read());
            wheel_set((now + steps as f64 * NOTCH_PERCENT / 100.0).clamp(-1.0, 1.0));
        }
    })
}

/// A slider lying down, `width` wide: a sunken track, filled in green to the value when `muted`
/// is given (grey while it says muted), and a thumb. `value` is from 0 to 1 (tracking); `set`
/// takes the value where it is pressed or dragged.
fn horizontal_slider(
    width: f32,
    label: &'static str,
    value: impl Fn() -> f32 + 'static,
    set: impl Fn(f32) + 'static,
    muted: Option<Rc<dyn Fn() -> bool>>,
) -> Div {
    let ui = Interaction::new();
    let value = memo(value);
    // The length the thumb's middle runs over: the track less its margins and its border.
    let span = width - TRACK_MARGIN * 2.0 - 2.0;
    let set_from = move |local_x: f32, rect_width: i32| {
        if rect_width <= 0 {
            return;
        }
        let x = local_x * width / rect_width as f32;
        set(((x - TRACK_MARGIN - 1.0) / span).clamp(0.0, 1.0));
    };
    let mut track = div()
        .absolute()
        .left(TRACK_MARGIN)
        .right(TRACK_MARGIN)
        .top(12.5)
        .h(5.0)
        .border(1.0)
        .dyn_style(|s| {
            let c = scheme();
            s.rounded(corner(2.0))
                .bg(if dark_look() { TRACK_DARK } else { TRACK_LIGHT })
                .border_color(c.field_border)
        });
    if let Some(muted) = muted {
        track = track.child(
            div()
                .absolute()
                .top(0.0)
                .bottom(0.0)
                .left(0.0)
                .dyn_style(move |s| {
                    s.w((span * value.get()).max(0.0))
                        .rounded(corner(1.0))
                        .bg(if muted() {
                            scheme().placeholder
                        } else {
                            LEVEL_GREEN
                        })
                }),
        );
    }
    let thumb = div()
        .absolute()
        .top(4.0)
        .w(10.0)
        .h(22.0)
        .border(1.0)
        .pointer_events_none()
        .fill(move || {
            let c = scheme();
            let (top, bottom) = if ui.hovered() {
                (c.button_hot_top, c.button_hot_bottom)
            } else {
                (c.button_top, c.button_bottom)
            };
            vertical_gradient(&[(0.0, top), (1.0, bottom)])
        })
        .dyn_style(move |s| {
            let c = scheme();
            s.left(1.0 + span * value.get())
                .rounded(corner(2.0))
                .border_color(if ui.hovered() || ui.pressed() {
                    c.field_focus
                } else {
                    c.field_border
                })
        });
    div()
        .relative()
        .flex_none()
        .w(width)
        .h(CONTROL_WIDTH)
        .interaction(ui)
        .grab_threshold(0.0)
        .access(move |a| a.role(Role::Slider).label(label))
        .on_grab(move |e| set_from(e.local.x, e.rect.width))
        .child(track)
        .child(thumb)
}

/// The mute button: the speaker, its waves for the level, the red mark when muted.
pub fn mute_button(control: VolumeControl) -> Div {
    let ui = Interaction::new();
    let level = {
        let control = control.clone();
        memo(move || control.level())
    };
    div()
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .w(CONTROL_WIDTH)
        .h(26.0)
        .border(1.0)
        .interaction(ui)
        .access(|a| a.role(Role::Button).label("Mute"))
        .on_click(move |_| control.toggle_mute())
        .dyn_style(move |s| {
            let c = scheme();
            let s = s.rounded(button_radius());
            if ui.hovered() {
                s.bg(c.hover).border_color(c.selection_border)
            } else {
                s.bg(Color::TRANSPARENT).border_color(Color::TRANSPARENT)
            }
        })
        .child(
            img(move || ink_asset(&format!("speaker-{}.svg", level.get())))
                .size(18.0)
                .object_fit(ObjectFit::Contain),
        )
}
