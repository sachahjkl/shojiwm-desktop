//! Popup menus in the manner of Windows 7 (QML `ContextMenu`, `MenuItemRow`), and the rows of
//! the Start menu (QML `MenuRow`).
//!
//! The menus are AntiWebUI's `menu_with`, which keeps what a menu does (where it hangs, the
//! keys, the lit row, the submenus, closing when a row is chosen or the pointer presses
//! outside), with the look of the theme: a pale band with a thin frame under the pointer.

use super::frame::popup_frame;
use super::icon::icon_box;
use super::{font, heading_font, ink_asset, text_fn};
use crate::settings::{motion, settings};
use crate::theme::{Align, away_from_bar, corner, item_radius, scheme, theme};
use anti_webui::prelude::*;
use std::rc::Rc;

/// The room of the icon, or of the check mark in its place, at the start of a menu line.
const GUTTER: f32 = 16.0;
/// The height of a separator line's row.
const SEPARATOR_HEIGHT: f32 = 7.0;
/// The room kept at the end of a line for the arrow of a submenu.
const ARROW_ROOM: f32 = 12.0;
/// How dim a line that cannot be chosen is.
const DISABLED_OPACITY: f32 = 0.45;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Item,
    Separator,
    Heading,
    Submenu,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mark {
    None,
    Check(bool),
    Radio(bool),
}

/// A line of a popup menu: a choice, a separator, a heading or a submenu. Build it with the
/// constructors, then add an icon, a check mark or a pin.
#[derive(Clone)]
pub struct MenuLine {
    kind: Kind,
    label: String,
    icon: String,
    mark: Mark,
    enabled: bool,
    run: Option<Rc<dyn Fn()>>,
    lines: Rc<Vec<MenuLine>>,
    pin: Option<(bool, Rc<dyn Fn()>)>,
}

impl MenuLine {
    fn new(kind: Kind, label: String) -> Self {
        Self {
            kind,
            label,
            icon: String::new(),
            mark: Mark::None,
            enabled: true,
            run: None,
            lines: Rc::new(Vec::new()),
            pin: None,
        }
    }

    /// A choice: `run` runs once the menu has been told to close.
    pub fn item(label: impl Into<String>, run: impl Fn() + 'static) -> Self {
        Self {
            run: Some(Rc::new(run)),
            ..Self::new(Kind::Item, label.into())
        }
    }

    /// A line between groups.
    pub fn separator() -> Self {
        Self::new(Kind::Separator, String::new())
    }

    /// The title of a group, such as "Recent" or "Tasks": not a choice.
    pub fn heading(label: impl Into<String>) -> Self {
        Self::new(Kind::Heading, label.into())
    }

    /// A line that opens a menu of `lines` beside it, when the pointer rests on it or clicks it.
    pub fn submenu(label: impl Into<String>, lines: Vec<MenuLine>) -> Self {
        Self {
            lines: Rc::new(lines),
            ..Self::new(Kind::Submenu, label.into())
        }
    }

    /// An icon in front of the label: a name of the icon theme, or a file.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = icon.into();
        self
    }

    /// A check box in front of the label, ticked when `on`.
    pub fn checked(mut self, on: bool) -> Self {
        self.mark = Mark::Check(on);
        self
    }

    /// A radio button in front of the label, chosen when `on`.
    pub fn radio(mut self, on: bool) -> Self {
        self.mark = Mark::Radio(on);
        self
    }

    /// `false` greys it out, and it cannot be chosen.
    pub fn enabled(mut self, on: bool) -> Self {
        self.enabled = on;
        self
    }

    /// The pin of a jump list file at the end of the line, under the pointer: upright when
    /// `pinned`, leaning when not. Clicking it runs `toggle` and keeps the menu open.
    pub fn pin(mut self, pinned: bool, toggle: impl Fn() + 'static) -> Self {
        self.pin = Some((pinned, Rc::new(toggle)));
        self
    }

    /// It has an icon or a mark: every line then keeps that column, as Windows 7 menus do, so
    /// that the labels line up.
    fn wants_gutter(&self) -> bool {
        !self.icon.is_empty() || self.mark != Mark::None
    }
}

