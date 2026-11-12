//! The desktop icons of one screen, over the wallpaper (QML `Desktop`): the Recycle Bin, the
//! system icons, then whatever is in the desktop folder. A drag moves an icon anywhere, to this
//! screen or another; "Align icons to grid" snaps it to a cell, and "Auto arrange icons" puts
//! each screen's icons in columns from the top left, in sort order, as Windows 7 does.
//!
//! A click picks an icon, Ctrl adds to the pick, a drag on the empty desktop draws a box round
//! several; a double click opens. Right click: the menu of the icon, or of the desktop. Keys:
//! Delete moves the picked icons to the Recycle Bin, F2 renames, Return opens, Ctrl+A picks every
//! icon, Escape drops the pick, Ctrl+wheel and Ctrl+plus or minus change the size of the icons.
//!
//! Each icon has a place in `desktop_places`: a screen and the top left of its cell. An icon
//! without one, or whose screen is not connected, shows on the main screen in the first free
//! cell. Its place is kept, so it goes back when its screen comes back. The icons keep out of
//! the bar.

use super::files::{self, Item, Kind};
use super::notes;
use crate::settings::{change, settings};
use crate::theme::{bar_above, bar_below, corner, theme};
use crate::ui::{MenuLine, font, shell_icon};
use anti_webui::prelude::*;
use anti_webui::shell::prelude::*;
use serde_json::{Map, Value, json};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::Duration;

/// What a drag of the shell's own icons carries to another program or another screen: the key of
/// the icon it started on. The icons themselves are read from [`dragged_icons`].
pub const ICONS_MIME: &str = "application/x-shojiwm-desktop-icons";

/// The icon sizes Ctrl+wheel steps through, up to 256 as on Windows 7. The View menu's Small,
/// Medium and Large are 32, 48 and 64.
const ICON_SIZES: [i64; 11] = [32, 40, 48, 56, 64, 80, 96, 128, 160, 192, 256];
/// The height of a line of an icon's name, to the size of the text.
const LINE_RATIO: f32 = 1.25;
/// How much further down a picked icon's name may go.
const PICKED_ROOM: f32 = 40.0;
/// How much bigger the icon of a folder or the Recycle Bin grows under a drag it would take.
const DROP_GROWTH: f32 = 1.12;

/// The plate under the pointer or the pick: pale glass, on any wallpaper, in both themes.
const PLATE_HOVER: Color = rgba(0xffffff26);
const PLATE_HOVER_BORDER: Color = rgba(0xffffff59);
const PLATE_PICKED: Color = rgba(0x86c0ec4d);
const PLATE_PICKED_HOT: Color = rgba(0x9fd2f75c);
const PLATE_PICKED_BORDER: Color = rgba(0xcfeaffa6);
/// The dark copy under a name, so that it reads on any wallpaper.
const NAME_SHADOW: Color = rgba(0x000000d9);
/// The shortcut arrow's square and its ink.
const ARROW_BORDER: Color = rgb(0x8c9aa6);
const ARROW_INK: Color = rgb(0x1f5fa8);
/// Renaming: a plain white field round the name, as on Windows 7.
const FIELD_SELECTION: Color = rgb(0x3399ff);
/// The pale place a drag over the empty desktop would drop at.
const LANDING_FILL: Color = rgba(0xffffff1a);
const LANDING_BORDER: Color = rgba(0xffffff80);
/// The box a drag draws on the empty desktop.
const BAND_FILL: Color = rgba(0x3399ff4d);
const BAND_BORDER: Color = rgba(0x3399ffcc);

// ----- the grid ------------------------------------------------------------------------------

/// The cells of a desktop: the screen less the bar, filled in columns. A cell holds the icon,
/// two lines of its name, and a gap before the next icon.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid {
    pub icon: f32,
    pub width: f32,
    pub height: f32,
    pub line: f32,
    pub area: (f32, f32),
    pub above: f32,
    pub below: f32,
    pub rows: i64,
    pub columns: i64,
}

impl Grid {
    fn now(area: Signal<(f32, f32)>) -> Self {
        let icon = settings()
            .with(|s| s.desktop_icon_size)
            .clamp(ICON_SIZES[0], ICON_SIZES[ICON_SIZES.len() - 1]) as f32;
        let line = (theme().font_size * LINE_RATIO).ceil();
        let width = (icon + 32.0).max(78.0);
        let height = 6.0 + icon + 4.0 + 2.0 * line + 12.0;
        let (w, h) = area.get();
        let (above, below) = (bar_above(), bar_below());
        Self {
            icon,
            width,
            height,
            line,
            area: (w, h),
            above,
            below,
            rows: ((h - above - below - 4.0) / height).floor().max(1.0) as i64,
            columns: ((w - 4.0) / width).floor().max(1.0) as i64,
        }
    }

    fn cell_point(&self, cell: i64) -> (f32, f32) {
        (
            2.0 + (cell / self.rows) as f32 * self.width,
            self.above + 2.0 + (cell % self.rows) as f32 * self.height,
        )
    }

    /// The cell nearest to a point.
    fn cell_at(&self, x: f32, y: f32) -> i64 {
        let column = ((x - 2.0) / self.width)
            .round()
            .clamp(0.0, (self.columns - 1) as f32) as i64;
        let row = ((y - self.above - 2.0) / self.height)
            .round()
            .clamp(0.0, (self.rows - 1) as f32) as i64;
        column * self.rows + row
    }

