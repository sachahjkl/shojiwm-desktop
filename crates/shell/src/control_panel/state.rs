//! Whether the Control Panel is open and on which page, the pages it has, and the tasks its
//! search box and the Start menu's find (QML `ControlPanelState`).

use anti_webui::prelude::*;
use std::cell::{Cell, RefCell};

/// A page of the Control Panel: its key, its title and the category it belongs to, as the
/// address bar shows them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Page {
    pub key: &'static str,
    pub title: &'static str,
    pub category: &'static str,
}

/// Every page, home first.
pub const PAGES: &[Page] = &[
    Page { key: "home", title: "Control Panel Home", category: "" },
    Page { key: "appearance", title: "Light, dark and glass", category: "Appearance and Personalization" },
    Page { key: "background", title: "Desktop Background", category: "Appearance and Personalization" },
    Page { key: "lockscreen", title: "Lock Screen", category: "Appearance and Personalization" },
    Page { key: "account", title: "Account Picture", category: "User Accounts" },
    Page { key: "taskbar", title: "Taskbar and Start Menu", category: "Appearance and Personalization" },
    Page { key: "notifications", title: "Notification Area Icons", category: "Appearance and Personalization" },
    Page { key: "clock", title: "Date and Time", category: "Clock and Region" },
    Page { key: "ease", title: "Ease of Access", category: "Ease of Access" },
    Page { key: "startup", title: "Startup Programs", category: "System" },
    Page { key: "defaults", title: "Default Programs", category: "Programs" },
    Page { key: "sound", title: "Sound", category: "Hardware and Sound" },
    Page { key: "devices", title: "Devices and Printers", category: "Hardware and Sound" },
    Page { key: "display", title: "Screen Resolution", category: "Appearance and Personalization" },
    Page { key: "about", title: "System", category: "System" },
];

/// The pages the pane of links lists, in its order.
pub const SIDE_PAGES: &[&str] = &[
    "appearance",
    "background",
    "display",
    "lockscreen",
    "account",
    "taskbar",
    "notifications",
    "clock",
    "ease",
    "devices",
    "sound",
    "defaults",
    "startup",
    "about",
];

/// The page of `key`; the home page for a key that is not one.
pub fn page_info(key: &str) -> Page {
    PAGES.iter().copied().find(|p| p.key == key).unwrap_or(PAGES[0])
}

/// A task one of the pages holds, and the words it is found by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Task {
    pub text: &'static str,
    pub page: &'static str,
    pub words: &'static str,
}

/// Every task the pages hold: the Control Panel's search box and the Start menu's find them.
pub const TASKS: &[Task] = &[
    Task { text: "Choose light or dark", page: "appearance", words: "dark light theme colour color scheme night appearance" },
    Task { text: "Change the colour of the taskbar and the Start menu", page: "appearance", words: "window color colour glass tint taskbar start menu intensity personalize" },
    Task { text: "Change how much the glass blurs", page: "appearance", words: "glass blur transparency aero frosted taskbar window colour color" },
    Task { text: "Change the desktop background on its own (slide show)", page: "background", words: "slide show slideshow wallpaper change rotate shuffle background folder" },
    Task { text: "Change desktop background", page: "background", words: "wallpaper picture image desktop background" },
    Task { text: "Change the size of desktop icons", page: "taskbar", words: "desktop icons size large small medium zoom" },
    Task { text: "Lock screen and its picture", page: "lockscreen", words: "lock screen picture password idle delay suspend sleep" },
    Task { text: "Change the screen saver", page: "lockscreen", words: "screen saver screensaver mystify idle wait preview" },
    Task { text: "Lock the computer after a delay", page: "lockscreen", words: "lock idle minutes delay timeout screensaver" },
    Task { text: "Change your account picture", page: "account", words: "account user picture avatar face photo profile" },
    Task { text: "Show or hide common icons on the desktop", page: "taskbar", words: "desktop icons computer user files control panel recycle bin show hide" },
    Task { text: "Combine taskbar buttons", page: "taskbar", words: "taskbar combine group buttons windows" },
    Task { text: "Show labels on taskbar buttons", page: "taskbar", words: "taskbar labels compact icons text" },
    Task { text: "Use Aero Peek to preview the desktop", page: "taskbar", words: "aero peek preview desktop show" },
    Task { text: "Show the programs used most in the Start menu", page: "taskbar", words: "start menu frequent recent programs" },
    Task { text: "Change what the power button does", page: "taskbar", words: "power button shut down shutdown restart sleep hibernate log off lock" },
    Task { text: "Select which icons appear on the taskbar", page: "notifications", words: "notification area tray icons hidden show" },
    Task { text: "Show clocks for other time zones", page: "clock", words: "additional clocks time zone world clock other city" },
    Task { text: "Change the clock format", page: "clock", words: "clock time date format 24 12 hour" },
    Task { text: "See the time zone and time synchronization", page: "clock", words: "time zone timezone ntp synchronization sync server internet" },
    Task { text: "Turn off all unnecessary animations", page: "ease", words: "animations motion reduce reduced effects ease access accessibility" },
    Task { text: "Change the animation speed", page: "ease", words: "animation speed fast faster slow slower motion duration" },
    Task { text: "Change the screen resolution and scale", page: "display", words: "display screen monitor resolution scale size dpi refresh rate hz mode text bigger" },
    Task { text: "View devices and printers", page: "devices", words: "devices printers usb hardware keyboard mouse webcam printer monitor" },
    Task { text: "Manage audio devices", page: "sound", words: "sound audio speakers headphones microphone recording playback default device output input" },
    Task { text: "Choose the programs that open your files and links", page: "defaults", words: "default programs apps browser web mail email open with file association mime pdf pictures music video folder" },
    Task { text: "Choose the programs that start when you log on", page: "startup", words: "startup start log on login boot autostart programs run session" },
    Task { text: "About this desktop", page: "about", words: "about system version information" },
];

