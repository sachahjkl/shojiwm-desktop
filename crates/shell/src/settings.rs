//! What the Control Panel changes, kept in `~/.config/shojiwm/settings.json`: beside the
//! installed configuration, which an update replaces, so the settings survive it. The keys and
//! their defaults are the QML shell's (`shell/Settings.qml`), so both read the same file while
//! the shell moves over. The file is watched: editing it by hand works too.
//!
//! The light or dark choice is not kept here: it is the system's own preference (dconf's
//! `color-scheme`), which the programs read as well ([`dark`], [`set_dark`]).

use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use serde_json::{Map, Value, json};
use std::cell::Cell;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// A place on the desktop, from the right and top edges of the screen less the bar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Place {
    pub right: f32,
    pub top: f32,
}

/// One of the Additional Clocks.
#[derive(Clone, Debug, PartialEq)]
pub struct ExtraClock {
    /// An IANA zone, as `America/New_York`.
    pub zone: String,
    pub label: String,
}

/// Every setting of the shell. The field names are the file's keys in snake_case.
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    /// Absolute paths; "" for the built-in picture. For the lock screen, "" is the desktop's
    /// picture and "builtin" the built-in one.
    pub wallpaper: String,
    pub lock_background: String,
    pub user_picture: String,
    pub combine_windows: bool,
    /// Taskbar buttons show the icon alone, as Windows 7's "hide labels".
    pub taskbar_compact: bool,
    /// The blur behind the glass: 0 for none, up to 4.
    pub glass_blur: i64,
    /// The square variant of Aero.
    pub square_corners: bool,
    /// "aero" or "kernel".
    pub theme: String,
    /// The programs the desktop starts, by key (terminal, fileManager, taskManager, clipboard,
    /// screenshot, ...): command lines. A key missing or empty: the session's.
    pub programs: Map<String, Value>,
    /// 1 is normal, 2 twice as fast.
    pub animation_speed: f64,
    /// Mode and scale per screen, by connector: `{ "DP-2": { width, height, refreshRate, scale } }`.
    pub display_choices: Map<String, Value>,
    pub extra_clocks: Vec<ExtraClock>,
    pub show_cpu_meter: bool,
    pub cpu_meter_place: Place,
    pub show_clock_gadget: bool,
    pub clock_gadget_place: Place,
    pub slideshow_folder: String,
    pub slideshow_minutes: i64,
    pub slideshow_shuffle: bool,
    /// Files pinned to a program's jump list, by .desktop file name.
    pub jump_pins: Map<String, Value>,
    /// "computer", "userfiles", "controlpanel".
    pub desktop_system_icons: Vec<String>,
    /// "" for the plain glass, else a colour laid over it at `glass_color_strength`.
    pub glass_color: String,
    pub glass_color_strength: f64,
    /// "fill", "fit", "stretch", "tile" or "center".
    pub wallpaper_position: String,
    /// "none" or "mystify".
    pub screensaver: String,
    pub screensaver_minutes: i64,
    pub taskbar_auto_hide: bool,
    pub start_menu_large_icons: bool,
    pub show_frequent: bool,
    pub launcher_recent_count: i64,
    pub launcher_width: i64,
    pub launcher_height: i64,
    /// 32 to 256.
    pub desktop_icon_size: i64,
    pub show_desktop_icons: bool,
    /// "name", "size", "type" or "date".
    pub desktop_sort: String,
    /// By connector name; "" for the first screen.
    pub primary_screen: String,
    pub last_color_scheme: String,
    pub desktop_auto_arrange: bool,
    pub desktop_align_to_grid: bool,
    /// Each desktop icon's place, by key: `{ screen, x, y }`.
    pub desktop_places: Map<String, Value>,
    /// "shutdown", "restart", "sleep", "hibernate", "lock" or "logoff".
    pub power_button_action: String,
    pub shutdown_dialog_action: String,
    pub peek: bool,
    /// "locale", "24h" or "12h".
    pub clock_format: String,
    pub show_date: bool,
    /// 0 for never.
    pub lock_after_minutes: i64,
    pub lock_on_suspend: bool,
    pub tray_show_all: bool,
    pub tray_shown: Vec<String>,
    pub tray_hidden: Vec<String>,
    /// .desktop file names, in order.
    pub start_pinned: Vec<String>,
    pub taskbar_pinned: Vec<String>,
    /// Keys this shell does not know, kept as they are.
    pub rest: Map<String, Value>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            wallpaper: String::new(),
            lock_background: String::new(),
            user_picture: String::new(),
            combine_windows: true,
            taskbar_compact: true,
            glass_blur: 2,
            square_corners: false,
            theme: "aero".into(),
            programs: Map::new(),
            animation_speed: 1.0,
            display_choices: Map::new(),
            extra_clocks: Vec::new(),
            show_cpu_meter: true,
            cpu_meter_place: Place { right: 24.0, top: 24.0 },
            show_clock_gadget: true,
            clock_gadget_place: Place { right: 52.0, top: 168.0 },
            slideshow_folder: String::new(),
            slideshow_minutes: 30,
            slideshow_shuffle: false,
            jump_pins: Map::new(),
            desktop_system_icons: vec!["computer".into()],
            glass_color: String::new(),
            glass_color_strength: 0.35,
            wallpaper_position: "fill".into(),
            screensaver: "none".into(),
            screensaver_minutes: 5,
            taskbar_auto_hide: false,
            start_menu_large_icons: false,
            show_frequent: true,
            launcher_recent_count: 25,
            launcher_width: 560,
            launcher_height: 520,
            desktop_icon_size: 48,
            show_desktop_icons: true,
            desktop_sort: "name".into(),
            primary_screen: String::new(),
            last_color_scheme: "default".into(),
            desktop_auto_arrange: false,
            desktop_align_to_grid: true,
            desktop_places: Map::new(),
            power_button_action: "shutdown".into(),
            shutdown_dialog_action: "logoff".into(),
            peek: true,
            clock_format: "locale".into(),
            show_date: true,
            lock_after_minutes: 10,
            lock_on_suspend: true,
            tray_show_all: false,
            tray_shown: Vec::new(),
            tray_hidden: Vec::new(),
            start_pinned: ["kitty.desktop", "thunar.desktop", "chromium.desktop"]
                .map(String::from)
                .to_vec(),
            taskbar_pinned: ["thunar.desktop", "chromium.desktop", "kitty.desktop"]
                .map(String::from)
                .to_vec(),
            rest: Map::new(),
        }
    }
}

