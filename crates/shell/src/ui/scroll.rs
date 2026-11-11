//! A box that scrolls with a scroll bar drawn as Windows 7 draws one (QML `AeroScrollBar`): a
//! pale trough, an arrow button at each end, and a thumb with a soft gradient and a grip that
//! turns blue under the pointer.
//!
//! AntiWebUI draws its own scroll bars with one colour only, so that one is hidden and this
//! bar stands beside the scrolled box, following its offset and moving it.

use super::dark_look;
use super::gradient::horizontal_gradient;
use crate::theme::corner;
use anti_webui::prelude::*;
use std::rc::Rc;
use std::time::Duration;

/// The width of the bar and the height of each arrow button.
const THICKNESS: f32 = 17.0;
/// The shortest the thumb gets, as a fraction of the track (Qt's `minimumSize`).
const MINIMUM_THUMB: f32 = 0.08;
/// A held arrow starts to repeat after this, then steps at the second interval.
const REPEAT_AFTER: Duration = Duration::from_millis(400);
const REPEAT_EVERY: Duration = Duration::from_millis(60);
/// An arrow steps a tenth of the content, as Qt's `increase()` does.
const STEP: f32 = 0.1;

/// The QML bar's colours, which are its own (`Scheme` has none for it).
struct BarColours {
    trough: [Color; 3],
    trough_border: Color,
    /// The border of an arrow while the bar is lit, and of the thumb at rest.
    lit_border: Color,
    thumb_border: Color,
    /// The border of what is under the pointer.
    hot_border: Color,
    arrow_lit: [Color; 2],
    arrow_hot: [Color; 2],
    arrow_pressed: [Color; 2],
    glyph: Color,
    glyph_hot: Color,
    thumb_rest: [Color; 4],
    thumb_hot: [Color; 4],
    thumb_pressed: [Color; 4],
    thumb_line: Color,
    grip: Color,
    grip_lit: Color,
}

const LIGHT: BarColours = BarColours {
    trough: [rgb(0xe3e3e3), rgb(0xf1f1f1), rgb(0xe3e3e3)],
    trough_border: rgb(0xd7d7d7),
    lit_border: rgb(0x9da8b2),
    thumb_border: rgb(0x9b9b9b),
    hot_border: rgb(0x3c7fb1),
    arrow_lit: [rgb(0xf3f3f3), rgb(0xdcdcdc)],
    arrow_hot: [rgb(0xe3f4fc), rgb(0xc6e8fa)],
    arrow_pressed: [rgb(0xc2e4f6), rgb(0xa9d9f2)],
    glyph: rgb(0x606060),
    glyph_hot: rgb(0x1c5180),
    thumb_rest: [rgb(0xf3f3f3), rgb(0xe8e8e8), rgb(0xd9d9d9), rgb(0xcdcdcd)],
    thumb_hot: [rgb(0xe3f4fc), rgb(0xd0ecfa), rgb(0xbde4f8), rgb(0xa9dcf6)],
    thumb_pressed: [rgb(0xcde9f8), rgb(0xb4def4), rgb(0x9fd4f0), rgb(0x8ccaec)],
    thumb_line: rgba(0xffffff66),
    grip: rgb(0x8e8e8e),
    grip_lit: rgb(0x3c7fb1),
};

const DARK: BarColours = BarColours {
    trough: [rgb(0x26303a), rgb(0x2c3743), rgb(0x26303a)],
    trough_border: rgb(0x1b232b),
    lit_border: rgb(0x56687a),
    thumb_border: rgb(0x56687a),
    hot_border: rgb(0x4b8fd0),
    arrow_lit: [rgb(0x3a4855), rgb(0x313e4a)],
    arrow_hot: [rgb(0x2a4660), rgb(0x223c53)],
    arrow_pressed: [rgb(0x2d5578), rgb(0x244a6b)],
    glyph: rgb(0xa7b4c0),
    glyph_hot: rgb(0xcfe6fa),
    thumb_rest: [rgb(0x4a5967), rgb(0x43515e), rgb(0x3c4955), rgb(0x36424d)],
    thumb_hot: [rgb(0x365b7d), rgb(0x305273), rgb(0x2a4968), rgb(0x25415d)],
    thumb_pressed: [rgb(0x3c6890), rgb(0x355f85), rgb(0x2e5579), rgb(0x284d6f)],
    thumb_line: rgba(0xffffff1f),
    grip: rgb(0x22303c),
    grip_lit: rgb(0x9cc8ee),
};