    /// A point kept on the screen, so that an icon is never out of reach.
    fn on_screen(&self, x: f32, y: f32) -> (f32, f32) {
        let (w, h) = self.area;
        (
            x.min(w - 2.0 - self.width).max(2.0),
            y.min(h - self.below - 2.0 - self.height)
                .max(self.above + 2.0),
        )
    }

    /// The plate of the icon whose cell is at `at`: what the pointer and the box hit.
    fn plate(&self, at: (f32, f32)) -> (f32, f32, f32, f32) {
        (at.0 + 2.0, at.1 + 2.0, self.width - 4.0, self.height - 4.0)
    }
}

// ----- the places ------------------------------------------------------------------------------

/// The place kept for an icon: its screen and the top left of its cell.
fn place_of(places: &Map<String, Value>, key: &str) -> Option<(String, f32, f32)> {
    let p = places.get(key)?;
    Some((
        p.get("screen")?.as_str()?.to_owned(),
        p.get("x")?.as_f64()? as f32,
        p.get("y")?.as_f64()? as f32,
    ))
}

fn place_value(screen: &str, (x, y): (f32, f32)) -> Value {
    json!({ "screen": screen, "x": x.round(), "y": y.round() })
}

thread_local! {
    static PLACES: Cell<Option<Memo<Map<String, Value>>>> = const { Cell::new(None) };
}

/// The places of the icons (`desktop_places`), one for every screen: what reads it is told only
/// when the places change, not at each change of the settings.
fn places() -> Memo<Map<String, Value>> {
    PLACES.with(|cell| {
        cell.get().unwrap_or_else(|| {
            let (places, _) =
                create_root(|| memo(|| settings().with(|s| s.desktop_places.clone())));
            cell.set(Some(places));
            places
        })
    })
}

/// Places to change, by key: a new place, or none to forget it.
type Changes = HashMap<String, Option<Value>>;

/// Changes the places of some icons; the others keep theirs.
fn set_places(changes: Changes) {
    if changes.is_empty() {
        return;
    }
    change(|s| {
        for (key, place) in changes {
            match place {
                Some(p) => {
                    s.desktop_places.insert(key, p);
                }
                None => {
                    s.desktop_places.remove(&key);
                }
            }
        }
    });
}

// ----- dragging --------------------------------------------------------------------------------

/// The shell's own icons being dragged: what a desktop needs to place them where they are
/// dropped, on any screen.
#[derive(Clone, Debug)]
pub struct IconDrag {
    pub keys: Vec<String>,
    /// Each icon's distance from the one the drag started on, in the order of `keys`.
    pub offsets: Vec<(f32, f32)>,
    /// The files among them, for a drop into a folder, the Recycle Bin or another program.
    pub paths: Vec<String>,
    /// Where the pointer took the icon, in its cell.
    pub grab: (f32, f32),
    /// The picture that goes with the pointer.
    pub icon: String,
    pub icon_size: f32,
}

thread_local! {
    /// The drag going on, for the desktops of the other screens: a drag to another surface comes
    /// to it as a drag from another program (QML `DragState`).
    static DRAG: RefCell<Option<IconDrag>> = const { RefCell::new(None) };
}

pub fn dragged_icons() -> Option<IconDrag> {
    DRAG.with(|d| d.borrow().clone())
}

/// Whether the drag going on carries the icon `key`.
fn dragging(key: &str) -> bool {
    DRAG.with(|d| {
        d.borrow()
            .as_ref()
            .is_some_and(|d| d.keys.iter().any(|k| k == key))
    })
}

/// Where the pointer took the icon of the drag going on, if it is the shell's own.
fn dragged_grab() -> Option<(f32, f32)> {
    DRAG.with(|d| d.borrow().as_ref().map(|d| d.grab))
}

/// The drag is over. Its icon says so too, but not when it went to another screen with it.
pub fn end_drag() {
    DRAG.with(|d| *d.borrow_mut() = None);
}

// ----- one desktop ---------------------------------------------------------------------------

/// Where a menu of the desktop was asked for.
#[derive(Clone, Debug, PartialEq)]
pub struct MenuAt {
    /// Where the pointer was, in logical pixels.
    pub point: (f32, f32),
    /// The icon it was asked on, or none for the desktop's own menu.
    pub item: Option<String>,
}

/// The state of one screen's desktop. Cheap to copy into handlers.
#[derive(Clone, Copy)]
pub struct Desk {
    /// The connector name of the screen.
    pub screen: Signal<String>,
    /// The size of the desktop in logical pixels.
    pub area: Signal<(f32, f32)>,
    /// The picked icons, by key.
    pub picked: Signal<Vec<String>>,
    /// The icon being renamed, or "", and the name in its field.
    pub rename_key: Signal<String>,
    pub rename_text: Signal<String>,
    /// The open menu, and the rectangle it hangs from (window physical pixels).
    pub menu: Signal<Option<MenuAt>>,
    pub menu_rect: Signal<Option<RectI>>,
    /// The Recycle Bin or folder a drag would drop into: it lights up as a picked icon does.
    pub drop_key: Signal<String>,
    /// Where a drag over the empty desktop would put its first icon.
    pub landing: Signal<Option<(f32, f32)>>,
    /// The box a drag draws on the empty desktop: x, y, width, height.
    pub band: Signal<Option<(f32, f32, f32, f32)>>,
    pub grid: Memo<Grid>,
    /// The icons on this screen, in sort order.
    pub shown: Memo<Vec<Item>>,
    /// The top left of each shown icon's cell, by key.
    pub positions: Memo<HashMap<String, (f32, f32)>>,
    /// The empty desktop, which takes the keyboard when clicked.
    pub background: NodeRef,
}