/// The keys of the file this shell reads, in its order.
const KEYS: &[&str] = &[
    "wallpaper",
    "lockBackground",
    "userPicture",
    "combineWindows",
    "taskbarCompact",
    "glassBlur",
    "squareCorners",
    "theme",
    "programs",
    "animationSpeed",
    "displayChoices",
    "extraClocks",
    "showCpuMeter",
    "cpuMeterPlace",
    "showClockGadget",
    "clockGadgetPlace",
    "slideshowFolder",
    "slideshowMinutes",
    "slideshowShuffle",
    "jumpPins",
    "desktopSystemIcons",
    "glassColor",
    "glassColorStrength",
    "wallpaperPosition",
    "screensaver",
    "screensaverMinutes",
    "taskbarAutoHide",
    "startMenuLargeIcons",
    "showFrequent",
    "launcherRecentCount",
    "launcherWidth",
    "launcherHeight",
    "desktopIconSize",
    "showDesktopIcons",
    "desktopSort",
    "primaryScreen",
    "lastColorScheme",
    "desktopAutoArrange",
    "desktopAlignToGrid",
    "desktopPlaces",
    "powerButtonAction",
    "shutdownDialogAction",
    "peek",
    "clockFormat",
    "showDate",
    "lockAfterMinutes",
    "lockOnSuspend",
    "trayShowAll",
    "trayShown",
    "trayHidden",
    "startPinned",
    "taskbarPinned",
];

