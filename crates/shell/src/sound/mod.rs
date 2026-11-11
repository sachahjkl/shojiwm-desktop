//! The Sound window: everything PipeWire can change about sound, in one normal window (the
//! compositor gives it its frame). A pane of links on the left, as the Control Panel's, chooses
//! the page:
//!
//! - Playback: a card for each program that plays, System Sounds first, with its volume, its
//!   mute button and the device it plays to; a program with several streams lists each.
//! - Recording: the same for the programs that record, and the device each records from.
//! - Output devices and Input devices: each device with its volume, its mute button, the default
//!   one chosen, its port (headphones, speakers, HDMI) and its balance.
//! - Configuration: each card with its profile (analog stereo, HDMI, off).
//!
//! Everything comes from PipeWire through `audio()`, follows it live, and every change applies
//! at once. There are no level meters: AntiWebUI does not read the sound itself.
//!
//! `shojiwm-shell msg sound open [page]` and `close` drive it; the pages are `playback`,
//! `recording`, `outputs`, `inputs` and `configuration`.

mod cards;
mod devices;
mod streams;

use crate::control_panel::{heading, note};
use crate::paths::asset;
use crate::theme::{scheme, theme};
use crate::ui::{VolumeControl, aero_scroll, cp_link, font, vertical_gradient};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use std::cell::{Cell, RefCell};

/// The window's title.
pub const TITLE: &str = "Sound";
/// The window's size when it opens.
const WIDTH: i32 = 820;
const HEIGHT: i32 = 580;
/// The width of the pane of links.
const SIDE_WIDTH: f32 = 190.0;
/// The room around a page.
const PAGE_PAD_X: f32 = 24.0;
const PAGE_PAD_TOP: f32 = 20.0;
const PAGE_PAD_BOTTOM: f32 = 24.0;
/// The icon of a link of the pane.
const LINK_ICON: f32 = 18.0;
/// The picture at the top of the pane.
const HEADER_ICON: f32 = 32.0;
/// The picture of a device or a card, at the left of its card.
const DEVICE_ICON: f32 = 48.0;

/// A page of the window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Playback,
    Recording,
    Outputs,
    Inputs,
    Configuration,
}

impl Page {
    const ALL: [Page; 5] = [
        Page::Playback,
        Page::Recording,
        Page::Outputs,
        Page::Inputs,
        Page::Configuration,
    ];

    /// Its name in the `sound open` command.
    fn key(self) -> &'static str {
        match self {
            Page::Playback => "playback",
            Page::Recording => "recording",
            Page::Outputs => "outputs",
            Page::Inputs => "inputs",
            Page::Configuration => "configuration",
        }
    }

    fn of(key: &str) -> Option<Page> {
        Page::ALL.into_iter().find(|p| p.key() == key)
    }

    /// Its title, in the pane and over the page.
    fn title(self) -> &'static str {
        match self {
            Page::Playback => "Playback",
            Page::Recording => "Recording",
            Page::Outputs => "Output Devices",
            Page::Inputs => "Input Devices",
            Page::Configuration => "Configuration",
        }
    }

    /// The picture of its link, in `assets/icons`.
    fn icon(self) -> &'static str {
        match self {
            Page::Playback => "sound-mixer",
            Page::Recording => "microphone",
            Page::Outputs => "speaker",
            Page::Inputs => "line-in",
            Page::Configuration => "sound-card",
        }
    }
}

thread_local! {
    static WINDOW: RefCell<Option<WindowHandle>> = const { RefCell::new(None) };
    static PAGE: Cell<Option<Signal<Page>>> = const { Cell::new(None) };
}

/// The page shown, a signal of the shell's life.
fn page_signal() -> Signal<Page> {
    PAGE.with(|c| {
        c.get().unwrap_or_else(|| {
            let (s, _) = create_root(|| signal(Page::Playback));
            c.set(Some(s));
            s
        })
    })
}

/// Answers the `sound` commands. Call once, from the shell's root.
pub fn install() {
    crate::ipc::handle("sound", |function, args| match function {
        "open" => {
            let key = args.first().map(String::as_str).unwrap_or("playback");
            let page = Page::of(key).ok_or_else(|| format!("no page {key}"))?;
            open_page(page);
            Ok(String::new())
        }
        "close" => {
            close();
            Ok(String::new())
        }
        _ => Err(format!("no function {function}")),
    });
}

fn window() -> Option<WindowHandle> {
    WINDOW.with(|w| w.borrow().clone().filter(WindowHandle::is_open))
}

fn set_window(handle: Option<WindowHandle>) {
    WINDOW.with(|w| *w.borrow_mut() = handle);
}