impl Desk {
    pub fn new(screen: String) -> Self {
        let screen = signal(screen);
        let area = signal((0.0f32, 0.0f32));
        let grid = memo(move || Grid::now(area));
        let places = places();
        let primary = memo(super::primary_output_name);
        let connected = memo(|| {
            outputs()
                .into_iter()
                .map(|o| o.name)
                .collect::<Vec<String>>()
        });
        let shown = memo(move || {
            let me = screen.get();
            let primary = primary.get();
            connected.with(|connected| {
                places.with(|places| {
                    files::items()
                        .into_iter()
                        .filter(|item| {
                            let on = match place_of(places, &item.key) {
                                Some((s, _, _)) if connected.contains(&s) => s,
                                _ => primary.clone(),
                            };
                            on == me
                        })
                        .collect::<Vec<_>>()
                })
            })
        });
        let arrangement =
            memo(|| settings().with(|s| (s.desktop_auto_arrange, s.desktop_align_to_grid)));
        let positions = memo(move || {
            let grid = grid.get();
            let me = screen.get();
            let (auto, align) = arrangement.get();
            let mut result = HashMap::new();
            let mut taken = HashSet::new();
            let mut waiting = Vec::new();
            for item in shown.get() {
                match places.with(|places| place_of(places, &item.key)) {
                    Some((s, x, y)) if !auto && s == me => {
                        let mut point = grid.on_screen(x, y);
                        let cell = grid.cell_at(point.0, point.1);
                        if align {
                            if taken.contains(&cell) {
                                waiting.push(item.key);
                                continue;
                            }
                            point = grid.cell_point(cell);
                        }
                        taken.insert(cell);
                        result.insert(item.key, point);
                    }
                    _ => waiting.push(item.key),
                }
            }
            let mut next = 0;
            for key in waiting {
                while taken.contains(&next) {
                    next += 1;
                }
                taken.insert(next);
                result.insert(key, grid.cell_point(next));
            }
            result
        });
        Self {
            screen,
            area,
            picked: signal(Vec::new()),
            rename_key: signal(String::new()),
            rename_text: signal(String::new()),
            menu: signal(None),
            menu_rect: signal(None),
            drop_key: signal(String::new()),
            landing: signal(None),
            band: signal(None),
            grid,
            shown,
            positions,
            background: node_ref(),
        }
    }

    fn me(&self) -> String {
        untrack(|| self.screen.get())
    }

    fn item(&self, key: &str) -> Option<Item> {
        untrack(|| {
            self.shown
                .with(|list| list.iter().find(|i| i.key == key).cloned())
        })
    }

    fn picked_items(&self) -> Vec<Item> {
        let picked = self.picked.get_untracked();
        untrack(|| {
            self.shown.with(|list| {
                list.iter()
                    .filter(|i| picked.contains(&i.key))
                    .cloned()
                    .collect()
            })
        })
    }

    fn pick(&self, key: &str, add: bool) {
        self.picked.update(|picked| {
            if !add {
                picked.clear();
                picked.push(key.to_owned());
            } else if let Some(i) = picked.iter().position(|k| k == key) {
                picked.remove(i);
            } else {
                picked.push(key.to_owned());
            }
        });
    }

    /// One size up (`step` 1) or down (-1).
    fn zoom(&self, step: i64) {
        let now = untrack(|| self.grid.get().icon) as i64;
        let index = (0..ICON_SIZES.len())
            .min_by_key(|&i| (ICON_SIZES[i] - now).abs())
            .unwrap_or(0) as i64;
        let next = (index + step).clamp(0, ICON_SIZES.len() as i64 - 1) as usize;
        change(|s| s.desktop_icon_size = ICON_SIZES[next]);
    }

    /// Every icon of this screen gets the place it shows at now, unless it has one here or is
    /// in `changes`. Moving one icon then leaves the others where they are, instead of letting
    /// the ones without a place fill the cell it left.
    fn keep_places(&self, changes: &mut Changes) {
        let me = self.me();
        let places = untrack(|| places().get());
        let positions = untrack(|| self.positions.get());
        for item in untrack(|| self.shown.get()) {
            let here = place_of(&places, &item.key).is_some_and(|(s, _, _)| s == me);
            if !here
                && !changes.contains_key(&item.key)
                && let Some(&at) = positions.get(&item.key)
            {
                changes.insert(item.key.clone(), Some(place_value(&me, at)));
            }
        }
    }

    /// "Sort by" without "Auto arrange icons": this screen's icons go to the first cells, in
    /// the new order, once.
    fn arrange(&self) {
        let me = self.me();
        let grid = untrack(|| self.grid.get());
        let changes = untrack(|| self.shown.get())
            .into_iter()
            .enumerate()
            .map(|(i, item)| (item.key, Some(place_value(&me, grid.cell_point(i as i64)))))
            .collect();
        set_places(changes);
    }

