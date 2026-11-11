//! The first pages of the Control Panel: light, dark and glass (with the theme), the desktop
//! background (with the slide show), the lock screen, the account picture, the taskbar and the
//! Start menu, and the icons of the notification area.

use super::picture_chooser::{built_in_wallpaper, picture_chooser, user_picture_source};
use super::{heading, note, sub_heading};
use crate::paths::asset;
use crate::settings::{Settings, change, dark, set_dark, settings};
use crate::theme::{corner, field_radius, is_kernel, parse_hex, scheme, theme};
use crate::ui::{
    cp_button, cp_check, cp_combo, cp_link, cp_radio, font, label_text, shell_icon,
    vertical_gradient,
};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::{TrayItem, tray};
use std::rc::Rc;

// ----- helpers ---------------------------------------------------------------------------------

/// A setting as a binding for a control: `get` reads it, `set` writes it.
fn setting<T: 'static>(get: fn(&Settings) -> T, set: fn(&mut Settings, T)) -> Binding<T> {
    Binding::new(move || settings().with(get), move |value| change(|s| set(s, value)))
}

/// A setting that takes one of `values`, as the index of a combo box. A value that is not in
/// the list reads as the first.
fn choice<T: PartialEq + Clone + 'static>(
    values: Vec<T>,
    get: fn(&Settings) -> T,
    set: fn(&mut Settings, T),
) -> Binding<usize> {
    let values = Rc::new(values);
    let known = values.clone();
    Binding::new(
        move || {
            let now = settings().with(get);
            known.iter().position(|v| *v == now).unwrap_or(0)
        },
        move |index| {
            if let Some(value) = values.get(index).cloned() {
                change(|s| set(s, value));
            }
        },
    )
}

fn labels(list: &[&str]) -> Vec<String> {
    list.iter().map(|l| (*l).to_owned()).collect()
}

/// A page: its parts one under the other.
fn page(gap: f32) -> Div {
    div().flex().flex_col().items_start().gap(gap)
}

/// Controls side by side, centred on their middle line.
fn row(gap: f32) -> Div {
    div().flex().flex_row().items_center().gap(gap)
}

/// Controls side by side that wrap to the next line when the page is narrow.
fn flow(gap: f32) -> Div {
    row(gap).flex_wrap().self_stretch()
}

/// A note as wide as the page, which wraps.
fn wide_note(label: impl Into<String>) -> Div {
    note(label).self_stretch()
}

/// A plain text field bound to `value`: its border in the focus colour while it has the
/// keyboard.
fn field(value: Signal<String>, width: f32) -> (Div, TextInput) {
    let focused = signal(false);
    let input = text_input()
        .value(value)
        .flex_1()
        .min_w(0.0)
        .on_focus_change(move |on| focused.set(on))
        .dyn_style(|s| {
            let c = scheme();
            s.font_family(font())
                .text_size(theme().font_size)
                .text_color(c.text)
                .caret_color(c.text)
        });
    let frame = div()
        .flex_none()
        .w(width)
        .h(24.0)
        .flex()
        .flex_row()
        .items_center()
        .px(6.0)
        .border(1.0)
        .dyn_style(move |s| {
            let c = scheme();
            s.rounded(field_radius()).bg(c.field).border_color(if focused.get() {
                c.field_focus
            } else {
                c.field_border
            })
        });
    (frame, input)
}

/// How far the pointer can be beside a slider's track and still move it.
const SLIDER_REACH: f32 = 6.0;
/// The height of a slider, and of its knob.
const SLIDER_HEIGHT: f32 = 24.0;
const KNOB: (f32, f32) = (10.0, 20.0);

