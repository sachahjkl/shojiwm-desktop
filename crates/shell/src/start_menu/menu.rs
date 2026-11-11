//! The Start menu, laid out as Windows 7 lays it out (QML `StartMenu`).
//!
//! Left: the pinned programs, the ones used most under them, and "All Programs", which swaps
//! the list for the programs in folders. Typing filters every program instead; the arrow keys
//! move through the results, Return starts the one picked, and with nothing found Return runs
//! what was typed.
//!
//! Right: the user, the places, and the power button, whose arrow opens log off, lock, restart,
//! sleep and hibernate. Resting on a program that has recent files shows them there instead.
//!
//! As Windows 7 builds it: the whole menu is a pane of glass with the frame's lines, the
//! programs sit in a white pane set into it on the left, and the places are white text on the
//! glass on the right. The account picture stands above the top edge.
//!
//! The view is built each time the menu opens, so it opens on its first page with an empty
//! search box.

use super::search::{self, Search, app_icon, program_lines};
use super::{close, places, user_tile};
use crate::paths::asset;
use crate::settings::{motion, settings};
use crate::theme::{corner, field_radius, menu_radius, parse_hex, popup_radius, scheme, theme};
use crate::ui::{RowLook, aero_scroll, font, menu_row, vertical_gradient, with_context_menu};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

/// The white line round the programs' pane, set into the glass (QML literal).
const PANE_LIGHT_LINE: Color = rgba(0xffffff99);
/// The dark line inside it (QML literal).
const PANE_DARK_LINE: Color = rgba(0x000000a6);
/// How long the pointer rests on a program before its jump list shows.
const JUMP_WAIT: Duration = Duration::from_millis(350);
/// The room the programs inside a folder are set in by.
const FOLDER_INDENT: f32 = 22.0;
/// The height of the line between the pinned programs and the ones used most.
const DIVIDER_ROOM: f32 = 9.0;

/// The page of the programs' pane.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    /// Pinned, then used most.
    Home,
    /// All Programs, in folders.
    All,
}

/// What the parts of the menu share while it is open.
#[derive(Clone)]
pub(super) struct Menu {
    /// The screen it is on.
    pub output_name: String,
    pub search: Search,
    /// The row Return chooses.
    pub pick: Signal<usize>,
    /// The program whose jump list shows in place of the places.
    pub jump: Signal<Option<Application>>,
    /// The colour picture of the place under the pointer, shown in the user's tile as Windows 7
    /// does ("documents", "computer", ...), or "" for the user's picture.
    pub hovered_place: Signal<&'static str>,
    jump_wait: Rc<Cell<Option<Timeout>>>,
}

impl Menu {
    /// Something is typed in the search box.
    pub fn searching(&self) -> bool {
        self.search.active()
    }

    /// The pointer comes onto a program (`inside`) or leaves it. A program with recent or pinned
    /// files shows them after a rest; another one takes its jump list away.
    fn hover_program(&self, app: &Application, inside: bool) {
        if let Some(t) = self.jump_wait.take() {
            t.cancel();
        }
        if !inside {
            return;
        }
        if has_jump_list(app) {
            let (jump, app) = (self.jump, app.clone());
            self.jump_wait.set(Some(set_timeout(JUMP_WAIT, move || {
                if jump.is_alive() {
                    jump.set(Some(app));
                }
            })));
        } else {
            self.jump.set(None);
        }
    }
}

fn has_jump_list(app: &Application) -> bool {
    !crate::apps::recent_for(app).is_empty() || !crate::apps::pinned_files_for(app).is_empty()
}

/// The Start menu of a screen, as it is when it opens.
pub(super) fn start_menu(output_name: String) -> Div {
    let menu = Menu {
        output_name,
        search: Search::new(),
        pick: signal(0),
        jump: signal(None),
        hovered_place: signal(""),
        jump_wait: Rc::default(),
    };
    // Each key starts the pick again at the top, and takes the jump list away.
    {
        let (query, pick, jump) = (menu.search.query, menu.pick, menu.jump);
        effect(move || {
            query.with(|_| ());
            pick.set(0);
            jump.set(None);
        });
    }
    div()
        .relative()
        .size_full()
        .select_none()
        .dyn_style(|s| {
            s.font_family(font())
                .text_size(theme().font_size)
                .text_color(scheme().text)
        })
        .child(glass())
        .child(places::places(menu.clone()))
        .child(programs(menu.clone()))
        .child(avatar(menu))
}