    /// The drag of the picked icons, started on `anchor` with the pointer at `grab` in its cell.
    fn drag_data(&self, anchor: &str, grab: (f32, f32), icon: String) -> IconDrag {
        let positions = untrack(|| self.positions.get());
        let origin = positions.get(anchor).copied().unwrap_or((0.0, 0.0));
        let mut drag = IconDrag {
            keys: Vec::new(),
            offsets: Vec::new(),
            paths: Vec::new(),
            grab,
            icon,
            icon_size: untrack(|| self.grid.get().icon),
        };
        for item in self.picked_items() {
            let Some(&at) = positions.get(&item.key) else {
                continue;
            };
            drag.keys.push(item.key.clone());
            drag.offsets.push((at.0 - origin.0, at.1 - origin.1));
            if !item.kind.is_system() {
                drag.paths.push(item.path.clone());
            }
        }
        if !untrack(|| settings().with(|s| s.desktop_auto_arrange)) {
            let mut changes = Changes::new();
            self.keep_places(&mut changes);
            set_places(changes);
        }
        drag
    }

    /// Dragged icons dropped at a point of this screen: the anchor's cell goes under the
    /// pointer where it was grabbed, the others keep their distance from it. On the grid, each
    /// takes the nearest free cell.
    fn place_dragged(&self, moving: &IconDrag, x: f32, y: f32) {
        let me = self.me();
        let grid = untrack(|| self.grid.get());
        let (auto, align) =
            untrack(|| settings().with(|s| (s.desktop_auto_arrange, s.desktop_align_to_grid)));
        let mut taken: HashSet<i64> = untrack(|| self.positions.get())
            .iter()
            .filter(|(key, _)| !moving.keys.contains(key))
            .map(|(_, at)| grid.cell_at(at.0, at.1))
            .collect();
        let mut changes = Changes::new();
        for (key, offset) in moving.keys.iter().zip(&moving.offsets) {
            let mut point =
                grid.on_screen(x - moving.grab.0 + offset.0, y - moving.grab.1 + offset.1);
            if align {
                let mut cell = grid.cell_at(point.0, point.1);
                while taken.contains(&cell) {
                    cell += 1;
                }
                taken.insert(cell);
                point = grid.cell_point(cell);
            }
            changes.insert(key.clone(), Some(place_value(&me, point)));
        }
        if !auto {
            self.keep_places(&mut changes);
        }
        set_places(changes);
    }

    /// Calls `f` with the icon under a point, or none. Nothing is copied: it runs at each move of
    /// a drag.
    fn with_item_at<R>(&self, x: f32, y: f32, f: impl FnOnce(Option<&Item>) -> R) -> R {
        let grid = untrack(|| self.grid.get());
        untrack(|| {
            self.positions.with(|positions| {
                self.shown.with(|shown| {
                    f(shown.iter().find(|item| {
                        positions.get(&item.key).is_some_and(|&at| {
                            let (px, py, w, h) = grid.plate(at);
                            x >= px && x < px + w && y >= py && y < py + h
                        })
                    }))
                })
            })
        })
    }

    /// The icon under a point, or none.
    fn item_at(&self, x: f32, y: f32) -> Option<Item> {
        self.with_item_at(x, y, |item| item.cloned())
    }

    /// The key of the Recycle Bin or folder under (x, y) that a drag can drop into, or "": not
    /// one of the icons being dragged.
    fn drop_target_at(&self, x: f32, y: f32) -> String {
        self.with_item_at(x, y, |target| match target {
            Some(t) if matches!(t.kind, Kind::Trash | Kind::Folder) && !dragging(&t.key) => {
                t.key.clone()
            }
            _ => String::new(),
        })
    }

    /// The top left a drop at (x, y) gives the first icon: the shell's own icons keep where they
    /// were taken, files come centred under the pointer.
    fn landing_at(&self, x: f32, y: f32) -> (f32, f32) {
        let grid = untrack(|| self.grid.get());
        let point = match dragged_grab() {
            Some(grab) => grid.on_screen(x - grab.0, y - grab.1),
            None => grid.on_screen(x - grid.width / 2.0, y - grid.height / 3.0),
        };
        if untrack(|| settings().with(|s| s.desktop_align_to_grid)) {
            grid.cell_point(grid.cell_at(point.0, point.1))
        } else {
            point
        }
    }

    /// Something is dragged over (x, y): light the icon it would drop into, or show where it
    /// would land.
    pub fn drag_moved(&self, x: f32, y: f32) {
        let key = self.drop_target_at(x, y);
        self.landing.set_if_changed(if key.is_empty() {
            Some(self.landing_at(x, y))
        } else {
            None
        });
        self.drop_key.set_if_changed(key);
    }

    pub fn drag_left(&self) {
        self.drop_key.set_if_changed(String::new());
        self.landing.set_if_changed(None);
    }

    /// The shell's own icons dropped at (x, y). On the Recycle Bin or a folder, their files go
    /// there; anywhere else, the icons move there, on this screen.
    pub fn drop_icons(&self, moving: &IconDrag, x: f32, y: f32, copy: bool) {
        self.drag_left();
        if let Some(target) = self.item_at(x, y)
            && !moving.keys.contains(&target.key)
            && matches!(target.kind, Kind::Trash | Kind::Folder)
        {
            self.drop_files(&moving.paths, x, y, copy);
            return;
        }
        self.place_dragged(moving, x, y);
    }