/// A popup menu of the lines `lines` gives, while `open` says so. Choosing a line, pressing
/// outside of it, Escape and Tab call `on_close`, which is the place to change what `open`
/// reads. `lines` runs each time it opens, so the lines show the state of then. `options`
/// say where it hangs: [`bar_menu_options`] for something on the bar, or AntiWebUI's
/// `MenuOptions` (`.anchor(node)`, `.at(rect)`, `.placement(..)`, `.gap(..)`). Put what comes
/// back in the element it belongs to, as a child.
pub fn popup_menu(
    open: impl Fn() -> bool + 'static,
    on_close: impl Fn() + 'static,
    options: MenuOptions,
    lines: impl Fn() -> Vec<MenuLine> + 'static,
) -> Show {
    menu_with(open, on_close, options, move |state| {
        let shown = fade_in(state);
        move || menu_surface(state, Rc::new(lines()), shown)
    })
}

/// Where a menu of something on the bar hangs: from `anchor`, away from the bar, with the
/// theme's gap. As the theme is when it is called.
pub fn bar_menu_options(anchor: NodeRef) -> MenuOptions {
    MenuOptions::new()
        .anchor(anchor)
        .placement(away_from_bar(Align::Center))
        .gap(theme().popup_gap as i32)
}

/// `target` with a popup menu of `lines` where the secondary button presses it. `target` is
/// what comes back.
pub fn with_context_menu(target: Div, lines: impl Fn() -> Vec<MenuLine> + 'static) -> Div {
    let at = signal(None::<RectI>);
    target
        .on_pointer_down(move |e| {
            if e.button == Some(PointerButton::Secondary) {
                let (x, y) = (e.position.x as i32, e.position.y as i32);
                at.set(Some(RectI::new(x, y, 1, 1)));
            }
        })
        .child(popup_menu(
            move || at.with(Option::is_some),
            move || at.set(None),
            MenuOptions::new().at(at),
            lines,
        ))
}

/// The menu fades in over the theme's `menu_fade`, and goes at once.
fn fade_in(state: MenuState) -> Animated<f32> {
    let length = untrack(|| motion(theme().menu_fade));
    follow(
        move || if state.open() { 1.0f32 } else { 0.0 },
        Transition::ms(length).ease_out(),
    )
}

fn menu_surface(state: MenuState, lines: Rc<Vec<MenuLine>>, shown: Animated<f32>) -> Div {
    let gutter = lines.iter().any(MenuLine::wants_gutter);
    popup_frame()
        .px(1.0)
        .dyn_style(move |s| {
            let t = theme();
            s.min_w(t.popup_min_width)
                .max_w(t.popup_max_width)
                .py(t.popup_pad_y)
                .opacity(shown.get().clamp(0.0, 1.0))
        })
        .children(
            lines
                .iter()
                .enumerate()
                .map(|(i, line)| menu_line(state, i, line, gutter))
                .collect::<Vec<_>>(),
        )
}

fn menu_line(state: MenuState, i: usize, line: &MenuLine, gutter: bool) -> View {
    match line.kind {
        Kind::Separator => div()
            .flex_none()
            .flex()
            .flex_row()
            .items_center()
            .h(SEPARATOR_HEIGHT)
            .child(div().flex_1().h(1.0).dyn_style(|s| {
                s.mx(theme().popup_pad_x / 2.0)
                    .bg(scheme().menu_divider)
            }))
            .into_view(),
        Kind::Heading => line_row(state, i, line, gutter, false).into_view(),
        Kind::Item => {
            let run = line.run.clone();
            let enabled = line.enabled;
            let choice = MenuChoice::new(move || {
                if let Some(run) = &run {
                    run();
                }
            })
            .disabled_when(move || !enabled);
            state
                .item(i, line_row(state, i, line, gutter, true), choice)
                .into_view()
        }
        Kind::Submenu => {
            let lines = line.lines.clone();
            let row = line_row(state, i, line, gutter, true);
            if !line.enabled {
                return state
                    .item(i, row, MenuChoice::default().disabled_when(|| true))
                    .into_view();
            }
            state
                .submenu(i, row, move |child| {
                    let shown = fade_in(child);
                    move || menu_surface(child, lines.clone(), shown)
                })
                .into_view()
        }
    }
}

