//! The live previews Windows 7 shows over a taskbar button (QML `ThumbnailPopup`): a card per
//! window with its title, a close button and a picture of it, and the media buttons when the
//! program plays media.
//!
//! They open on their own layer surface over the bar, not as a popup of the bar: they open
//! when the pointer rests on a button, with no press, and take clicks without closing when the
//! pointer goes back to the bar. The pictures are AntiWebUI's `capture` of each window
//! (`ext_image_copy_capture` or ShojiWM's `hyprland_toplevel_export`).

use super::Bar;
use super::media::{player_program, transport};
use crate::apps::{icon_for, title_of, window_groups};
use crate::theme::{bar_at_top, corner, scheme, theme};
use crate::ui::{font, popup_frame, shell_icon, vertical_gradient};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

/// How often a live picture is taken again while the window changes.
const LIVE_EVERY: Duration = Duration::from_millis(250);
/// The close button of a card: red under the pointer, as the caption's.
const CLOSE_BORDER: Color = rgb(0x7a1a12);
const CLOSE_TOP: Color = rgb(0xf08a7c);
const CLOSE_BOTTOM: Color = rgb(0xc42b1c);
const CLOSE_WIDTH: f32 = 20.0;
const CLOSE_HEIGHT: f32 = 17.0;
/// The size of the program's icon on a card with no picture.
const CARD_ICON: f32 = 32.0;
/// The room between the cards and the media buttons.
const TOOLBAR_GAP: f32 = 8.0;

/// The windows of the group shown.
fn shown_windows(bar: &Bar) -> Vec<Toplevel> {
    let Some(key) = bar.preview.with(|p| p.as_ref().map(|p| p.key.clone())) else {
        return Vec::new();
    };
    window_groups()
        .into_iter()
        .find(|g| g.key == key)
        .map(|g| g.windows)
        .unwrap_or_default()
}

/// Where the layer goes: centred on the button, kept on the screen, away from the bar.
fn margin(bar: &Bar, count: usize) -> Margin {
    let t = theme();
    let n = count.max(1) as f32;
    let width = t.thumb_padding * 2.0 + n * t.thumb_width + (n - 1.0) * t.thumb_gap;
    let screen = bar
        .output
        .current()
        .map_or(bar.output.logical_size.width, |o| o.logical_size.width) as f32;
    let center = bar.preview.with(|p| p.as_ref().map_or(0.0, |p| p.center));
    let left = (center - width / 2.0)
        .clamp(0.0, (screen - width).max(0.0))
        .round() as i32;
    let away = (t.taskbar_height + t.popup_gap).round() as i32;
    if bar_at_top() {
        Margin {
            top: away,
            left,
            ..Default::default()
        }
    } else {
        Margin {
            bottom: away,
            left,
            ..Default::default()
        }
    }
}

/// Opens the thumbnails' layer while the bar shows a preview, and keeps it over the button.
pub(super) fn follow(bar: Bar) {
    let open = {
        let bar = bar.clone();
        memo(move || bar.preview.with(Option::is_some))
    };
    let handle: Rc<RefCell<Option<LayerHandle>>> = Rc::default();
    {
        let bar = bar.clone();
        let handle = handle.clone();
        effect(move || {
            let on = open.get();
            untrack(|| {
                if let Some(old) = handle.borrow_mut().take() {
                    old.close();
                }
                if !on {
                    return;
                }
                let edge = if bar_at_top() {
                    Anchor::TOP
                } else {
                    Anchor::BOTTOM
                };
                let options = LayerOptions::new()
                    .layer(Layer::Top)
                    .anchor(edge | Anchor::LEFT)
                    .exclusive(Exclusive::Ignore)
                    .keyboard(Keyboard::None)
                    .fit_content()
                    .margin(margin(&bar, shown_windows(&bar).len()))
                    .output(&bar.output)
                    .namespace("shojiwm-win7-thumbnails");
                let shown = bar.clone();
                *handle.borrow_mut() = Some(open_layer(options, move || view(shown)));
            });
        });
    }
    effect(move || {
        let place = margin(&bar, shown_windows(&bar).len());
        if let Some(layer) = handle.borrow().as_ref() {
            layer.set_margin(place);
        }
    });
}

/// The media player of the windows' program, for the thumbnail toolbar Windows 7 gives media
/// players: the one whose name on the bus is the program's.
fn player_of(windows: &[Toplevel]) -> Option<Player> {
    let app_id = windows.first()?.app_id.to_lowercase();
    if app_id.is_empty() {
        return None;
    }
    let short = app_id.rsplit('.').next().unwrap_or(&app_id).to_owned();
    media().players().into_iter().find(|p| {
        let name = player_program(p);
        !name.is_empty() && (app_id == name || app_id.contains(&name) || name.contains(&short))
    })
}

