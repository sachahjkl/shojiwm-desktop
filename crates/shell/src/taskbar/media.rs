//! The player that plays now (QML `MediaControls`), for the volume popup: its cover, the title
//! and the artist, and the previous, play or pause, and next buttons. Any program with an MPRIS
//! interface counts (browsers, Spotify, mpv). Nothing shows while no program plays media.
//!
//! The buttons are the thumbnail toolbar's too ([`media_button`]).

use crate::theme::{button_radius, corner, scheme, theme};
use crate::ui::{dark_look, font, ink_asset, shell_icon, vertical_gradient};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;

/// The square behind the cover: white glass on dark colours, blue ink on light ones.
const COVER_BACK_DARK: Color = rgba(0xffffff33);
const COVER_BACK_LIGHT: Color = rgba(0x2c4a661a);
/// How dim a button that cannot be used is.
const UNAVAILABLE_OPACITY: f32 = 0.4;

/// The player shown: the one that plays, else the first.
fn player() -> Option<Player> {
    let all = media().players();
    all.iter()
        .find(|p| p.status == PlaybackStatus::Playing)
        .or(all.first())
        .cloned()
}

/// The name a player's program goes by on the bus (`org.mpris.MediaPlayer2.firefox.instance_1`
/// is `firefox`), in lower case.
pub(super) fn player_program(player: &Player) -> String {
    let rest = player
        .name
        .strip_prefix("org.mpris.MediaPlayer2.")
        .unwrap_or(&player.name);
    rest.split('.').next().unwrap_or(rest).to_lowercase()
}

/// The cover of what plays, as a file, when the player gives one.
fn cover_file(player: &Player) -> Option<std::path::PathBuf> {
    let url = player.art_url.as_deref()?;
    let path = url.strip_prefix("file://")?;
    Some(std::path::PathBuf::from(path))
}

/// The player now and what its controls do.
pub fn media_controls() -> Div {
    let now = memo(player);
    // The cover's file, else the player's program: the cover is made again only when it changes.
    let picture = memo(move || {
        now.with(|p| {
            let p = p.as_ref()?;
            Some(cover_file(p).ok_or_else(|| player_program(p)))
        })
    });
    let cover = div()
        .flex_none()
        .size(40.0)
        .border(1.0)
        .overflow_clip()
        .flex()
        .items_center()
        .justify_center()
        .dyn_style(|s| {
            let c = scheme();
            s.rounded(corner(2.0))
                .border_color(c.field_border)
                .bg(if dark_look() {
                    COVER_BACK_DARK
                } else {
                    COVER_BACK_LIGHT
                })
        })
        .child(dyn_view(move || {
            Some(match picture.get()? {
                Ok(file) => img(file)
                    .size(38.0)
                    .object_fit(ObjectFit::Cover)
                    .into_view(),
                // The program's icon, else a note: a worker asks the icon theme.
                Err(program) => {
                    let icon = crate::status::icon_or_signal(&program, "audio-x-generic");
                    shell_icon(move || icon.get(), 24.0).into_view()
                }
            })
        }));
    let words = div()
        .flex_1()
        .min_w(0.0)
        .flex()
        .flex_col()
        .justify_center()
        .gap(1.0)
        .child(
            div()
                .overflow_clip()
                .whitespace_nowrap()
                .font_weight(FontWeight::BOLD)
                .dyn_style(|s| {
                    s.font_family(font())
                        .text_size(theme().font_size)
                        .text_color(scheme().text)
                })
                .child(text(move || {
                    now.get()
                        .map(|p| {
                            if p.title.is_empty() {
                                p.identity
                            } else {
                                p.title
                            }
                        })
                        .unwrap_or_default()
                })),
        )
        .child(
            div()
                .overflow_clip()
                .whitespace_nowrap()
                .dyn_style(|s| {
                    s.font_family(font())
                        .text_size(theme().font_size_small)
                        .text_color(scheme().text_muted)
                })
                .child(text(move || {
                    now.get()
                        .map(|p| {
                            if p.artists.is_empty() {
                                p.identity
                            } else {
                                p.artists.join(", ")
                            }
                        })
                        .unwrap_or_default()
                })),
        );
    div()
        .flex()
        .flex_col()
        .gap(6.0)
        .dyn_style(move |s| {
            if now.with(Option::is_none) {
                s.hidden()
            } else {
                s
            }
        })
        .child(div().flex().flex_row().gap(8.0).child(cover).child(words))
        .child(transport(move || now.get()))
        .child(div().h(1.0).dyn_style(|s| s.bg(scheme().menu_divider)))
}

/// Previous, play or pause, and next, for the player `player` gives.
pub(super) fn transport(player: impl Fn() -> Option<Player> + Clone + 'static) -> Div {
    let (a, b, c) = (player.clone(), player.clone(), player.clone());
    let (d, e, f) = (player.clone(), player.clone(), player);
    div()
        .flex()
        .flex_row()
        .justify_center()
        .gap(6.0)
        .child(media_button(
            || "previous",
            move || a().is_some_and(|p| p.can_go_previous),
            move || {
                if let Some(p) = b() {
                    let _ = p.previous();
                }
            },
        ))
        .child(media_button(
            {
                let c = c.clone();
                move || {
                    if c().is_some_and(|p| p.status == PlaybackStatus::Playing) {
                        "pause"
                    } else {
                        "play"
                    }
                }
            },
            move || c().is_some_and(|p| p.can_play || p.can_pause),
            move || {
                if let Some(p) = d() {
                    let _ = p.play_pause();
                }
            },
        ))
        .child(media_button(
            || "next",
            move || e().is_some_and(|p| p.can_go_next),
            move || {
                if let Some(p) = f() {
                    let _ = p.next();
                }
            },
        ))
}

/// A small push button with a media glyph (`previous`, `play`, `pause`, `next`): the scheme's
/// button gradient, lit under the pointer, dim when it cannot be used.
pub(super) fn media_button(
    glyph: impl Fn() -> &'static str + 'static,
    available: impl Fn() -> bool + 'static,
    run: impl Fn() + 'static,
) -> Div {
    let ui = Interaction::new();
    let available = std::rc::Rc::new(available);
    let can = available.clone();
    let style_can = available.clone();
    div()
        .flex_none()
        .w(34.0)
        .h(26.0)
        .border(1.0)
        .flex()
        .items_center()
        .justify_center()
        .interaction(ui)
        .disabled(move || !can())
        .fill(move || {
            let c = scheme();
            let (top, bottom) = if ui.pressed() {
                (c.button_pressed_top, c.button_pressed_bottom)
            } else if ui.hovered() {
                (c.button_hot_top, c.button_hot_bottom)
            } else {
                (c.button_top, c.button_bottom)
            };
            vertical_gradient(&[(0.0, top), (1.0, bottom)])
        })
        .dyn_style(move |s| {
            let c = scheme();
            s.rounded(button_radius())
                .border_color(if ui.hovered() {
                    c.selection_border
                } else {
                    c.field_border
                })
                .opacity(if style_can() {
                    1.0
                } else {
                    UNAVAILABLE_OPACITY
                })
        })
        .on_click(move |_| {
            if available() {
                run();
            }
        })
        .child(
            img(move || ink_asset(&format!("media-{}.svg", glyph())))
                .size(14.0)
                .object_fit(ObjectFit::Contain),
        )
}
