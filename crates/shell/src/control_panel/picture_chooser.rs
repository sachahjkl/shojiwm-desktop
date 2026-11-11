//! A grid of pictures to pick from, as the Windows 7 background chooser shows them (QML
//! `PictureChooser`): the built-in picture first, then the pictures in the Pictures folder and
//! in the usual backgrounds folders. A path can also be typed. And the pictures the pages show:
//! the built-in desktop picture and the account picture.

use crate::paths::asset;
use crate::settings::{dark, home, settings};
use crate::theme::{field_radius, is_kernel, item_radius, scheme, theme};
use crate::ui::{cp_button, font};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::Process;
use std::cell::OnceCell;
use std::path::PathBuf;
use std::rc::Rc;

/// The size of a cell: wide for desktop pictures, square for account pictures.
const CELL: (f32, f32) = (140.0, 88.0);
const SQUARE_CELL: f32 = 96.0;
/// The room between a cell's edge and its picture.
const CELL_PAD: f32 = 6.0;
/// The widest the field for a typed path gets.
const PATH_FIELD_WIDTH: f32 = 420.0;
/// The most pictures listed, and how deep the folders are searched.
const MOST_PICTURES: usize = 120;
const DEPTH: usize = 3;

/// The desktop's built-in picture: the theme's own. Aero's has a night version while the
/// system prefers dark; Kernel's is a dark desk under a grid of dots. Reading it tracks both.
pub fn built_in_wallpaper() -> PathBuf {
    if is_kernel() {
        asset("kernel/wallpaper.svg")
    } else if dark() {
        asset("wallpaper-dark.svg")
    } else {
        asset("wallpaper.svg")
    }
}

thread_local! {
    static SYSTEM_PICTURE: OnceCell<Option<PathBuf>> = const { OnceCell::new() };
}

/// The first readable, non-empty file of the places log-on screens read the account picture
/// from: AccountsService's icon, `~/.face`, then `~/.face.icon`. Looked for once.
fn system_picture() -> Option<PathBuf> {
    SYSTEM_PICTURE.with(|cell| {
        cell.get_or_init(|| {
            let user = std::env::var("USER").unwrap_or_default();
            let mut places = vec![PathBuf::from("/var/lib/AccountsService/icons").join(user)];
            if let Some(home) = home() {
                places.push(home.join(".face"));
                places.push(home.join(".face.icon"));
            }
            places.into_iter().find(|path| {
                std::fs::metadata(path).is_ok_and(|m| m.is_file() && m.len() > 0)
                    && std::fs::File::open(path).is_ok()
            })
        })
        .clone()
    })
}

pub use crate::account::user_picture as user_picture_source;

/// The folders searched for pictures.
fn folders() -> Vec<String> {
    let home = home().map(|h| h.display().to_string()).unwrap_or_default();
    vec![
        crate::apps::user_dir("PICTURES", "Pictures"),
        format!("{home}/.local/share/backgrounds"),
        "/run/current-system/sw/share/backgrounds".into(),
        "/usr/share/backgrounds".into(),
        "/usr/share/wallpapers".into(),
    ]
}