/// A track `width` wide with a knob on it at `fraction` (0 to 1); pressing or dragging along it
/// calls `set` with the fraction under the pointer.
fn slider(
    width: f32,
    fraction: impl Fn() -> f32 + 'static,
    set: impl Fn(f32) + 'static,
) -> Div {
    let ui = Interaction::new();
    let held = signal(false);
    let lit = move || ui.hovered() || held.get();
    let track = div()
        .absolute()
        .left(SLIDER_REACH)
        .top((SLIDER_HEIGHT - 4.0) / 2.0)
        .w(width)
        .h(4.0)
        .border(1.0)
        .dyn_style(|s| {
            let c = scheme();
            s.rounded(corner(2.0)).bg(c.field).border_color(c.field_border)
        });
    let knob = div()
        .absolute()
        .top((SLIDER_HEIGHT - KNOB.1) / 2.0)
        .w(KNOB.0)
        .h(KNOB.1)
        .border(1.0)
        .fill(move || {
            let c = scheme();
            let [top, bottom] = if ui.hovered() {
                [c.button_hot_top, c.button_hot_bottom]
            } else {
                [c.button_top, c.button_bottom]
            };
            vertical_gradient(&[(0.0, top), (1.0, bottom)])
        })
        .dyn_style(move |s| {
            let c = scheme();
            let at = fraction().clamp(0.0, 1.0) * width;
            s.left(SLIDER_REACH + at - KNOB.0 / 2.0)
                .rounded(corner(2.0))
                .border_color(if lit() { c.field_focus } else { c.field_border })
        });
    div()
        .relative()
        .flex_none()
        .w(width + 2.0 * SLIDER_REACH)
        .h(SLIDER_HEIGHT)
        .select_none()
        .interaction(ui)
        .grab_threshold(0.0)
        .on_grab(move |e| match e.phase {
            GrabPhase::Start | GrabPhase::Move => {
                held.set(true);
                let x = e.local.x / window_scale().as_f32() - SLIDER_REACH;
                set((x / width).clamp(0.0, 1.0));
            }
            GrabPhase::End | GrabPhase::Cancel => held.set(false),
        })
        .child(track)
        .child(knob)
}

// ----- light, dark and glass -------------------------------------------------------------------

/// The small window each light or dark choice shows: its outline, its caption's gradient, its
/// page and its lines of text, light then dark.
const PREVIEW_OUTLINE: Color = rgb(0x5d7489);
const PREVIEW_CAPTION: [Color; 2] = [rgb(0xa6c8e6), rgb(0x6f9cc4)];
const PREVIEW_PAGE: [Color; 2] = [rgb(0xffffff), rgb(0x1e262e)];
const PREVIEW_LINES: [Color; 2] = [rgb(0xc3d3e0), rgb(0x5d7489)];
/// A warning that the choice reaches no program.
const ERROR_TEXT: Color = rgb(0xc42b1c);
/// The taskbar's glass under each colour swatch, as the swatches of Windows 7 show it.
const SWATCH_GLASS: [Color; 2] = [rgb(0x7fa3c4), rgb(0x1d3550)];
/// How strongly a swatch shows its colour over the glass.
const SWATCH_TINT: f32 = 0.75;
const SWATCH_SIZE: f32 = 40.0;

/// The colours of the glass, by value ("" for the plain glass) and name.
const GLASS_COLOURS: &[(&str, &str)] = &[
    ("", "Plain glass"),
    ("#3b8fd6", "Sky"),
    ("#1f5aa8", "Twilight"),
    ("#1aa2a8", "Sea"),
    ("#4fa83d", "Leaf"),
    ("#9cc93a", "Lime"),
    ("#e8c22f", "Sun"),
    ("#e8892f", "Pumpkin"),
    ("#e0607e", "Blush"),
    ("#a64fb3", "Violet"),
    ("#b49bd9", "Lavender"),
    ("#a69b85", "Taupe"),
    ("#6b4a2f", "Chocolate"),
    ("#6b7b8c", "Slate"),
    ("#e6eef2", "Frost"),
];

/// The levels of blur behind the glass.
const BLUR_LEVELS: &[(i64, &str)] = &[
    (0, "Clear"),
    (1, "Light"),
    (2, "Medium"),
    (3, "Strong"),
    (4, "Frosted"),
];

/// The range of the colour's intensity.
const STRENGTH_LOW: f64 = 0.1;
const STRENGTH_HIGH: f64 = 0.9;

const DCONF_SCHEME_KEY: &str = "/org/gnome/desktop/interface/color-scheme";

thread_local! {
    static SCHEME_REACHABLE: std::cell::Cell<Option<Signal<bool>>> =
        const { std::cell::Cell::new(None) };
}

