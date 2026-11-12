//! The pieces every surface of the shell shares, in the look of the theme in use: the popup
//! frame and the tooltip, the popup menus and the Start menu's rows, the section titles, the
//! icons, the scroll bar, and the controls of the Control Panel and the dialogs.
//!
//! Each piece reads the theme in reactive closures, so it follows a change of theme at once.
//! Each returns a plain builder (`Div`, `Text` or `Show`) that the caller can style further.

mod controls;
mod frame;
mod gradient;
mod icon;
mod menu;
mod scroll;
mod text;
mod volume;

pub use controls::{
    cp_button, cp_check, cp_combo, cp_default_button, cp_link, cp_link_marked, cp_link_tinted,
    cp_radio,
};
pub use frame::{popup_frame, shell_tooltip};
pub use gradient::{horizontal_gradient, vertical_gradient};
pub use icon::shell_icon;
// For the modules that still call `anti_webui::shell::icon_path` on the UI thread.
#[allow(unused_imports)]
pub use icon::resolved_icon;
pub use menu::{
    MenuLine, RowLook, bar_menu_options, marked_text, menu_row, popup_menu, with_context_menu,
};
pub use scroll::aero_scroll;
pub use text::{label_text, section_title, section_title_with, shadow_text, soft_shadowed};
pub use volume::{VolumeControl, balance_bar, mute_button, volume_bar, volume_slider, wheel_steps};

use crate::settings::dark;
use crate::theme::{is_kernel, theme};
use anti_webui::core::view::TextSource;
use anti_webui::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;

/// The colours are dark: Kernel always, Aero when the system prefers dark. The QML shell's
/// `Scheme.dark` read the system alone, which gave Kernel light hover colours and dark ink on
/// its dark panels when the system preferred light.
pub fn dark_look() -> bool {
    is_kernel() || dark()
}

/// The face of the text, with its fallback, as a font list.
pub fn font() -> FontFamily {
    let t = theme();
    FontFamily(format!("{}, {}", t.font_family, t.font_fallback).into())
}

/// The face of the section titles (monospaced in Kernel), with the text's face as fallback.
pub fn heading_font() -> FontFamily {
    let t = theme();
    FontFamily(
        format!(
            "{}, {}, {}",
            t.heading_family, t.font_family, t.font_fallback
        )
        .into(),
    )
}

/// The face of the clock and its tooltip (monospaced in Kernel).
pub fn clock_font() -> FontFamily {
    let t = theme();
    FontFamily(format!("{}, {}, {}", t.clock_family, t.font_family, t.font_fallback).into())
}

/// A picture of `shell/assets` drawn for the colours in use: the plain one on dark colours, its
/// `ink-` variant (dark lines) on light ones.
pub fn ink_asset(name: &str) -> PathBuf {
    let prefix = if dark_look() { "" } else { "ink-" };
    crate::paths::asset(&format!("{prefix}{name}"))
}

/// `color` lighter by `factor`, as Qt's `Qt.lighter` makes it: its value in HSV times
/// `factor`, and what goes over the top taken from the saturation.
pub fn lighter(color: Color, factor: f32) -> Color {
    let (r, g, b) = (color.r, color.g, color.b);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let hue = if delta <= 0.0 {
        0.0
    } else if max == r {
        60.0 * ((g - b) / delta).rem_euclid(6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    let mut saturation = if max <= 0.0 { 0.0 } else { delta / max };
    let mut value = max * factor;
    if value > 1.0 {
        saturation = (saturation - (value - 1.0)).max(0.0);
        value = 1.0;
    }
    let chroma = value * saturation;
    let x = chroma * (1.0 - ((hue / 60.0).rem_euclid(2.0) - 1.0).abs());
    let m = value - chroma;
    let (r, g, b) = match (hue / 60.0) as u32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    Color::new(r + m, g + m, b + m, color.a)
}

/// A text source as a function that gives the text now, for a piece that shows it more than
/// once or changes it (capitals, shadows).
pub(crate) fn text_fn(label: impl IntoTextSource) -> Rc<dyn Fn() -> String> {
    match label.into_text_source() {
        TextSource::Static(s) => Rc::new(move || s.clone()),
        TextSource::Dynamic(f) => Rc::from(f),
    }
}