fn strings(v: &Value) -> Option<Vec<String>> {
    v.as_array()
        .map(|list| list.iter().filter_map(|x| x.as_str().map(String::from)).collect())
}

fn place(v: &Value) -> Option<Place> {
    Some(Place {
        right: v.get("right")?.as_f64()? as f32,
        top: v.get("top")?.as_f64()? as f32,
    })
}

impl Settings {
    /// What is in the file over the defaults: a key that is missing or of the wrong type keeps
    /// its default.
    pub fn from_json(v: &Value) -> Self {
        let mut s = Self::default();
        let Some(obj) = v.as_object() else {
            return s;
        };
        let st = |k: &str| obj.get(k).and_then(Value::as_str).map(String::from);
        let b = |k: &str| obj.get(k).and_then(Value::as_bool);
        let i = |k: &str| obj.get(k).and_then(Value::as_i64);
        let f = |k: &str| obj.get(k).and_then(Value::as_f64);
        let m = |k: &str| obj.get(k).and_then(Value::as_object).cloned();
        let l = |k: &str| obj.get(k).and_then(strings);
        macro_rules! set {
            ($field:ident, $get:expr) => {
                if let Some(value) = $get {
                    s.$field = value;
                }
            };
        }
        set!(wallpaper, st("wallpaper"));
        set!(lock_background, st("lockBackground"));
        set!(user_picture, st("userPicture"));
        set!(combine_windows, b("combineWindows"));
        set!(taskbar_compact, b("taskbarCompact"));
        set!(glass_blur, i("glassBlur"));
        set!(square_corners, b("squareCorners"));
        set!(theme, st("theme"));
        set!(programs, m("programs"));
        set!(animation_speed, f("animationSpeed"));
        set!(display_choices, m("displayChoices"));
        if let Some(list) = obj.get("extraClocks").and_then(Value::as_array) {
            s.extra_clocks = list
                .iter()
                .filter_map(|c| {
                    Some(ExtraClock {
                        zone: c.get("zone")?.as_str()?.into(),
                        label: c.get("label").and_then(Value::as_str).unwrap_or("").into(),
                    })
                })
                .collect();
        }
        set!(show_cpu_meter, b("showCpuMeter"));
        set!(cpu_meter_place, obj.get("cpuMeterPlace").and_then(place));
        set!(show_clock_gadget, b("showClockGadget"));
        set!(clock_gadget_place, obj.get("clockGadgetPlace").and_then(place));
        set!(slideshow_folder, st("slideshowFolder"));
        set!(slideshow_minutes, i("slideshowMinutes"));
        set!(slideshow_shuffle, b("slideshowShuffle"));
        set!(jump_pins, m("jumpPins"));
        set!(desktop_system_icons, l("desktopSystemIcons"));
        set!(glass_color, st("glassColor"));
        set!(glass_color_strength, f("glassColorStrength"));
        set!(wallpaper_position, st("wallpaperPosition"));
        set!(screensaver, st("screensaver"));
        set!(screensaver_minutes, i("screensaverMinutes"));
        set!(taskbar_auto_hide, b("taskbarAutoHide"));
        set!(start_menu_large_icons, b("startMenuLargeIcons"));
        set!(show_frequent, b("showFrequent"));
        set!(launcher_recent_count, i("launcherRecentCount"));
        set!(launcher_width, i("launcherWidth"));
        set!(launcher_height, i("launcherHeight"));
        set!(desktop_icon_size, i("desktopIconSize"));
        set!(show_desktop_icons, b("showDesktopIcons"));
        set!(desktop_sort, st("desktopSort"));
        set!(primary_screen, st("primaryScreen"));
        set!(last_color_scheme, st("lastColorScheme"));
        set!(desktop_auto_arrange, b("desktopAutoArrange"));
        set!(desktop_align_to_grid, b("desktopAlignToGrid"));
        set!(desktop_places, m("desktopPlaces"));
        set!(power_button_action, st("powerButtonAction"));
        set!(shutdown_dialog_action, st("shutdownDialogAction"));
        set!(peek, b("peek"));
        set!(clock_format, st("clockFormat"));
        set!(show_date, b("showDate"));
        set!(lock_after_minutes, i("lockAfterMinutes"));
        set!(lock_on_suspend, b("lockOnSuspend"));
        set!(tray_show_all, b("trayShowAll"));
        set!(tray_shown, l("trayShown"));
        set!(tray_hidden, l("trayHidden"));
        set!(start_pinned, l("startPinned"));
        set!(taskbar_pinned, l("taskbarPinned"));
        s.rest = obj
            .iter()
            .filter(|(k, _)| !KEYS.contains(&k.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        s
    }

    pub fn to_json(&self) -> Value {
        let place = |p: &Place| json!({ "right": p.right, "top": p.top });
        let mut v = json!({
            "wallpaper": self.wallpaper,
            "lockBackground": self.lock_background,
            "userPicture": self.user_picture,
            "combineWindows": self.combine_windows,
            "taskbarCompact": self.taskbar_compact,
            "glassBlur": self.glass_blur,
            "squareCorners": self.square_corners,
            "theme": self.theme,
            "programs": self.programs,
            "animationSpeed": self.animation_speed,
            "displayChoices": self.display_choices,
            "extraClocks": self.extra_clocks.iter()
                .map(|c| json!({ "zone": c.zone, "label": c.label }))
                .collect::<Vec<_>>(),
            "showCpuMeter": self.show_cpu_meter,
            "cpuMeterPlace": place(&self.cpu_meter_place),
            "showClockGadget": self.show_clock_gadget,
            "clockGadgetPlace": place(&self.clock_gadget_place),
            "slideshowFolder": self.slideshow_folder,
            "slideshowMinutes": self.slideshow_minutes,
            "slideshowShuffle": self.slideshow_shuffle,
            "jumpPins": self.jump_pins,
            "desktopSystemIcons": self.desktop_system_icons,
            "glassColor": self.glass_color,
            "glassColorStrength": self.glass_color_strength,
            "wallpaperPosition": self.wallpaper_position,
            "screensaver": self.screensaver,
            "screensaverMinutes": self.screensaver_minutes,
            "taskbarAutoHide": self.taskbar_auto_hide,
            "startMenuLargeIcons": self.start_menu_large_icons,
            "showFrequent": self.show_frequent,
            "launcherRecentCount": self.launcher_recent_count,
            "launcherWidth": self.launcher_width,
            "launcherHeight": self.launcher_height,
            "desktopIconSize": self.desktop_icon_size,
            "showDesktopIcons": self.show_desktop_icons,
            "desktopSort": self.desktop_sort,
            "primaryScreen": self.primary_screen,
            "lastColorScheme": self.last_color_scheme,
            "desktopAutoArrange": self.desktop_auto_arrange,
            "desktopAlignToGrid": self.desktop_align_to_grid,
            "desktopPlaces": self.desktop_places,
            "powerButtonAction": self.power_button_action,
            "shutdownDialogAction": self.shutdown_dialog_action,
            "peek": self.peek,
            "clockFormat": self.clock_format,
            "showDate": self.show_date,
            "lockAfterMinutes": self.lock_after_minutes,
            "lockOnSuspend": self.lock_on_suspend,
            "trayShowAll": self.tray_show_all,
            "trayShown": self.tray_shown,
            "trayHidden": self.tray_hidden,
            "startPinned": self.start_pinned,
            "taskbarPinned": self.taskbar_pinned,
        });
        if let Some(obj) = v.as_object_mut() {
            for (k, value) in &self.rest {
                obj.insert(k.clone(), value.clone());
            }
        }
        v
    }

    /// `list` without `id`, and with it at the end when `on` (Settings.qml's `withMember`).
    pub fn with_member(list: &[String], id: &str, on: bool) -> Vec<String> {
        let mut next: Vec<String> = list.iter().filter(|x| *x != id).cloned().collect();
        if on {
            next.push(id.to_owned());
        }
        next
    }
}

/// `$XDG_CONFIG_HOME/shojiwm/settings.json`.
pub fn path() -> PathBuf {
    config_home().join("shojiwm").join("settings.json")
}

pub fn config_home() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| home().map(|h| h.join(".config")))
        .unwrap_or_else(|| PathBuf::from("/tmp"))
}