/// A line's look: the band under the pointer (`choosable`), the mark or the icon, the label,
/// the arrow of a submenu and the pin.
fn line_row(state: MenuState, i: usize, line: &MenuLine, gutter: bool, choosable: bool) -> Div {
    let enabled = line.enabled;
    let heading = line.kind == Kind::Heading;
    let arrow = line.kind == Kind::Submenu;
    let lit = move || choosable && enabled && (state.is_lit(i) || state.is_open(i));
    let mark = line.mark;
    let lead: Option<Div> = if !line.icon.is_empty() {
        let icon = line.icon.clone();
        Some(icon_box(Rc::new(move || icon.clone()), GUTTER))
    } else if gutter {
        Some(
            div()
                .flex_none()
                .w(GUTTER)
                .flex()
                .justify_center()
                .dyn_style(|s| s.text_size(theme().font_size_small))
                .child(match mark {
                    Mark::Check(true) => "\u{2714}",
                    Mark::Radio(true) => "\u{25CF}",
                    _ => "",
                }),
        )
    } else {
        None
    };
    let words = line.label.clone();
    let label = div()
        .flex_1()
        .min_w(0.0)
        .pr(ARROW_ROOM)
        .overflow_clip()
        .whitespace_nowrap()
        .dyn_style(move |s| {
            let t = theme();
            let c = scheme();
            let s = s.opacity(if enabled { 1.0 } else { DISABLED_OPACITY });
            if heading {
                s.font_family(heading_font())
                    .text_size(t.font_size_small)
                    .font_weight(FontWeight::BOLD)
                    .letter_spacing(t.heading_spacing)
                    .text_color(c.menu_heading)
            } else {
                s.font_family(font()).text_size(t.font_size).text_color(c.text)
            }
        })
        .child(text(move || {
            if heading && theme().heading_caps {
                words.to_uppercase()
            } else {
                words.clone()
            }
        }));
    let row = div()
        .relative()
        .flex_none()
        .flex()
        .flex_row()
        .items_center()
        .gap(7.0)
        .mx(2.0)
        .border(1.0)
        .dyn_style(move |s| {
            let t = theme();
            let c = scheme();
            // The border is there lit or not, so that nothing moves; the padding puts the
            // content where the QML row had it, `popup_pad_x` from the menu's edge.
            let s = s
                .h(t.popup_item_height)
                .pl(t.popup_pad_x - 3.0)
                .pr(t.popup_pad_x - 3.0)
                .rounded(item_radius());
            if lit() {
                s.bg(c.selection).border_color(c.selection_border)
            } else {
                s.bg(Color::TRANSPARENT).border_color(Color::TRANSPARENT)
            }
        })
        .children(lead)
        .child(label);
    let row = if arrow {
        row.child(
            div()
                .flex_none()
                .dyn_style(move |s| {
                    s.text_size(theme().font_size)
                        .text_color(scheme().text)
                        .opacity(if enabled { 1.0 } else { DISABLED_OPACITY })
                })
                .child("\u{25B8}"),
        )
    } else {
        row
    };
    match &line.pin {
        Some((pinned, toggle)) => row.child(pin_button(*pinned, toggle.clone(), lit)),
        None => row,
    }
}

/// The pin at the end of a jump list line, shown while `shown` says so (the line is lit): a
/// press runs `toggle`, and goes no further than the pin.
fn pin_button(pinned: bool, toggle: Rc<dyn Fn()>, shown: impl Fn() -> bool + 'static) -> Div {
    let ui = Interaction::new();
    div()
        .absolute()
        .right(4.0)
        .top(2.0)
        .bottom(2.0)
        .w(22.0)
        .border(1.0)
        .flex()
        .items_center()
        .justify_center()
        .interaction(ui)
        .on_click(move |e| {
            e.stop_propagation();
            toggle();
        })
        .dyn_style(move |s| {
            let c = scheme();
            let s = s.rounded(corner(2.0));
            let s = if ui.hovered() {
                s.bg(c.hover).border_color(c.selection_border)
            } else {
                s.bg(Color::TRANSPARENT).border_color(Color::TRANSPARENT)
            };
            if shown() || ui.hovered() { s } else { s.hidden() }
        })
        .child(
            img(move || ink_asset(if pinned { "pin-on.svg" } else { "pin.svg" }))
                .size(14.0)
                .object_fit(ObjectFit::Contain),
        )
}

