//! Icons from the icon theme, sharp at any scale.

use super::text_fn;
use anti_webui::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;

/// An icon `size` logical pixels square: `name` from the icon theme (`firefox`), or a file
/// (`/path/to/icon.svg`, `file:///path`). The theme is asked for the size in the screen's own
/// pixels, so a theme with fixed sizes gives a picture that the scale does not blur. An empty
/// name takes no room; a name the theme lacks leaves the room empty.
pub fn shell_icon(name: impl IntoTextSource, size: f32) -> Div {
    icon_box(text_fn(name), size)
}

pub(crate) fn icon_box(name: Rc<dyn Fn() -> String>, size: f32) -> Div {
    let wanted = name.clone();
    let path = memo(move || {
        let pixels = (size * window_scale().as_f32()).ceil().max(1.0) as u32;
        find_icon(&wanted(), pixels)
    });
    let empty = memo(move || name().is_empty());
    div()
        .flex_none()
        .size(size)
        .dyn_style(move |s| if empty.get() { s.hidden() } else { s })
        .child(dyn_view(move || {
            path.get()
                .map(|file| img(file).size(size).object_fit(ObjectFit::Contain))
        }))
}

/// The file of an icon: a path as it is, else the theme's at about `pixels` pixels. Takes the
/// forms Quickshell gave (`file://`, `image://icon/name?path=..`).
fn find_icon(name: &str, pixels: u32) -> Option<PathBuf> {
    let name = name.trim();
    let name = name.strip_prefix("file://").unwrap_or(name);
    let name = name.strip_prefix("image://icon/").unwrap_or(name);
    let name = name.split('?').next().unwrap_or(name);
    if name.is_empty() {
        return None;
    }
    anti_webui::shell::icon_path(name, pixels)
}