    /// Files dropped at (x, y): into the Recycle Bin or a folder icon under it, else onto the
    /// desktop. Moved unless the drag asks for a copy; files already where they are dropped
    /// stay as they are. Files that come to the desktop show where they are dropped.
    pub fn drop_files(&self, paths: &[String], x: f32, y: f32, copy: bool) {
        self.drag_left();
        let target = self.item_at(x, y);
        if let Some(t) = &target
            && t.kind == Kind::Trash
        {
            files::trash_paths(&paths.iter().map(String::as_str).collect::<Vec<_>>());
            return;
        }
        let Some(desktop) = files::folder_string() else {
            return;
        };
        let folder = match &target {
            Some(t) if t.kind == Kind::Folder => t.path.clone(),
            _ => desktop.clone(),
        };
        let moving: Vec<String> = paths
            .iter()
            .filter(|p| **p != folder && files::parent_of(p) != folder)
            .cloned()
            .collect();
        if moving.is_empty() {
            return;
        }
        if folder == desktop {
            let me = self.me();
            let grid = untrack(|| self.grid.get());
            let changes = moving
                .iter()
                .enumerate()
                .map(|(j, path)| {
                    let point = grid.on_screen(
                        x - grid.width / 2.0 + j as f32 * grid.width,
                        y - grid.height / 3.0,
                    );
                    let name = path.rsplit('/').next().unwrap_or(path);
                    (format!("{folder}/{name}"), Some(place_value(&me, point)))
                })
                .collect();
            set_places(changes);
        }
        files::move_into(&moving, &folder, copy);
    }

    // ----- renaming ------------------------------------------------------------------------

    fn start_rename(&self, key: &str) {
        let Some(item) = self.item(key) else { return };
        if item.kind.is_system() {
            return;
        }
        self.rename_text.set(item.name);
        self.rename_key.set(key.to_owned());
    }

    /// The name in the field becomes the file's. The icon keeps its place under its new name.
    fn commit_rename(&self) {
        let key = self.rename_key.get_untracked();
        if key.is_empty() {
            return;
        }
        self.rename_key.set(String::new());
        self.background.focus();
        let Some(item) = self.item(&key) else { return };
        let name = self.rename_text.get_untracked();
        let kept = untrack(|| places().with(|p| p.get(&key).cloned()));
        files::rename(&item, &name, move |path| {
            let mut changes = Changes::new();
            changes.insert(path, kept);
            changes.insert(key, None);
            set_places(changes);
        });
    }

    fn cancel_rename(&self) {
        self.rename_key.set(String::new());
        self.background.focus();
    }

    // ----- the menus -----------------------------------------------------------------------

    /// Opens a menu where the pointer was pressed (`position`, window physical pixels).
    fn open_menu(&self, position: PointF, item: Option<String>) {
        let scale = untrack(|| window_scale().as_f32());
        let rect = RectI::new(position.x as i32, position.y as i32, 1, 1);
        batch(|| {
            self.menu_rect.set(Some(rect));
            self.menu.set(Some(MenuAt {
                point: (position.x / scale, position.y / scale),
                item,
            }));
        });
    }

    pub fn close_menu(&self) {
        self.menu.set_if_changed(None);
    }

    /// The lines of the open menu, as things are now.
    pub fn menu_lines(&self) -> Vec<MenuLine> {
        let Some(at) = self.menu.get_untracked() else {
            return Vec::new();
        };
        match at.item.and_then(|key| self.item(&key)) {
            Some(item) => self.item_lines(item),
            None => self.desktop_lines(at.point),
        }
    }

