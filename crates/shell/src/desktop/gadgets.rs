//! The desktop gadgets (QML `GadgetFrame`, `CpuMeter`, `ClockGadget`, `SystemStats`), on the
//! main screen. Each keeps its distance from the right and top edges of the desktop (the screen
//! less the bar) in a setting, and a drag anywhere on it moves it. Its buttons, if any, show on
//! its right edge under the pointer, as Windows 7's gadget toolbar does.
//!
//! The gadgets are drawn here, in the manner of the Windows 7 gadgets, not copied from them. Their
//! colours are their own (dark glass, a pale clock face), in both themes, as on Windows.

use crate::programs::{program, run};
use crate::settings::{Place, change, motion, settings};
use crate::theme::{bar_above, bar_below, corner};
use crate::ui::{font, shell_icon, vertical_gradient};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use std::cell::Cell;
use std::f64::consts::PI;
use std::rc::Rc;
use std::time::Duration;

/// How often the CPU Meter reads the processor and the memory.
const STATS_EVERY: Duration = Duration::from_millis(1500);
/// OutCubic, the easing of the QML gadgets.
const OUT_CUBIC: (f32, f32, f32, f32) = (0.33, 1.0, 0.68, 1.0);

/// The gadget toolbar's buttons.
const TOOL_SIZE: f32 = 20.0;
const TOOL_ICON: f32 = 14.0;
const TOOL_HOT: Color = rgba(0xffffff66);
const TOOL_REST: Color = rgba(0x00000033);
const TOOL_BORDER: Color = rgba(0xffffff80);

/// Which gadget, for its setting.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Gadget {
    CpuMeter,
    Clock,
}

impl Gadget {
    fn place(self) -> Place {
        settings().with(|s| match self {
            Gadget::CpuMeter => s.cpu_meter_place,
            Gadget::Clock => s.clock_gadget_place,
        })
    }

    fn set_place(self, place: Place) {
        change(|s| match self {
            Gadget::CpuMeter => s.cpu_meter_place = place,
            Gadget::Clock => s.clock_gadget_place = place,
        });
    }
}

/// The gadgets that are switched on, for the main screen's desktop of size `area`.
pub fn view(area: Signal<(f32, f32)>) -> impl IntoView {
    fragment([
        show(
            || settings().with(|s| s.show_cpu_meter),
            move || {
                let monitor: Rc<dyn Fn()> = Rc::new(|| run(&program("systemMonitor")));
                frame(
                    Gadget::CpuMeter,
                    area,
                    (METER_WIDTH, METER_HEIGHT),
                    vec![("utilities-system-monitor", monitor)],
                    cpu_meter(),
                )
            },
        ),
        show(
            || settings().with(|s| s.show_clock_gadget),
            move || {
                frame(
                    Gadget::Clock,
                    area,
                    (CLOCK_SIZE, CLOCK_SIZE),
                    Vec::new(),
                    clock(),
                )
            },
        ),
    ])
}