/// The words of `query`, in lower case.
fn words(query: &str) -> Vec<String> {
    query.split_whitespace().map(str::to_lowercase).collect()
}

/// The tasks whose text and words hold every word of `query`, for the Start menu. None for an
/// empty query.
pub fn find_tasks(query: &str) -> Vec<Task> {
    let wanted = words(query);
    if wanted.is_empty() {
        return Vec::new();
    }
    TASKS
        .iter()
        .copied()
        .filter(|task| {
            let haystack = format!("{} {}", task.text, task.words).to_lowercase();
            wanted.iter().all(|w| haystack.contains(w.as_str()))
        })
        .collect()
}

/// The tasks for the Control Panel's own search box: as [`find_tasks`], and the title and the
/// category of the task's page count too.
pub fn search(query: &str) -> Vec<Task> {
    let wanted = words(query);
    if wanted.is_empty() {
        return Vec::new();
    }
    TASKS
        .iter()
        .copied()
        .filter(|task| {
            let page = page_info(task.page);
            let haystack =
                format!("{} {} {} {}", task.text, task.words, page.title, page.category)
                    .to_lowercase();
            wanted.iter().all(|w| haystack.contains(w.as_str()))
        })
        .collect()
}

thread_local! {
    static PAGE: Cell<Option<Signal<String>>> = const { Cell::new(None) };
    static QUERY: Cell<Option<Signal<String>>> = const { Cell::new(None) };
    static WINDOW: RefCell<Option<WindowHandle>> = const { RefCell::new(None) };
}

/// A signal of the shell's life, made under a root of its own the first time it is asked for.
fn lasting(cell: &'static std::thread::LocalKey<Cell<Option<Signal<String>>>>, first: &str) -> Signal<String> {
    cell.with(|c| {
        c.get().unwrap_or_else(|| {
            let first = first.to_owned();
            let (s, _) = create_root(move || signal(first));
            c.set(Some(s));
            s
        })
    })
}

fn page_signal() -> Signal<String> {
    lasting(&PAGE, "home")
}

/// The page shown, by key. Reading it tracks it.
pub fn page() -> String {
    page_signal().get()
}

/// Shows the page `key`; an empty or unknown key shows the home page.
pub fn set_page(key: &str) {
    let key = page_info(key).key;
    page_signal().set(key.to_owned());
}

/// What is typed in the search box. While it holds a word, the search results replace the page.
pub fn query() -> Signal<String> {
    lasting(&QUERY, "")
}

/// Goes to a page: the search box empties, and the page shows from its top.
pub fn go(key: &str) {
    batch(|| {
        query().set(String::new());
        set_page(key);
    });
}

/// The window, while it is open.
pub fn window() -> Option<WindowHandle> {
    WINDOW.with(|w| w.borrow().clone().filter(WindowHandle::is_open))
}

pub fn set_window(handle: Option<WindowHandle>) {
    WINDOW.with(|w| *w.borrow_mut() = handle);
}
