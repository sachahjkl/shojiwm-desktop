//! The Control Panel: a normal window, so the compositor gives it its frame, laid out as the
//! Windows 7 one (QML `ControlPanel`). An address bar with the path and the search box, a pane
//! of links on the left, and the page on the right. The home page lists the categories; each
//! link opens a page of settings, which apply at once.
//!
//! Everything here is kept in the settings, except the light or dark choice, which is the
//! system's own and is read by every program.
//!
//! `shojiwm-shell msg controlpanel open`, `openPage <page>` and `close` drive it.

pub mod home;
pub mod pages_a;
pub mod pages_b;
pub mod picture_chooser;
pub mod state;

use crate::paths::asset;
use crate::theme::{field_radius, scheme, theme};
use crate::ui::{
    aero_scroll, cp_link, cp_link_tinted, font, section_title_with, vertical_gradient,
};
use anti_webui::prelude::*;

/// The window's title. The taskbar knows the Control Panel's window by it.
pub const TITLE: &str = "Control Panel";
/// The window's size when it opens.
const WIDTH: i32 = 860;
const HEIGHT: i32 = 600;
/// The height of the address bar.
const BAR_HEIGHT: f32 = 40.0;
/// The width of the pane of links.
const SIDE_WIDTH: f32 = 210.0;
/// The width of the search box.
const SEARCH_WIDTH: f32 = 210.0;
/// The height of the path and the search box.
const FIELD_HEIGHT: f32 = 24.0;
/// The room around a page.
const PAGE_PAD_X: f32 = 28.0;
const PAGE_PAD_TOP: f32 = 22.0;
const PAGE_PAD_BOTTOM: f32 = 26.0;

/// The round back button, as Explorer's: its border, and the top of its gradient at rest and
/// under the pointer, then its bottom.
const BACK_BORDER: Color = rgb(0x1d4f86);
const BACK_TOP: Color = rgb(0x5aa9f0);
const BACK_TOP_HOT: Color = rgb(0x7cc0ff);
const BACK_BOTTOM: Color = rgb(0x1c63ae);
/// How faint the back button is on the home page, where it does nothing.
const BACK_IDLE_OPACITY: f32 = 0.45;

/// Answers the `controlpanel` commands. Call once, from the shell's root.
pub fn install() {
    crate::ipc::handle("controlpanel", |function, args| match function {
        "open" => {
            open();
            Ok(String::new())
        }
        "openPage" => {
            open_page(args.first().map(String::as_str).unwrap_or(""));
            Ok(String::new())
        }
        "close" => {
            close();
            Ok(String::new())
        }
        _ => Err(format!("no function {function}")),
    });
}

/// Opens the Control Panel on its home page.
pub fn open() {
    open_page("");
}

/// Opens the Control Panel on `page` ("appearance", "background", ...; "" for the home page).
/// When it is open already, it goes to that page.
pub fn open_page(page: &str) {
    state::go(page);
    if state::window().is_some() {
        return;
    }
    let options = WindowOptions::new()
        .title(TITLE)
        .size(WIDTH, HEIGHT)
        .top_level()
        .background(untrack(|| scheme().page));
    state::set_window(Some(open_window(options, window_view)));
}

/// Closes the Control Panel.
pub fn close() {
    if let Some(window) = state::window() {
        window.close();
    }
    state::set_window(None);
}

// ----- the page styles, for every page ----------------------------------------------------------

/// A page's title, in the page heading colour, 8 px over the text.
pub fn heading(label: impl Into<String>) -> Div {
    heading_text(label.into())
}

/// [`heading`], with text that can change.
pub fn heading_text(label: impl IntoTextSource) -> Div {
    div()
        .dyn_style(|s| {
            s.font_family(font())
                .text_size(theme().font_size + 8.0)
                .text_color(scheme().page_heading)
        })
        .child(text(label))
}

/// The title of a part of a page: the theme's section title in the page heading colour, 2 px
/// over the text, with room above it.
pub fn sub_heading(label: impl Into<String>) -> Div {
    section_title_with(
        label.into(),
        || scheme().page_heading,
        || theme().calendar_today,
    )
    .pt(10.0)
    .dyn_style(|s| s.text_size(theme().font_size + 2.0))
}