/// A button of a gadget's toolbar: its icon's name and what it does.
type GadgetAction = (&'static str, Rc<dyn Fn()>);

/// A gadget's place: `content`, `size` big, kept where its setting says and moved by a drag.
fn frame(
    gadget: Gadget,
    area: Signal<(f32, f32)>,
    size: (f32, f32),
    actions: Vec<GadgetAction>,
    content: Div,
) -> Div {
    let (w, h) = size;
    let ui = Interaction::new();
    let from = Rc::new(Cell::new(Place {
        right: 0.0,
        top: 0.0,
    }));
    let has_tools = !actions.is_empty();
    let tools = div()
        .absolute()
        .right(4.0)
        .top(4.0)
        .flex()
        .flex_col()
        .gap(2.0)
        .dyn_style(move |s| {
            if has_tools && ui.hovered() {
                s
            } else {
                s.hidden()
            }
        })
        .children(
            actions
                .into_iter()
                .map(|(icon, run)| tool_button(icon, run)),
        );
    div()
        .absolute()
        .w(w)
        .h(h)
        .select_none()
        .interaction(ui)
        .dyn_style(move |s| {
            let place = gadget.place();
            let (aw, _) = area.get();
            let s = s.left(aw - w - place.right).top(bar_above() + place.top);
            s.cursor(if ui.pressed() {
                CursorIcon::Grabbing
            } else {
                CursorIcon::Default
            })
        })
        .on_grab(move |g| match g.phase {
            GrabPhase::Start => from.set(untrack(|| gadget.place())),
            GrabPhase::Move => {
                let start = from.get();
                let (aw, ah) = area.get_untracked();
                let room = untrack(|| ah - bar_above() - bar_below());
                gadget.set_place(Place {
                    right: (start.right - g.total.x).min(aw - w).max(0.0).round(),
                    top: (start.top + g.total.y).min(room - h).max(0.0).round(),
                });
            }
            GrabPhase::End => {}
            GrabPhase::Cancel => gadget.set_place(from.get()),
        })
        .child(content)
        .child(tools)
}

fn tool_button(icon: &'static str, run: Rc<dyn Fn()>) -> Div {
    let ui = Interaction::new();
    div()
        .w(TOOL_SIZE)
        .h(TOOL_SIZE)
        .border(1.0)
        .border_color(TOOL_BORDER)
        .flex()
        .items_center()
        .justify_center()
        .interaction(ui)
        .dyn_style(move |s| {
            s.rounded(corner(3.0))
                .bg(if ui.hovered() { TOOL_HOT } else { TOOL_REST })
        })
        .on_click(move |e| {
            e.stop_propagation();
            run();
        })
        .child(shell_icon(icon, TOOL_ICON))
}

// ----- the CPU Meter ---------------------------------------------------------------------------

const METER_WIDTH: f32 = 196.0;
const METER_HEIGHT: f32 = 124.0;
const METER_SHADOW: Color = rgba(0x00000055);
const METER_BORDER: Color = rgba(0x0b1620cc);
const METER_TOP: Color = rgba(0x455868e6);
const METER_MIDDLE: Color = rgba(0x1b2733e6);
const METER_BOTTOM: Color = rgba(0x0c141ce6);
const METER_INNER_LINE: Color = rgba(0xffffff40);
const METER_GLOSS: Color = rgba(0xffffff38);
const METER_GLOSS_END: Color = rgba(0xffffff00);
const DIAL_TRACK: Color = rgba(0x0000008c);
const DIAL_TRACK_LIGHT: Color = rgba(0xffffff1f);
/// The value goes from green through yellow to red.
const DIAL_LOW: Color = rgb(0x5fd35a);
const DIAL_MIDDLE: Color = rgb(0xf2c335);
const DIAL_HIGH: Color = rgb(0xe5463a);
const DIAL_VALUE_SHADOW: Color = rgba(0x00000080);
const DIAL_LABEL: Color = rgb(0xb8cbdc);
const DIAL_DETAIL: Color = rgb(0x8fa6ba);
const GIB: f64 = 1_073_741_824.0;

/// Two dials on a plate of dark glass: the processor on the left and the memory on the right,
/// each with its percentage. It reads only while it shows.
fn cpu_meter() -> Div {
    let stats = system_every(STATS_EVERY);
    let cpu = {
        let stats = stats.clone();
        move || stats.cpu() as f32
    };
    let memory = {
        let stats = stats.clone();
        move || {
            let m = stats.memory();
            if m.total > 0 {
                m.used as f32 / m.total as f32
            } else {
                0.0
            }
        }
    };
    let detail = move || {
        let m = stats.memory();
        if m.total > 0 {
            format!(
                "{:.1} / {} GB",
                m.used as f64 / GIB,
                (m.total as f64 / GIB).round() as u64
            )
        } else {
            String::new()
        }
    };
    div()
        .relative()
        .w(METER_WIDTH)
        .h(METER_HEIGHT)
        // A drop shadow, then the glass.
        .child(
            div()
                .absolute()
                .left(2.0)
                .top(3.0)
                .right(-2.0)
                .bottom(-4.0)
                .bg(METER_SHADOW)
                .dyn_style(|s| s.rounded(corner(14.0))),
        )
        .child(
            div()
                .absolute()
                .inset(0.0)
                .border(1.0)
                .border_color(METER_BORDER)
                .fill(|| {
                    vertical_gradient(&[(0.0, METER_TOP), (0.5, METER_MIDDLE), (1.0, METER_BOTTOM)])
                })
                .dyn_style(|s| {
                    // The light line inside the outline.
                    s.rounded(corner(14.0)).shadow(BoxShadow {
                        spread: 1.0,
                        color: METER_INNER_LINE,
                        inset: true,
                        ..Default::default()
                    })
                })
                // The gloss on the upper half.
                .child(
                    div()
                        .absolute()
                        .left(2.0)
                        .top(2.0)
                        .right(2.0)
                        .h(METER_HEIGHT * 0.45)
                        .fill(|| vertical_gradient(&[(0.0, METER_GLOSS), (1.0, METER_GLOSS_END)]))
                        .dyn_style(|s| s.rounded(corner(12.0))),
                ),
        )
        .child(dial((8.0, 8.0), 104.0, cpu, "CPU", String::new))
        .child(dial((116.0, 26.0), 72.0, memory, "RAM", detail))
}

/// The SVG path of an arc about (`cx`, `cy`), from `start` degrees (clockwise from three
/// o'clock, y down) over `sweep` degrees.
fn arc(cx: f64, cy: f64, r: f64, start: f64, sweep: f64) -> String {
    let point = |deg: f64| {
        let a = deg * PI / 180.0;
        (cx + r * a.cos(), cy + r * a.sin())
    };
    let (x0, y0) = point(start);
    let (x1, y1) = point(start + sweep);
    let large = if sweep.abs() > 180.0 { 1 } else { 0 };
    let clockwise = if sweep >= 0.0 { 1 } else { 0 };
    format!("M {x0} {y0} A {r} {r} 0 {large} {clockwise} {x1} {y1}")
}

/// A gauge from 7 to 5 o'clock: the track, then the value in a green to red gradient, the
/// percentage and the label in the middle and a detail under it.
fn dial(
    at: (f32, f32),
    size: f32,
    value: impl Fn() -> f32 + Clone + 'static,
    label: &'static str,
    detail: impl Fn() -> String + 'static,
) -> Div {
    let cx = (size / 2.0) as f64;
    let cy = (size / 2.0 + 2.0) as f64;
    let r = (size / 2.0 - 6.0) as f64;
    let target = value.clone();
    let shown = follow(
        move || target().clamp(0.0, 1.0),
        Transition::ms(untrack(|| motion(400.0))).cubic_bezier(
            OUT_CUBIC.0,
            OUT_CUBIC.1,
            OUT_CUBIC.2,
            OUT_CUBIC.3,
        ),
    );
    let track = arc(cx, cy, r, 135.0, 270.0);
    let band = Gradient::linear((cx - r) as f32, 0.0, (cx + r) as f32, 0.0)
        .units(GradientUnits::UserSpace)
        .stop(0.0, DIAL_LOW)
        .stop(0.6, DIAL_MIDDLE)
        .stop(1.0, DIAL_HIGH);
    let gauge = svg().absolute().left(0.0).top(0.0).w(size).h(size).child(
        svg::group()
            .fill_none()
            .line_cap(LineCap::Round)
            .child(
                svg::path(track.clone())
                    .stroke(DIAL_TRACK)
                    .stroke_width(6.0),
            )
            .child(svg::path(track).stroke(DIAL_TRACK_LIGHT).stroke_width(4.0))
            .child(
                svg::path(move || {
                    let sweep = 270.0 * shown.get() as f64;
                    // Nothing for nothing: a round cap alone would be a dot.
                    if sweep < 1.35 {
                        String::new()
                    } else {
                        arc(cx, cy, r, 135.0, sweep)
                    }
                })
                .stroke(band)
                .stroke_width(4.0),
            ),
    );
    let big = size > 90.0;
    let percent = move || format!("{}%", (value() * 100.0).round() as i64);
    let percent_shadow = percent.clone();
    div()
        .absolute()
        .left(at.0)
        .top(at.1)
        .w(size)
        .h(size)
        .dyn_style(|s| s.font_family(font()))
        .child(gauge)
        .child(
            div()
                .absolute()
                .inset(0.0)
                .pt(8.0)
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .child(
                    // Raised: a dark copy a pixel below.
                    div()
                        .relative()
                        .text_size(if big { 18.0 } else { 14.0 })
                        .font_weight(FontWeight::BOLD)
                        .child(
                            div()
                                .absolute()
                                .left(0.0)
                                .top(1.0)
                                .text_color(DIAL_VALUE_SHADOW)
                                .child(text(percent_shadow)),
                        )
                        .child(
                            div()
                                .relative()
                                .text_color(Color::WHITE)
                                .child(text(percent)),
                        ),
                )
                .child(
                    div()
                        .mt(-1.0)
                        .text_size(10.0)
                        .text_color(DIAL_LABEL)
                        .child(label),
                ),
        )
        .child(
            div()
                .absolute()
                .left(0.0)
                .right(0.0)
                .bottom(-2.0)
                .flex()
                .justify_center()
                .text_size(9.0)
                .text_color(DIAL_DETAIL)
                .whitespace_nowrap()
                .child(text(detail)),
        )
}

// ----- the Clock -------------------------------------------------------------------------------

const CLOCK_SIZE: f32 = 140.0;
const CLOCK_SHADOW: Color = rgba(0x00000050);
const RIM_TOP: Color = rgb(0x5d6f80);
const RIM_BOTTOM: Color = rgb(0x141c24);
const FACE_LIGHT: Color = rgb(0xffffff);
const FACE: Color = rgb(0xd6e2ec);
const MARKS: Color = rgb(0x3a4c5c);
const HANDS: Color = rgb(0x1f2b36);
const SECONDS_HAND: Color = rgb(0xc42b1c);
const GLOSS: Color = rgba(0xffffff8c);
const GLOSS_END: Color = rgba(0xffffff00);
const DATE: Color = rgb(0x4d6578);

/// A round face of pale glass in a dark rim, twelve marks, the hour and minute hands and a thin
/// red seconds hand, and the day under the centre.
fn clock() -> Div {
    let c = (CLOCK_SIZE / 2.0) as f64;
    let r = (CLOCK_SIZE / 2.0 - 3.0) as f64;
    // The rim: dark, lighter at the top.
    let rim = svg::circle(c, c, r).fill(
        Gradient::linear(0.0, 0.0, 0.0, 1.0)
            .stop(0.0, RIM_TOP)
            .stop(1.0, RIM_BOTTOM),
    );
    // The face: pale glass, brighter towards the top left.
    let glass = Gradient {
        shape: GradientShape::Radial {
            cx: c as f32,
            cy: c as f32,
            r: (r * 0.9) as f32,
            fx: (c - r * 0.3) as f32,
            fy: (c - r * 0.35) as f32,
        },
        ..Gradient::radial(0.0, 0.0, 0.0)
    }
    .units(GradientUnits::UserSpace)
    .stop(0.0, FACE_LIGHT)
    .stop(1.0, FACE);
    let face = svg::circle(c, c, r - 7.0).fill(glass);
    // The marks: long at the quarters.
    let marks = (0..12).map(|i| {
        let angle = i as f64 * PI / 6.0;
        let quarter = i % 3 == 0;
        let outer = r - 12.0;
        let inner = outer - if quarter { 9.0 } else { 5.0 };
        let (s, k) = angle.sin_cos();
        svg::line(c + s * inner, c - k * inner, c + s * outer, c - k * outer)
            .stroke(MARKS)
            .stroke_width(if quarter { 2.5 } else { 1.5 })
    });
    // One hand: a rounded bar from a little behind the centre to its tip, turned about the
    // centre; `turn` gives degrees clockwise from twelve.
    let hand =
        move |length: f64, back: f64, width: f32, color: Color, turn: Rc<dyn Fn() -> f64>| {
            let cap = width as f64 / 2.0;
            svg::path(move || {
                let (s, k) = turn().to_radians().sin_cos();
                let (from, to) = (-(back - cap), length - cap);
                format!(
                    "M {} {} L {} {}",
                    c + s * from,
                    c - k * from,
                    c + s * to,
                    c - k * to
                )
            })
            .stroke(color)
            .stroke_width(width)
        };
    let now = || local_time(Every::Second);
    let seconds = move || now().second as f64;
    let minutes = move || now().minute as f64 + seconds() / 60.0;
    let hours = move || (now().hour % 12) as f64 + minutes() / 60.0;
    // The gloss over the upper half of the glass.
    let gloss_top = c - r + 9.0;
    let gloss = svg::ellipse(c, gloss_top + r * 0.45, r - 12.0, r * 0.45).fill(
        Gradient::linear(0.0, (c - r) as f32, 0.0, c as f32)
            .units(GradientUnits::UserSpace)
            .stop(0.0, GLOSS)
            .stop(1.0, GLOSS_END),
    );
    let picture = svg()
        .absolute()
        .left(0.0)
        .top(0.0)
        .w(CLOCK_SIZE)
        .h(CLOCK_SIZE)
        .child(
            svg::group()
                .child(rim)
                .child(face)
                .child(
                    svg::group()
                        .fill_none()
                        .line_cap(LineCap::Round)
                        .children(marks),
                )
                .child(
                    svg::group()
                        .fill_none()
                        .line_cap(LineCap::Round)
                        .child(hand(
                            r * 0.45,
                            6.0,
                            4.5,
                            HANDS,
                            Rc::new(move || hours() * 30.0),
                        ))
                        .child(hand(
                            r * 0.68,
                            8.0,
                            3.0,
                            HANDS,
                            Rc::new(move || minutes() * 6.0),
                        ))
                        .child(hand(
                            r * 0.74,
                            12.0,
                            1.2,
                            SECONDS_HAND,
                            Rc::new(move || seconds() * 6.0),
                        )),
                )
                .child(svg::circle(c, c, 3.0).fill(SECONDS_HAND))
                .child(gloss),
        );
    div()
        .relative()
        .w(CLOCK_SIZE)
        .h(CLOCK_SIZE)
        // A soft shadow under the face.
        .child(
            div()
                .absolute()
                .left(4.0)
                .top(6.0)
                .w(CLOCK_SIZE - 4.0)
                .h(CLOCK_SIZE - 4.0)
                .rounded((CLOCK_SIZE - 4.0) / 2.0)
                .bg(CLOCK_SHADOW),
        )
        .child(picture)
        .child(
            div()
                .absolute()
                .left(0.0)
                .right(0.0)
                .top(CLOCK_SIZE * 0.64)
                .flex()
                .justify_center()
                .text_size(10.0)
                .text_color(DATE)
                .dyn_style(|s| s.font_family(font()))
                .child(text(move || now().format("%a %-d"))),
        )
}