    fn desktop_lines(&self, point: (f32, f32)) -> Vec<MenuLine> {
        let desk = *self;
        let s = settings().get_untracked();
        let size = |label: &str, value: i64| {
            MenuLine::item(label, move || change(|s| s.desktop_icon_size = value))
                .radio(s.desktop_icon_size == value)
        };
        let sort = |label: &str, value: &'static str| {
            MenuLine::item(label, move || {
                change(|s| s.desktop_sort = value.to_owned());
                if !untrack(|| settings().with(|s| s.desktop_auto_arrange)) {
                    desk.arrange();
                }
            })
            .radio(s.desktop_sort == value)
        };
        let icon = |name: &str| {
            crate::paths::asset(&format!("icons/{name}"))
                .to_string_lossy()
                .into_owned()
        };
        vec![
            MenuLine::submenu(
                "View",
                vec![
                    size("Large icons", 64),
                    size("Medium icons", 48),
                    size("Small icons", 32),
                    MenuLine::separator(),
                    MenuLine::item("Auto arrange icons", || {
                        change(|s| s.desktop_auto_arrange = !s.desktop_auto_arrange)
                    })
                    .checked(s.desktop_auto_arrange),
                    MenuLine::item("Align icons to grid", || {
                        change(|s| s.desktop_align_to_grid = !s.desktop_align_to_grid)
                    })
                    .checked(s.desktop_align_to_grid),
                    MenuLine::separator(),
                    MenuLine::item("Show desktop icons", || {
                        change(|s| s.show_desktop_icons = !s.show_desktop_icons)
                    })
                    .checked(s.show_desktop_icons),
                ],
            ),
            MenuLine::submenu(
                "Sort by",
                vec![
                    sort("Name", "name"),
                    sort("Size", "size"),
                    sort("Item type", "type"),
                    sort("Date modified", "date"),
                ],
            ),
            MenuLine::item("Refresh", files::refresh),
            MenuLine::separator(),
            MenuLine::submenu(
                "New",
                vec![
                    MenuLine::item("Folder", files::new_folder).icon(icon("folder.png")),
                    MenuLine::separator(),
                    MenuLine::item("Text Document", files::new_text_document)
                        .icon(icon("file.png")),
                    MenuLine::item("Sticky Note", move || {
                        let (w, h) = desk.area.get_untracked();
                        notes::add(&desk.me(), point.0.min(w - 230.0), point.1.min(h - 210.0));
                    }),
                ],
            ),
            MenuLine::item("Open in Terminal", files::open_terminal).icon(icon("run.png")),
            MenuLine::separator(),
            MenuLine::item("Screen resolution", || {
                crate::control_panel::open_page("display")
            })
            .icon(icon("computer.png")),
            MenuLine::submenu(
                "Gadgets",
                vec![
                    MenuLine::item("CPU Meter", || {
                        change(|s| s.show_cpu_meter = !s.show_cpu_meter)
                    })
                    .checked(s.show_cpu_meter),
                    MenuLine::item("Clock", || {
                        change(|s| s.show_clock_gadget = !s.show_clock_gadget)
                    })
                    .checked(s.show_clock_gadget),
                ],
            ),
            MenuLine::item("Personalize", || {
                crate::control_panel::open_page("appearance")
            })
            .icon(icon("control-panel.png")),
        ]
    }

    fn item_lines(&self, item: Item) -> Vec<MenuLine> {
        let desk = *self;
        let open = {
            let item = item.clone();
            move || files::open(&item)
        };
        let kind = item.kind;
        match kind {
            Kind::Computer | Kind::UserFiles | Kind::ControlPanel => vec![
                MenuLine::item("Open", open),
                MenuLine::separator(),
                MenuLine::item("Remove from the desktop", move || {
                    files::remove_system_icon(kind)
                }),
            ],
            Kind::Trash => vec![
                MenuLine::item("Open", open),
                MenuLine::separator(),
                MenuLine::item("Empty Recycle Bin", files::empty_trash)
                    .enabled(untrack(files::trash_full)),
            ],
            _ => {
                let list = Rc::new(self.picked_items());
                let opened = list.clone();
                let one = list.len() == 1;
                let location = files::parent_of(&item.path);
                vec![
                    MenuLine::item("Open", move || opened.iter().for_each(files::open)),
                    MenuLine::item("Open file location", move || files::open_path(&location)),
                    MenuLine::separator(),
                    MenuLine::item("Delete", move || {
                        files::trash(&list);
                        desk.picked.set(Vec::new());
                    }),
                    MenuLine::item("Rename", move || desk.start_rename(&item.key)).enabled(one),
                ]
            }
        }
    }
}

// ----- the view ----------------------------------------------------------------------------

/// The empty desktop (picks, the box, the menu, the keys), the place a drag would land, the
/// icons, and the box.
pub fn view(desk: Desk) -> impl IntoView {
    let shown = memo(|| settings().with(|s| s.show_desktop_icons));
    fragment([
        background(desk).into_view(),
        div()
            .absolute()
            .border(1.0)
            .border_color(LANDING_BORDER)
            .bg(LANDING_FILL)
            .pointer_events_none()
            .dyn_style(move |s| {
                let grid = desk.grid.get();
                match desk.landing.get() {
                    Some((x, y)) if shown.get() => s
                        .left(x + 2.0)
                        .top(y + 2.0)
                        .w(grid.width - 4.0)
                        .h(grid.height - 4.0)
                        .rounded(corner(3.0)),
                    _ => s.hidden(),
                }
            })
            .into_view(),
        div()
            .absolute()
            .inset(0.0)
            .pointer_events_none()
            .dyn_style(move |s| if shown.get() { s } else { s.hidden() })
            .child(for_each(
                move || desk.shown.get(),
                |item: &Item| (item.key.clone(), item.label(), item.shortcut_icon.clone()),
                move |item| icon_cell(desk, item),
            ))
            .into_view(),
        div()
            .absolute()
            .border(1.0)
            .border_color(BAND_BORDER)
            .bg(BAND_FILL)
            .pointer_events_none()
            .dyn_style(move |s| match desk.band.get() {
                Some((x, y, w, h)) => s.left(x).top(y).w(w).h(h),
                None => s.hidden(),
            })
            .into_view(),
    ])
}

