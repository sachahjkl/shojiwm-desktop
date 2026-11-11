//! The titles of the Alt+Tab panel, each under its window's thumbnail (QML `AltTabTitles`). The
//! compositor draws the panel (`src/alt-tab.tsx`), but a composition draws no text: it tells the
//! shell where its cells are with the event `alt-tab` on the shell's line, and the shell draws
//! the titles on a layer over the panel. The layer takes no input: the panel's grab has the
//! keyboard and the pointer.

use crate::theme::theme;
use crate::ui::font;
use anti_webui::prelude::*;
use serde_json::Value;
use std::sync::OnceLock;

/// The titles' text, and the line round its letters.
const TITLE: Color = Color::WHITE;
const TITLE_OUTLINE: Color = rgba(0x00000066);
/// The room kept at each side of a title, in its cell.
const TITLE_INSET: f32 = 6.0;

/// A cell of the panel: where its title goes, in the output's logical pixels, and the title.
#[derive(Clone, Debug, PartialEq)]
struct Cell {
    x: f32,
    y: f32,
    width: f32,
    title: String,
}

/// The Alt+Tab panel while it is open.
#[derive(Clone, Debug, PartialEq)]
struct Panel {
    output: String,
    selected: usize,
    title_height: f32,
    cells: Vec<Cell>,
}

static PANEL: OnceLock<SharedSignal<Option<Panel>>> = OnceLock::new();

fn panel() -> &'static SharedSignal<Option<Panel>> {
    PANEL.get_or_init(|| shared_signal(None))
}

fn number(v: &Value, key: &str) -> f32 {
    v.get(key).and_then(Value::as_f64).unwrap_or(0.0) as f32
}

fn parse(payload: &Value) -> Option<Panel> {
    let cells = payload
        .get("cells")?
        .as_array()?
        .iter()
        .map(|c| Cell {
            x: number(c, "x"),
            y: number(c, "y"),
            width: number(c, "width"),
            title: c
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned(),
        })
        .collect();
    Some(Panel {
        output: payload.get("output")?.as_str()?.to_owned(),
        selected: payload.get("selected").and_then(Value::as_u64).unwrap_or(0) as usize,
        title_height: number(payload, "titleHeight"),
        cells,
    })
}

/// The compositor's `alt-tab` event: `{ output, selected, titleHeight, cells: [{ x, y, width,
/// title }] }`, or `null` when the panel closes. Can be called from any thread.
pub fn alt_tab_event(payload: Option<&Value>) {
    panel().set(payload.and_then(parse));
}

/// Whether the panel is open on the output named `output_name`. Reading it tracks it.
pub(super) fn shown_on(output_name: &str) -> bool {
    panel().with(|p| p.as_ref().is_some_and(|p| p.output == output_name))
}

/// The titles of the panel on the output named `output_name`.
pub(super) fn view(output_name: String) -> Div {
    div()
        .size_full()
        .relative()
        .select_none()
        .child(dyn_view(move || {
            let Some(now) = panel().get().filter(|p| p.output == output_name) else {
                return div();
            };
            let titles: Vec<Div> = now
                .cells
                .into_iter()
                .enumerate()
                .map(|(i, cell)| title(cell, now.title_height, i == now.selected))
                .collect();
            div().absolute().inset(0.0).children(titles)
        }))
}

/// A title in its cell: centred, in white with a dark line round its letters, bold for the
/// selected window.
fn title(cell: Cell, height: f32, selected: bool) -> Div {
    let words = cell.title;
    let layer = |dx: f32, dy: f32, color: Color| {
        div()
            .absolute()
            .left(dx)
            .top(dy)
            .w_full()
            .h_full()
            .flex()
            .items_center()
            .justify_center()
            .overflow_clip()
            .whitespace_nowrap()
            .text_color(color)
            .child(words.clone())
    };
    div()
        .absolute()
        .left(cell.x + TITLE_INSET)
        .top(cell.y)
        .w((cell.width - 2.0 * TITLE_INSET).max(0.0))
        .h(height)
        .dyn_style(move |s| {
            let s = s.font_family(font()).text_size(theme().font_size);
            if selected {
                s.font_weight(FontWeight::BOLD)
            } else {
                s
            }
        })
        .child(layer(-1.0, 0.0, TITLE_OUTLINE))
        .child(layer(1.0, 0.0, TITLE_OUTLINE))
        .child(layer(0.0, -1.0, TITLE_OUTLINE))
        .child(layer(0.0, 1.0, TITLE_OUTLINE))
        .child(layer(0.0, 0.0, TITLE))
}
