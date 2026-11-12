//! "Close all tabs?": asked when the close button of a window that has tabs is pressed. It says
//! how many tabs close and which ones (their titles; past a few, the rest in a tooltip), and
//! closes them all on "Close all tabs". The compositor names the windows by their
//! `ext_foreign_toplevel_list` identifiers, and the shell closes them as it closes a window from
//! the taskbar.

use super::close_group_close;
use super::frame::{button_band, dialog_frame, dialog_page};
use crate::paths::asset;
use crate::theme::{bar_at_top, theme};
use crate::ui::{cp_button, cp_default_button, font, shell_tooltip};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;

/// The width of the dialog.
const WIDTH: f32 = 420.0;
/// The room from the screen's edges.
const MARGIN: f32 = 12.0;
/// The titles listed in the dialog; the others are in the tooltip of the line after them.
const LISTED: usize = 8;
/// The band of buttons, on dark and on light colours.
const BAND_DARK: Color = rgb(0x1b242d);
const BAND_LIGHT: Color = rgb(0xf0f0f0);

/// The titles of the windows named by `identifiers`, in their order; a window gone is left out.
fn titles(identifiers: &[String]) -> Vec<String> {
    let all = toplevels().all();
    identifiers
        .iter()
        .filter_map(|id| {
            all.iter()
                .find(|t| t.identifier.as_deref() == Some(id.as_str()))
                .map(|t| t.title.clone())
        })
        .collect()
}

/// Closes the windows named by `identifiers`.
fn close_all(identifiers: &[String]) {
    for t in toplevels().all() {
        if t.identifier
            .as_ref()
            .is_some_and(|id| identifiers.contains(id))
        {
            let _ = t.close();
        }
    }
}

/// The dialog, over the screen; `identifiers` the windows of the group.
pub fn view(identifiers: Vec<String>) -> Div {
    let listed = titles(&identifiers);
    let count = listed.len().max(identifiers.len());
    let shown: Vec<String> = listed.iter().take(LISTED).cloned().collect();
    let rest: Vec<String> = listed.iter().skip(LISTED).cloned().collect();

    let confirm = {
        let identifiers = identifiers.clone();
        move || {
            close_all(&identifiers);
            close_group_close();
        }
    };

    let list = div()
        .flex()
        .flex_col()
        .gap(2.0)
        .children(shown.into_iter().map(|title| {
            div()
                .whitespace_nowrap()
                .overflow_clip()
                .child(text(format!("\u{2022}  {title}")))
        }));
    let more = (!rest.is_empty()).then(|| {
        let names = rest.join("\n");
        shell_tooltip(
            div().child(text(format!("and {} more\u{2026}", rest.len()))),
            Placement::BelowStart,
            move || names.clone(),
        )
    });

    let page = dialog_page()
        .child(
            div()
                .flex()
                .flex_row()
                .gap(14.0)
                .pt(16.0)
                .px(14.0)
                .pb(14.0)
                .child(
                    img(asset("icons/run.png"))
                        .flex_none()
                        .size(32.0)
                        .object_fit(ObjectFit::Contain),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w(0.0)
                        .flex()
                        .flex_col()
                        .gap(8.0)
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .child(text(format!("Close all {count} tabs?"))),
                        )
                        .child(text("These windows close:"))
                        .child(list)
                        .children(more),
                ),
        )
        .child(button_band(
            (BAND_DARK, BAND_LIGHT),
            None,
            fragment([
                cp_default_button("Close all tabs", {
                    let confirm = confirm.clone();
                    move || confirm()
                })
                .into_view(),
                cp_button("Cancel", close_group_close).into_view(),
            ]),
        ));

    let frame = dialog_frame("Close the window", WIDTH, page)
        .absolute()
        .left(MARGIN)
        .dyn_style(|s| {
            let away = theme().taskbar_height + MARGIN;
            if bar_at_top() {
                s.top(away)
            } else {
                s.bottom(away)
            }
        })
        .on_pointer_down(|e| e.stop_propagation());

    let root = node_ref();
    request_frame(move |_| root.focus());
    div()
        .size_full()
        .relative()
        .node_ref(root)
        .focusable()
        .dyn_style(|s| s.font_family(font()).text_size(theme().font_size))
        .on_key_down(move |e| match &e.key {
            Key::Named(NamedKey::Escape) => {
                e.prevent_default();
                close_group_close();
            }
            Key::Named(NamedKey::Enter) => {
                e.prevent_default();
                confirm();
            }
            _ => {}
        })
        .on_pointer_down(|_| close_group_close())
        .child(frame)
}
