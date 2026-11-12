//! "Share your screen": the desktop's own picker for the screen-cast portal. A program asks to
//! record the screen; the portal runs `shojiwm-shell pick-source`, which hands the request to
//! this dialog through a deferred IPC call ([`crate::ipc::handle_later`]) and prints the chosen
//! source's id.
//!
//! The dialog has two tabs, Screens and Windows, each a grid of cards: a live picture of the
//! screen or the window (the program's icon until the first picture comes), and its name. Share
//! answers the call with the chosen card's id; Cancel, Escape and the dialog going away answer
//! "cancelled". A second request while one is open cancels the first. When the portal gives up
//! (it kills the picker), the IPC socket thread sees the caller go and the dialog closes.
//!
//! The keys: Tab and Shift+Tab move between the tabs, the cards, Share and Cancel; Ctrl+Tab and
//! Ctrl+Page Down switch the tab. On the tabs, Left and Right switch the tab; on the cards, the
//! arrows, Home and End move the choice. Return shares (Cancel when it has the keyboard), Space
//! presses the button that has the keyboard, and Escape cancels.

use super::frame::{button_band, dialog_frame, dialog_page};
use super::screen_or_primary;
use crate::apps::{entry_for, icon_for};
use crate::ipc::Reply;
use crate::theme::{corner, is_kernel, scheme, theme};
use crate::ui::{
    aero_scroll, cp_button, cp_default_button, dark_look, font, shell_icon, shell_tooltip,
    vertical_gradient,
};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use serde_json::Value;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::Duration;

/// The dimmed screen round the dialog: it waits for an answer, so a stray press does not close
/// it.
const DIM: Color = rgba(0x00000073);
/// The cards: their width, the picture's height, the room inside them, and the label's height.
const CARD_WIDTH: f32 = 168.0;
const PICTURE_HEIGHT: f32 = 100.0;
const CARD_PADDING: f32 = 6.0;
const LABEL_HEIGHT: f32 = 20.0;
/// The room between the cards and round the grid; the cards in a row.
const CARD_GAP: f32 = 8.0;
const GRID_PADDING: f32 = 10.0;
const COLUMNS: usize = 3;
/// The grid's width holds exactly [`COLUMNS`] cards, so that Up and Down move by a row. Two
/// pixels more: each box is rounded to physical pixels, and a row must not lose a card to it.
const GRID_WIDTH: f32 =
    COLUMNS as f32 * CARD_WIDTH + (COLUMNS as f32 - 1.0) * CARD_GAP + 2.0 * GRID_PADDING + 2.0;
/// The scrolled box shows two rows; more scroll.
const CARD_HEIGHT: f32 = 2.0 + 2.0 * CARD_PADDING + PICTURE_HEIGHT + 6.0 + LABEL_HEIGHT;
const GRID_HEIGHT: f32 = 2.0 * CARD_HEIGHT + CARD_GAP + 2.0 * GRID_PADDING;
/// The scroll bar's room beside the grid.
const SCROLL_BAR: f32 = 17.0;
/// The page's margins round the tabs.
const PAGE_MARGIN: f32 = 14.0;
/// The dialog: the panel, the page's margins and outline, the frame's margins and line.
const WIDTH: f32 = GRID_WIDTH + SCROLL_BAR + 2.0 + 2.0 * PAGE_MARGIN + 2.0 + 14.0 + 2.0;
/// The picture's width in a card.
const PICTURE_WIDTH: f32 = CARD_WIDTH - 2.0 - 2.0 * CARD_PADDING;
/// The program's icon in the header, a card's icon while it has no picture, a window's icon by
/// its title.
const HEADER_ICON: f32 = 32.0;
const CARD_ICON: f32 = 32.0;
const TITLE_ICON: f32 = 16.0;
/// How often a picture is taken again: enough to tell what is on it.
const REFRESH: Duration = Duration::from_secs(1);
/// What is round a picture: dark under a screen's or a window's own colours.
const PICTURE_BACK_DARK: Color = rgba(0x00000066);
const PICTURE_BACK_LIGHT: Color = rgba(0x0000001f);
/// The line inside the card that has the keyboard, as a button's.
const FOCUS_LINE_DARK: Color = rgba(0xffffff99);
const FOCUS_LINE_LIGHT: Color = rgba(0x00000099);
/// The tabs: the strip's height, an Aero tab's height when chosen and when not, Kernel's bar
/// under the chosen tab.
const TAB_STRIP_HEIGHT: f32 = 26.0;
const TAB_HEIGHT: f32 = 24.0;
const TAB_HEIGHT_BACK: f32 = 21.0;
const KERNEL_TAB_BAR: f32 = 2.0;
/// The band of buttons and the line along its top, as "Shut Down Windows" has them.
const BAND_DARK: Color = rgb(0x1a232c);
const BAND_LIGHT: Color = rgb(0xf0f0f0);
const BAND_LINE_DARK: Color = rgba(0xffffff33);
const BAND_LINE_LIGHT: Color = rgb(0xdfdfdf);