pub fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn modified() -> Option<SystemTime> {
    std::fs::metadata(path()).and_then(|m| m.modified()).ok()
}

fn load() -> Option<Settings> {
    let text = std::fs::read_to_string(path()).ok()?;
    let value = serde_json::from_str::<Value>(&text).ok()?;
    Some(Settings::from_json(&value))
}

fn save(settings: &Settings) {
    let path = path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let text = serde_json::to_string_pretty(&settings.to_json()).unwrap_or_default();
    // Written next to it and moved over it, so that a crash never leaves half a file.
    let partial = path.with_extension("json.partial");
    if std::fs::write(&partial, text + "\n").is_ok() {
        let _ = std::fs::rename(&partial, &path);
    }
}

thread_local! {
    static SETTINGS: Cell<Option<Signal<Settings>>> = const { Cell::new(None) };
    static SAVING: Cell<bool> = const { Cell::new(true) };
}

/// Keeps changes from being written, for pictures of the shell (`preview`).
pub fn never_save() {
    SAVING.with(|s| s.set(false));
}

/// How often the file is looked at for changes made by hand or by another program.
const WATCH: Duration = Duration::from_millis(700);

/// The settings. Reading them tracks them; writing them (`settings().update(..)`) saves them a
/// moment later. A change to the file from elsewhere is read back.
pub fn settings() -> Signal<Settings> {
    SETTINGS.with(|cell| {
        cell.get().unwrap_or_else(|| {
            let (s, _) = create_root(|| {
                let first = load();
                let missing = first.is_none();
                let s = signal(first.unwrap_or_default());
                // The first run: write the defaults out, so there is a file to edit.
                if missing && SAVING.with(Cell::get) {
                    save(&s.get_untracked());
                }
                let seen: std::rc::Rc<Cell<Option<SystemTime>>> =
                    std::rc::Rc::new(Cell::new(modified()));
                // Saved once things have settled, not on every step of a slider.
                let pending: Cell<Option<Timeout>> = Cell::new(None);
                let skip = std::rc::Rc::new(Cell::new(true));
                {
                    let seen = seen.clone();
                    let skip = skip.clone();
                    effect(move || {
                        let now = s.get();
                        if skip.replace(false) || !SAVING.with(Cell::get) {
                            return;
                        }
                        if let Some(t) = pending.take() {
                            t.cancel();
                        }
                        let seen = seen.clone();
                        pending.set(Some(set_timeout(Duration::from_millis(300), move || {
                            save(&now);
                            seen.set(modified());
                        })));
                    });
                }
                watch(s, seen, skip);
                s
            });
            cell.set(Some(s));
            s
        })
    })
}