/// The pictures to choose from: a cell for each, the built-in one first, which stands for ""
/// (`built_in` is the file it shows). The one `selected` says is lit; a click calls
/// `on_chosen` with its path. Under them, the path chosen now, and a field to type a path in.
/// `square` cells are for account pictures.
pub fn picture_chooser(
    selected: impl Fn() -> String + 'static,
    built_in: impl Fn() -> PathBuf + 'static,
    built_in_label: &'static str,
    square: bool,
    on_chosen: impl Fn(String) + 'static,
) -> Div {
    let selected: Rc<dyn Fn() -> String> = Rc::new(selected);
    let built_in: Rc<dyn Fn() -> PathBuf> = Rc::new(built_in);
    let on_chosen: Rc<dyn Fn(String)> = Rc::new(on_chosen);
    let script = format!(
        "find \"$@\" -maxdepth {DEPTH} -type f \\( -iname '*.jpg' -o -iname '*.jpeg' \
         -o -iname '*.png' -o -iname '*.webp' -o -iname '*.svg' \\) \
         2>/dev/null | sort | head -n {MOST_PICTURES}"
    );
    let finder = Process::new("sh")
        .args(["-c".to_owned(), script, "find".to_owned()])
        .args(folders())
        .spawn();
    let pictures = memo(move || {
        finder
            .lines()
            .into_iter()
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
    });
    let cells = {
        let selected = selected.clone();
        let built_in = built_in.clone();
        let on_chosen = on_chosen.clone();
        for_each(
            move || {
                std::iter::once(String::new())
                    .chain(pictures.get())
                    .collect::<Vec<_>>()
            },
            String::clone,
            move |path: String| {
                cell(path, square, selected.clone(), built_in.clone(), on_chosen.clone())
            },
        )
    };
    let chosen_now = {
        let selected = selected.clone();
        move || {
            if pictures.with(Vec::is_empty) {
                "No picture found in your Pictures folder. Type a path below.".to_owned()
            } else {
                let now = selected();
                if now.is_empty() { built_in_label.to_owned() } else { now }
            }
        }
    };
    div()
        .flex()
        .flex_col()
        .gap(10.0)
        .child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap(10.0)
                .child(cells),
        )
        .child(
            div()
                .min_w(0.0)
                .overflow_clip()
                .whitespace_nowrap()
                .dyn_style(|s| {
                    s.font_family(font())
                        .text_size(theme().font_size_small)
                        .text_color(scheme().text_muted)
                })
                .child(text(chosen_now)),
        )
        .child(path_row(on_chosen))
}

/// A picture to choose: lit while it is the one chosen, and under the pointer.
fn cell(
    path: String,
    square: bool,
    selected: Rc<dyn Fn() -> String>,
    built_in: Rc<dyn Fn() -> PathBuf>,
    on_chosen: Rc<dyn Fn(String)>,
) -> Div {
    let ui = Interaction::new();
    let (width, height) = if square { (SQUARE_CELL, SQUARE_CELL) } else { CELL };
    let is_selected = {
        let path = path.clone();
        memo(move || selected() == path)
    };
    let source = {
        let path = path.clone();
        move || {
            if path.is_empty() { built_in() } else { PathBuf::from(&path) }
        }
    };
    div()
        .relative()
        .flex_none()
        .w(width)
        .h(height)
        .border(1.0)
        .interaction(ui)
        .on_click(move |_| on_chosen(path.clone()))
        .dyn_style(move |s| {
            let c = scheme();
            let s = s.rounded(item_radius());
            if is_selected.get() {
                s.bg(c.selection).border_color(c.selection_border)
            } else if ui.hovered() {
                s.bg(c.hover).border_color(c.selection_border)
            } else {
                s.bg(Color::TRANSPARENT).border_color(Color::TRANSPARENT)
            }
        })
        .child(
            img(source)
                .absolute()
                .inset(CELL_PAD - 1.0)
                .object_fit(ObjectFit::Cover)
                .overflow_clip(),
        )
}

/// A field to type a picture's path in, and its button. `~/` is the home folder.
fn path_row(on_chosen: Rc<dyn Fn(String)>) -> Div {
    let typed = signal(String::new());
    let focused = signal(false);
    let use_path = Rc::new(move || {
        let path = typed.get_untracked().trim().to_owned();
        let path = match (path.strip_prefix("~/"), home()) {
            (Some(rest), Some(home)) => home.join(rest).display().to_string(),
            _ => path,
        };
        if path.is_empty() {
            return;
        }
        on_chosen(path);
        typed.set(String::new());
    });
    let on_submit = use_path.clone();
    let field = div()
        .flex_1()
        .min_w(0.0)
        .max_w(PATH_FIELD_WIDTH)
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
        })
        .child(
            text_input()
                .value(typed)
                .placeholder("/path/to/picture.jpg")
                .flex_1()
                .min_w(0.0)
                .on_focus_change(move |on| focused.set(on))
                .on_submit(move |_| on_submit())
                .dyn_style(|s| {
                    let c = scheme();
                    s.font_family(font())
                        .text_size(theme().font_size)
                        .text_color(c.text)
                        .caret_color(c.text)
                }),
        );
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(8.0)
        .child(field)
        .child(cp_button("Use this", move || use_path()))
}