// ----- the request -------------------------------------------------------------------------------

/// Something the program can be given to see.
#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    /// What the portal gets back: an output's name, or a window's `ext_foreign_toplevel_list`
    /// identifier.
    id: String,
    /// The output's connector name, for a screen; empty for a window.
    output: String,
    /// The line under the picture.
    label: String,
    /// The window's program; empty for a screen.
    app_id: String,
    /// More about it, in the label's tooltip: the screen's description.
    detail: String,
}

/// A request of the portal, as the dialog shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    /// Tells a request from the one before it: a second one on the same screen builds the dialog
    /// again.
    serial: u64,
    /// The screen the dialog is on, by connector name.
    on: String,
    /// The program that asks.
    app_id: String,
    /// It asked for a window first.
    prefer_windows: bool,
    screens: Vec<Source>,
    windows: Vec<Source>,
}

static REQUEST: OnceLock<SharedSignal<Option<Request>>> = OnceLock::new();

/// The request shown, if any.
fn request_slot() -> &'static SharedSignal<Option<Request>> {
    REQUEST.get_or_init(|| shared_signal(None))
}

thread_local! {
    /// The reply to the request shown, with the request's serial. Only the thread that answers
    /// the IPC target holds it: a reply is answered on the thread it came to.
    static PENDING: RefCell<Option<(u64, Reply)>> = const { RefCell::new(None) };
    static NEXT_SERIAL: Cell<u64> = const { Cell::new(1) };
}

/// Whether the dialog shows on `output`. Reading it tracks it.
pub fn shown_on(output: &str) -> bool {
    request_slot().with(|r| r.as_ref().is_some_and(|r| r.on == output))
}

fn string(v: &Value, key: &str) -> String {
    v.get(key).and_then(Value::as_str).unwrap_or("").to_owned()
}

/// The portal's request: `{ app_id, prefer, sources: [{ id, kind, ... }] }`. Gives the program,
/// whether it prefers a window, the screens and the windows.
fn parse(text: &str) -> Result<(String, bool, Vec<Source>, Vec<Source>), String> {
    let v: Value = serde_json::from_str(text).map_err(|e| format!("bad request: {e}"))?;
    let mut screens = Vec::new();
    let mut windows = Vec::new();
    let sources = v.get("sources").and_then(Value::as_array);
    for s in sources.into_iter().flatten() {
        let id = string(s, "id");
        if id.is_empty() {
            continue;
        }
        match string(s, "kind").as_str() {
            "output" => {
                let mut name = string(s, "name");
                if name.is_empty() {
                    name = id.clone();
                }
                let size = |key| s.get(key).and_then(Value::as_i64).unwrap_or(0);
                let (width, height) = (size("width"), size("height"));
                let number = screens.len() + 1;
                // The size goes to the tooltip: with it the line does not fit the card.
                let label = format!("Screen {number} \u{2014} {name}");
                let description = string(s, "description");
                let detail = if width > 0 && height > 0 {
                    format!("{width}\u{00d7}{height}  {description}")
                } else {
                    description
                };
                screens.push(Source {
                    id,
                    output: name,
                    label,
                    app_id: String::new(),
                    detail: detail.trim_end().to_owned(),
                });
            }
            "window" => {
                let app_id = string(s, "app_id");
                let title = string(s, "title");
                let label = if !title.is_empty() {
                    title
                } else if !app_id.is_empty() {
                    app_id.clone()
                } else {
                    "Window".into()
                };
                windows.push(Source {
                    id,
                    output: String::new(),
                    label,
                    app_id,
                    detail: String::new(),
                });
            }
            _ => {}
        }
    }
    let prefer_windows = string(&v, "prefer") == "window";
    Ok((string(&v, "app_id"), prefer_windows, screens, windows))
}