/// The empty desktop: a press drops the pick (Ctrl keeps it) and a drag draws a box round the
/// icons to pick; the secondary button opens the desktop's menu. It takes the keyboard.
fn background(desk: Desk) -> Div {
    let from = Rc::new(Cell::new((0.0f32, 0.0f32)));
    let pressed_at = from.clone();
    div()
        .absolute()
        .inset(0.0)
        .focusable()
        .node_ref(desk.background)
        .on_pointer_down(move |e| {
            desk.close_menu();
            desk.commit_rename();
            desk.background.focus();
            if e.button == Some(PointerButton::Secondary) {
                desk.picked.set(Vec::new());
                desk.open_menu(e.position, None);
                return;
            }
            if !e.modifiers.ctrl {
                desk.picked.set(Vec::new());
            }
            let scale = untrack(|| window_scale().as_f32());
            pressed_at.set((e.position.x / scale, e.position.y / scale));
        })
        .on_grab(move |g| match g.phase {
            GrabPhase::Start | GrabPhase::Move => {
                let (x0, y0) = from.get();
                let (x1, y1) = (x0 + g.total.x, y0 + g.total.y);
                let rect = (x0.min(x1), y0.min(y1), (x1 - x0).abs(), (y1 - y0).abs());
                desk.band.set(Some(rect));
                let grid = untrack(|| desk.grid.get());
                let (bx, by, bw, bh) = rect;
                let hit = |at: (f32, f32)| {
                    let (x, y, w, h) = grid.plate(at);
                    x < bx + bw && x + w > bx && y < by + bh && y + h > by
                };
                // The pick changes only when the box takes or leaves an icon: nothing is copied
                // while it stays the same.
                let inside = untrack(|| {
                    desk.positions.with(|positions| {
                        let same = desk.picked.with_untracked(|picked| {
                            picked.is_sorted()
                                && positions.iter().filter(|(_, at)| hit(**at)).count()
                                    == picked.len()
                                && picked
                                    .iter()
                                    .all(|k| positions.get(k).is_some_and(|at| hit(*at)))
                        });
                        (!same).then(|| {
                            let mut inside: Vec<String> = positions
                                .iter()
                                .filter(|(_, at)| hit(**at))
                                .map(|(key, _)| key.clone())
                                .collect();
                            inside.sort();
                            inside
                        })
                    })
                });
                if let Some(inside) = inside {
                    desk.picked.set(inside);
                }
            }
            GrabPhase::End | GrabPhase::Cancel => desk.band.set(None),
        })
        .on_scroll(move |e| {
            if !e.modifiers.ctrl || e.delta_y == 0.0 {
                return;
            }
            e.stop_propagation();
            desk.zoom(if e.delta_y < 0.0 { 1 } else { -1 });
        })
        .on_key_down(move |e| {
            if !desk.rename_key.get_untracked().is_empty() {
                return;
            }
            let ctrl = e.modifiers.ctrl;
            match &e.key {
                Key::Named(NamedKey::Delete) => {
                    files::trash(&desk.picked_items());
                    desk.picked.set(Vec::new());
                }
                Key::Named(NamedKey::F(2)) => {
                    let list = desk.picked_items();
                    if let [one] = list.as_slice() {
                        desk.start_rename(&one.key);
                    }
                }
                Key::Named(NamedKey::Enter) => desk.picked_items().iter().for_each(files::open),
                Key::Named(NamedKey::Escape) => desk.picked.set(Vec::new()),
                key if ctrl && key.is_char('a') => {
                    let all: Vec<String> = untrack(|| {
                        desk.shown
                            .with(|l| l.iter().map(|i| i.key.clone()).collect())
                    });
                    desk.picked.set(all);
                }
                key if ctrl && (key.is_char('+') || key.is_char('=')) => desk.zoom(1),
                key if ctrl && key.is_char('-') => desk.zoom(-1),
                _ => return,
            }
            e.prevent_default();
        })
}

