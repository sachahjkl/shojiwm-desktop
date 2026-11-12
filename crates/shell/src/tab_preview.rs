//! The preview of a tab: while the pointer rests on a tab of a window group, the window behind it
//! shows small and live beside the tab, under it when there is room on the screen, else over it.
//! The compositor draws the tabs and tells where the pointer rests, over IPC:
//!
//!   shojiwm-shell msg tabpreview show <screen> <identifier> <x> <y> <width> <height>
//!   shojiwm-shell msg tabpreview hide
//!
//! The rectangle is the tab's, in logical pixels from the screen's top-left; the window is named
//! by its `ext_foreign_toplevel_list` identifier.

use crate::theme::theme;
use crate::ui::{font, popup_frame};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::Duration;

/// The picture's room, the frame's padding round it, and the title's height over it.
const PICTURE_WIDTH: f32 = 260.0;
const PICTURE_HEIGHT: f32 = 160.0;
const PADDING: f32 = 6.0;
const TITLE_HEIGHT: f32 = 20.0;
/// The room between the tab and the preview.
const GAP: f32 = 6.0;
/// How long the pointer rests on a tab before its preview shows.
const SHOW_AFTER: Duration = Duration::from_millis(400);
/// How often the live picture is taken again.
const LIVE_EVERY: Duration = Duration::from_millis(250);

#[derive(Clone, Debug, PartialEq)]
struct Request {
    screen: String,
    identifier: String,
    /// The tab: x, y, width, height.
    tab: (f32, f32, f32, f32),
}

static WANTED: OnceLock<SharedSignal<Option<Request>>> = OnceLock::new();

fn wanted() -> &'static SharedSignal<Option<Request>> {
    WANTED.get_or_init(|| shared_signal(None))
}

/// The whole size of the preview's frame.
fn card_size() -> (f32, f32) {
    (
        PICTURE_WIDTH + 2.0 * PADDING + 2.0,
        TITLE_HEIGHT + PICTURE_HEIGHT + 2.0 * PADDING + 2.0,
    )
}

/// Where the preview goes on a screen `screen` big: centred under the tab when it fits there,
/// else over it; kept on the screen.
fn place(tab: (f32, f32, f32, f32), card: (f32, f32), screen: (f32, f32)) -> (f32, f32) {
    let (x, y, width, height) = tab;
    let left = (x + width / 2.0 - card.0 / 2.0).clamp(0.0, (screen.0 - card.0).max(0.0));
    let below = y + height + GAP;
    let top = if below + card.1 <= screen.1 {
        below
    } else {
        (y - GAP - card.1).max(0.0)
    };
    (left.round(), top.round())
}

/// Answers the IPC target `tabpreview` and opens the preview's layer on the screen it is asked
/// for. Call once, from the root.
pub fn install() {
    crate::ipc::handle("tabpreview", |function, args| {
        match function {
            "show" => {
                let number =
                    |i: usize| -> f32 { args.get(i).and_then(|v| v.parse().ok()).unwrap_or(0.0) };
                let (Some(screen), Some(identifier)) = (args.first(), args.get(1)) else {
                    return Err("show <screen> <identifier> <x> <y> <width> <height>".into());
                };
                wanted().set(Some(Request {
                    screen: screen.clone(),
                    identifier: identifier.clone(),
                    tab: (number(2), number(3), number(4), number(5)),
                }));
            }
            "hide" => wanted().set(None),
            _ => return Err(format!("no function {function}")),
        }
        Ok(String::new())
    });
    for_each_output(|output| {
        let handle: Rc<RefCell<Option<LayerHandle>>> = Rc::default();
        let timer: Rc<RefCell<Option<Timeout>>> = Rc::default();
        let name = output.name.clone();
        effect(move || {
            let request = wanted().get().filter(|r| r.screen == name);
            untrack(|| {
                if let Some(t) = timer.borrow_mut().take() {
                    t.cancel();
                }
                if let Some(old) = handle.borrow_mut().take() {
                    old.close();
                }
                let Some(request) = request else {
                    return;
                };
                let output = output.clone();
                let handle = handle.clone();
                *timer.borrow_mut() = Some(set_timeout(SHOW_AFTER, move || {
                    let screen = output
                        .current()
                        .map_or(output.logical_size, |o| o.logical_size);
                    let card = card_size();
                    let (left, top) = place(
                        request.tab,
                        card,
                        (screen.width as f32, screen.height as f32),
                    );
                    let options = LayerOptions::new()
                        .layer(Layer::Overlay)
                        .anchor(Anchor::TOP | Anchor::LEFT)
                        .exclusive(Exclusive::Ignore)
                        .keyboard(Keyboard::None)
                        .input_region(InputRegion::Empty)
                        .fit_content()
                        .margin(Margin {
                            top: top as i32,
                            left: left as i32,
                            ..Default::default()
                        })
                        .output(&output)
                        .namespace("shojiwm-desktop-tab-preview");
                    let scale = output.scale.max(1.0);
                    let identifier = request.identifier.clone();
                    *handle.borrow_mut() =
                        Some(open_layer(options, move || view(&identifier, scale)));
                }));
            });
        });
    });
}

/// The preview of the window named `identifier`: its title over its live picture.
fn view(identifier: &str, scale: f32) -> Div {
    let window = toplevels()
        .all()
        .into_iter()
        .find(|t| t.identifier.as_deref() == Some(identifier));
    let title = window.as_ref().map(|t| t.title.clone()).unwrap_or_default();
    let picture = window.map(|t| {
        let shot = capture(
            &t,
            CaptureOptions::default()
                .max_size(
                    (PICTURE_WIDTH * scale).ceil() as u32,
                    (PICTURE_HEIGHT * scale).ceil() as u32,
                )
                .every(LIVE_EVERY),
        );
        bitmap_view(move || shot.picture())
            .w(PICTURE_WIDTH)
            .h(PICTURE_HEIGHT)
            .object_fit(ObjectFit::Contain)
    });
    popup_frame()
        .p(PADDING)
        .child(
            div()
                .h(TITLE_HEIGHT)
                .w(PICTURE_WIDTH)
                .flex()
                .items_center()
                .overflow_clip()
                .whitespace_nowrap()
                .dyn_style(|s| s.font_family(font()).text_size(theme().font_size))
                .child(text(title)),
        )
        .children(picture)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: (f32, f32) = (1920.0, 1080.0);
    const CARD: (f32, f32) = (274.0, 194.0);

    #[test]
    fn a_tab_with_room_under_it_has_its_preview_under_it() {
        assert_eq!(
            place((400.0, 100.0, 160.0, 28.0), CARD, SCREEN),
            (343.0, 134.0)
        );
    }

    #[test]
    fn a_tab_low_on_the_screen_has_its_preview_over_it() {
        assert_eq!(
            place((400.0, 1000.0, 160.0, 28.0), CARD, SCREEN),
            (343.0, 800.0)
        );
    }

    #[test]
    fn the_preview_stays_on_the_screen() {
        assert_eq!(place((0.0, 100.0, 40.0, 28.0), CARD, SCREEN).0, 0.0);
        assert_eq!(place((1900.0, 100.0, 40.0, 28.0), CARD, SCREEN).0, 1646.0);
    }
}