fn colours() -> &'static BarColours {
    if dark_look() { &DARK } else { &LIGHT }
}

/// Where the thumb is, in logical pixels from the top of the bar.
#[derive(Clone, Copy, PartialEq)]
struct Thumb {
    top: f32,
    length: f32,
    /// How far the content scrolls for a pixel the thumb moves.
    ratio: f32,
}

/// `content` in a box that scrolls up and down, with the Windows 7 bar at its right while the
/// content is taller than the box. Give what comes back its height (`h`, `flex_1`, `max_h`).
pub fn aero_scroll(content: impl IntoView) -> Div {
    let view = node_ref();
    let offset = signal(0.0f32);
    let view_height = signal(0.0f32);
    let content_height = signal(0.0f32);
    let viewport = div()
        .flex_1()
        .min_w(0.0)
        .min_h(0.0)
        .flex()
        .flex_col()
        .overflow_y_scroll()
        .scrollbar_color(Color::TRANSPARENT)
        .node_ref(view)
        .on_scrolled(move |p| offset.set(p.y))
        .on_resized(move |_, h| view_height.set(h))
        .child(
            div()
                .flex_none()
                .flex()
                .flex_col()
                .on_resized(move |_, h| content_height.set(h))
                .child(content),
        );
    let overflows = memo(move || content_height.get() > view_height.get() + 0.5);
    div()
        .flex()
        .flex_row()
        .min_h(0.0)
        .child(viewport)
        .child(show(
            move || overflows.get(),
            move || bar(view, offset, view_height, content_height),
        ))
}

fn bar(
    view: NodeRef,
    offset: Signal<f32>,
    view_height: Signal<f32>,
    content_height: Signal<f32>,
) -> Div {
    let bar_ui = Interaction::new();
    let dragging = signal(false);
    let drag_from = signal(0.0f32);
    let thumb = memo(move || {
        let (view_h, content_h) = (view_height.get(), content_height.get());
        let track = (view_h - 2.0 * THICKNESS).max(0.0);
        let length = (track * view_h / content_h.max(1.0))
            .max(track * MINIMUM_THUMB)
            .min(track);
        let range = (content_h - view_h).max(0.0);
        let room = (track - length).max(0.0);
        let top = THICKNESS
            + if range > 0.0 {
                room * offset.get() / range
            } else {
                0.0
            };
        Thumb {
            top,
            length,
            ratio: if room > 0.0 { range / room } else { 0.0 },
        }
    });
    let scroll_by = move |by: f32| {
        view.scroll_to(0.0, offset.get_untracked() + by);
    };
    let lit = move || bar_ui.hovered() || dragging.get();
    // A press on the trough pages towards it.
    let page = move |e: &PointerEvent| {
        let y = e.local.y / window_scale().as_f32();
        let t = untrack(|| thumb.get());
        let page = view_height.get_untracked();
        if y < t.top {
            scroll_by(-page);
        } else if y > t.top + t.length {
            scroll_by(page);
        }
    };
    let grip = div()
        .flex()
        .flex_col()
        .gap(1.0)
        .dyn_style(move |s| {
            if thumb.get().length > 20.0 {
                s
            } else {
                s.hidden()
            }
        })
        .children((0..4).map(move |_| {
            div().w(7.0).h(1.0).dyn_style(move |s| {
                let c = colours();
                s.bg(if lit() { c.grip_lit } else { c.grip })
            })
        }));
    // Placed from the bar's outer edge, over its border, as the QML thumb was.
    let thumb_box = div()
        .absolute()
        .left(-1.0)
        .w(THICKNESS)
        .border(1.0)
        .flex()
        .items_center()
        .justify_center()
        .grab_threshold(0.0)
        .on_click(|e| e.stop_propagation())
        .on_grab(move |e| match e.phase {
            GrabPhase::Start => {
                drag_from.set(offset.get_untracked());
                dragging.set(true);
            }
            GrabPhase::Move => {
                let to = drag_from.get_untracked() + e.total.y * untrack(|| thumb.get()).ratio;
                view.scroll_to(0.0, to);
            }
            GrabPhase::End | GrabPhase::Cancel => dragging.set(false),
        })
        .fill(move || {
            let c = colours();
            let stops = if dragging.get() {
                c.thumb_pressed
            } else if bar_ui.hovered() {
                c.thumb_hot
            } else {
                c.thumb_rest
            };
            horizontal_gradient(&[
                (0.0, stops[0]),
                (0.45, stops[1]),
                (0.55, stops[2]),
                (1.0, stops[3]),
            ])
        })
        .dyn_style(move |s| {
            let c = colours();
            let t = thumb.get();
            s.top(t.top - 1.0)
                .h(t.length)
                .rounded(corner(3.0))
                .border_color(if lit() { c.hot_border } else { c.thumb_border })
                .shadow(BoxShadow {
                    spread: 1.0,
                    color: c.thumb_line,
                    inset: true,
                    ..Default::default()
                })
        })
        .child(grip);
    let step = move || content_height.get_untracked() * STEP;
    div()
        .relative()
        .flex_none()
        .w(THICKNESS)
        .border(1.0)
        .select_none()
        .interaction(bar_ui)
        .on_click(page)
        .fill(|| {
            let c = colours();
            horizontal_gradient(&[(0.0, c.trough[0]), (0.5, c.trough[1]), (1.0, c.trough[2])])
        })
        .dyn_style(|s| s.border_color(colours().trough_border))
        .child(arrow(false, lit, Rc::new(move || scroll_by(-step()))))
        .child(arrow(true, lit, Rc::new(move || scroll_by(step()))))
        .child(thumb_box)
}

