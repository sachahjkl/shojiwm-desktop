//! The theme chosen in the Control Panel (`settings().theme`), read the way the surfaces need it:
//! its sizes and colours ([`theme`]), the colour tokens for light or dark ([`scheme`]), its
//! corners and the side of the screen its bar stands on.
//!
//! Every function here reads the settings, so a view that calls it in a reactive closure
//! (`dyn_style`, `text(move || ..)`, `memo`) follows a change of theme at once.

use crate::settings::{dark, settings};
use anti_webui::prelude::*;

pub use crate::theme_values::{AERO, AERO_DARK, AERO_LIGHT, KERNEL, KERNEL_SCHEME, Scheme, Theme};

/// The theme's name: "aero" or "kernel" (anything else reads as Aero).
pub fn name() -> String {
    settings().with(|s| s.theme.clone())
}

pub fn is_kernel() -> bool {
    settings().with(|s| s.theme == "kernel")
}

/// The sizes, colours, faces and timings of the theme in use.
pub fn theme() -> Theme {
    if is_kernel() { KERNEL } else { AERO }
}

/// The colour tokens: Kernel's (always dark), else Aero's light or dark ones.
pub fn scheme() -> Scheme {
    if is_kernel() {
        KERNEL_SCHEME
    } else if dark() {
        AERO_DARK
    } else {
        AERO_LIGHT
    }
}

/// A corner's radius: `radius` in Aero, 0 in its square variant, at most 2 in Kernel. Every
/// rounded corner goes through it; a circle does not.
pub fn corner(radius: f32) -> f32 {
    if is_kernel() {
        radius.min(2.0)
    } else if settings().with(|s| s.square_corners) {
        0.0
    } else {
        radius
    }
}

pub fn popup_radius() -> f32 {
    corner(4.0)
}
pub fn menu_radius() -> f32 {
    corner(5.0)
}
pub fn item_radius() -> f32 {
    corner(3.0)
}
pub fn field_radius() -> f32 {
    corner(2.0)
}
pub fn button_radius() -> f32 {
    corner(3.0)
}

/// The bar stands at the top of the screen (Kernel), not the bottom (Aero).
pub fn bar_at_top() -> bool {
    theme().bar_side == "top"
}

/// The room the bar takes at the top of the screen and at its bottom: what is placed on the
/// desktop (icons, gadgets) keeps out of it.
pub fn bar_above() -> f32 {
    if bar_at_top() { theme().taskbar_height } else { 0.0 }
}
pub fn bar_below() -> f32 {
    if bar_at_top() { 0.0 } else { theme().taskbar_height }
}

/// Where a popup opened from something on the bar goes: away from the bar, lined up with
/// that thing's start, end or middle.
pub fn away_from_bar(align: Align) -> Placement {
    match (bar_at_top(), align) {
        (true, Align::Start) => Placement::BelowStart,
        (true, Align::End) => Placement::BelowEnd,
        (true, Align::Center) => Placement::BelowCenter,
        (false, Align::Start) => Placement::AboveStart,
        (false, Align::End) => Placement::AboveEnd,
        (false, Align::Center) => Placement::AboveCenter,
    }
}

/// Which side of the thing a popup lines up with ([`away_from_bar`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Start,
    End,
    Center,
}

/// The face of the text, with its fallback.
pub fn font_families() -> [&'static str; 2] {
    let t = theme();
    [t.font_family, t.font_fallback]
}

/// `"#rrggbb"` or `"#aarrggbb"` (Qt's order, as the settings keep glass colours) to a colour.
pub fn parse_hex(hex: &str) -> Option<Color> {
    let h = hex.trim().trim_start_matches('#');
    let v = u32::from_str_radix(h, 16).ok()?;
    match h.len() {
        6 => Some(rgb(v)),
        8 => Some(rgba((v << 8) | (v >> 24))),
        _ => None,
    }
}
