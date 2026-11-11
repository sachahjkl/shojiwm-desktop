//! The right column of the Start menu: the account's name and the places as white text on the
//! glass (QML `PlaceRow`), Recent Items with its submenu, the jump list of the program under the
//! pointer in their place, and the power button at the foot.

use super::menu::Menu;
use super::search::open_file;
use super::{close, power, row_height};
use crate::settings::{home, settings};
use crate::theme::{corner, item_radius, menu_tile_overhang, scheme, theme};
use crate::ui::{MenuLine, RowLook, font, menu_row, popup_menu, vertical_gradient};
use anti_webui::prelude::*;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

/// How long the pointer rests on Recent Items before its submenu opens.
const RECENT_WAIT: Duration = Duration::from_millis(350);
/// The most files Recent Items lists.
const RECENT_LIMIT: usize = 15;
/// The height of a separator's row, and the length of its line across the column.
const SEPARATOR_HEIGHT: f32 = 11.0;
const SEPARATOR_LENGTH: f32 = 0.72;

/// The etched line between groups of places: dark over light (QML literals).
const ETCH_DARK: Color = rgba(0x0000004d);
const ETCH_LIGHT: Color = rgba(0xffffff33);
/// A place's text, white on the glass (QML literal).
const PLACE_TEXT: Color = Color::WHITE;
/// The band of glass under the pointer: its rim, the darker line outside it, and its fill
/// from top to bottom, at rest and pressed (QML literals).
const BAND_RIM: Color = rgba(0xffffff80);
const BAND_OUTLINE: Color = rgba(0x00000040);
const BAND_FILL: [Color; 3] = [rgba(0xffffff4d), rgba(0xffffff2e), rgba(0xffffff40)];
const BAND_FILL_PRESSED: [Color; 3] = [rgba(0xffffff1a), rgba(0xffffff14), rgba(0xffffff26)];

/// What a line of the places does.
#[derive(Clone)]
enum Action {
    /// Opens a folder in the file manager.
    Folder(String),
    /// Opens the Recent Items submenu.
    Recent,
    ControlPanel,
    Run,
}

/// A line of the places, or `None` for a separator.
struct Place {
    label: &'static str,
    /// The colour picture shown in the user's tile while the pointer is on it.
    tile: &'static str,
    action: Action,
}

fn places_list() -> Vec<Option<Place>> {
    let dir = |key, fallback| Action::Folder(crate::apps::user_dir(key, fallback));
    vec![
        Some(Place {
            label: "Documents",
            tile: "documents",
            action: dir("DOCUMENTS", "Documents"),
        }),
        Some(Place {
            label: "Downloads",
            tile: "downloads",
            action: dir("DOWNLOAD", "Downloads"),
        }),
        Some(Place {
            label: "Pictures",
            tile: "pictures",
            action: dir("PICTURES", "Pictures"),
        }),
        Some(Place {
            label: "Music",
            tile: "music",
            action: dir("MUSIC", "Music"),
        }),
        Some(Place {
            label: "Videos",
            tile: "videos",
            action: dir("VIDEOS", "Videos"),
        }),
        Some(Place {
            label: "Recent Items",
            tile: "documents",
            action: Action::Recent,
        }),
        None,
        Some(Place {
            label: "Computer",
            tile: "computer",
            action: Action::Folder("/".into()),
        }),
        None,
        Some(Place {
            label: "Control Panel",
            tile: "control-panel",
            action: Action::ControlPanel,
        }),
        Some(Place {
            label: "Run...",
            tile: "run",
            action: Action::Run,
        }),
    ]
}

/// The right column: from the edge of the programs to the menu's right edge, and from the top of
/// the glass to its foot.
pub(super) fn places(menu: Menu) -> Div {
    let showing_jump = {
        let menu = menu.clone();
        memo(move || menu.jump.with(Option::is_some) && !menu.searching())
    };
    let jump = {
        let menu = menu.clone();
        show(move || showing_jump.get(), move || jump_list(menu.clone()))
    };
    let column = {
        let menu = menu.clone();
        show(
            move || !showing_jump.get(),
            move || place_column(menu.clone()),
        )
    };
    div()
        .absolute()
        .right(1.0)
        .bottom(1.0)
        .dyn_style(|s| {
            let t = theme();
            s.top(menu_tile_overhang() + 1.0)
                .w(t.menu_width - t.menu_programs_width - 2.0)
        })
        .child(jump)
        .child(column)
        .child(power::power_button())
}

