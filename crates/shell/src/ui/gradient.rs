//! Gradients that fill a box. AntiWebUI's boxes take one colour as background; Qt's
//! `Gradient` (every Windows 7 surface has one) is a shader here, given to `Div::fill`:
//!
//! ```ignore
//! div().fill(|| vertical_gradient(&[(0.0, scheme().popup_top), (1.0, scheme().popup_bottom)]))
//! ```
//!
//! The fill is cut to the box's rounded corners and drawn under its border.

use anti_webui::prelude::*;

/// Up to four stops, as `(offset, colour)`, along the height or the width of the box.
#[derive(ShaderParams, Clone, Copy)]
struct Stops {
    c0: Color,
    c1: Color,
    c2: Color,
    c3: Color,
    /// The offset of each stop, from 0 to 1, in order.
    offsets: [f32; 4],
    /// 1 for along the width, 0 for along the height.
    horizontal: f32,
}

static GRADIENT: Shader<Stops> = Shader::new(
    "shell_gradient",
    r#"
fn paint(f: Frag) -> vec4<f32> {
    let size = vec2<f32>(aw_box.size);
    var t = clamp(f.px.y / max(size.y, 1.0), 0.0, 1.0);
    if (params.horizontal > 0.5) {
        t = clamp(f.px.x / max(size.x, 1.0), 0.0, 1.0);
    }
    let o = params.offsets;
    if (t <= o.x) {
        return params.c0;
    }
    if (t <= o.y) {
        return mix(params.c0, params.c1, (t - o.x) / max(o.y - o.x, 0.0001));
    }
    if (t <= o.z) {
        return mix(params.c1, params.c2, (t - o.y) / max(o.z - o.y, 0.0001));
    }
    if (t <= o.w) {
        return mix(params.c2, params.c3, (t - o.z) / max(o.w - o.z, 0.0001));
    }
    return params.c3;
}
"#,
);

/// A gradient from the top of the box to its bottom: `(offset, colour)` stops, offsets from 0
/// to 1 in order. Up to four stops; more are ignored.
pub fn vertical_gradient(stops: &[(f32, Color)]) -> ShaderPaint {
    gradient(stops, false)
}

/// A gradient from the left of the box to its right, as [`vertical_gradient`].
pub fn horizontal_gradient(stops: &[(f32, Color)]) -> ShaderPaint {
    gradient(stops, true)
}

fn gradient(stops: &[(f32, Color)], horizontal: bool) -> ShaderPaint {
    let count = stops.len().min(4);
    // The missing stops repeat the last one, at the end.
    let stop = |i: usize| -> (f32, Color) {
        match stops.get(i).filter(|_| i < count) {
            Some(&s) => s,
            None => (
                1.0,
                stops
                    .get(count.wrapping_sub(1))
                    .map_or(Color::TRANSPARENT, |s| s.1),
            ),
        }
    };
    let (o0, c0) = stop(0);
    let (o1, c1) = stop(1);
    let (o2, c2) = stop(2);
    let (o3, c3) = stop(3);
    GRADIENT.params(Stops {
        c0,
        c1,
        c2,
        c3,
        offsets: [o0, o1.max(o0), o2.max(o1), o3.max(o2)],
        horizontal: if horizontal { 1.0 } else { 0.0 },
    })
}
