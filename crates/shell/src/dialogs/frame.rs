//! The window of the Run and Shut Down dialogs, as Windows 7 draws a dialog: a glass frame with
//! its caption, round a white page, with the buttons on a grey band at the bottom of the page.

use crate::theme::{corner, scheme, theme};
use crate::ui::{dark_look, font, vertical_gradient};
use anti_webui::prelude::*;

/// The glass of the frame, top and bottom, on dark colours and on light ones.
const GLASS_DARK: [Color; 2] = [rgba(0x566f88f0), rgba(0x324a60f0)];
const GLASS_LIGHT: [Color; 2] = [rgba(0xb9d1eae6), rgba(0x8fb3d6e6)];
/// The dark line round the frame, and the light one just inside it.
const FRAME_OUTLINE: Color = rgba(0x000000b3);
const FRAME_INNER_LINE: Color = rgba(0xffffff59);
/// The caption on the glass.
const CAPTION_DARK: Color = Color::WHITE;
const CAPTION_LIGHT: Color = rgb(0x1e395b);
/// The line round the page.
const PAGE_OUTLINE: Color = rgba(0x00000080);
/// The height of the caption, and of the band of buttons.
const CAPTION_HEIGHT: f32 = 28.0;
const BAND_HEIGHT: f32 = 44.0;

/// The frame, `width` wide, with `caption` on its glass and `page` in it. Clicks on it stay in
/// it; the caller adds what a press on it does besides.
pub fn dialog_frame(caption: &'static str, width: f32, page: Div) -> Div {
    div()
        .flex()
        .flex_col()
        .flex_none()
        .w(width)
        .border(1.0)
        .select_none()
        .fill(|| {
            let [top, bottom] = if dark_look() { GLASS_DARK } else { GLASS_LIGHT };
            vertical_gradient(&[(0.0, top), (1.0, bottom)])
        })
        .dyn_style(|s| {
            s.rounded(corner(7.0))
                .border_color(FRAME_OUTLINE)
                .shadow(BoxShadow {
                    spread: 1.0,
                    color: FRAME_INNER_LINE,
                    inset: true,
                    ..Default::default()
                })
                .font_family(font())
                .text_size(theme().font_size)
        })
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                // The frame's own line takes a pixel of each.
                .h(CAPTION_HEIGHT - 1.0)
                .pl(9.0)
                .dyn_style(|s| s.text_color(if dark_look() { CAPTION_DARK } else { CAPTION_LIGHT }))
                .child(caption),
        )
        .child(page.mx(7.0).mb(8.0))
}

/// The white page in the frame: a column, outlined.
pub fn dialog_page() -> Div {
    div()
        .relative()
        .flex()
        .flex_col()
        .border(1.0)
        .dyn_style(|s| {
            let c = scheme();
            s.bg(c.page)
                .border_color(PAGE_OUTLINE)
                .text_color(c.text)
                .font_family(font())
                .text_size(theme().font_size)
        })
}

/// The band at the bottom of the page, in `band` (dark, light), with `buttons` at its right end
/// and, when given, a line (dark, light) along its top.
pub fn button_band(
    band: (Color, Color),
    top_line: Option<(Color, Color)>,
    buttons: impl IntoView,
) -> Div {
    let line = top_line.map(|(dark, light)| {
        div()
            .absolute()
            .left(0.0)
            .right(0.0)
            .top(0.0)
            .h(1.0)
            .dyn_style(move |s| s.bg(if dark_look() { dark } else { light }))
    });
    div()
        .relative()
        .flex()
        .flex_row()
        .flex_none()
        .items_center()
        .justify_end()
        .gap(8.0)
        .h(BAND_HEIGHT)
        .pr(12.0)
        .dyn_style(move |s| s.bg(if dark_look() { band.0 } else { band.1 }))
        .children(line)
        .child(buttons)
}