/// Whether dconf answers, so that programs see the light or dark choice. Asked once; true
/// until it has answered. Reading it tracks it.
fn system_scheme_available() -> bool {
    SCHEME_REACHABLE.with(|cell| {
        cell.get()
            .unwrap_or_else(|| {
                let (reachable, _) = create_root(|| {
                    let reachable = signal(true);
                    let read = anti_webui::shell::prelude::Process::new("dconf")
                        .args(["read", DCONF_SCHEME_KEY])
                        .spawn();
                    read.on_exit(move |status| reachable.set(status == Some(0)));
                    reachable
                });
                cell.set(Some(reachable));
                reachable
            })
            .get()
    })
}

/// "Choose light or dark", the theme, the corners and the glass.
pub fn appearance_page() -> Div {
    let theme_choice = setting(|s| s.theme.clone(), |s, v| s.theme = v);
    let kernel_theme = theme_choice.clone();
    let corners = setting(|s| s.square_corners, |s, v| s.square_corners = v);
    let square = corners.clone();
    page(12.0)
        .child(heading("Choose light or dark"))
        .child(wide_note(
            "This is the system's own setting: programs that follow it (GTK, libadwaita, Qt, \
             Firefox, Chromium) change with it, and so do the menus of this desktop. The \
             taskbar and the window frames stay dark glass, as on Windows 7.",
        ))
        .child(
            row(18.0)
                .child(scheme_choice(false, "Light"))
                .child(scheme_choice(true, "Dark")),
        )
        .child(show(
            || !system_scheme_available(),
            || {
                wide_note(
                    "dconf could not be reached, so programs will not see this choice; only \
                     this desktop will, until it restarts. On NixOS: programs.dconf.enable = \
                     true (the shojiwm module of nixconfig sets it).",
                )
                .dyn_style(|s| s.text_color(ERROR_TEXT))
            },
        ))
        .child(sub_heading("Theme"))
        .child(wide_note(
            "How the window frames and this desktop's own surfaces are drawn. Aero: Windows \
             7's glass. Kernel: a deep blue-black desk, frames with a steel-blue caption plate \
             and a dark outline.",
        ))
        .child(
            flow(16.0)
                .child(cp_radio("Aero", theme_choice, "aero".to_owned()))
                .child(cp_radio("Kernel", kernel_theme, "kernel".to_owned())),
        )
        .child(show(
            || !is_kernel(),
            move || {
                page(12.0)
                    .self_stretch()
                    .child(sub_heading("Corners"))
                    .child(wide_note(
                        "The shape of the windows, menus, buttons and fields. What is round by \
                         nature, as the Start button, stays round.",
                    ))
                    .child(
                        flow(16.0)
                            .child(cp_radio("Rounded", corners.clone(), false))
                            .child(cp_radio("Square", square.clone(), true)),
                    )
            },
        ))
        .child(sub_heading("Glass"))
        .child(wide_note(
            "How much the caption bars, the taskbar and the menus blur what is behind them. \
             More blur costs more of the graphics card.",
        ))
        .child(flow(16.0).children(BLUR_LEVELS.iter().map(|&(level, label)| {
            cp_radio(label, setting(|s| s.glass_blur, |s, v| s.glass_blur = v), level)
        })))
        .child(wide_note("The colour of the taskbar's and the Start menu's glass:"))
        .child(flow(8.0).children(GLASS_COLOURS.iter().map(|&(value, name)| swatch(value, name))))
        .child(label_text(|| {
            let now = settings().with(|s| s.glass_color.clone());
            let name = GLASS_COLOURS
                .iter()
                .find(|(value, _)| *value == now)
                .map_or(now.clone(), |(_, name)| (*name).to_owned());
            format!("Colour: {name}")
        }))
        .child(show(
            || settings().with(|s| !s.glass_color.is_empty()),
            || {
                row(12.0).child(note("Colour intensity:")).child(slider(
                    220.0,
                    || {
                        let strength = settings().with(|s| s.glass_color_strength);
                        ((strength - STRENGTH_LOW) / (STRENGTH_HIGH - STRENGTH_LOW)) as f32
                    },
                    |fraction| {
                        let strength =
                            STRENGTH_LOW + fraction as f64 * (STRENGTH_HIGH - STRENGTH_LOW);
                        let strength = (strength * 100.0).round() / 100.0;
                        change(|s| s.glass_color_strength = strength);
                    },
                ))
            },
        ))
}