/// Opens the Sound window on `page`; when it is open already, it goes to that page.
pub fn open_page(page: Page) {
    page_signal().set(page);
    if window().is_some() {
        return;
    }
    let options = WindowOptions::new()
        .title(TITLE)
        .size(WIDTH, HEIGHT)
        .top_level()
        .background(untrack(|| scheme().page));
    set_window(Some(open_window(options, window_view)));
}

/// Closes the Sound window.
pub fn close() {
    if let Some(window) = window() {
        window.close();
    }
    set_window(None);
}

// ----- what the pages share --------------------------------------------------------------------

/// The default output, or the default input when `input`.
pub fn default_device(input: bool) -> Option<AudioDevice> {
    if input {
        audio().default_source()
    } else {
        audio().default_sink()
    }
}

/// The devices: the outputs, or the inputs when `input`.
pub fn devices(input: bool) -> Vec<AudioDevice> {
    if input {
        audio().sources()
    } else {
        audio().sinks()
    }
}

/// The volume of the default output, or of the default input when `input`.
pub fn default_device_control(input: bool) -> VolumeControl {
    let device = move || default_device(input);
    VolumeControl::new(
        move || device().map(|d| (d.volume, d.muted)),
        move |volume| {
            if let Some(d) = untrack(device) {
                let _ = d.set_volume(volume);
            }
        },
        move |muted| {
            if let Some(d) = untrack(device) {
                let _ = d.set_muted(muted);
            }
        },
    )
}

/// The name of a device to show: its description, else its name.
fn device_label(device: &AudioDevice) -> String {
    if device.description.is_empty() {
        device.name.clone()
    } else {
        device.description.clone()
    }
}

/// The volume as a percentage, beside a slider, in a column wide enough for "100 %".
fn percent_text(control: VolumeControl) -> Div {
    div()
        .flex_none()
        .w(40.0)
        .dyn_style(|s| {
            s.text_size(theme().font_size_small)
                .text_color(scheme().text_muted)
        })
        .child(text(move || format!("{} %", control.percent())))
}

/// A card of a page: a box with a soft border, its parts one under another.
fn card_box() -> Div {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(6.0)
        .p(10.0)
        .border(1.0)
        .dyn_style(|s| {
            let c = scheme();
            s.rounded(crate::theme::item_radius())
                .bg(c.field)
                .border_color(c.menu_divider)
        })
}

/// A row of controls lined up on their middles.
fn row(gap: f32) -> Div {
    div().flex().flex_row().items_center().gap(gap)
}

/// Text in the theme's face and size and the text colour, on one line.
fn line_text(label: impl IntoTextSource) -> Text {
    text(label).whitespace_nowrap().dyn_style(|s| {
        s.font_family(font())
            .text_size(theme().font_size)
            .text_color(scheme().text)
    })
}

/// Text in the small size and the muted colour, on one line.
fn small_text(label: impl IntoTextSource) -> Text {
    text(label).whitespace_nowrap().dyn_style(|s| {
        s.font_family(font())
            .text_size(theme().font_size_small)
            .text_color(scheme().text_muted)
    })
}

/// The picture `name` of `assets/icons`, `side` pixels square.
fn picture(name: &str, side: f32) -> Img {
    img(asset(&format!("icons/{name}.png")))
        .flex_none()
        .size(side)
        .object_fit(ObjectFit::Contain)
}

/// The picture of a device, in `assets/icons`, from what it says of itself: the texts are its
/// active port's name and description, its name and description, and its card's icon name, in
/// that order; the first that names a kind of device chooses. An input is a webcam, a headset,
/// a line in or else a microphone; an output is a screen (HDMI, DisplayPort, digital), a pair of
/// headphones, a headset or else a speaker.
fn device_picture(input: bool, texts: &[&str]) -> &'static str {
    for text in texts {
        let text = text.to_lowercase();
        let has = |word: &str| text.contains(word);
        // "Line In", "analog-input-linein", "[In] Line": a word starting with "line".
        let line = text
            .split(|c: char| !c.is_alphanumeric())
            .any(|w| w.starts_with("line"));
        let found = if input {
            if has("webcam") || has("camera") {
                Some("webcam")
            } else if has("headset") {
                Some("headset")
            } else if line {
                Some("line-in")
            } else {
                None
            }
        } else if has("hdmi") || has("displayport") || has("iec958") {
            Some("display")
        } else if has("headphone") {
            Some("headphones")
        } else if has("headset") {
            Some("headset")
        } else {
            None
        };
        if let Some(name) = found {
            return name;
        }
    }
    if input { "microphone" } else { "speaker" }
}

/// [`device_picture`] of a device as PipeWire gives it.
fn device_picture_of(input: bool, device: &AudioDevice) -> &'static str {
    let port = device
        .port
        .and_then(|index| device.ports.iter().find(|p| p.index == index));
    let card_icon = device
        .card
        .and_then(|id| audio().cards().into_iter().find(|c| c.id == id))
        .map(|c| c.icon_name)
        .unwrap_or_default();
    device_picture(
        input,
        &[
            port.map_or("", |p| p.name.as_str()),
            port.map_or("", |p| p.description.as_str()),
            &device.name,
            &device.description,
            &card_icon,
        ],
    )
}