/// A line of explanation, in the muted text colour. It wraps.
pub fn note(label: impl Into<String>) -> Div {
    note_text(label.into())
}

/// [`note`], with text that can change.
pub fn note_text(label: impl IntoTextSource) -> Div {
    div()
        .dyn_style(|s| {
            s.font_family(font())
                .text_size(theme().font_size)
                .text_color(scheme().text_muted)
        })
        .child(text(label))
}

// ----- the window ----------------------------------------------------------------------------

fn window_view() -> Div {
    on_cleanup(|| {
        state::set_window(None);
        state::query().set(String::new());
    });
    // The search results while something is typed, else the page.
    let shown = memo(|| {
        if state::query().with(|q| q.trim().is_empty()) {
            state::page()
        } else {
            "search".to_owned()
        }
    });
    let content = div()
        .flex_1()
        .min_w(0.0)
        .min_h(0.0)
        .flex()
        .flex_col()
        // Built again for each page, so that each page shows from its top.
        .child(dyn_view(move || {
            let key = shown.get();
            aero_scroll(
                div()
                    .flex()
                    .flex_col()
                    .px(PAGE_PAD_X)
                    .pt(PAGE_PAD_TOP)
                    .pb(PAGE_PAD_BOTTOM)
                    .child(untrack(|| page_view(&key))),
            )
            .flex_1()
            .min_h(0.0)
        }));
    div()
        .w_full()
        .h_full()
        .flex()
        .flex_col()
        .dyn_style(|s| {
            let c = scheme();
            s.bg(c.page)
                .font_family(font())
                .text_size(theme().font_size)
                .text_color(c.text)
        })
        .child(address_bar())
        .child(
            div()
                .flex_1()
                .min_h(0.0)
                .flex()
                .flex_row()
                .child(side_pane())
                .child(content),
        )
}

fn page_view(key: &str) -> View {
    match key {
        "search" => home::search_page().into_view(),
        "appearance" => pages_a::appearance_page().into_view(),
        "background" => pages_a::background_page().into_view(),
        "lockscreen" => pages_a::lock_page().into_view(),
        "account" => pages_a::account_page().into_view(),
        "taskbar" => pages_a::taskbar_page().into_view(),
        "notifications" => pages_a::notifications_page().into_view(),
        "clock" => pages_b::date_time_page().into_view(),
        "ease" => pages_b::ease_page().into_view(),
        "startup" => pages_b::startup_page().into_view(),
        "defaults" => pages_b::default_programs_page().into_view(),
        "sound" => pages_b::sound_page().into_view(),
        "devices" => pages_b::devices_page().into_view(),
        "display" => pages_b::display_page().into_view(),
        "about" => pages_b::about_page().into_view(),
        _ => home::home_page().into_view(),
    }
}

/// The pale gradient of the address bar and the pane of links.
fn side_fill() -> ShaderPaint {
    let c = scheme();
    vertical_gradient(&[(0.0, c.side_top), (1.0, c.side_bottom)])
}

fn on_home() -> bool {
    state::page() == "home"
}

// ----- the address bar -------------------------------------------------------------------------

fn address_bar() -> Div {
    div()
        .flex_none()
        .h(BAR_HEIGHT)
        .flex()
        .flex_row()
        .items_center()
        .gap(8.0)
        .px(8.0)
        .border_b(1.0)
        .fill(side_fill)
        .dyn_style(|s| s.border_color(scheme().menu_divider))
        .child(back_button())
        .child(path_box())
        .child(search_box())
}