/// A light or dark choice: a small window in that scheme over its radio button. A click
/// anywhere on it chooses it.
fn scheme_choice(is_dark: bool, label: &'static str) -> Div {
    let ui = Interaction::new();
    let chosen = move || dark() == is_dark;
    let which = usize::from(is_dark);
    let lines = (0..4).map(move |i| {
        div()
            .flex_none()
            .w(120.0 - i as f32 * 18.0)
            .h(6.0)
            .bg(PREVIEW_LINES[which])
            .dyn_style(|s| s.rounded(corner(3.0)))
    });
    let window = div()
        .absolute()
        .left(16.0)
        .top(12.0)
        .w(200.0 - 32.0)
        .h(96.0)
        .border(1.0)
        .border_color(PREVIEW_OUTLINE)
        .bg(PREVIEW_PAGE[which])
        .overflow_clip()
        .dyn_style(|s| s.rounded(corner(4.0)))
        .child(
            div()
                .absolute()
                .left(0.0)
                .top(0.0)
                .right(0.0)
                .h(18.0)
                .fill(|| {
                    vertical_gradient(&[(0.0, PREVIEW_CAPTION[0]), (1.0, PREVIEW_CAPTION[1])])
                })
                .dyn_style(|s| s.rounded(corner(4.0))),
        )
        .child(
            div()
                .absolute()
                .left(10.0)
                .top(28.0)
                .flex()
                .flex_col()
                .gap(6.0)
                .children(lines),
        );
    let radio = div()
        .absolute()
        .left(0.0)
        .right(0.0)
        .bottom(10.0)
        .flex()
        .justify_center()
        .child(
            cp_radio(label, Binding::new(dark, set_dark), is_dark)
                // The box below chooses it too: once is enough.
                .on_click(|e| e.stop_propagation()),
        );
    div()
        .relative()
        .flex_none()
        .w(200.0)
        .h(150.0)
        .border(1.0)
        .interaction(ui)
        .on_click(move |_| set_dark(is_dark))
        .dyn_style(move |s| {
            let c = scheme();
            let s = s.rounded(corner(4.0));
            if chosen() {
                s.bg(c.selection).border_color(c.selection_border)
            } else if ui.hovered() {
                s.bg(c.hover).border_color(c.selection_border)
            } else {
                s.bg(Color::TRANSPARENT).border_color(Color::TRANSPARENT)
            }
        })
        .child(window)
        .child(radio)
}

/// A colour of the glass: the colour over the taskbar's glass. A click chooses it.
fn swatch(value: &'static str, name: &'static str) -> Div {
    let ui = Interaction::new();
    let chosen = move || settings().with(|s| s.glass_color == value);
    let tint = div()
        .absolute()
        .inset(2.0)
        .opacity(SWATCH_TINT)
        .bg(parse_hex(value).unwrap_or(Color::TRANSPARENT))
        .dyn_style(|s| s.rounded(corner(2.0)));
    div()
        .relative()
        .flex_none()
        .size(SWATCH_SIZE)
        .interaction(ui)
        .access(move |a| a.role(Role::Radio).label(name).checked(chosen()))
        .on_click(move |_| change(|s| s.glass_color = value.to_owned()))
        .fill(|| vertical_gradient(&[(0.0, SWATCH_GLASS[0]), (1.0, SWATCH_GLASS[1])]))
        .dyn_style(move |s| {
            let c = scheme();
            let s = s.rounded(corner(4.0));
            if chosen() {
                s.border(2.0).border_color(c.field_focus)
            } else {
                s.border(1.0).border_color(if ui.hovered() {
                    c.selection_border
                } else {
                    c.field_border
                })
            }
        })
        .child(tint)
}

// ----- desktop background ----------------------------------------------------------------------

const POSITIONS: &[&str] = &["fill", "fit", "stretch", "tile", "center"];
const POSITION_LABELS: &[&str] = &["Fill", "Fit", "Stretch", "Tile", "Center"];
const SLIDESHOW_MINUTES: &[i64] = &[1, 10, 30, 60, 360, 1440];
const SLIDESHOW_LABELS: &[&str] =
    &["1 minute", "10 minutes", "30 minutes", "1 hour", "6 hours", "1 day"];
