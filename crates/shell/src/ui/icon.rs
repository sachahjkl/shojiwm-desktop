//! Icons from the icon theme, sharp at any scale.
//!
//! Finding an icon's file looks at the disk (the theme's folders and indexes): the UI thread
//! never does it. [`resolved_icon`] answers from a cache and, for an icon not seen yet, asks the
//! resolver thread; the answer comes back to the UI thread and wakes what read it.

use super::text_fn;
use anti_webui::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Mutex, OnceLock};

/// An icon `size` logical pixels square: `name` from the icon theme (`firefox`), or a file
/// (`/path/to/icon.svg`, `file:///path`). The theme is asked for the size in the screen's own
/// pixels, so a theme with fixed sizes gives a picture that the scale does not blur. An empty
/// name takes no room; a name the theme lacks leaves the room empty, and so does a name not
/// resolved yet.
pub fn shell_icon(name: impl IntoTextSource, size: f32) -> Div {
    icon_box(text_fn(name), size)
}

pub(crate) fn icon_box(name: Rc<dyn Fn() -> String>, size: f32) -> Div {
    let wanted = name.clone();
    let path = memo(move || {
        let pixels = (size * window_scale().as_f32()).ceil().max(1.0) as u32;
        resolved_icon(&wanted(), pixels)
    });
    let empty = memo(move || name().is_empty());
    div()
        .flex_none()
        .size(size)
        .dyn_style(move |s| if empty.get() { s.hidden() } else { s })
        .child(dyn_view(move || {
            path.get()
                .map(|file| img(file).size(size).object_fit(ObjectFit::Contain))
        }))
}

/// The file of an icon at about `pixels` screen pixels: a path as it is, else the theme's. It
/// takes the forms Quickshell gave (`file://`, `image://icon/name?path=..`). `None` while the
/// resolver looks for it, and when there is none. Reading it tracks it: the answer wakes the
/// reader. It never looks at the disk on this thread.
pub fn resolved_icon(name: &str, pixels: u32) -> Option<PathBuf> {
    let key = IconKey::new(name, pixels)?;
    icon_signal(key).get().flatten()
}

/// What the cache knows an icon by: its name without the Quickshell forms, and the pixels.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct IconKey {
    name: String,
    pixels: u32,
}

impl IconKey {
    /// `None` for an empty name.
    fn new(name: &str, pixels: u32) -> Option<Self> {
        let name = name.trim();
        let name = name.strip_prefix("file://").unwrap_or(name);
        let name = name.strip_prefix("image://icon/").unwrap_or(name);
        let name = name.split('?').next().unwrap_or(name);
        (!name.is_empty()).then(|| Self {
            name: name.to_owned(),
            pixels: pixels.max(1),
        })
    }
}

/// `None` while the resolver looks; then what it found.
type Answer = Option<Option<PathBuf>>;

/// What the resolver sends back to a UI thread.
type Resolved = (IconKey, Option<PathBuf>);

/// The icons found, for every UI thread. Only the resolver writes it.
fn found() -> &'static Mutex<HashMap<IconKey, Option<PathBuf>>> {
    static FOUND: OnceLock<Mutex<HashMap<IconKey, Option<PathBuf>>>> = OnceLock::new();
    FOUND.get_or_init(Mutex::default)
}

/// Asks the resolver thread for `key`; the answer goes to `answer`.
fn ask(key: IconKey, answer: UiSender<Resolved>) {
    static RESOLVER: OnceLock<Sender<(IconKey, UiSender<Resolved>)>> = OnceLock::new();
    let resolver = RESOLVER.get_or_init(|| {
        let (sender, requests) = channel::<(IconKey, UiSender<Resolved>)>();
        std::thread::Builder::new()
            .name("icons".into())
            .spawn(move || {
                for (key, answer) in requests {
                    let known = found().lock().ok().and_then(|f| f.get(&key).cloned());
                    let path = known.unwrap_or_else(|| {
                        let path = anti_webui::shell::icon_path(&key.name, key.pixels);
                        if let Ok(mut f) = found().lock() {
                            f.insert(key.clone(), path.clone());
                        }
                        path
                    });
                    answer.send((key, path));
                }
            })
            .expect("the icon thread starts");
        sender
    });
    let _ = resolver.send((key, answer));
}

/// The icons of this UI thread: one signal for each, so an answer wakes only its readers.
struct Local {
    owner: Owner,
    answers: UiSender<Resolved>,
    icons: HashMap<IconKey, Signal<Answer>>,
}

impl Local {
    fn new() -> Self {
        // Under a root of its own: the cache lives as long as the thread.
        let (answers, owner) = create_root(|| {
            ui_channel(|(key, path): Resolved| {
                let icon =
                    LOCAL.with(|l| l.borrow().as_ref().and_then(|l| l.icons.get(&key).copied()));
                if let Some(icon) = icon {
                    icon.set(Some(path));
                }
            })
        });
        Self {
            owner,
            answers,
            icons: HashMap::new(),
        }
    }
}

thread_local! {
    static LOCAL: RefCell<Option<Local>> = const { RefCell::new(None) };
}

/// The signal of `key` on this thread; made, and the resolver asked, the first time.
fn icon_signal(key: IconKey) -> Signal<Answer> {
    LOCAL.with(|l| {
        let mut l = l.borrow_mut();
        let local = l.get_or_insert_with(Local::new);
        if let Some(icon) = local.icons.get(&key) {
            return *icon;
        }
        let known = found().lock().ok().and_then(|f| f.get(&key).cloned());
        let icon = local.owner.with(|| signal(known.clone()));
        local.icons.insert(key.clone(), icon);
        if known.is_none() {
            ask(key, local.answers.clone());
        }
        icon
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_forms_of_one_icon_have_one_key() {
        let plain = IconKey::new("/usr/share/icons/a.svg", 24);
        assert_eq!(IconKey::new("file:///usr/share/icons/a.svg", 24), plain);
        assert_eq!(
            IconKey::new("image://icon/firefox?path=/opt", 24),
            IconKey::new(" firefox ", 24)
        );
    }

    #[test]
    fn sizes_are_apart_and_an_empty_name_has_no_key() {
        assert_ne!(IconKey::new("firefox", 24), IconKey::new("firefox", 48));
        assert_eq!(IconKey::new("  ", 24), None);
        assert_eq!(IconKey::new("x", 0).map(|k| k.pixels), Some(1));
    }
}
