//! The desktop of each screen (the wallpaper and desktop layers of `shell.qml`): the picture, the
//! icons, the sticky notes made on that screen and, on the main screen, the gadgets.
//!
//! The desktop is a background layer that asks for no blur (`no_blur`). It takes the keyboard when
//! clicked, for Delete, F2, Return and Ctrl+A, and holds it while an icon is renamed or a note is
//! typed in. Its right-click menu opens in an overlay layer of its own over everything while it
//! is open: ShojiWM hit-tests the windows before the popups of a background surface, so a popup
//! of the desktop itself would be drawn over a window and let the window take its clicks. A
//! click anywhere else on the screen closes it, as on Windows 7, and so does Escape.

mod files;
mod gadgets;
mod icons;
mod notes;
pub mod wallpaper;

use crate::settings::settings;
use crate::ui::popup_menu;
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use icons::{Desk, ICONS_MIME, IconDrag, dragged_icons, end_drag};

/// Opens the desktop of `output`. Call it for each output, from the shell's root (as
/// `for_each_output` does): the first call starts what every desktop shares (the folder's
/// watcher, the notes, the slide show) in roots of their own.
pub fn open(output: Output) {
    files::install();
    notes::install();
    wallpaper::install_slideshow();

    let desk = Desk::new(output.name.clone());
    let layer = open_layer(
        LayerOptions::background()
            .output(&output)
            .namespace("no_blur")
            .keyboard(Keyboard::OnDemand),
        move || view(desk),
    );
    // The keyboard stays with the desktop while an icon is renamed or a note is typed in.
    let holds = memo(move || desk.rename_key.with(|k| !k.is_empty()) || notes::editing());
    effect(move || {
        layer.set_keyboard(if holds.get() {
            Keyboard::Exclusive
        } else {
            Keyboard::OnDemand
        });
    });

    let menu_open = memo(move || desk.menu.with(Option::is_some));
    effect(move || {
        if !menu_open.get() {
            return;
        }
        // Closed with this run of the effect, when the menu closes.
        open_layer(
            LayerOptions::new()
                .layer(Layer::Overlay)
                .anchor(Anchor::ALL)
                .exclusive(Exclusive::Ignore)
                .keyboard(Keyboard::Exclusive)
                .output(&output)
                .namespace("no_blur"),
            move || menu_layer(desk),
        );
    });
}

/// The main screen: the one `primary_screen` (else `SHOJIWM_WIN7_PRIMARY_SCREEN`) names, when it
/// is connected, else the first output. It is not the screen with the focus: it is where the
/// shell puts what has no screen of its own (the gadgets, the icons without a place). Reading
/// it tracks the settings and the outputs.
pub fn primary_output_name() -> String {
    let mut wanted = settings().with(|s| s.primary_screen.clone());
    if wanted.is_empty() {
        wanted = std::env::var("SHOJIWM_WIN7_PRIMARY_SCREEN").unwrap_or_default();
    }
    let list = outputs();
    if !wanted.is_empty() && list.iter().any(|o| o.name == wanted) {
        return wanted;
    }
    list.into_iter().next().map(|o| o.name).unwrap_or_default()
}

/// A point of a drop, from window physical pixels to the desktop's logical ones.
fn logical(position: PointF) -> (f32, f32) {
    let scale = untrack(|| window_scale().as_f32());
    (position.x / scale, position.y / scale)
}

/// The desktop: the picture, the icons, the notes and the gadgets. Whatever is dropped on it
/// lands here: the shell's own icons from this screen (an [`IconDrag`]) or another
/// ([`ICONS_MIME`]), and files from other programs.
fn view(desk: Desk) -> Div {
    let screen = desk.screen.get_untracked();
    let on_primary = {
        let screen = screen.clone();
        move || primary_output_name() == screen
    };
    div()
        .relative()
        .size_full()
        .overflow_hidden()
        .bg(wallpaper::BACKDROP)
        .on_resized(move |w, h| desk.area.set_if_changed((w, h)))
        // A note keeps the keyboard until the pointer goes to a window or a panel: the desktop
        // is then under it no more.
        .on_hover(|inside| {
            if !inside {
                notes::stop_editing();
            }
        })
        .drop_target(move |drag: &IconDrag, e: &DropEvent| {
            let (x, y) = logical(e.position);
            desk.drop_icons(drag, x, y, e.action == DragAction::Copy);
            end_drag();
        })
        .on_drop_data(ICONS_MIME, move |_, e| {
            if let Some(drag) = dragged_icons() {
                let (x, y) = logical(e.position);
                desk.drop_icons(&drag, x, y, e.action == DragAction::Copy);
            }
            end_drag();
        })
        .on_drop_files(move |paths, e| {
            let (x, y) = logical(e.position);
            let paths: Vec<String> = paths
                .iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect();
            desk.drop_files(&paths, x, y, e.action == DragAction::Copy);
        })
        .on_drag_over(move |_: &IconDrag, e: &DropEvent| {
            let (x, y) = logical(e.position);
            desk.drag_moved(x, y);
        })
        .on_drag_over(move |_: &ExternalDrag, e: &DropEvent| {
            let (x, y) = logical(e.position);
            desk.drag_moved(x, y);
        })
        .on_drag_hover(move |inside| {
            if !inside {
                desk.drag_left();
            }
        })
        .child(wallpaper::view())
        .child(icons::view(desk))
        .child(notes::view(screen, desk.area))
        .child(show(on_primary, move || gadgets::view(desk.area)))
}

/// The layer of the desktop's menu: the menu where the pointer was pressed. A press outside of
/// it closes it.
fn menu_layer(desk: Desk) -> Div {
    // The menu has no grab (`PopupKind::Bubble`): a press anywhere else on this layer, which
    // covers the screen, closes it. The backdrop stands beside the menu, not around it, so a
    // press in the menu does not reach it.
    let backdrop = div()
        .absolute()
        .inset(0.0)
        .on_pointer_down(move |_| desk.close_menu())
        .on_aux_click(move |_| desk.close_menu());
    div()
        .size_full()
        .relative()
        .child(backdrop)
        .child(popup_menu(
            move || desk.menu.with(Option::is_some),
            move || desk.close_menu(),
            // Inside this layer, which takes every press around the menu: no grab, which the layer
            // could not make anyway, never having had a press of its own.
            MenuOptions::new()
                .at(desk.menu_rect)
                .kind(PopupKind::Bubble),
            move || desk.menu_lines(),
        ))
}