/// An arrow button at an end of the bar: `down` at the bottom. It steps once when clicked, and
/// again and again while it is held.
fn arrow(down: bool, bar_lit: impl Fn() -> bool + 'static, step: Rc<dyn Fn()>) -> Div {
    let bar_lit = Rc::new(bar_lit);
    let lit_for_border = bar_lit.clone();
    let ui = Interaction::new();
    let held = signal(false);
    let repeat = step.clone();
    let el = div().absolute().left(-1.0).w(THICKNESS).h(THICKNESS);
    let el = if down { el.bottom(-1.0) } else { el.top(-1.0) };
    el.border(1.0)
        .flex()
        .items_center()
        .justify_center()
        .interaction(ui)
        .on_active(move |on| {
            held.set(on);
            if on {
                repeat_while(held, repeat.clone(), REPEAT_AFTER);
            }
        })
        .on_click(move |e| {
            e.stop_propagation();
            step();
        })
        .fill(move || {
            let c = colours();
            let stops = if ui.pressed() {
                c.arrow_pressed
            } else if ui.hovered() {
                c.arrow_hot
            } else if bar_lit() {
                c.arrow_lit
            } else {
                [Color::TRANSPARENT; 2]
            };
            horizontal_gradient(&[(0.0, stops[0]), (1.0, stops[1])])
        })
        .dyn_style(move |s| {
            let c = colours();
            let border = if ui.hovered() {
                c.hot_border
            } else if lit_for_border() {
                c.lit_border
            } else {
                Color::TRANSPARENT
            };
            s.rounded(corner(2.0))
                .border_color(border)
                .text_color(if ui.hovered() { c.glyph_hot } else { c.glyph })
        })
        .child(text(if down { "\u{25BC}" } else { "\u{25B2}" }).text_size(7.0))
}

/// Runs `step` after `wait`, then every [`REPEAT_EVERY`], for as long as `held` stays true.
fn repeat_while(held: Signal<bool>, step: Rc<dyn Fn()>, wait: Duration) {
    set_timeout(wait, move || {
        if held.is_alive() && held.get_untracked() {
            step();
            repeat_while(held, step, REPEAT_EVERY);
        }
    });
}