/// One icon: its plate, its picture (with the shortcut arrow) and its name, or the field that
/// renames it.
fn icon_cell(desk: Desk, item: Item) -> Div {
    let key: Rc<str> = item.key.clone().into();
    let item = Rc::new(item);
    let ui = Interaction::new();
    let picked = {
        let key = key.clone();
        memo(move || desk.picked.with(|p| p.iter().any(|k| **k == *key)))
    };
    let target = {
        let key = key.clone();
        memo(move || desk.drop_key.with(|k| **k == *key))
    };
    let editing = {
        let key = key.clone();
        memo(move || desk.rename_key.with(|k| **k == *key))
    };
    let icon_size = memo(move || desk.grid.get().icon);
    // Where the pointer took the icon, in its cell, for a drag.
    let grab = Rc::new(Cell::new((0.0f32, 0.0f32)));

    let picture = {
        let item = item.clone();
        dyn_view(move || {
            let size = icon_size.get();
            let shown = item.clone();
            let arrow = (item.kind == Kind::Shortcut).then(|| shortcut_arrow((size / 3.0).round()));
            div()
                .relative()
                .flex_none()
                .size(size)
                // It grows a little under a drag it would take.
                .transition_of(
                    [Prop::Scale],
                    Transition::ms(untrack(|| crate::settings::motion(120.0)))
                        .cubic_bezier(0.33, 1.0, 0.68, 1.0),
                )
                .dyn_style(move |s| s.scale(if target.get() { DROP_GROWTH } else { 1.0 }))
                .child(shell_icon(move || shown.icon(), size))
                .children(arrow)
        })
    };

    let label = {
        let item = item.clone();
        move || item.label()
    };
    let name = {
        let label = label.clone();
        move || {
            let shadow = label.clone();
            let words = label.clone();
            div()
                .relative()
                .w_full()
                .px(1.0)
                .overflow_clip()
                .text_align(TextAlign::Center)
                .dyn_style(move |s| {
                    let grid = desk.grid.get();
                    let lines = if picked.get() { 6.0 } else { 2.0 };
                    s.font_family(font())
                        .text_size(theme().font_size)
                        .line_height(LineHeight::Px(grid.line))
                        .max_h(lines * grid.line)
                })
                .child(
                    div()
                        .absolute()
                        .left(2.0)
                        .top(1.0)
                        .right(0.0)
                        .text_color(NAME_SHADOW)
                        .z_index(0)
                        .blur(1.0)
                        .child(text(shadow)),
                )
                .child(
                    div()
                        .relative()
                        .z_index(1)
                        .text_color(Color::WHITE)
                        .child(text(words)),
                )
        }
    };

    let field_ref = node_ref();
    let field = {
        let item = item.clone();
        move || {
            // The name without its extension is picked, as Windows does.
            let name = item.name.clone();
            let stem = if item.is_dir() {
                name.len()
            } else {
                name.rfind('.').filter(|&i| i > 0).unwrap_or(name.len())
            };
            set_timeout(Duration::ZERO, move || {
                field_ref.focus();
                field_ref.set_text(&name, (0, stem));
            });
            text_input()
                .node_ref(field_ref)
                .value(desk.rename_text)
                .w_full()
                .px(2.0)
                .border(1.0)
                .border_color(Color::BLACK)
                .bg(Color::WHITE)
                .text_color(Color::BLACK)
                .selection_color(FIELD_SELECTION)
                .text_align(TextAlign::Center)
                .dyn_style(|s| s.font_family(font()).text_size(theme().font_size))
                .on_submit(move |_| desk.commit_rename())
                .on_key_down(move |e| {
                    if e.key == Key::Named(NamedKey::Escape) {
                        e.stop_propagation();
                        desk.cancel_rename();
                    }
                })
        }
    };

    let press_key = key.clone();
    let press_grab = grab.clone();
    let click_key = key.clone();
    let click_item = item.clone();
    let drag_key = key.clone();
    let drag_item = item.clone();
    div()
        .absolute()
        .flex()
        .flex_col()
        .items_center()
        .gap(4.0)
        .pt(4.0)
        .pb(6.0)
        .border(1.0)
        .overflow_clip()
        .select_none()
        .pointer_events_auto()
        .interaction(ui)
        .dyn_style(move |s| {
            let grid = desk.grid.get();
            let at = desk
                .positions
                .with(|p| p.get(&*key).copied())
                .unwrap_or((2.0, 2.0));
            let (x, y, w, _) = grid.plate(at);
            let picked = picked.get();
            let lit = picked || target.get();
            let hot = ui.hovered() || target.get();
            let s = s
                .left(x)
                .top(y)
                .w(w)
                .max_h(grid.height - 4.0 + if picked { PICKED_ROOM } else { 0.0 })
                .z_index(if picked || editing.get() { 2 } else { 0 })
                .rounded(corner(3.0));
            if lit {
                s.bg(if hot { PLATE_PICKED_HOT } else { PLATE_PICKED })
                    .border_color(PLATE_PICKED_BORDER)
            } else if ui.hovered() {
                s.bg(PLATE_HOVER).border_color(PLATE_HOVER_BORDER)
            } else {
                s.bg(Color::TRANSPARENT).border_color(Color::TRANSPARENT)
            }
        })
        .on_pointer_down(move |e| {
            if untrack(|| editing.get()) {
                return;
            }
            e.stop_propagation();
            desk.close_menu();
            desk.commit_rename();
            desk.background.focus();
            let scale = untrack(|| window_scale().as_f32());
            // From the plate to its cell.
            press_grab.set((e.local.x / scale + 2.0, e.local.y / scale + 2.0));
            if e.button == Some(PointerButton::Secondary) {
                if !untrack(|| picked.get()) {
                    desk.pick(&press_key, false);
                }
                desk.open_menu(e.position, Some(press_key.to_string()));
                return;
            }
            let add = e.modifiers.ctrl;
            if add || !untrack(|| picked.get()) {
                desk.pick(&press_key, add);
            }
        })
        .on_click(move |e| {
            if untrack(|| editing.get()) || e.button == Some(PointerButton::Secondary) {
                return;
            }
            if e.click_count >= 2 {
                files::open(&click_item);
                return;
            }
            // A click on one of several picked icons picks it alone.
            if !e.modifiers.ctrl {
                desk.pick(&click_key, false);
            }
        })
        .draggable(move || {
            let drag = desk.drag_data(&drag_key, grab.get(), drag_item.icon());
            DRAG.with(|d| *d.borrow_mut() = Some(drag.clone()));
            let paths = drag.paths.clone();
            let data = DragData::new(drag).data(ICONS_MIME, drag_key.as_bytes().to_vec());
            let data = if paths.is_empty() {
                data
            } else {
                data.files(paths)
            };
            data.actions(DragActions::Both)
        })
        .drag_preview(|d: &IconDrag| shell_icon(d.icon.clone(), d.icon_size))
        .on_drag_end(|_| end_drag())
        .child(picture)
        .child(show(move || !editing.get(), name))
        .child(show(move || editing.get(), field))
}

/// The shortcut arrow, in the bottom left corner of the icon.
fn shortcut_arrow(size: f32) -> Div {
    div()
        .absolute()
        .left(0.0)
        .bottom(0.0)
        .size(size)
        .flex()
        .items_center()
        .justify_center()
        .bg(Color::WHITE)
        .border(1.0)
        .border_color(ARROW_BORDER)
        .text_color(ARROW_INK)
        .text_size((size - 3.0).max(1.0))
        .font_weight(FontWeight::BOLD)
        .dyn_style(|s| s.rounded(corner(1.0)))
        .child("\u{2197}")
}