// ----- the window ------------------------------------------------------------------------------

fn window_view() -> Div {
    on_cleanup(|| set_window(None));
    let page = page_signal();
    let content = div()
        .flex_1()
        .min_w(0.0)
        .min_h(0.0)
        .flex()
        .flex_col()
        // Built again for each page, so that each page shows from its top.
        .child(dyn_view(move || {
            let shown = page.get();
            aero_scroll(
                div()
                    .flex()
                    .flex_col()
                    .gap(10.0)
                    .px(PAGE_PAD_X)
                    .pt(PAGE_PAD_TOP)
                    .pb(PAGE_PAD_BOTTOM)
                    .child(heading(shown.title()))
                    .child(dyn_view(move || {
                        if !audio().available() {
                            return note("PipeWire is not running: there is no sound to control.")
                                .into_view();
                        }
                        untrack(|| page_view(shown))
                    })),
            )
            .flex_1()
            .min_h(0.0)
        }));
    div()
        .w_full()
        .h_full()
        .flex()
        .flex_row()
        .dyn_style(|s| {
            let c = scheme();
            s.bg(c.page)
                .font_family(font())
                .text_size(theme().font_size)
                .text_color(c.text)
        })
        .child(side_pane())
        .child(content)
}

fn page_view(page: Page) -> View {
    match page {
        Page::Playback => streams::page(false).into_view(),
        Page::Recording => streams::page(true).into_view(),
        Page::Outputs => devices::page(false).into_view(),
        Page::Inputs => devices::page(true).into_view(),
        Page::Configuration => cards::page().into_view(),
    }
}

/// The pane of links, in the Control Panel's pale gradient: the page shown in bold.
fn side_pane() -> Div {
    let page = page_signal();
    let link = move |p: Page| {
        row(8.0).child(picture(p.icon(), LINK_ICON)).child(
            cp_link(p.title(), false, move || page.set(p)).dyn_style(move |s| {
                if page.get() == p {
                    s.font_weight(FontWeight::BOLD)
                } else {
                    s
                }
            }),
        )
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
        .fill(|| {
            let c = scheme();
            vertical_gradient(&[(0.0, c.side_top), (1.0, c.side_bottom)])
        })
        .dyn_style(|s| s.border_color(scheme().menu_divider))
        .child(
            row(10.0)
                .pb(6.0)
                .child(picture("sound", HEADER_ICON))
                .child(text(TITLE).whitespace_nowrap().dyn_style(|s| {
                    s.font_family(font())
                        .text_size(theme().font_size + 4.0)
                        .text_color(scheme().page_heading)
                })),
        )
        .children(Page::ALL.map(link))
        .child(div().flex_1())
        .child(small_text("See also"))
        .child(cp_link("Sound in Control Panel", false, || {
            crate::control_panel::open_page("sound");
        }))
}

#[cfg(test)]
mod tests {
    use super::device_picture;

    #[test]
    fn outputs() {
        assert_eq!(
            device_picture(false, &["hdmi-output-0", "HDMI / DisplayPort"]),
            "display"
        );
        assert_eq!(
            device_picture(false, &["", "", "alsa_output.x.iec958-stereo"]),
            "display"
        );
        assert_eq!(
            device_picture(false, &["analog-output-headphones", "Headphones"]),
            "headphones"
        );
        assert_eq!(
            device_picture(false, &["", "", "", "USB Headset"]),
            "headset"
        );
        assert_eq!(
            device_picture(false, &["analog-output-speaker", "Speakers"]),
            "speaker"
        );
        assert_eq!(device_picture(false, &[]), "speaker");
    }

    #[test]
    fn inputs() {
        assert_eq!(
            device_picture(true, &["", "", "", "", "camera-web"]),
            "webcam"
        );
        assert_eq!(
            device_picture(true, &["", "", "", "HD Pro Webcam C920"]),
            "webcam"
        );
        assert_eq!(
            device_picture(true, &["analog-input-linein", "Line In"]),
            "line-in"
        );
        assert_eq!(
            device_picture(true, &["analog-input-mic", "Microphone"]),
            "microphone"
        );
        assert_eq!(
            device_picture(true, &["", "", "", "Headset Microphone"]),
            "headset"
        );
        // The port chooses before the device's name.
        assert_eq!(
            device_picture(true, &["analog-input-linein", "Line In", "", "Webcam"]),
            "line-in"
        );
        // "online" is not a line in.
        assert_eq!(
            device_picture(true, &["", "", "", "Online Microphone"]),
            "microphone"
        );
    }
}