/// The account's name, then the places. Room is kept at the top for the picture, which hangs
/// over this corner.
fn place_column(menu: Menu) -> Div {
    let recent_open = signal(false);
    let recent_wait: Rc<Cell<Option<Timeout>>> = Rc::default();
    let recent_anchor = node_ref();
    let user = place_row(crate::apps::user_name, |_| {}).on_click(|_| {
        if let Some(home) = home() {
            crate::apps::open_path(&home.to_string_lossy());
        }
        close();
    });
    let lines: Vec<Div> = places_list()
        .into_iter()
        .map(|place| match place {
            None => separator(),
            Some(place) => place_line(
                &menu,
                place,
                recent_open,
                recent_wait.clone(),
                recent_anchor,
            ),
        })
        .collect();
    div()
        .absolute()
        .left(0.0)
        .right(0.0)
        .top(0.0)
        .flex()
        .flex_col()
        .gap(2.0)
        .dyn_style(|s| {
            let t = theme();
            s.p(t.menu_padding)
        })
        .child(div().flex_none().dyn_style(|s| {
            let t = theme();
            s.h(t.menu_tile_size - menu_tile_overhang())
        }))
        .child(user)
        .children(lines)
        .child(recent_menu(recent_open, recent_anchor))
}

/// An etched line, dark over light, short of both edges.
fn separator() -> Div {
    let line = |colour: Color| {
        div()
            .h(1.0)
            .w(Length::Percent(SEPARATOR_LENGTH * 100.0))
            .bg(colour)
    };
    div()
        .flex_none()
        .h(SEPARATOR_HEIGHT)
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .child(line(ETCH_DARK))
        .child(line(ETCH_LIGHT))
}

fn place_line(
    menu: &Menu,
    place: Place,
    recent_open: Signal<bool>,
    recent_wait: Rc<Cell<Option<Timeout>>>,
    recent_anchor: NodeRef,
) -> Div {
    let hovered_place = menu.hovered_place;
    let tile = place.tile;
    let recent = matches!(place.action, Action::Recent);
    let output_name = menu.output_name.clone();
    let action = place.action.clone();
    let wait = recent_wait;
    let on_hover = move |inside: bool| {
        if inside {
            hovered_place.set(tile);
        } else if hovered_place.get_untracked() == tile {
            hovered_place.set("");
        }
        // Recent Items opens its submenu after a rest; another place closes it. Only leaving
        // Recent Items itself cancels the wait: another line's leave can come after this one's
        // enter.
        if inside && recent {
            if let Some(t) = wait.take() {
                t.cancel();
            }
            wait.set(Some(set_timeout(RECENT_WAIT, move || {
                if recent_open.is_alive() {
                    recent_open.set(true);
                }
            })));
        } else if inside {
            if let Some(t) = wait.take() {
                t.cancel();
            }
            recent_open.set(false);
        } else if recent && let Some(t) = wait.take() {
            t.cancel();
        }
    };
    let row = place_row(place.label, on_hover).on_click(move |_| match &action {
        Action::Recent => recent_open.set(true),
        Action::Run => {
            // The Run dialog, on this menu's screen.
            let screen = output_name.clone();
            close();
            crate::dialogs::run_toggle_on(&screen);
        }
        Action::ControlPanel => {
            crate::control_panel::open();
            close();
        }
        Action::Folder(path) => {
            crate::apps::open_path(path);
            close();
        }
    });
    if !recent {
        return row;
    }
    // The arrow of the line that opens a submenu.
    row.node_ref(recent_anchor).child(
        div()
            .absolute()
            .right(8.0)
            .top(0.0)
            .bottom(0.0)
            .flex()
            .items_center()
            .pointer_events_none()
            .dyn_style(|s| {
                s.text_size(theme().font_size)
                    .text_color(scheme().places_text)
            })
            .child("\u{25B8}"),
    )
}