/// What a Start menu row looks like besides its icon and label.
#[derive(Clone, Default)]
pub struct RowLook {
    /// Pinned programs and "All Programs" are set in bold, as on Windows.
    pub bold: bool,
    /// A section title: smaller, in `menu_heading`, not lit under the pointer.
    pub heading: bool,
    /// The program has a jump list: an arrow at the end.
    pub arrow: bool,
    /// Extra room at the start, for the programs inside a folder.
    pub indent: f32,
    /// On the places pane: the label in `places_text`.
    pub places: bool,
    /// The pin of a jump list file: pinned or not, and what clicking it does.
    pub pin: Option<(bool, Rc<dyn Fn()>)>,
}

/// A row of the Start menu: an icon (a name of the icon theme or a file; "" for none) and a
/// label, lit under the pointer or while `selected` says so (the keys are on it). Its height
/// and icon size follow the "large icons" setting. The caller adds what it does
/// (`on_click`, `on_aux_click`, `draggable`).
pub fn menu_row(
    icon: impl IntoTextSource,
    label: impl IntoTextSource,
    look: RowLook,
    selected: impl Fn() -> bool + 'static,
) -> Div {
    let ui = Interaction::new();
    let heading = look.heading;
    let lit = move || !heading && (selected() || ui.hovered());
    let icon = text_fn(icon);
    let large = memo(|| settings().with(|s| s.start_menu_large_icons));
    let icon = dyn_view(move || {
        let t = theme();
        let size = if large.get() { t.menu_icon_size } else { t.menu_icon_size_small };
        icon_box(icon.clone(), size)
    });
    let words = text_fn(label);
    let (bold, places) = (look.bold, look.places);
    let label = div()
        .flex_1()
        .min_w(0.0)
        .overflow_clip()
        .whitespace_nowrap()
        .dyn_style(move |s| {
            let t = theme();
            let c = scheme();
            let s = if bold || heading { s.font_weight(FontWeight::BOLD) } else { s };
            if heading {
                s.font_family(heading_font())
                    .text_size(t.font_size_small)
                    .letter_spacing(t.heading_spacing)
                    .text_color(c.menu_heading)
            } else {
                s.font_family(font())
                    .text_size(t.font_size)
                    .text_color(if places { c.places_text } else { c.text })
            }
        })
        .child(text(move || {
            let now = words();
            if heading && theme().heading_caps {
                now.to_uppercase()
            } else {
                now
            }
        }));
    let row = div()
        .relative()
        .flex_none()
        .flex()
        .flex_row()
        .items_center()
        .gap(8.0)
        .border(1.0)
        .pl(5.0 + look.indent)
        .pr(5.0)
        .select_none()
        .interaction(ui)
        .dyn_style(move |s| {
            let t = theme();
            let c = scheme();
            let height = if large.get() { t.menu_row_height } else { t.menu_row_height_small };
            let s = s.h(height).rounded(item_radius());
            if lit() {
                s.bg(c.selection).border_color(c.selection_border)
            } else {
                s.bg(Color::TRANSPARENT).border_color(Color::TRANSPARENT)
            }
        })
        .child(icon)
        .child(label);
    let row = if look.arrow {
        row.child(
            div()
                .flex_none()
                .dyn_style(|s| s.text_size(theme().font_size).text_color(scheme().text_muted))
                .child("\u{25B8}"),
        )
    } else {
        row
    };
    match look.pin {
        Some((pinned, toggle)) => row.child(pin_button(pinned, toggle, move || ui.hovered())),
        None => row,
    }
}