/// The width of the slide show's folder field.
const FOLDER_FIELD_WIDTH: f32 = 320.0;
/// The height of each line of the slide show's settings.
const LINE_HEIGHT: f32 = 24.0;

/// "Choose your desktop background", how it fills the screen, and the slide show.
pub fn background_page() -> Div {
    page(12.0)
        .child(heading("Choose your desktop background"))
        .child(wide_note("Click a picture to make it your desktop background."))
        .child(
            picture_chooser(
                || settings().with(|s| s.wallpaper.clone()),
                built_in_wallpaper,
                "Windows 7 style (built in)",
                false,
                |path| change(|s| s.wallpaper = path),
            )
            .self_stretch(),
        )
        .child(
            row(12.0).child(note("Picture position:")).child(
                cp_combo(
                    labels(POSITION_LABELS),
                    choice(
                        POSITIONS.iter().map(|p| (*p).to_owned()).collect::<Vec<String>>(),
                        |s| s.wallpaper_position.clone(),
                        |s, v| s.wallpaper_position = v,
                    ),
                )
                .w(160.0),
            ),
        )
        .child(sub_heading("Slide show"))
        .child(cp_check(
            "Change the picture on its own, from a folder",
            Binding::new(
                || settings().with(|s| !s.slideshow_folder.is_empty()),
                |on| {
                    let folder = if on {
                        crate::apps::user_dir("PICTURES", "Pictures")
                    } else {
                        String::new()
                    };
                    change(|s| s.slideshow_folder = folder);
                },
            ),
        ))
        .child(show(
            || settings().with(|s| !s.slideshow_folder.is_empty()),
            slideshow_settings,
        ))
}

/// The folder, how often the picture changes, the order, and a button for the next picture:
/// labels on the left, controls on the right.
fn slideshow_settings() -> Div {
    let folder = signal(untrack(|| settings().with(|s| s.slideshow_folder.clone())));
    // The field follows the setting when it changes elsewhere.
    effect(move || {
        let now = settings().with(|s| s.slideshow_folder.clone());
        if untrack(|| folder.get()) != now {
            folder.set(now);
        }
    });
    let commit = move || {
        let typed = folder.get_untracked().trim().to_owned();
        if !typed.is_empty() && settings().with_untracked(|s| s.slideshow_folder != typed) {
            change(|s| s.slideshow_folder = typed);
        }
    };
    let (frame, input) = field(folder, FOLDER_FIELD_WIDTH);
    let input = input
        .on_submit(move |_| commit())
        .on_focus_change(move |on| {
            if !on {
                commit();
            }
        });
    let line = |content: Div| div().flex_none().h(LINE_HEIGHT).flex().items_center().child(content);
    div()
        .flex()
        .flex_row()
        .gap(12.0)
        .pl(24.0)
        .child(
            div()
                .flex()
                .flex_col()
                .gap(8.0)
                .child(line(note("Folder:")))
                .child(line(note("Change every:"))),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .items_start()
                .gap(8.0)
                .child(line(frame.child(input)))
                .child(line(
                    cp_combo(
                        labels(SLIDESHOW_LABELS),
                        choice(
                            SLIDESHOW_MINUTES.to_vec(),
                            |s| s.slideshow_minutes,
                            |s, v| s.slideshow_minutes = v,
                        ),
                    )
                    .w(160.0),
                ))
                .child(cp_check(
                    "Shuffle",
                    setting(|s| s.slideshow_shuffle, |s, v| s.slideshow_shuffle = v),
                ))
                .child(cp_button("Next picture", crate::desktop::wallpaper::next)),
        )
}

// ----- lock screen -----------------------------------------------------------------------------

const SAVERS: &[&str] = &["none", "mystify"];
const SAVER_LABELS: &[&str] = &["(None)", "Mystify"];
const SAVER_MINUTES: &[i64] = &[1, 2, 5, 10, 15, 30];
const SAVER_MINUTE_LABELS: &[&str] = &[
    "1 minute",
    "2 minutes",
    "5 minutes",
    "10 minutes",
    "15 minutes",
    "30 minutes",
];
/// The delays before the lock, in minutes (0 for never).
const LOCK_AFTER: &[(i64, &str)] = &[
    (1, "1 minute"),
    (5, "5 minutes"),
    (10, "10 minutes"),
    (15, "15 minutes"),
    (30, "30 minutes"),
    (60, "1 hour"),
    (0, "Never"),
];