/// Back to the home page, as the round arrow of Explorer.
fn back_button() -> Div {
    let ui = Interaction::new();
    div()
        .flex_none()
        .size(26.0)
        .rounded(13.0)
        .border(1.0)
        .border_color(BACK_BORDER)
        .flex()
        .items_center()
        .justify_center()
        .select_none()
        .interaction(ui)
        .access(|a| a.role(Role::Button).label("Back"))
        .on_click(|_| {
            if !untrack(on_home) {
                state::go("home");
            }
        })
        .fill(move || {
            let top = if ui.hovered() && !on_home() {
                BACK_TOP_HOT
            } else {
                BACK_TOP
            };
            vertical_gradient(&[(0.0, top), (1.0, BACK_BOTTOM)])
        })
        .dyn_style(|s| s.opacity(if on_home() { BACK_IDLE_OPACITY } else { 1.0 }))
        .child(
            img(asset("arrow-right.svg"))
                .size(14.0)
                .rotate(180.0)
                .object_fit(ObjectFit::Contain),
        )
}

/// A field of the address bar: the path, or the search box.
fn field_box() -> Div {
    div()
        .h(FIELD_HEIGHT)
        .flex()
        .flex_row()
        .items_center()
        .border(1.0)
        .overflow_clip()
}

/// The path: Control Panel ▸ category ▸ page.
fn path_box() -> Div {
    field_box()
        .flex_1()
        .min_w(0.0)
        .gap(6.0)
        .pl(6.0)
        .whitespace_nowrap()
        .dyn_style(|s| {
            let c = scheme();
            s.rounded(field_radius())
                .bg(c.field)
                .border_color(c.field_border)
        })
        .child(
            img(asset("icons/control-panel.png"))
                .flex_none()
                .size(16.0)
                .object_fit(ObjectFit::Contain),
        )
        .child(cp_link_tinted(
            TITLE,
            false,
            || scheme().text,
            || state::go("home"),
        ))
        .child(
            div()
                .flex_none()
                .dyn_style(|s| if on_home() { s.hidden() } else { s })
                .child(text(|| {
                    let page = state::page_info(&state::page());
                    format!("▸  {}  ▸  {}", page.category, page.title)
                })),
        )
}

/// "Search Control Panel": typing replaces the page with the tasks that match every word.
/// Enter goes to the first; Escape empties it.
fn search_box() -> Div {
    let query = state::query();
    let focused = signal(false);
    let input = text_input()
        .value(query)
        .placeholder("Search Control Panel")
        .flex_1()
        .min_w(0.0)
        .on_focus_change(move |on| focused.set(on))
        .on_submit(|typed| {
            if let Some(task) = state::search(typed).first() {
                state::go(task.page);
            }
        })
        .on_key_down(move |e| {
            if e.key == Key::Named(NamedKey::Escape) {
                query.set(String::new());
            }
        })
        .dyn_style(|s| {
            let c = scheme();
            s.font_family(font())
                .text_size(theme().font_size)
                .text_color(c.text)
                .caret_color(c.text)
        });
    field_box()
        .flex_none()
        .w(SEARCH_WIDTH)
        .gap(4.0)
        .pl(6.0)
        .pr(6.0)
        .dyn_style(move |s| {
            let c = scheme();
            s.rounded(field_radius())
                .bg(c.field)
                .border_color(if focused.get() {
                    c.field_focus
                } else {
                    c.field_border
                })
        })
        .child(input)
        .child(
            img(asset("search.svg"))
                .flex_none()
                .size(14.0)
                .object_fit(ObjectFit::Contain),
        )
}

// ----- the pane of links -----------------------------------------------------------------------

fn side_pane() -> Div {
    let link = |key: &'static str, label: &'static str| {
        cp_link(label, false, move || state::go(key)).dyn_style(move |s| {
            if state::page() == key {
                s.font_weight(FontWeight::BOLD)
            } else {
                s
            }
        })
    };
    div()
        .flex_none()
        .w(SIDE_WIDTH)
        .flex()
        .flex_col()
        .gap(9.0)
        .p(16.0)
        .border_r(1.0)
        .overflow_clip()
        .fill(side_fill)
        .dyn_style(|s| s.border_color(scheme().menu_divider))
        .child(link("home", "Control Panel Home"))
        .child(div().flex_none().h(6.0))
        .children(
            state::SIDE_PAGES
                .iter()
                .map(|&key| link(key, state::page_info(key).title)),
        )
}
