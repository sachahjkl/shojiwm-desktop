//! The account picture in the frame the Windows 7 log-on screen puts round it (QML `UserTile`),
//! and the picture itself.
//!
//! The frame was measured on a Windows 7 Ultimate screenshot at 800 x 600 (116 px round a 98 px
//! picture) and is drawn here for 136 px; everything scales with the size, so the Start menu
//! uses it small:
//!
//! - a dark blue outer line, a one-pixel highlight inside it;
//! - a band of glass, nearly white at the top, cyan-blue down the sides, blue at the bottom;
//! - a light line, then a dark grey-blue line round the picture, which is square and fills its
//!   place.
//!
//! Each layer is one rounded box inside the last, so there is one corner, not several. On dark
//! colours the glass is smoked.

use crate::paths::asset;
use crate::settings::{home, settings};
use crate::theme::corner;
use crate::ui::{dark_look, vertical_gradient};
use anti_webui::prelude::*;
use std::path::PathBuf;

/// The size the frame was measured for.
const DESIGN_SIZE: f32 = 136.0;
/// The shadow under the frame.
const SHADOW: Color = rgba(0x001a3338);
/// The outer line, light and dark.
const OUTER: [Color; 2] = [rgb(0x1b5f8c), rgb(0x06121c)];
/// The highlight just inside it.
const HIGHLIGHT: [Color; 2] = [rgb(0xb8ecff), rgb(0x7f9db4)];
/// The light line round the picture.
const LIGHT_LINE: [Color; 2] = [rgb(0xa8d8f0), rgb(0x5f7d95)];
/// The dark line round the picture.
const DARK_LINE: [Color; 2] = [rgb(0x476f8a), rgb(0x0e1a24)];
/// Where the glass's gradient is split in two: the gradients take four stops, the glass has six.
const GLASS_SPLIT: f32 = 0.6;
/// The glass above the split: stops at 0, 0.1, 0.35 and 0.6 of the whole height.
const GLASS_TOP: [Color; 4] = [rgb(0xe0f4f6), rgb(0xd2ecf0), rgb(0x8fc0d2), rgb(0x5f9fc0)];
const SMOKED_TOP: [Color; 4] = [rgb(0xa9bccb), rgb(0x8ea4b6), rgb(0x4f6678), rgb(0x2f4556)];
/// The glass below it: stops at 0.6, 0.9 and 1.
const GLASS_BOTTOM: [Color; 3] = [rgb(0x5f9fc0), rgb(0x3f8dbf), rgb(0x4a9ccf)];
const SMOKED_BOTTOM: [Color; 3] = [rgb(0x2f4556), rgb(0x1f3a52), rgb(0x2a5170)];

/// A colour of a pair, for light or dark colours.
fn pick(pair: [Color; 2]) -> Color {
    pair[usize::from(dark_look())]
}

pub use crate::account::user_picture;

/// The framed picture of `source`, `size` logical pixels square.
pub fn user_tile(source: impl Fn() -> PathBuf + 'static, size: f32) -> Div {
    let k = size / DESIGN_SIZE;
    let band = (10.0 * k).round().max(3.0);
    let r = (8.0 * k).max(3.0);
    let shadow_drop = (3.0 * k).round().max(1.0);
    let glass_height = size - 4.0;
    let top_height = (glass_height * GLASS_SPLIT).round();
    let glass = div()
        .absolute()
        .inset(2.0)
        .flex()
        .flex_col()
        .overflow_clip()
        .dyn_style(move |s| s.rounded(corner((r - 2.0).max(0.0))))
        .child(div().flex_none().h(top_height).fill(|| {
            let c = if dark_look() { SMOKED_TOP } else { GLASS_TOP };
            vertical_gradient(&[
                (0.0, c[0]),
                (0.1 / GLASS_SPLIT, c[1]),
                (0.35 / GLASS_SPLIT, c[2]),
                (1.0, c[3]),
            ])
        }))
        .child(div().flex_1().fill(|| {
            let c = if dark_look() { SMOKED_BOTTOM } else { GLASS_BOTTOM };
            vertical_gradient(&[
                (0.0, c[0]),
                ((0.9 - GLASS_SPLIT) / (1.0 - GLASS_SPLIT), c[1]),
                (1.0, c[2]),
            ])
        }));
    div()
        .relative()
        .flex_none()
        .size(size)
        .select_none()
        // The shadow under the frame.
        .child(
            div()
                .absolute()
                .left(-2.0)
                .right(-2.0)
                .top(0.0)
                .bottom(-shadow_drop)
                .bg(SHADOW)
                .dyn_style(move |s| s.rounded(corner(r + 2.0))),
        )
        // The outer line.
        .child(
            div()
                .absolute()
                .inset(0.0)
                .dyn_style(move |s| s.rounded(corner(r)).bg(pick(OUTER))),
        )
        // The highlight just inside it.
        .child(
            div()
                .absolute()
                .inset(1.0)
                .dyn_style(move |s| s.rounded(corner((r - 1.0).max(0.0))).bg(pick(HIGHLIGHT))),
        )
        .child(glass)
        // The light line, then the dark one, round the picture.
        .child(
            div()
                .absolute()
                .inset(band - 2.0)
                .dyn_style(move |s| s.rounded(corner((2.0 * k).max(1.0))).bg(pick(LIGHT_LINE))),
        )
        .child(
            div()
                .absolute()
                .inset(band - 1.0)
                .dyn_style(|s| s.bg(pick(DARK_LINE))),
        )
        .child(
            img(source)
                .absolute()
                .inset(band)
                .object_fit(ObjectFit::Cover),
        )
}