/// Reads the file again whenever its time changes and it is not this shell's own write.
fn watch(s: Signal<Settings>, seen: std::rc::Rc<Cell<Option<SystemTime>>>, skip: std::rc::Rc<Cell<bool>>) {
    set_timeout(WATCH, move || {
        let now = modified();
        if now.is_some() && now != seen.get() {
            seen.set(now);
            if let Some(read) = load() {
                if read != s.get_untracked() {
                    // Not written back: it is what the file says already.
                    skip.set(true);
                    s.set(read);
                }
            }
        }
        watch(s, seen, skip);
    });
}

/// `settings().update(f)`, for short.
pub fn change(f: impl FnOnce(&mut Settings)) {
    settings().update(f);
}

// ----- light or dark ---------------------------------------------------------------------------

const SCHEME_KEY: &str = "/org/gnome/desktop/interface/color-scheme";
const GTK_THEME_KEY: &str = "/org/gnome/desktop/interface/gtk-theme";
const ANIMATIONS_KEY: &str = "/org/gnome/desktop/interface/enable-animations";

thread_local! {
    static DARK: Cell<Option<Signal<bool>>> = const { Cell::new(None) };
    static ANIMATIONS: Cell<Option<Signal<bool>>> = const { Cell::new(None) };
}

/// `'prefer-dark'` (with its quotes, as dconf prints it) to the scheme's name.
fn parse_scheme(line: &str) -> Option<String> {
    let start = line.find('\'')? + 1;
    let end = start + line[start..].find('\'')?;
    Some(line[start..end].to_owned())
}