/// Shows the portal's request `text` (one line of JSON) and answers `reply` once the user has
/// chosen. A request already shown is cancelled.
pub fn ask(text: &str, reply: Reply) {
    let (app_id, prefer_windows, screens, windows) = match parse(text) {
        Ok(parsed) => parsed,
        Err(e) => return reply.send(Err(e)),
    };
    if screens.is_empty() && windows.is_empty() {
        return reply.send(Err("nothing to share".into()));
    }
    // The screen of the focused window: the program that asks is almost always the one in use.
    let on = untrack(|| {
        let focused = toplevels()
            .active()
            .and_then(|t| t.outputs.first().cloned())
            .unwrap_or_default();
        screen_or_primary(&focused)
    });
    if on.is_empty() {
        return reply.send(Err("no screen".into()));
    }
    let serial = NEXT_SERIAL.with(|n| n.replace(n.get() + 1));
    reply.on_gone(move || close(serial, None));
    crate::start_menu::close();
    let previous = PENDING.with(|p| p.borrow_mut().replace((serial, reply)));
    // Its caller hears "cancelled".
    drop(previous);
    request_slot().set(Some(Request {
        serial,
        on,
        app_id,
        prefer_windows,
        screens,
        windows,
    }));
}

/// Ends the request `serial` if it is still the one shown: `chosen` is the id to share, `None`
/// cancels.
fn close(serial: u64, chosen: Option<String>) {
    let pending = PENDING.with(|p| {
        let mut p = p.borrow_mut();
        if p.as_ref().is_some_and(|(s, _)| *s == serial) {
            p.take()
        } else {
            None
        }
    });
    let Some((_, reply)) = pending else {
        return;
    };
    match chosen {
        Some(id) => reply.send(Ok(id)),
        // Dropped unsent: "cancelled".
        None => drop(reply),
    }
    request_slot().set(None);
}

/// The program's name, from its app id; "A program" when it is not known.
fn program_name(app_id: &str) -> String {
    if app_id.is_empty() {
        return "A program".into();
    }
    entry_for(app_id)
        .map(|e| e.name)
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "A program".into())
}

// ----- the state ---------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tab {
    Screens,
    Windows,
}

impl Tab {
    fn index(self) -> usize {
        match self {
            Tab::Screens => 0,
            Tab::Windows => 1,
        }
    }

    fn other(self) -> Self {
        match self {
            Tab::Screens => Tab::Windows,
            Tab::Windows => Tab::Screens,
        }
    }
}

/// What has the keyboard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Focus {
    Tabs,
    Cards,
    Share,
    Cancel,
}

impl Focus {
    fn step(self, by: i32) -> Self {
        let all = [Focus::Tabs, Focus::Cards, Focus::Share, Focus::Cancel];
        let at = all.iter().position(|f| *f == self).unwrap_or(0) as i32;
        all[(at + by).rem_euclid(all.len() as i32) as usize]
    }
}

#[derive(Clone, Copy)]
struct State {
    tab: Signal<Tab>,
    /// The chosen card of each tab, by index: screens, then windows. Each tab keeps its own.
    chosen: [Signal<Option<usize>>; 2],
    focus: Signal<Focus>,
    /// The cards of each tab.
    counts: [usize; 2],
}

impl State {
    /// The chosen card of the tab shown. Reading it tracks it.
    fn chosen_now(&self) -> Option<usize> {
        self.chosen[self.tab.get().index()].get()
    }

    /// Moves the choice in the tab shown by `by` cards, kept in the grid.
    fn move_by(&self, by: i32) {
        let tab = self.tab.get_untracked().index();
        let last = self.counts[tab] as i32 - 1;
        if last < 0 {
            return;
        }
        self.chosen[tab].update(|c| {
            let at = c.map_or(0, |c| c as i32);
            *c = Some((at + by).clamp(0, last) as usize);
        });
    }

    /// Chooses the first card (`last` false) or the last one of the tab shown.
    fn move_to_end(&self, last: bool) {
        let tab = self.tab.get_untracked().index();
        let count = self.counts[tab];
        if count > 0 {
            self.chosen[tab].set(Some(if last { count - 1 } else { 0 }));
        }
    }
}