/// The pane of glass under the whole menu: the outline, the glass with its light rim, and the
/// colour of the glass chosen in the Control Panel over it.
fn glass() -> Div {
    let tint = memo(|| {
        settings().with(|s| (parse_hex(&s.glass_color), s.glass_color_strength as f32))
    });
    div()
        .absolute()
        .left(0.0)
        .right(0.0)
        .bottom(0.0)
        .dyn_style(|s| s.top(theme().menu_tile_overhang))
        .child(
            div()
                .absolute()
                .inset(0.0)
                .dyn_style(|s| s.rounded(corner(6.0)).bg(scheme().start_glass_outline)),
        )
        .child(
            div()
                .absolute()
                .inset(1.0)
                .border(1.0)
                .fill(|| {
                    let c = scheme();
                    vertical_gradient(&[
                        (0.0, c.start_glass_top),
                        (0.55, c.start_glass_middle),
                        (1.0, c.start_glass_bottom),
                    ])
                })
                .dyn_style(|s| s.rounded(menu_radius()).border_color(scheme().start_glass_highlight)),
        )
        .child(div().absolute().inset(1.0).dyn_style(move |s| {
            let s = s.rounded(menu_radius());
            match tint.get() {
                (Some(colour), strength) => s.bg(colour).opacity(strength),
                (None, _) => s.hidden(),
            }
        }))
}

/// The left column: the white pane, its list (pinned and used most, All Programs, or the
/// results), the All Programs row and the search box.
fn programs(menu: Menu) -> Div {
    let page = signal(Page::Home);
    let open_folder = signal(String::new());
    let searching = {
        let menu = menu.clone();
        memo(move || menu.searching())
    };
    let home = {
        let menu = menu.clone();
        show(
            move || !searching.get() && page.get() == Page::Home,
            move || aero_scroll(home_list(menu.clone())).flex_1(),
        )
    };
    let all = show(
        move || !searching.get() && page.get() == Page::All,
        move || aero_scroll(folder_list(open_folder)).flex_1(),
    );
    let results = {
        let menu = menu.clone();
        show(
            move || searching.get(),
            move || aero_scroll(search::results(menu.search, menu.pick, close, true)).flex_1(),
        )
    };
    let list = div()
        .flex_1()
        .min_h(0.0)
        .flex()
        .flex_col()
        .dyn_style(|s| s.m(theme().menu_padding))
        .child(home)
        .child(all)
        .child(results);
    let all_programs = show(
        move || !searching.get(),
        move || {
            div()
                .flex_none()
                .flex()
                .flex_col()
                .justify_between()
                .mb(4.0)
                .dyn_style(|s| s.mx(theme().menu_padding).h(super::row_height() + DIVIDER_ROOM))
                .child(
                    div()
                        .mt(4.0)
                        .h(1.0)
                        .dyn_style(|s| s.bg(scheme().menu_divider)),
                )
                .child(
                    menu_row(
                        "",
                        move || {
                            if page.get() == Page::All {
                                "\u{25C2}  Back".to_owned()
                            } else {
                                "All Programs  \u{25B8}".to_owned()
                            }
                        },
                        RowLook {
                            bold: true,
                            ..Default::default()
                        },
                        || false,
                    )
                    .on_click(move |_| {
                        page.update(|p| {
                            *p = if *p == Page::All { Page::Home } else { Page::All }
                        });
                        open_folder.set(String::new());
                    }),
                )
        },
    );
    div()
        .absolute()
        .dyn_style(|s| {
            let t = theme();
            s.left(t.menu_glass_border)
                .top(t.menu_tile_overhang + t.menu_glass_border)
                .bottom(t.menu_glass_border)
                .w(t.menu_programs_width - t.menu_glass_border)
        })
        // The white pane, set into the glass with the frame's two lines.
        .child(
            div()
                .absolute()
                .inset(-2.0)
                .bg(PANE_LIGHT_LINE)
                .dyn_style(|s| s.rounded(corner(popup_radius() + 2.0)))
                .child(
                    div()
                        .absolute()
                        .inset(1.0)
                        .bg(PANE_DARK_LINE)
                        .dyn_style(|s| s.rounded(corner(popup_radius() + 1.0))),
                ),
        )
        .child(
            div()
                .absolute()
                .inset(0.0)
                .fill(|| {
                    let c = scheme();
                    vertical_gradient(&[(0.0, c.menu_top), (1.0, c.menu_bottom)])
                })
                .dyn_style(|s| s.rounded(popup_radius())),
        )
        .child(
            div()
                .absolute()
                .inset(0.0)
                .flex()
                .flex_col()
                .child(list)
                .child(all_programs)
                .child(search_box(menu)),
        )
}