/// A dconf key, followed: `on_value` gets each line of `dconf read`, then of `dconf watch`.
fn follow_dconf(key: &'static str, on_value: impl Fn(&str) + Clone + 'static) {
    let read = Process::new("dconf").args(["read", key]).spawn();
    {
        let on_value = on_value.clone();
        read.on_line(move |line| on_value(line));
    }
    let watch = Process::new("dconf").args(["watch", key]).spawn();
    watch.on_line(move |line| on_value(line));
    // The processes live as long as the root they were made in: the whole shell.
    std::mem::forget(read);
    std::mem::forget(watch);
}

/// The system prefers dark. It starts as the last one seen, kept in the settings, so the shell
/// does not show light for a moment until dconf answers. Reading it tracks it.
pub fn dark() -> bool {
    dark_signal().get()
}

fn dark_signal() -> Signal<bool> {
    DARK.with(|cell| {
        cell.get().unwrap_or_else(|| {
            let (d, _) = create_root(|| {
                let d = signal(settings().get_untracked().last_color_scheme == "prefer-dark");
                follow_dconf(SCHEME_KEY, move |line| {
                    if let Some(scheme) = parse_scheme(line) {
                        d.set(scheme == "prefer-dark");
                        if settings().get_untracked().last_color_scheme != scheme {
                            change(|s| s.last_color_scheme = scheme);
                        }
                    }
                });
                d
            });
            cell.set(Some(d));
            d
        })
    })
}

/// Sets the preference for every program: the colour scheme (GTK 4, libadwaita, Qt, the
/// browsers) and, for GTK 3, the light or dark variant of the GTK theme in use (`gtk-switch.sh`).
pub fn set_dark(on: bool) {
    dark_signal().set(on);
    let value = if on { "'prefer-dark'" } else { "'default'" };
    let _ = std::process::Command::new("dconf")
        .args(["write", SCHEME_KEY, value])
        .spawn();
    if let Some(script) = crate::paths::shell_file("gtk-switch.sh") {
        let _ = std::process::Command::new("sh")
            .arg(script)
            .args(["dconf", GTK_THEME_KEY, if on { "dark" } else { "light" }])
            .spawn();
    }
}

/// Whether the system asks for animations (`enable-animations`). Reading it tracks it.
pub fn animations() -> bool {
    ANIMATIONS.with(|cell| {
        cell.get().unwrap_or_else(|| {
            let (a, _) = create_root(|| {
                let a = signal(true);
                follow_dconf(ANIMATIONS_KEY, move |line| match line.trim() {
                    "true" => a.set(true),
                    "false" => a.set(false),
                    _ => {}
                });
                a
            });
            cell.set(Some(a));
            a
        })
        .get()
    })
}

pub fn set_animations(on: bool) {
    let _ = std::process::Command::new("dconf")
        .args(["write", ANIMATIONS_KEY, if on { "true" } else { "false" }])
        .spawn();
}

/// The length of an animation: `ms` at the chosen speed, or 0 when the system asks for none.
pub fn motion(ms: f32) -> u64 {
    if !animations() {
        return 0;
    }
    let speed = settings().get().animation_speed;
    let speed = if speed > 0.0 { speed } else { 1.0 };
    (ms as f64 / speed).round() as u64
}