/// The lock screen's picture, the screen saver, and when the computer locks.
pub fn lock_page() -> Div {
    page(12.0)
        .child(heading("Lock screen"))
        .child(sub_heading("Picture"))
        .child(cp_check(
            "Use the desktop background",
            Binding::new(
                || settings().with(|s| s.lock_background.is_empty()),
                |on| {
                    change(|s| {
                        s.lock_background = if on {
                            String::new()
                        } else if s.wallpaper.is_empty() {
                            "builtin".to_owned()
                        } else {
                            s.wallpaper.clone()
                        };
                    })
                },
            ),
        ))
        .child(show(
            || settings().with(|s| !s.lock_background.is_empty()),
            || {
                picture_chooser(
                    || {
                        settings().with(|s| {
                            if s.lock_background == "builtin" {
                                String::new()
                            } else {
                                s.lock_background.clone()
                            }
                        })
                    },
                    built_in_wallpaper,
                    "Windows 7 style (built in)",
                    false,
                    // "" means "the desktop's": keep a real choice instead.
                    |path| {
                        change(|s| {
                            s.lock_background =
                                if path.is_empty() { "builtin".to_owned() } else { path }
                        })
                    },
                )
                .self_stretch()
            },
        ))
        .child(sub_heading("Screen saver"))
        .child(
            row(12.0)
                .child(
                    cp_combo(
                        labels(SAVER_LABELS),
                        choice(
                            SAVERS.iter().map(|s| (*s).to_owned()).collect::<Vec<String>>(),
                            |s| s.screensaver.clone(),
                            |s, v| s.screensaver = v,
                        ),
                    )
                    .w(160.0),
                )
                .child(note("Wait:"))
                .child(
                    cp_combo(
                        labels(SAVER_MINUTE_LABELS),
                        choice(
                            SAVER_MINUTES.to_vec(),
                            |s| s.screensaver_minutes,
                            |s, v| s.screensaver_minutes = v,
                        ),
                    )
                    .w(120.0),
                )
                .child(show(
                    || settings().with(|s| s.screensaver != "none"),
                    || cp_button("Preview", crate::dialogs::preview_screen_saver),
                )),
        )
        .child(wide_note(
            "It runs while the computer is left alone, before the lock; the mouse or a key \
             ends it.",
        ))
        .child(sub_heading("Lock the computer"))
        .child(note("After the mouse and the keyboard have been left alone for:"))
        .child(flow(16.0).children(LOCK_AFTER.iter().map(|&(minutes, label)| {
            cp_radio(
                label,
                setting(|s| s.lock_after_minutes, |s, v| s.lock_after_minutes = v),
                minutes,
            )
        })))
        .child(cp_check(
            "Lock when the computer goes to sleep",
            setting(|s| s.lock_on_suspend, |s, v| s.lock_on_suspend = v),
        ))
        .child(
            row(10.0)
                .pt(8.0)
                .child(cp_button("Lock now", crate::lock::lock))
                .child(note("Also Super+L, and loginctl lock-session.")),
        )
}

// ----- account picture -------------------------------------------------------------------------

/// The mat the account picture sits on.
const PICTURE_MAT: Color = rgb(0xffffff);
const PICTURE_SIZE: f32 = 96.0;

/// "Choose your account picture": the picture now, the user's name, and the pictures to pick
/// from.
pub fn account_page() -> Div {
    let picture = div()
        .flex_none()
        .size(PICTURE_SIZE)
        .p(4.0)
        .border(1.0)
        .bg(PICTURE_MAT)
        .overflow_clip()
        .dyn_style(|s| s.rounded(corner(6.0)).border_color(scheme().field_border))
        .child(
            img(user_picture_source)
                .w_full()
                .h_full()
                .object_fit(ObjectFit::Cover),
        );
    let about = div()
        .flex()
        .flex_col()
        .items_start()
        .gap(6.0)
        .child(
            div()
                .dyn_style(|s| {
                    s.font_family(font())
                        .text_size(theme().font_size + 4.0)
                        .text_color(scheme().text)
                })
                .child(text(crate::apps::user_name)),
        )
        .child(show(
            || settings().with(|s| !s.user_picture.is_empty()),
            || {
                cp_link("Go back to the default picture", false, || {
                    change(|s| s.user_picture = String::new())
                })
            },
        ));
    page(12.0)
        .child(heading("Choose your account picture"))
        .child(wide_note(
            "Shown in the Start menu and on the lock screen. With none chosen here, the \
             system's is used, as log-in screens read it: AccountsService's icon, ~/.face, \
             then ~/.face.icon.",
        ))
        .child(row(16.0).child(picture).child(about))
        .child(
            picture_chooser(
                || settings().with(|s| s.user_picture.clone()),
                || asset("user.svg"),
                "The default picture",
                true,
                |path| change(|s| s.user_picture = path),
            )
            .self_stretch(),
        )
}