// ----- the dialog --------------------------------------------------------------------------------

/// The dialog on `output` while a request is shown there; built again for each request.
pub fn view(output: Output) -> View {
    let serial = memo(|| request_slot().with(|r| r.as_ref().map(|r| r.serial)));
    dyn_view(move || {
        serial.get()?;
        let request = untrack(|| request_slot().with(Clone::clone))?;
        Some(dialog(request, &output))
    })
}

fn dialog(request: Request, output: &Output) -> Div {
    let serial = request.serial;
    // The dialog went away with no answer (its screen was unplugged): the program must not wait.
    // A frame later, out of the teardown; an answered or replaced request is left alone.
    on_cleanup(move || request_frame(move |_| close(serial, None)));

    let sources: Rc<[Vec<Source>; 2]> = Rc::new([request.screens, request.windows]);
    let first = |list: &Vec<Source>| (!list.is_empty()).then_some(0);
    let opened_on = if (request.prefer_windows && !sources[1].is_empty()) || sources[0].is_empty() {
        Tab::Windows
    } else {
        Tab::Screens
    };
    let state = State {
        tab: signal(opened_on),
        chosen: [signal(first(&sources[0])), signal(first(&sources[1]))],
        focus: signal(Focus::Cards),
        counts: [sources[0].len(), sources[1].len()],
    };
    let share: Rc<dyn Fn()> = {
        let sources = sources.clone();
        Rc::new(move || {
            let tab = state.tab.get_untracked().index();
            let id = state.chosen[tab]
                .get_untracked()
                .and_then(|i| sources[tab].get(i))
                .map(|s| s.id.clone());
            if id.is_some() {
                close(serial, id);
            }
        })
    };
    let cancel: Rc<dyn Fn()> = Rc::new(move || close(serial, None));
    let keys = node_ref();
    request_frame(move |_| keys.focus());
    let scale = output.scale.max(1.0);

    let cards = {
        let sources = sources.clone();
        let share = share.clone();
        // Only the tab shown takes pictures.
        dyn_view(move || {
            let tab = state.tab.get();
            grid(
                state,
                tab,
                &sources[tab.index()],
                scale,
                keys,
                share.clone(),
            )
        })
    };
    let panel = div()
        .relative()
        .flex()
        .flex_col()
        .border(1.0)
        .dyn_style(|s| {
            let c = scheme();
            s.border_color(c.field_border).bg(c.page)
        })
        .child(aero_scroll(cards).h(GRID_HEIGHT));

    let page = dialog_page()
        .child(header(request.app_id))
        .child(
            div()
                .flex()
                .flex_col()
                .px(PAGE_MARGIN)
                .pb(PAGE_MARGIN)
                .child(tab_strip(state, keys))
                .child(panel),
        )
        .child(button_band(
            (BAND_DARK, BAND_LIGHT),
            Some((BAND_LINE_DARK, BAND_LINE_LIGHT)),
            buttons(state, share.clone(), cancel.clone()),
        ));
    let frame = dialog_frame("Share your screen", WIDTH, page).on_pointer_down(move |e| {
        e.stop_propagation();
        keys.focus();
    });

    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(DIM)
        .node_ref(keys)
        .focusable()
        .dyn_style(|s| s.font_family(font()).text_size(theme().font_size))
        .on_pointer_down(move |_| keys.focus())
        .on_key_down(move |e| {
            let focus = state.focus.get_untracked();
            let ctrl = e.modifiers.ctrl;
            match &e.key {
                Key::Named(NamedKey::Tab) if ctrl => state.tab.update(|t| *t = t.other()),
                Key::Named(NamedKey::PageDown | NamedKey::PageUp) if ctrl => {
                    state.tab.update(|t| *t = t.other())
                }
                Key::Named(NamedKey::Tab) => {
                    let by = if e.modifiers.shift { -1 } else { 1 };
                    state.focus.update(|f| *f = f.step(by));
                }
                Key::Named(NamedKey::ArrowLeft) if focus == Focus::Tabs => {
                    state.tab.set(Tab::Screens)
                }
                Key::Named(NamedKey::ArrowRight) if focus == Focus::Tabs => {
                    state.tab.set(Tab::Windows)
                }
                Key::Named(NamedKey::ArrowDown) if focus == Focus::Tabs => {
                    state.focus.set(Focus::Cards)
                }
                Key::Named(NamedKey::ArrowLeft) if focus == Focus::Cards => state.move_by(-1),
                Key::Named(NamedKey::ArrowRight) if focus == Focus::Cards => state.move_by(1),
                Key::Named(NamedKey::ArrowUp) if focus == Focus::Cards => {
                    state.move_by(-(COLUMNS as i32))
                }
                Key::Named(NamedKey::ArrowDown) if focus == Focus::Cards => {
                    state.move_by(COLUMNS as i32)
                }
                Key::Named(NamedKey::Home) if focus == Focus::Cards => state.move_to_end(false),
                Key::Named(NamedKey::End) if focus == Focus::Cards => state.move_to_end(true),
                Key::Named(NamedKey::ArrowLeft) if focus == Focus::Cancel => {
                    state.focus.set(Focus::Share)
                }
                Key::Named(NamedKey::ArrowRight) if focus == Focus::Share => {
                    state.focus.set(Focus::Cancel)
                }
                Key::Named(NamedKey::Enter) => {
                    if focus == Focus::Cancel {
                        cancel();
                    } else {
                        share();
                    }
                }
                Key::Named(NamedKey::Space) if focus == Focus::Share => share(),
                Key::Named(NamedKey::Space) if focus == Focus::Cancel => cancel(),
                Key::Named(NamedKey::Escape) => cancel(),
                _ => return,
            }
            e.prevent_default();
            e.stop_propagation();
        })
        .child(frame)
}

