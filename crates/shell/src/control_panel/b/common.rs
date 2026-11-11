//! What the second pages of the Control Panel share: the page's column, the small and muted
//! texts, the text field of the pages, and running a command for its output.

use crate::theme::{field_radius, scheme, theme};
use crate::ui::font;
use anti_webui::prelude::*;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// The height of a text field and of the labels beside the fields and combo boxes.
pub const FIELD_HEIGHT: f32 = 24.0;

/// A page: its parts one under another, `gap` apart, as wide as the page.
pub fn page_column(gap: f32) -> Div {
    div().flex().flex_col().items_start().w_full().gap(gap)
}

/// A row of controls, `gap` apart, lined up on their middles.
pub fn control_row(gap: f32) -> Div {
    div().flex().flex_row().items_center().gap(gap)
}

/// Text in the theme's face and size, in a colour of the scheme.
pub fn tinted_text(label: impl IntoTextSource, color: impl Fn() -> Color + 'static) -> Text {
    text(label).dyn_style(move |s| {
        s.font_family(font())
            .text_size(theme().font_size)
            .text_color(color())
    })
}

/// Text in the small size and the muted colour: a detail under or beside something.
pub fn muted_small(label: impl IntoTextSource) -> Text {
    text(label).dyn_style(|s| {
        s.font_family(font())
            .text_size(theme().font_size_small)
            .text_color(scheme().text_muted)
    })
}

/// A label of a form's row (`Display:`, `Time zone:`): muted, as high as the field beside it,
/// in a column of `width`.
pub fn form_label(label: impl IntoTextSource, width: f32) -> Div {
    div()
        .flex()
        .items_center()
        .flex_none()
        .min_w(width)
        .h(FIELD_HEIGHT)
        .child(tinted_text(label, || scheme().text_muted))
}

/// A one-line text field of the pages, `width` wide: the field's colour, and its border in
/// `border(focused)`. `input` keeps its own handlers; its face and colour are set here.
pub fn cp_field(input: TextInput, width: f32, border: impl Fn(bool) -> Color + 'static) -> Div {
    let focused = signal(false);
    div()
        .flex()
        .flex_row()
        .items_center()
        .flex_none()
        .w(width)
        .h(FIELD_HEIGHT)
        .px(6.0)
        .border(1.0)
        .overflow_clip()
        .dyn_style(move |s| {
            s.rounded(field_radius())
                .bg(scheme().field)
                .border_color(border(focused.get()))
        })
        .child(
            input
                .flex_1()
                .min_w(0.0)
                .on_focus_change(move |on| focused.set(on))
                .dyn_style(|s| {
                    s.font_family(font())
                        .text_size(theme().font_size)
                        .text_color(scheme().text)
                }),
        )
}

/// The border of a field: the focus colour while it has the keyboard.
pub fn field_border(focused: bool) -> Color {
    let c = scheme();
    if focused { c.field_focus } else { c.field_border }
}

/// Runs `program` with `args` and gives what it printed; "" when it cannot run. It blocks:
/// call it on a thread (`task`).
pub fn output_of(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

/// The variable `name`, else `fallback` when it is unset or empty.
pub fn env_or(name: &str, fallback: impl FnOnce() -> String) -> String {
    std::env::var(name)
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(fallback)
}

/// The directories of data, in order: `$XDG_DATA_HOME` (`~/.local/share`), then
/// `$XDG_DATA_DIRS` (`/usr/share`).
pub fn data_dirs() -> Vec<PathBuf> {
    let home = env_or("XDG_DATA_HOME", || {
        crate::settings::home()
            .map(|h| h.join(".local/share").display().to_string())
            .unwrap_or_default()
    });
    let rest = env_or("XDG_DATA_DIRS", || "/usr/share".into());
    std::iter::once(home.as_str())
        .chain(rest.split(':'))
        .filter(|d| !d.is_empty())
        .map(PathBuf::from)
        .collect()
}

/// The `.desktop` file of an application, by its file name, from the data directories.
pub fn desktop_file(id: &str) -> Option<PathBuf> {
    data_dirs()
        .into_iter()
        .map(|dir| dir.join("applications").join(id))
        .find(|path| path.is_file())
}

/// The keys of a `.desktop` file's `[Desktop Entry]` group, in order, as they are written.
pub fn desktop_entry_keys(text: &str) -> Vec<(String, String)> {
    let mut group = "";
    let mut keys = Vec::new();
    for line in text.lines() {
        let line = line.trim_end();
        if line.starts_with('[') {
            group = line;
            continue;
        }
        if group != "[Desktop Entry]" {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            keys.push((key.trim().to_owned(), value.trim_start().to_owned()));
        }
    }
    keys
}

/// An icon of the theme by `name`, else `fallback` when the theme lacks it (Quickshell's
/// `iconPath(name, fallback)`). A path stays as it is.
pub fn icon_or(name: &str, fallback: &str) -> String {
    if name.is_empty() {
        return fallback.to_owned();
    }
    if name.starts_with('/') || anti_webui::shell::icon_path(name, 48).is_some() {
        name.to_owned()
    } else {
        fallback.to_owned()
    }
}
