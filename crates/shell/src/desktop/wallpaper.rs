//! The desktop's picture (the wallpaper layer of `shell.qml`) and the slide show of Windows 7's
//! Desktop Background (QML `Slideshow`).
//!
//! The picture is the one chosen in the Control Panel, else the theme's built-in one (Settings.qml
//! `builtInWallpaper`): Aero's, lighter or darker with the system, or Kernel's dark desk under a
//! grid of dots. It is placed as the "Picture position" says: fill, fit, stretch, tile or center.

use crate::paths::asset;
use crate::settings::{change, dark, settings};
use crate::theme::is_kernel;
use anti_webui::prelude::*;
use std::cell::Cell;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Under a picture that does not cover the screen (fit, center): the dark blue of Windows 7's
/// background colour.
pub const BACKDROP: Color = rgb(0x1d3550);

/// The pictures a slide show takes.
const SLIDE_SUFFIXES: &[&str] = &["jpg", "jpeg", "png", "webp"];

/// The theme's own picture.
pub fn built_in() -> PathBuf {
    if is_kernel() {
        asset("kernel/wallpaper.svg")
    } else if dark() {
        asset("wallpaper-dark.svg")
    } else {
        asset("wallpaper.svg")
    }
}

/// The desktop's picture now. Reading it tracks the settings and the theme.
pub fn source() -> PathBuf {
    let chosen = settings().with(|s| s.wallpaper.clone());
    if chosen.is_empty() { built_in() } else { PathBuf::from(chosen) }
}

/// The picture, the size of the desktop, placed as the settings say.
pub fn view() -> View {
    let position = memo(|| settings().with(|s| s.wallpaper_position.clone()));
    dyn_view(move || {
        let picture = || img(source).absolute().inset(0.0).size_full();
        match position.get().as_str() {
            // Tile and center keep the picture's own size; tiles start at the top left.
            "tile" => div()
                .absolute()
                .inset(0.0)
                .bg_repeat(BackgroundRepeat::Repeat)
                .bg_size(BackgroundSize::Auto)
                .bg_position(px(0.0), px(0.0))
                .dyn_style(|s| s.bg_image(source()))
                .into_view(),
            "center" => picture().object_fit(ObjectFit::None).into_view(),
            "fit" => picture().object_fit(ObjectFit::Contain).into_view(),
            "stretch" => picture().object_fit(ObjectFit::Fill).into_view(),
            _ => picture().object_fit(ObjectFit::Cover).into_view(),
        }
    })
}

// ----- the slide show --------------------------------------------------------------------------

thread_local! {
    static SLIDESHOW: Cell<bool> = const { Cell::new(false) };
}

/// Starts the slide show: while a folder is chosen, its pictures become the wallpaper in turn,
/// every few minutes, in name order or shuffled. The folder is read again at each change, so
/// pictures added to it join the show. Starting a show, or changing its folder, shows a picture
/// of it at once. Call it once, from the shell's root.
pub fn install_slideshow() {
    if SLIDESHOW.with(|s| s.replace(true)) {
        return;
    }
    create_root(|| {
        let folder = memo(|| settings().with(|s| s.slideshow_folder.clone()));
        let minutes = memo(|| settings().with(|s| s.slideshow_minutes.max(1)));
        // A new folder or a new interval starts the turns again; a new folder shows at once.
        let turn: std::rc::Rc<Cell<Option<Timeout>>> = std::rc::Rc::default();
        // The folder of the show when the shell starts: its picture is already the wallpaper.
        let shown_folder = std::rc::Rc::new(std::cell::RefCell::new(untrack(|| folder.get())));
        effect(move || {
            let now = folder.get();
            let every = Duration::from_secs(minutes.get() as u64 * 60);
            if let Some(t) = turn.take() {
                t.cancel();
            }
            if now.is_empty() {
                shown_folder.borrow_mut().clear();
                return;
            }
            if *shown_folder.borrow() != now {
                *shown_folder.borrow_mut() = now.clone();
                next();
            }
            schedule(turn.clone(), every);
        });
    });
}

/// The next turn, `every` from now, and the one after it.
fn schedule(turn: std::rc::Rc<Cell<Option<Timeout>>>, every: Duration) {
    let again = turn.clone();
    turn.set(Some(set_timeout(every, move || {
        next();
        schedule(again, every);
    })));
}

/// Shows the next picture of the show now.
pub fn next() {
    let (folder, shuffle, current) = untrack(|| {
        settings().with(|s| (s.slideshow_folder.clone(), s.slideshow_shuffle, s.wallpaper.clone()))
    });
    if folder.is_empty() {
        return;
    }
    let mut pictures: Vec<String> = std::fs::read_dir(&folder)
        .map(|dir| {
            dir.flatten()
                .map(|e| e.path())
                .filter(|p| p.is_file())
                .filter(|p| {
                    p.extension()
                        .and_then(|e| e.to_str())
                        .is_some_and(|e| SLIDE_SUFFIXES.contains(&e.to_lowercase().as_str()))
                })
                .map(|p| p.to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    if pictures.is_empty() {
        return;
    }
    pictures.sort();
    let at = pictures.iter().position(|p| *p == current);
    let index = if shuffle && pictures.len() > 1 {
        // Any other picture than the one showing.
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.subsec_nanos() as usize ^ d.as_secs() as usize);
        let pick = seed % (pictures.len() - 1);
        match at {
            Some(i) if pick >= i => pick + 1,
            _ => pick,
        }
    } else {
        at.map_or(0, |i| (i + 1) % pictures.len())
    };
    let chosen = pictures.swap_remove(index);
    change(|s| s.wallpaper = chosen);
}