/// The program's icon and what it asks.
fn header(app_id: String) -> Div {
    let icon_app = app_id.clone();
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(12.0)
        .pt(PAGE_MARGIN)
        .px(PAGE_MARGIN)
        .pb(12.0)
        // Only a program the menu knows: an unknown one would leave an empty square.
        .children(
            untrack(|| entry_for(&icon_app).is_some())
                .then(|| shell_icon(move || icon_for(&icon_app), HEADER_ICON)),
        )
        .child(
            div()
                .flex_1()
                .min_w(0.0)
                .dyn_style(|s| {
                    s.text_color(scheme().page_heading)
                        .text_size(theme().font_size + 2.0)
                })
                .child(text(move || {
                    format!(
                        "{} wants to see your screen. Choose what it can see.",
                        program_name(&app_id)
                    )
                })),
        )
}

// ----- the tabs ----------------------------------------------------------------------------------

/// The two tabs over the panel. Kernel's are flat, the chosen one marked by a bar under it;
/// Aero's are a property sheet's, the chosen one taller and joined to the panel.
fn tab_strip(state: State, keys: NodeRef) -> Div {
    div()
        .relative()
        // Over the panel: the chosen Aero tab covers the panel's top line under it.
        .z_index(1)
        .flex()
        .flex_row()
        .items_end()
        .h(TAB_STRIP_HEIGHT)
        .dyn_style(|s| s.pl(if is_kernel() { 0.0 } else { 4.0 }))
        .child(tab_button(state, Tab::Screens, "Screens", keys))
        .child(tab_button(state, Tab::Windows, "Windows", keys))
}