/// A program of the lists: a click starts it, a right click opens its menu, and resting on it
/// shows its jump list. Lit while its jump list shows.
fn program_row(menu: &Menu, app: Application, bold: bool) -> Div {
    let jump = menu.jump;
    let mine = app.clone();
    let has_arrow = has_jump_list(&app);
    let launched = app.clone();
    let hovered = (menu.clone(), app.clone());
    let row = menu_row(
        app_icon(&app),
        app.name.clone(),
        RowLook {
            bold,
            arrow: has_arrow,
            ..Default::default()
        },
        move || jump.with(|j| j.as_ref() == Some(&mine)),
    )
    .on_hover(move |inside| hovered.0.hover_program(&hovered.1, inside))
    .on_click(move |_| {
        crate::apps::launch(&launched);
        close();
    });
    with_context_menu(row, move || program_lines(&app, close))
}

/// The pinned programs, a line, then the ones used most.
fn home_list(menu: Menu) -> View {
    dyn_view(move || {
        let pinned = crate::apps::pinned();
        let frequent = if settings().with(|s| s.show_frequent) {
            crate::apps::frequent()
        } else {
            Vec::new()
        };
        let divider = (!pinned.is_empty() && !frequent.is_empty()).then(|| {
            div()
                .flex_none()
                .h(DIVIDER_ROOM)
                .flex()
                .items_center()
                .child(div().flex_1().h(1.0).dyn_style(|s| s.bg(scheme().menu_divider)))
        });
        let pinned_rows: Vec<Div> = pinned
            .into_iter()
            .map(|app| program_row(&menu, app, true))
            .collect();
        let frequent_rows: Vec<Div> = frequent
            .into_iter()
            .map(|app| program_row(&menu, app, false))
            .collect();
        div()
            .flex()
            .flex_col()
            .gap(1.0)
            .children(pinned_rows)
            .children(divider)
            .children(frequent_rows)
    })
}

/// Every program, in folders; a click on a folder opens it in place, and closes the one open.
fn folder_list(open_folder: Signal<String>) -> View {
    dyn_view(move || {
        let folders = crate::apps::folders();
        div().flex().flex_col().gap(1.0).children(
            folders
                .into_iter()
                .map(|folder| {
                    let name = folder.name.clone();
                    let expanded = {
                        let name = name.clone();
                        move || open_folder.with(|o| *o == name)
                    };
                    let expanded_icon = expanded.clone();
                    let expanded_row = expanded.clone();
                    let header = menu_row(
                        move || {
                            let file = if expanded_icon() {
                                "icons/folder-open.png"
                            } else {
                                "icons/folder.png"
                            };
                            asset(file).to_string_lossy().into_owned()
                        },
                        name.clone(),
                        RowLook::default(),
                        expanded_row,
                    )
                    .on_click({
                        let name = name.clone();
                        move |_| {
                            open_folder.update(|o| {
                                *o = if *o == name { String::new() } else { name.clone() }
                            })
                        }
                    });
                    let entries = folder.entries;
                    let inside = show(expanded, move || {
                        div().flex().flex_col().children(
                            entries
                                .iter()
                                .cloned()
                                .map(folder_entry)
                                .collect::<Vec<_>>(),
                        )
                    });
                    div().flex().flex_col().child(header).child(inside)
                })
                .collect::<Vec<_>>(),
        )
    })
}

/// A program inside a folder of All Programs.
fn folder_entry(app: Application) -> Div {
    let launched = app.clone();
    let row = menu_row(
        app_icon(&app),
        app.name.clone(),
        RowLook {
            indent: FOLDER_INDENT,
            ..Default::default()
        },
        || false,
    )
    .on_click(move |_| {
        crate::apps::launch(&launched);
        close();
    });
    with_context_menu(row, move || program_lines(&app, close))
}