// ----- taskbar and Start menu ------------------------------------------------------------------

/// The icons the desktop can show beside the Recycle Bin, by kind.
const SYSTEM_ICONS: &[(&str, &str)] = &[
    ("computer", "Computer"),
    ("userfiles", "User's Files"),
    ("controlpanel", "Control Panel"),
];
/// How many programs started last the launcher lists (0 for none).
const LAUNCHER_COUNTS: &[i64] = &[0, 10, 15, 20, 25, 30, 40, 50];

/// The taskbar, the desktop icons, the Start menu and the power button.
pub fn taskbar_page() -> Div {
    let check = |label: &'static str, get: fn(&Settings) -> bool, set: fn(&mut Settings, bool)| {
        cp_check(label, setting(get, set))
    };
    let power_choices = |get: fn() -> String, set: fn(&mut Settings, String)| {
        flow(16.0).children(
            crate::power::POWER_ACTIONS
                .iter()
                .filter(|action| action.id != "separator")
                .map(move |action| {
                    cp_radio(
                        action.label.to_string(),
                        Binding::new(get, move |id| change(|s| set(s, id))),
                        action.id.to_string(),
                    )
                }),
        )
    };
    page(10.0)
        .child(heading("Taskbar and Start menu"))
        .child(sub_heading("Taskbar"))
        .child(check(
            "Combine the windows of a program into one button",
            |s| s.combine_windows,
            |s, v| s.combine_windows = v,
        ))
        .child(check(
            "Compact: show the icons alone, without labels",
            |s| s.taskbar_compact,
            |s, v| s.taskbar_compact = v,
        ))
        .child(check(
            "Auto-hide the taskbar",
            |s| s.taskbar_auto_hide,
            |s, v| s.taskbar_auto_hide = v,
        ))
        .child(check(
            "Use Aero Peek to preview the desktop",
            |s| s.peek,
            |s, v| s.peek = v,
        ))
        .child(sub_heading("Desktop icons"))
        .child(note("Show these on the desktop, beside the Recycle Bin:"))
        .children(SYSTEM_ICONS.iter().map(|&(kind, label)| {
            cp_check(
                label,
                Binding::new(
                    move || settings().with(|s| s.desktop_system_icons.iter().any(|k| k == kind)),
                    move |on| {
                        change(|s| {
                            s.desktop_system_icons =
                                Settings::with_member(&s.desktop_system_icons, kind, on)
                        })
                    },
                ),
            )
        }))
        .child(sub_heading("Start menu"))
        .child(check(
            "Use large icons",
            |s| s.start_menu_large_icons,
            |s, v| s.start_menu_large_icons = v,
        ))
        .child(check(
            "Show the programs used most",
            |s| s.show_frequent,
            |s, v| s.show_frequent = v,
        ))
        .child(
            row(10.0)
                .child(note("Programs started last in the launcher (Windows+Shift+D):"))
                .child(
                    cp_combo(
                        LAUNCHER_COUNTS
                            .iter()
                            .map(|&n| if n == 0 { "None".to_owned() } else { n.to_string() })
                            .collect(),
                        choice(
                            LAUNCHER_COUNTS.to_vec(),
                            |s| s.launcher_recent_count,
                            |s, v| s.launcher_recent_count = v,
                        ),
                    )
                    .w(80.0),
                ),
        )
        .child(note("The power button:"))
        .child(power_choices(|| crate::power::power_button_action().to_string(), |s, v| {
            s.power_button_action = v
        }))
        .child(note("What the Shut Down dialog (Super+Shift+E) selects:"))
        .child(power_choices(|| crate::power::shutdown_dialog_action().to_string(), |s, v| {
            s.shutdown_dialog_action = v
        }))
}

