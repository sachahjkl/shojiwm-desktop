//! The home page of the Control Panel, which lists the categories and their links, and the page
//! of search results that replaces it while something is typed in the search box.

use super::state::{self, Task};
use super::{heading, heading_text, note_text};
use crate::paths::asset;
use crate::theme::{scheme, theme};
use crate::ui::{cp_link, cp_link_tinted};
use anti_webui::prelude::*;

/// The room between the two columns of categories, and between their rows.
const COLUMN_GAP: f32 = 40.0;
const ROW_GAP: f32 = 26.0;
/// The page is two columns of categories from this width, one below it.
const TWO_COLUMNS_FROM: f32 = 640.0;
/// The side of a category's icon.
const CATEGORY_ICON: f32 = 48.0;

/// A category of the home page: its icon (a picture of `assets/icons`), its name, the page its
/// name opens, and its links with the page each opens.
struct Category {
    icon: &'static str,
    name: &'static str,
    target: &'static str,
    links: &'static [(&'static str, &'static str)],
}

const CATEGORIES: &[Category] = &[
    Category {
        icon: "pictures",
        name: "Appearance and Personalization",
        target: "appearance",
        links: &[
            ("Choose light or dark", "appearance"),
            ("Change desktop background", "background"),
            ("Taskbar and Start menu", "taskbar"),
            ("Notification area icons", "notifications"),
        ],
    },
    Category {
        icon: "control-panel",
        name: "User Accounts",
        target: "account",
        links: &[
            ("Change your account picture", "account"),
            ("Lock screen and its picture", "lockscreen"),
        ],
    },
    Category {
        icon: "calendar",
        name: "Clock and Region",
        target: "clock",
        links: &[("Change the clock format", "clock")],
    },
    Category {
        icon: "run",
        name: "Ease of Access",
        target: "ease",
        links: &[("Turn off all unnecessary animations", "ease")],
    },
    Category {
        icon: "computer",
        name: "System",
        target: "about",
        links: &[
            (
                "Choose the programs that open your files and links",
                "defaults",
            ),
            ("Choose the programs that start when you log on", "startup"),
            ("About this desktop", "about"),
        ],
    },
];

/// `link` with its text wrapped at the width of its column, so that a long one is never cut off.
fn wrapping(link: Div) -> Div {
    link.min_w(0.0).dyn_style(|mut s| {
        s.white_space = Some(WhiteSpace::PreWrap);
        s
    })
}

/// "Adjust your computer's settings": the categories in two columns, each half the page, or in
/// one in a narrow window.
pub fn home_page() -> Div {
    let width = signal(0.0f32);
    let cell = memo(move || {
        let width = width.get();
        let columns = if width >= TWO_COLUMNS_FROM { 2.0 } else { 1.0 };
        ((width - COLUMN_GAP * (columns - 1.0)) / columns).max(0.0)
    });
    div()
        .flex()
        .flex_col()
        .gap(18.0)
        .child(heading("Adjust your computer's settings"))
        .child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .items_start()
                .column_gap(COLUMN_GAP)
                .row_gap(ROW_GAP)
                .on_resized(move |w, _| width.set(w))
                .children(
                    CATEGORIES
                        .iter()
                        .enumerate()
                        .map(|(order, category)| category_view(category, order, cell)),
                ),
        )
}

/// A category: in Kernel, a bar in the category's colour (`category_colours`, by its place on
/// the page); then its icon, its name, and its links under the name.
fn category_view(category: &'static Category, order: usize, cell: Memo<f32>) -> Div {
    // The category's colour, if the theme has them.
    let colour = move || {
        let colours = theme().category_colours;
        (!colours.is_empty()).then(|| colours[order % colours.len()])
    };
    let bar = div()
        .flex_none()
        .self_stretch()
        .dyn_style(move |s| match colour() {
            Some(c) => s.w(theme().heading_bar).bg(c),
            None => s.hidden(),
        });
    let name = wrapping(cp_link_tinted(
        category.name,
        true,
        move || colour().unwrap_or_else(|| scheme().link2),
        move || state::go(category.target),
    ))
    .font_weight(FontWeight::BOLD);
    let links = category
        .links
        .iter()
        .map(|&(label, page)| wrapping(cp_link(label, false, move || state::go(page))));
    div()
        .flex_none()
        .flex()
        .flex_row()
        .items_start()
        .gap(14.0)
        .dyn_style(move |s| s.w(cell.get()))
        .child(bar)
        .child(
            img(asset(&format!("icons/{}.png", category.icon)))
                .flex_none()
                .size(CATEGORY_ICON)
                .object_fit(ObjectFit::Contain),
        )
        .child(
            div()
                .flex_1()
                .min_w(0.0)
                .flex()
                .flex_col()
                .gap(4.0)
                .child(name)
                .children(links),
        )
}

/// The tasks that match every word typed in the search box, each with the page it is on.
pub fn search_page() -> Div {
    let query = state::query();
    let found = memo(move || state::search(&query.get()));
    div()
        .flex()
        .flex_col()
        .gap(10.0)
        .child(heading_text(move || {
            let typed = query.with(|q| q.trim().to_owned());
            if found.with(Vec::is_empty) {
                format!("No setting matches “{typed}”")
            } else {
                format!("Results for “{typed}”")
            }
        }))
        .child(for_each(
            move || found.get(),
            |task: &Task| task.text,
            |task: Task| {
                let page = state::page_info(task.page);
                div()
                    .flex()
                    .flex_col()
                    .items_start()
                    .gap(2.0)
                    .child(cp_link(task.text, true, move || state::go(task.page)))
                    .child(note_text(format!("{}  ▸  {}", page.category, page.title)))
            },
        ))
}