/// The search box at the foot of the programs: it has the keyboard as the menu opens.
fn search_box(menu: Menu) -> Div {
    let query = menu.search.query;
    let focused = signal(false);
    let input = node_ref();
    request_frame(move |_| input.focus());
    let search = menu.search;
    let pick = menu.pick;
    let field = text_input()
        .value(query)
        .node_ref(input)
        .w_full()
        .on_focus_change(move |on| focused.set(on))
        .on_submit(move |_| {
            if search.accept(pick.get_untracked()) {
                close();
            }
        })
        .on_key_down(move |e| match &e.key {
            Key::Named(NamedKey::Escape) => {
                e.prevent_default();
                if query.with_untracked(String::is_empty) {
                    close();
                } else {
                    query.set(String::new());
                }
            }
            Key::Named(NamedKey::ArrowUp) => {
                e.prevent_default();
                if untrack(|| search.active()) && pick.get_untracked() > 0 {
                    pick.update(|p| *p -= 1);
                }
            }
            Key::Named(NamedKey::ArrowDown) => {
                e.prevent_default();
                let count = untrack(|| search.pick_count.get());
                if untrack(|| search.active()) && pick.get_untracked() + 1 < count {
                    pick.update(|p| *p += 1);
                }
            }
            _ => {}
        })
        .dyn_style(|s| {
            let c = scheme();
            s.font_family(font())
                .text_size(theme().font_size)
                .text_color(c.text)
                .selection_color(c.selection)
        });
    div()
        .flex_none()
        .flex()
        .flex_row()
        .items_center()
        .border(1.0)
        .dyn_style(move |s| {
            let t = theme();
            let c = scheme();
            s.mx(t.menu_padding)
                .mb(t.menu_padding)
                .h(t.menu_search_height)
                .rounded(field_radius())
                .bg(c.field)
                .border_color(if focused.get() { c.field_focus } else { c.field_border })
        })
        .child(
            img(asset("search.svg"))
                .size(14.0)
                .ml(6.0)
                .flex_none()
                .object_fit(ObjectFit::Contain),
        )
        .child(
            div()
                .relative()
                .flex_1()
                .min_w(0.0)
                .mx(6.0)
                .flex()
                .items_center()
                .overflow_clip()
                .child(field)
                .child(search::placeholder(
                    query,
                    "Search programs, or type a command",
                    || theme().font_size,
                )),
        )
}

/// The account picture over the top edge, above the places; the place under the pointer shows
/// in the same frame, as Windows 7 shows it.
fn avatar(menu: Menu) -> Div {
    let hovered = menu.hovered_place;
    // The last place shown, kept while its picture fades out.
    let last = signal("");
    effect(move || {
        let now = hovered.get();
        if !now.is_empty() {
            last.set(now);
        }
    });
    let fade = untrack(|| motion(theme().fade));
    let place_shown = follow(
        move || if hovered.with(|h| !h.is_empty()) { 1.0f32 } else { 0.0 },
        Transition::ms(fade).ease_out(),
    );
    let tile_size = untrack(|| theme().menu_tile_size);
    div()
        .absolute()
        .top(0.0)
        .dyn_style(|s| {
            let t = theme();
            let places_width = t.menu_width - t.menu_programs_width - 2.0;
            let places_left = t.menu_programs_width + 1.0;
            s.left(places_left + (places_width - t.menu_tile_size) / 2.0)
                .size(t.menu_tile_size)
        })
        .child(
            user_tile::user_tile(super::user_picture, tile_size)
                .absolute()
                .inset(0.0)
                .dyn_style(move |s| s.opacity(1.0 - place_shown.get().clamp(0.0, 1.0))),
        )
        .child(
            user_tile::user_tile(
                move || asset(&format!("icons/{}.png", last.get())),
                tile_size,
            )
            .absolute()
            .inset(0.0)
            .dyn_style(move |s| {
                let shown = place_shown.get().clamp(0.0, 1.0);
                if last.with(|l| l.is_empty()) { s.hidden() } else { s.opacity(shown) }
            }),
        )
}