// ----- notification area icons -----------------------------------------------------------------

/// Up to this many notification-area icons, the ones with no choice made show on the taskbar;
/// beyond it they wait in the popover (Settings.qml's `trayShownByDefault`).
const TRAY_SHOWN_BY_DEFAULT: usize = 3;
/// The size of an icon in the list, and the width of its name.
const TRAY_ICON_SIZE: f32 = 20.0;
const TRAY_NAME_WIDTH: f32 = 260.0;
/// How faint the list is while every icon shows anyway.
const OVERRIDDEN_OPACITY: f32 = 0.5;

/// Whether a notification-area icon shows on the taskbar, or only in the popover of hidden
/// icons: the choice made for it, else shown while there are few icons.
fn tray_visible(id: &str) -> bool {
    let (all, shown, hidden) = settings().with(|s| {
        (
            s.tray_show_all,
            s.tray_shown.iter().any(|x| x == id),
            s.tray_hidden.iter().any(|x| x == id),
        )
    });
    if all || shown {
        return true;
    }
    if hidden {
        return false;
    }
    tray().items().len() <= TRAY_SHOWN_BY_DEFAULT
}

/// Shows the icon `id` on the taskbar, or puts it in the popover. Hiding one while every icon
/// is shown keeps the others shown, now by choice.
fn set_tray_visible(id: &str, on: bool) {
    let others: Vec<String> = tray()
        .items()
        .into_iter()
        .map(|item| item.id)
        .filter(|other| other != id)
        .collect();
    change(|s| {
        if !on && s.tray_show_all {
            s.tray_shown = others;
            s.tray_show_all = false;
        }
        s.tray_shown = Settings::with_member(&s.tray_shown, id, on);
        s.tray_hidden = Settings::with_member(&s.tray_hidden, id, !on);
    });
}

/// "Select which icons appear on the taskbar": a line for each icon of the notification area.
pub fn notifications_page() -> Div {
    page(10.0)
        .child(heading("Select which icons appear on the taskbar"))
        .child(wide_note(
            "An icon left off the taskbar waits in the popover the arrow beside the clock \
             opens. A program that starts later puts its icon there until you choose \
             otherwise.",
        ))
        .child(cp_check(
            "Always show all icons on the taskbar",
            setting(|s| s.tray_show_all, |s, v| s.tray_show_all = v),
        ))
        .child(div().flex_none().h(4.0))
        .child(for_each(
            || tray().items(),
            |item: &TrayItem| item.id.clone(),
            tray_line,
        ))
        .child(show(
            || tray().items().is_empty(),
            || wide_note("No program has an icon in the notification area now."),
        ))
}

/// An icon of the notification area: its picture, its name, and whether it shows on the
/// taskbar.
fn tray_line(item: TrayItem) -> Div {
    let id = item.id.clone();
    let name = item
        .tooltip
        .as_ref()
        .map(|t| t.title.clone())
        .filter(|t| !t.is_empty())
        .or_else(|| Some(item.title.clone()).filter(|t| !t.is_empty()))
        .unwrap_or_else(|| item.id.clone());
    let icon = match item.icon.name.clone().filter(|n| !n.is_empty()) {
        Some(name) => shell_icon(name, TRAY_ICON_SIZE).into_view(),
        None => {
            let picture = item.icon.best((TRAY_ICON_SIZE * 2.0) as u32);
            bitmap_view(move || picture.clone())
                .flex_none()
                .size(TRAY_ICON_SIZE)
                .into_view()
        }
    };
    let shown = id.clone();
    row(12.0)
        .dyn_style(|s| {
            let all = settings().with(|s| s.tray_show_all);
            s.opacity(if all { OVERRIDDEN_OPACITY } else { 1.0 })
        })
        .child(icon)
        .child(
            div()
                .flex_none()
                .w(TRAY_NAME_WIDTH)
                .overflow_clip()
                .whitespace_nowrap()
                .child(label_text(name)),
        )
        .child(cp_check(
            "Show on the taskbar",
            Binding::new(move || tray_visible(&shown), move |on| set_tray_visible(&id, on)),
        ))
}