fn tab_button(state: State, tab: Tab, label: &'static str, keys: NodeRef) -> Div {
    let ui = Interaction::new();
    let on = move || state.tab.get() == tab;
    let focus_line = div()
        .absolute()
        .inset(3.0)
        .border(1.0)
        .pointer_events_none()
        .dyn_style(move |s| {
            let s = s.border_color(if dark_look() {
                FOCUS_LINE_DARK
            } else {
                FOCUS_LINE_LIGHT
            });
            if on() && state.focus.get() == Focus::Tabs {
                s
            } else {
                s.hidden()
            }
        });
    let kernel_bar = div()
        .absolute()
        .left(0.0)
        .right(0.0)
        .bottom(0.0)
        .h(KERNEL_TAB_BAR)
        .dyn_style(move |s| {
            let s = s.bg(scheme().selection_border);
            if is_kernel() && on() { s } else { s.hidden() }
        });
    div()
        .relative()
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .flex_none()
        .min_w(84.0)
        .px(14.0)
        .border_t(1.0)
        .border_l(1.0)
        .border_r(1.0)
        .select_none()
        .interaction(ui)
        .access(move |a| a.role(Role::Tab))
        .on_pointer_down(move |e| {
            e.stop_propagation();
            state.tab.set(tab);
            state.focus.set(Focus::Tabs);
            keys.focus();
        })
        .fill(move || {
            let c = scheme();
            let [top, bottom] = if is_kernel() {
                let flat = if on() {
                    c.page
                } else if ui.hovered() {
                    c.hover
                } else {
                    Color::TRANSPARENT
                };
                [flat, flat]
            } else if on() {
                [c.page, c.page]
            } else if ui.hovered() {
                [c.button_hot_top, c.button_hot_bottom]
            } else {
                [c.button_top, c.button_bottom]
            };
            vertical_gradient(&[(0.0, top), (1.0, bottom)])
        })
        .dyn_style(move |s| {
            let c = scheme();
            let chosen = on();
            if is_kernel() {
                s.h(TAB_STRIP_HEIGHT)
                    .mb(0.0)
                    .border_color(Color::TRANSPARENT)
                    .text_color(if chosen { c.text } else { c.text_muted })
            } else {
                // The chosen tab stands a line lower, over the panel's edge, and the others
                // overlap it by their side lines.
                s.h(if chosen { TAB_HEIGHT } else { TAB_HEIGHT_BACK })
                    .mb(if chosen { -1.0 } else { 0.0 })
                    .ml(if tab == Tab::Screens { 0.0 } else { -1.0 })
                    .rounded_t(corner(3.0))
                    .border_color(c.field_border)
                    .text_color(c.text)
            }
        })
        .child(text(label))
        .child(kernel_bar)
        .child(focus_line)
}

// ----- the cards ---------------------------------------------------------------------------------

/// The cards of one tab, in rows of [`COLUMNS`].
fn grid(
    state: State,
    tab: Tab,
    sources: &[Source],
    scale: f32,
    keys: NodeRef,
    share: Rc<dyn Fn()>,
) -> Div {
    if sources.is_empty() {
        let empty = match tab {
            Tab::Screens => "No screen can be shared.",
            Tab::Windows => "No window is open.",
        };
        return div()
            .p(GRID_PADDING + 6.0)
            .dyn_style(|s| s.text_color(scheme().text_muted))
            .child(empty);
    }
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .gap(CARD_GAP)
        .p(GRID_PADDING)
        .w(GRID_WIDTH)
        .children(sources.iter().enumerate().map(|(index, source)| {
            card(
                state,
                tab,
                index,
                source.clone(),
                scale,
                keys,
                share.clone(),
            )
        }))
}

/// Live pictures of `source`, sized for a card at `scale`; `None` when the screen or the window
/// is gone.
fn pictures_of(tab: Tab, source: &Source, scale: f32) -> Option<Capture> {
    let options = CaptureOptions::default()
        .max_size(
            (PICTURE_WIDTH * scale).ceil() as u32,
            (PICTURE_HEIGHT * scale).ceil() as u32,
        )
        .every(REFRESH);
    untrack(|| match tab {
        Tab::Screens => outputs()
            .into_iter()
            .find(|o| o.name == source.output)
            .map(|o| capture(&o, options)),
        Tab::Windows => toplevels()
            .all()
            .into_iter()
            .find(|t| t.identifier.as_deref() == Some(source.id.as_str()))
            .map(|t| capture(&t, options)),
    })
}

/// `color` at half its strength: a card under the pointer, lighter than the chosen one.
fn faint(color: Color) -> Color {
    Color::new(color.r, color.g, color.b, color.a * 0.5)
}