/// One line of the places: white text on the glass, lit under the pointer by a pale band of
/// glass with a light rim, as Windows 7 draws them. No icon: the picture above the menu shows
/// the place instead. `on_hover` hears the pointer come and go.
fn place_row(label: impl IntoTextSource, on_hover: impl Fn(bool) + 'static) -> Div {
    let ui = Interaction::new();
    let band = div()
        .absolute()
        .inset(0.0)
        .border(1.0)
        .border_color(BAND_RIM)
        .fill(move || {
            let f = if ui.pressed() {
                BAND_FILL_PRESSED
            } else {
                BAND_FILL
            };
            vertical_gradient(&[(0.0, f[0]), (0.5, f[1]), (1.0, f[2])])
        })
        .dyn_style(move |s| {
            let s = s.rounded(item_radius());
            if ui.hovered() { s } else { s.hidden() }
        })
        // The darker line outside the rim. Placed from inside the band's border, as absolute
        // boxes are.
        .child(
            div()
                .absolute()
                .inset(-2.0)
                .border(1.0)
                .border_color(BAND_OUTLINE)
                .dyn_style(|s| s.rounded(corner(item_radius() + 1.0))),
        );
    div()
        .relative()
        .flex_none()
        .select_none()
        .interaction(ui)
        .on_hover(on_hover)
        .dyn_style(|s| s.h(row_height()))
        .child(band)
        .child(
            div()
                .absolute()
                .left(10.0)
                .right(8.0)
                .top(0.0)
                .bottom(0.0)
                .flex()
                .items_center()
                .overflow_clip()
                .whitespace_nowrap()
                .text_color(PLACE_TEXT)
                .dyn_style(|s| s.font_family(font()).text_size(theme().font_size))
                .child(text(label)),
        )
}

/// The Recent Items submenu: the files the desktop opened last, beside the line. It lets the
/// pointer go to the other places, which close it.
fn recent_menu(open: Signal<bool>, anchor: NodeRef) -> Show {
    let options = MenuOptions::new()
        .anchor(anchor)
        .placement(Placement::RightStart)
        .gap(0)
        .not_modal();
    popup_menu(
        move || open.get(),
        move || open.set(false),
        options,
        || {
            let files = crate::apps::recent_files();
            if files.is_empty() {
                return vec![MenuLine::item("No recent items", || {}).enabled(false)];
            }
            files
                .into_iter()
                .take(RECENT_LIMIT)
                .map(|file| {
                    let path = file.path.clone();
                    MenuLine::item(file.name.clone(), move || {
                        open_file(&path);
                        close();
                    })
                    .icon(crate::apps::file_icon(&file.name))
                })
                .collect()
        },
    )
}

/// The jump list of the program under the pointer, in place of the places: its pinned files,
/// then its recent ones. A click opens one with the program; the pin pins it or takes it off.
fn jump_list(menu: Menu) -> Div {
    let jump = menu.jump;
    let list = dyn_view(move || {
        let Some(app) = jump.get() else {
            return div();
        };
        // Read so that pinning a file shows at once.
        settings().with(|s| s.jump_pins.len());
        let pinned = crate::apps::pinned_files_for(&app);
        let recent = crate::apps::recent_unpinned(&app);
        let _heading = |label: &'static str| {
            menu_row(
                "",
                label,
                RowLook {
                    heading: true,
                    ..Default::default()
                },
                || false,
            )
        };
        let file_row = |file: crate::apps::JumpFile, pinned: bool| {
            let (app, path) = (app.clone(), file.path.clone());
            let opened = (app.clone(), path.clone());
            menu_row(
                crate::apps::file_icon(&file.name),
                file.name.clone(),
                RowLook {
                    places: true,
                    pin: Some((
                        pinned,
                        Rc::new(move || crate::apps::set_file_pinned(&app, &path, !pinned)),
                    )),
                    ..Default::default()
                },
                || false,
            )
            .on_click(move |_| {
                crate::apps::open_with(&opened.0, &opened.1);
                close();
            })
        };
        let mut rows: Vec<Div> = Vec::new();
        if !pinned.is_empty() {
            rows.push(crate::ui::section_title("Pinned"));
            rows.extend(pinned.into_iter().map(|f| file_row(f, true)));
        }
        if !recent.is_empty() {
            rows.push(crate::ui::section_title("Recent"));
            rows.extend(recent.into_iter().map(|f| file_row(f, false)));
        }
        div().flex().flex_col().gap(1.0).children(rows)
    });
    div()
        .absolute()
        .left(0.0)
        .right(0.0)
        .top(0.0)
        .dyn_style(|s| {
            let t = theme();
            s.p(t.menu_padding)
                .pt(t.menu_padding + t.menu_tile_size - menu_tile_overhang())
        })
        .child(list)
}
