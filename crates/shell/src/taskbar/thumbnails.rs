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
use crate::settings::motion;
use crate::theme::{bar_at_top, corner, scheme, theme};
use crate::ui::{font, popup_frame, shell_icon, vertical_gradient};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use serde_json::json;
use std::cell::RefCell;
use std::collections::HashMap;
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
/// How long a closed window's card takes to fade and fold, while the others close up and the
/// layer narrows around them.
const FOLD: f32 = 220.0;

fn fold() -> Transition {
    Transition::ms(motion(FOLD)).ease_out()
}

thread_local! {
    /// The `ext_foreign_toplevel_list` identifier of the window the compositor highlights.
    static HIGHLIGHTED: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Asks the compositor to single out a window on the screen while the pointer rests on its
/// card, or none. The compositor finds the window by its `ext_foreign_toplevel_list`
/// identifier: the one name both sides know for the window's whole life.
fn highlight(identifier: Option<String>) {
    let changed = HIGHLIGHTED.with(|h| {
        let mut h = h.borrow_mut();
        let changed = *h != identifier;
        *h = identifier.clone();
        changed
    });
    if changed {
        crate::compositor::send("highlight", json!({ "window": identifier }));
    }
}

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

/// The width of the frame around `count` cards (a fraction while one folds).
fn frame_width(count: f32) -> f32 {
    let t = theme();
    let n = count.max(1.0);
    t.thumb_padding * 2.0 + n * t.thumb_width + (n - 1.0) * t.thumb_gap
}

/// Where the frame around `count` cards starts on the screen: centred on the button, kept on
/// the screen.
fn frame_left(bar: &Bar, count: f32) -> f32 {
    let width = frame_width(count);
    let screen = bar
        .output
        .current()
        .map_or(bar.output.logical_size.width, |o| o.logical_size.width) as f32;
    let center = bar.preview.with(|p| p.as_ref().map_or(0.0, |p| p.center));
    (center - width / 2.0).clamp(0.0, (screen - width).max(0.0))
}

/// How many cards the frame is drawn for, and how many the layer has room for.
///
/// The layer never narrows while the thumbnails are shown: the compositor sizes it a frame or
/// more after it is asked to, and a frame folding inside a layer that narrows late would slide
/// aside. So the layer keeps the room of the most cards it has had, and the frame folds inside
/// it, where the input region follows it.
#[derive(Clone, Copy)]
struct Place {
    count: Animated<f32>,
    room: Signal<f32>,
}

impl Place {
    /// Where the frame is in the layer.
    fn offset(&self, bar: &Bar) -> f32 {
        frame_left(bar, self.count.get()) - frame_left(bar, self.room.get())
    }
}

/// Where the layer goes: around the frame of `count` cards, away from the bar.
fn margin(bar: &Bar, count: f32) -> Margin {
    let t = theme();
    let left = frame_left(bar, count).round() as i32;
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
    // Before the layer's effect: the layer opens with the room this gives.
    let place = Place {
        count: animated(0.0_f32),
        room: signal(0.0_f32),
    };
    let shown_key: Rc<RefCell<Option<String>>> = Rc::default();
    {
        let bar = bar.clone();
        effect(move || {
            let key = bar.preview.with(|p| p.as_ref().map(|p| p.key.clone()));
            let n = shown_windows(&bar).len() as f32;
            untrack(|| {
                let same = *shown_key.borrow() == key;
                if same && n < place.count.get_untracked() {
                    // A window closed: its card folds, the layer keeps its room.
                    place.count.to(n, fold());
                } else if same && n <= place.room.get_untracked() {
                    place.count.set(n);
                } else {
                    place.count.set(n);
                    place.room.set(n);
                }
                *shown_key.borrow_mut() = key;
            });
        });
    }
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
                    // The pointer may never have left a card.
                    highlight(None);
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
                    .margin(margin(&bar, place.room.get_untracked()))
                    .output(&bar.output)
                    .namespace("shojiwm-desktop-thumbnails");
                let shown = bar.clone();
                *handle.borrow_mut() = Some(open_layer(options, move || view(shown, place)));
            });
        });
    }
    effect(move || {
        let at = margin(&bar, place.room.get());
        // The rest of the layer lets the pointer through to what is under it.
        let input = InputRegion::Rects(vec![RectI {
            x: place.offset(&bar).floor() as i32,
            y: 0,
            width: frame_width(place.count.get()).ceil() as i32 + 1,
            height: i32::from(u16::MAX),
        }]);
        if let Some(layer) = handle.borrow().as_ref() {
            layer.set_margin(at);
            layer.set_input_region(input);
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

fn view(bar: Bar, place: Place) -> Div {
    let windows = {
        let bar = bar.clone();
        memo(move || shown_windows(&bar))
    };
    // The last window of the group closed: nothing more to show.
    {
        let bar = bar.clone();
        effect(move || {
            if windows.with(Vec::is_empty) {
                // After the last card has folded, and not while this layer is being worked on:
                // the change closes it.
                // The bar's own state, not this layer's: the layer may be gone by then.
                let bar = bar.clone();
                set_timeout(Duration::from_millis(motion(FOLD)), move || {
                    if untrack(|| shown_windows(&bar).is_empty()) {
                        bar.preview.set(None);
                    }
                });
            }
        });
    }
    let player = memo(move || windows.with(|w| player_of(w)));
    let hover = bar.clone();
    let cards = bar.clone();
    let room = bar.clone();
    let frame = popup_frame()
        .items_center()
        // Each card carries half the gap on either side, so that a folding card takes its gap
        // with it: the padding gives that half back at the ends.
        .dyn_style(|s| {
            let t = theme();
            s.py(t.thumb_padding)
                .px(t.thumb_padding - t.thumb_gap / 2.0)
        })
        .on_hover(move |inside| {
            hover.preview_inside.set(inside);
            if !inside {
                hover.hide_preview_soon();
            }
        })
        .child(
            div().flex().flex_row().child(
                for_each(
                    move || windows.get(),
                    |t| t.id(),
                    move |t| card(cards.clone(), t),
                )
                .exit(|s| s.opacity(0.0).w(0.0))
                .transition(fold()),
            ),
        )
        .child(show(
            move || player.with(Option::is_some),
            move || div().mt(TOOLBAR_GAP).child(transport(move || player.get())),
        ));
    // The layer's room, with the frame where the cards it shows are centred.
    div()
        .flex()
        .flex_col()
        .items_start()
        .dyn_style(move |s| s.w(frame_width(place.room.get())))
        .child(frame.dyn_style(move |s| s.ml(place.offset(&room))))
}

thread_local! {
    static WINDOWS_BY_ID: std::cell::Cell<Option<Memo<HashMap<u64, Toplevel>>>> =
        const { std::cell::Cell::new(None) };
}

/// Every window by its id, made once for all the cards: a card finds its own without a pass
/// over the list. Reading it tracks it.
fn windows_by_id() -> Memo<HashMap<u64, Toplevel>> {
    WINDOWS_BY_ID.with(|cell| {
        cell.get().unwrap_or_else(|| {
            let (m, _) = untrack(|| {
                create_root(|| {
                    memo(|| {
                        toplevels()
                            .all()
                            .into_iter()
                            .map(|t| (t.id(), t))
                            .collect::<HashMap<_, _>>()
                    })
                })
            });
            cell.set(Some(m));
            m
        })
    })
}

/// One window: its title and close button, and its picture, which shows it on a click.
fn card(bar: Bar, window: Toplevel) -> Div {
    let id = window.id();
    let by_id = windows_by_id();
    let now = memo(move || by_id.with(|all| all.get(&id).cloned()));
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
        // The thumbnails stay, as Windows 7's do: the card folds and the others close up.
        .on_aux_click(move |e| {
            if e.button == Some(PointerButton::Middle) {
                close();
            }
        });
    // The window stands out on the screen while the pointer is on its card; a card that goes
    // away (its window closed, another button's thumbnails) takes the highlight with it.
    let first = window.identifier.clone();
    // The identifier this card asked the compositor to highlight.
    let asked: Rc<RefCell<Option<String>>> = Rc::default();
    // Only its own: the next card may already have taken the highlight.
    let release = {
        let asked = asked.clone();
        move || {
            let id = asked.borrow_mut().take();
            if id.is_some() && HIGHLIGHTED.with(|h| *h.borrow() == id) {
                highlight(None);
            }
        }
    };
    on_cleanup(release.clone());
    // The slot: the card and half the gap on either side. It folds to nothing when the window
    // closes, clipping the card, which keeps its own width.
    div()
        .flex_none()
        .flex()
        .flex_row()
        .overflow_clip()
        .dyn_style(|s| {
            let t = theme();
            s.w(t.thumb_width + t.thumb_gap)
        })
        .child(card_body(card_ui, first, asked, release, now, title, frame))
}

/// The card itself, in its slot.
fn card_body(
    card_ui: Interaction,
    first: Option<String>,
    asked: Rc<RefCell<Option<String>>>,
    release: impl Fn() + 'static,
    now: Memo<Option<Toplevel>>,
    title: Div,
    frame: Div,
) -> Div {
    div()
        .flex_none()
        .flex()
        .flex_col()
        .interaction(card_ui)
        .on_hover(move |inside| {
            if inside {
                let id = untrack(|| now.get())
                    .and_then(|w| w.identifier)
                    .or_else(|| first.clone());
                *asked.borrow_mut() = id.clone();
                highlight(id);
            } else {
                release();
            }
        })
        .dyn_style(|s| {
            let t = theme();
            s.w(t.thumb_width).ml(t.thumb_gap / 2.0)
        })
        .child(title)
        .child(frame)
}