fn card(
    state: State,
    tab: Tab,
    index: usize,
    source: Source,
    scale: f32,
    keys: NodeRef,
    share: Rc<dyn Fn()>,
) -> Div {
    let ui = Interaction::new();
    let me = node_ref();
    let t = tab.index();
    let selected = move || state.chosen[t].get() == Some(index);
    // The arrows can choose a card out of view.
    effect(move || {
        if selected() {
            untrack(|| me.scroll_into_view());
        }
    });

    let shot = pictures_of(tab, &source, scale);
    let has_picture = {
        let shot = shot.clone();
        memo(move || shot.as_ref().is_some_and(|s| s.picture().is_some()))
    };
    let picture = shot.map(|shot| {
        bitmap_view(move || shot.picture())
            .absolute()
            .left(0.0)
            .top(0.0)
            .right(0.0)
            .bottom(0.0)
            .object_fit(ObjectFit::Contain)
    });
    let icon = match tab {
        Tab::Screens => shell_icon("video-display", CARD_ICON),
        Tab::Windows => {
            let app_id = source.app_id.clone();
            shell_icon(move || icon_for(&app_id), CARD_ICON)
        }
    };
    let picture_box = div()
        .relative()
        .flex_none()
        .w_full()
        .h(PICTURE_HEIGHT)
        .flex()
        .items_center()
        .justify_center()
        .overflow_clip()
        .dyn_style(|s| {
            s.bg(if dark_look() {
                PICTURE_BACK_DARK
            } else {
                PICTURE_BACK_LIGHT
            })
        })
        // The icon until the first picture, and when none comes: no card is left blank.
        .child(icon.dyn_style(move |s| if has_picture.get() { s.hidden() } else { s }))
        .children(picture);

    let title_icon = (tab == Tab::Windows).then(|| {
        let app_id = source.app_id.clone();
        shell_icon(move || icon_for(&app_id), TITLE_ICON)
    });
    let tip = if source.detail.is_empty() {
        source.label.clone()
    } else {
        format!("{}\n{}", source.label, source.detail)
    };
    let label = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(6.0)
        .h(LABEL_HEIGHT)
        .min_w(0.0)
        .children(title_icon)
        .child(
            div()
                .flex_1()
                .min_w(0.0)
                .overflow_clip()
                .whitespace_nowrap()
                .child(text(source.label)),
        );
    let label = shell_tooltip(label, Placement::BelowStart, move || tip.clone());

    let focus_line = div()
        .absolute()
        .inset(2.0)
        .border(1.0)
        .pointer_events_none()
        .dyn_style(move |s| {
            let s = s.rounded(corner(2.0)).border_color(if dark_look() {
                FOCUS_LINE_DARK
            } else {
                FOCUS_LINE_LIGHT
            });
            if selected() && state.focus.get() == Focus::Cards {
                s
            } else {
                s.hidden()
            }
        });

    div()
        .relative()
        .flex_none()
        .flex()
        .flex_col()
        .gap(6.0)
        .w(CARD_WIDTH)
        .p(CARD_PADDING)
        .border(1.0)
        .select_none()
        .node_ref(me)
        .interaction(ui)
        .access(move |a| a.role(Role::Button))
        .on_click(move |e| {
            state.chosen[t].set(Some(index));
            state.focus.set(Focus::Cards);
            keys.focus();
            if e.click_count >= 2 {
                share();
            }
        })
        .dyn_style(move |s| {
            let c = scheme();
            let (back, line) = if selected() {
                (c.selection, c.selection_border)
            } else if ui.hovered() {
                (faint(c.selection), faint(c.selection_border))
            } else {
                (Color::TRANSPARENT, Color::TRANSPARENT)
            };
            s.rounded(corner(3.0))
                .bg(back)
                .border_color(line)
                .text_color(c.text)
        })
        .child(picture_box)
        .child(label)
        .child(focus_line)
}

// ----- the buttons -------------------------------------------------------------------------------

/// Share and Cancel. Share is the default button, greyed until a card is chosen; the line moves
/// to Cancel while it has the keyboard.
fn buttons(state: State, share: Rc<dyn Fn()>, cancel: Rc<dyn Fn()>) -> View {
    let can_share = memo(move || state.chosen_now().is_some());
    dyn_view(move || {
        let focus = state.focus.get();
        let enabled = can_share.get();
        let share_button = {
            let share = share.clone();
            let press = move || share();
            if enabled && focus != Focus::Cancel {
                cp_default_button("Share", press)
            } else {
                cp_button("Share", press)
            }
        };
        let share_button = if enabled {
            share_button
        } else {
            share_button.opacity(0.5).pointer_events_none()
        };
        let cancel_button = {
            let cancel = cancel.clone();
            let press = move || cancel();
            if focus == Focus::Cancel {
                cp_default_button("Cancel", press)
            } else {
                cp_button("Cancel", press)
            }
        };
        div()
            .flex()
            .flex_row()
            .gap(8.0)
            .child(share_button)
            .child(cancel_button)
    })
}