fn view(bar: Bar) -> Div {
    let windows = {
        let bar = bar.clone();
        memo(move || shown_windows(&bar))
    };
    // The last window of the group closed: nothing more to show.
    {
        let bar = bar.clone();
        effect(move || {
            if windows.with(Vec::is_empty) {
                // Not while this layer is being worked on: the change closes it.
                let preview = bar.preview;
                set_timeout(Duration::ZERO, move || preview.set(None));
            }
        });
    }
    let player = memo(move || windows.with(|w| player_of(w)));
    let hover = bar.clone();
    let cards = bar.clone();
    popup_frame()
        .items_center()
        .dyn_style(|s| s.p(theme().thumb_padding))
        .on_hover(move |inside| {
            hover.preview_inside.set(inside);
            if !inside {
                hover.hide_preview_soon();
            }
        })
        .child(
            div()
                .flex()
                .flex_row()
                .dyn_style(|s| s.gap(theme().thumb_gap))
                .child(for_each(
                    move || windows.get(),
                    |t| t.id(),
                    move |t| card(cards.clone(), t),
                )),
        )
        .child(show(
            move || player.with(Option::is_some),
            move || div().mt(TOOLBAR_GAP).child(transport(move || player.get())),
        ))
}

/// One window: its title and close button, and its picture, which shows it on a click.
fn card(bar: Bar, window: Toplevel) -> Div {
    let id = window.id();
    let now = memo(move || toplevels().all().into_iter().find(|t| t.id() == id));
    let card_ui = Interaction::new();
    let frame_ui = Interaction::new();
    let close_ui = Interaction::new();
    let t = untrack(theme);
    let shot = t.live_thumbnails.then(|| {
        let scale = bar.output.scale.max(1.0);
        let (w, h) = (
            (t.thumb_width * scale).ceil() as u32,
            (t.thumb_height * scale).ceil() as u32,
        );
        capture(
            &window,
            CaptureOptions::default().max_size(w, h).every(LIVE_EVERY),
        )
    });
    let has_picture = {
        let shot = shot.clone();
        memo(move || shot.as_ref().is_some_and(|s| s.picture().is_some()))
    };
    let picture = shot.map(|shot| {
        bitmap_view(move || shot.picture())
            .absolute()
            .left(1.0)
            .top(1.0)
            .right(1.0)
            .bottom(1.0)
            .object_fit(ObjectFit::Contain)
    });
    let close = move || {
        if let Some(w) = untrack(|| now.get()) {
            let _ = w.close();
        }
    };
    let close_button = div()
        .absolute()
        .right(0.0)
        .top(1.0)
        .w(CLOSE_WIDTH)
        .h(CLOSE_HEIGHT)
        .border(1.0)
        .flex()
        .items_center()
        .justify_center()
        .font_weight(FontWeight::BOLD)
        .text_size(15.0)
        .interaction(close_ui)
        .fill(move || {
            if close_ui.hovered() {
                vertical_gradient(&[(0.0, CLOSE_TOP), (1.0, CLOSE_BOTTOM)])
            } else {
                vertical_gradient(&[(0.0, Color::TRANSPARENT), (1.0, Color::TRANSPARENT)])
            }
        })
        .dyn_style(move |s| {
            let hot = close_ui.hovered();
            let s = s
                .rounded(corner(3.0))
                .border_color(if hot {
                    CLOSE_BORDER
                } else {
                    Color::TRANSPARENT
                })
                .text_color(if hot {
                    Color::WHITE
                } else {
                    scheme().text_muted
                });
            if card_ui.hovered() { s } else { s.hidden() }
        })
        .on_click(move |e| {
            e.stop_propagation();
            close();
        })
        .child("\u{00D7}");
    let title = div()
        .relative()
        .flex_none()
        .dyn_style(|s| s.h(theme().thumb_title_height))
        .child(
            div()
                .absolute()
                .left(0.0)
                .top(0.0)
                .bottom(0.0)
                .flex()
                .items_center()
                .overflow_clip()
                .whitespace_nowrap()
                .dyn_style(move |s| {
                    let t = theme();
                    s.right(if card_ui.hovered() {
                        CLOSE_WIDTH + 2.0
                    } else {
                        0.0
                    })
                    .font_family(font())
                    .text_size(t.font_size_small)
                    .text_color(scheme().text)
                })
                .child(text(move || {
                    now.get().map(|w| title_of(&w)).unwrap_or_default()
                })),
        )
        .child(close_button);
    let app_id = window.app_id.clone();
    let frame_bar = bar.clone();
    let frame = div()
        .relative()
        .flex_none()
        .border(1.0)
        .flex()
        .items_center()
        .justify_center()
        .interaction(frame_ui)
        .dyn_style(move |s| {
            let t = theme();
            let c = scheme();
            s.h(t.thumb_height)
                .bg(t.thumb_frame)
                .border_color(if frame_ui.hovered() {
                    c.selection_border
                } else {
                    c.popup_outline
                })
        })
        // The program's icon until the first picture, and when there is none: a capture that
        // never delivers one leaves no blank card.
        .child(
            shell_icon(icon_for(&app_id), CARD_ICON)
                .dyn_style(move |s| if has_picture.get() { s.hidden() } else { s }),
        )
        .children(picture)
        .on_click(move |_| {
            if let Some(w) = untrack(|| now.get()) {
                let _ = w.set_minimized(false);
                let _ = w.activate();
            }
            frame_bar.preview.set(None);
        })
        .on_aux_click(move |e| {
            if e.button == Some(PointerButton::Middle) {
                close();
                bar.preview.set(None);
            }
        });
    div()
        .flex_none()
        .flex()
        .flex_col()
        .interaction(card_ui)
        .dyn_style(|s| s.w(theme().thumb_width))
        .child(title)
        .child(frame)
}
