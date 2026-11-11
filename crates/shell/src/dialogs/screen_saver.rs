//! The screen saver, as Windows 7 runs one (QML `ScreenSaver`): after some minutes without
//! input, over everything, until a key or the mouse moves. "Mystify" here is drawn for this
//! desktop: two four-sided shapes whose corners bounce off the edges, each leaving a trail of
//! fading copies in slowly changing colours.

use super::saver_preview;
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant, SystemTime};

/// The time between two steps of the shapes.
const STEP: f32 = 0.033;
/// How many copies a shape's trail keeps, itself included.
const TRAIL: usize = 7;
/// The width of the lines.
const LINE_WIDTH: f32 = 1.6;
/// How far the hue goes on at each step.
const HUE_STEP: f32 = 0.002;
/// How far apart the two shapes' hues are.
const HUE_APART: f32 = 0.45;
/// The colour of the shapes: saturation and lightness.
const SATURATION: f32 = 0.9;
const LIGHTNESS: f32 = 0.6;
/// After the preview starts, how long the mouse can move before it ends it: time to let go of
/// the button that started it.
const PREVIEW_GRACE: Duration = Duration::from_millis(800);

/// A corner of a shape and its speed, in logical pixels per step.
#[derive(Clone, Copy)]
struct Corner {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
}

/// The shapes, their trails, and the hue.
struct Mystify {
    width: f32,
    height: f32,
    shapes: [[Corner; 4]; 2],
    /// Each shape's last places, the newest first.
    trails: [Vec<[(f32, f32); 4]>; 2],
    hue: f32,
    steps: u64,
}

/// A small random number source (xorshift), seeded from the clock.
struct Random(u64);

impl Random {
    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_or(1, |d| d.as_nanos() as u64);
        Self(nanos | 1)
    }

    /// From 0 to 1.
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }

    /// A speed from 1.5 to 3.5, either way.
    fn speed(&mut self) -> f32 {
        let size = self.next() * 2.0 + 1.5;
        if self.next() < 0.5 { -size } else { size }
    }
}

impl Mystify {
    fn new(width: f32, height: f32) -> Self {
        let mut random = Random::new();
        let mut corner = || Corner {
            x: random.next() * width,
            y: random.next() * height,
            dx: random.speed(),
            dy: random.speed(),
        };
        let shapes = [
            [corner(), corner(), corner(), corner()],
            [corner(), corner(), corner(), corner()],
        ];
        Self {
            width,
            height,
            shapes,
            trails: [Vec::new(), Vec::new()],
            hue: 0.0,
            steps: 0,
        }
    }

    fn step(&mut self) {
        for (shape, trail) in self.shapes.iter_mut().zip(self.trails.iter_mut()) {
            for p in shape.iter_mut() {
                p.x += p.dx;
                p.y += p.dy;
                if p.x < 0.0 || p.x > self.width {
                    p.dx = -p.dx;
                    p.x = p.x.clamp(0.0, self.width);
                }
                if p.y < 0.0 || p.y > self.height {
                    p.dy = -p.dy;
                    p.y = p.y.clamp(0.0, self.height);
                }
            }
            trail.insert(0, (*shape).map(|p| (p.x, p.y)));
            trail.truncate(TRAIL);
        }
        self.hue = (self.hue + HUE_STEP) % 1.0;
        self.steps += 1;
    }

    /// The `copy`th place of a shape, as path data; "" when the trail is not that long yet.
    fn outline(&self, shape: usize, copy: usize) -> String {
        let Some(corners) = self.trails[shape].get(copy) else {
            return String::new();
        };
        let [a, b, c, d] = corners;
        format!(
            "M{} {} L{} {} L{} {} L{} {} Z",
            a.0, a.1, b.0, b.1, c.0, c.1, d.0, d.1
        )
    }

    /// The colour of the `copy`th place of a shape: older copies fade.
    fn colour(&self, shape: usize, copy: usize) -> Color {
        let length = self.trails[shape].len().max(1) as f32;
        let hue = (self.hue + shape as f32 * HUE_APART) % 1.0;
        hsla(hue, SATURATION, LIGHTNESS, 1.0 - copy as f32 / length)
    }
}

/// A colour from hue, saturation and lightness, each from 0 to 1 (Qt's `Qt.hsla`).
fn hsla(hue: f32, saturation: f32, lightness: f32, alpha: f32) -> Color {
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let h = hue * 6.0;
    let x = chroma * (1.0 - (h % 2.0 - 1.0).abs());
    let (r, g, b) = match h as u32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let m = lightness - chroma / 2.0;
    Color::new(r + m, g + m, b + m, alpha.clamp(0.0, 1.0))
}

/// The screen saver on `output`: black, with the shapes moving on it. Moving the mouse ends a
/// preview.
pub(super) fn view(output: Output) -> Div {
    let size = untrack(|| {
        output
            .current()
            .map_or(output.logical_size, |o| o.logical_size)
    });
    let mystify = Rc::new(RefCell::new(Mystify::new(
        size.width as f32,
        size.height as f32,
    )));
    // A step each 33 ms, counted from the clock, which goes on frame by frame while this lives.
    let time = clock();
    let steps = signal(0u64);
    {
        let mystify = mystify.clone();
        effect(move || {
            let due = (time.get() / STEP) as u64;
            let mut m = mystify.borrow_mut();
            // After a long wait, as when the screen was off, it does not run to catch up.
            if due > m.steps + 4 {
                m.steps = due - 4;
            }
            while m.steps < due {
                m.step();
            }
            steps.set(m.steps);
        });
    }
    let mut lines = Vec::new();
    for shape in 0..2 {
        for copy in 0..TRAIL {
            let outline = mystify.clone();
            let colour = mystify.clone();
            lines.push(
                svg::path(move || {
                    steps.get();
                    outline.borrow().outline(shape, copy)
                })
                .dyn_style(move |s| {
                    steps.get();
                    s.fill_none()
                        .stroke(colour.borrow().colour(shape, copy))
                        .stroke_width(LINE_WIDTH)
                }),
            );
        }
    }
    let shown_at = Instant::now();
    div()
        .size_full()
        .bg(Color::BLACK)
        .on_pointer_move(move |_| {
            let preview = saver_preview();
            if preview.get_untracked() && shown_at.elapsed() > PREVIEW_GRACE {
                preview.set(false);
            }
        })
        .child(
            svg()
                .w(size.width as f32)
                .h(size.height as f32)
                .children(lines),
        )
}
