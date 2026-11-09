pragma ComponentBehavior: Bound

import QtQuick
import Qt.labs.folderlistmodel
import Quickshell
import Quickshell.Io
import ".."
import "../theme.js" as Theme
import "../tools.js" as Tools

// The desktop icons of one screen, over the wallpaper: the Recycle Bin, the
// system icons, then whatever is in the desktop folder (XDG_DESKTOP_DIR,
// ~/Desktop by default). A drag moves an icon anywhere, to this screen or
// another; "Align icons to grid" snaps it to a cell, and "Auto arrange
// icons" puts each screen's icons in columns from the top left, in sort
// order, as Windows 7 does.
//
// A click picks an icon, Ctrl adds to the pick, a drag on the empty desktop
// draws a box round several; a double click opens. A .desktop file is a
// shortcut: it launches its program and wears the shortcut arrow. Right
// click: the menu of the icon, or of the desktop. Keys: Delete moves the
// picked icons to the Recycle Bin, F2 renames, Return opens, Ctrl+A picks
// every icon, Escape drops the pick, Ctrl+wheel and Ctrl+plus or minus
// change the size of the icons.
Item {
    id: desktop

    focus: true

    Keys.onPressed: function (event) {
        if (desktop.renaming)
            return;
        var ctrl = (event.modifiers & Qt.ControlModifier) !== 0;
        if (event.key === Qt.Key_Delete) {
            desktop.trash(desktop.pickedItems());
        } else if (event.key === Qt.Key_F2) {
            var list = desktop.pickedItems();
            if (list.length === 1 && !desktop.isSystem(list[0]))
                desktop.renameKey = list[0].key;
        } else if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
            var open = desktop.pickedItems();
            for (var i = 0; i < open.length; i++)
                desktop.open(open[i]);
        } else if (ctrl && event.key === Qt.Key_A) {
            var all = ({});
            for (var k = 0; k < desktop.shown.length; k++)
                all[desktop.shown[k].key] = true;
            desktop.picked = all;
        } else if (ctrl && (event.key === Qt.Key_Plus || event.key === Qt.Key_Equal)) {
            desktop.zoom(1);
        } else if (ctrl && event.key === Qt.Key_Minus) {
            desktop.zoom(-1);
        } else if (event.key === Qt.Key_Escape) {
            desktop.picked = ({});
        } else {
            return;
        }
        event.accepted = true;
    }

    /// An icon is being renamed: the window takes the keyboard meanwhile.
    readonly property bool renaming: desktop.renameKey !== ""
    property string renameKey: ""

    readonly property string folder: Apps.userDir("DESKTOP", "Desktop")

    /// The icon sizes Ctrl+wheel steps through, up to 256 as on Windows 7.
    /// The View menu's Small, Medium and Large are 32, 48 and 64.
    readonly property var iconSizes: [32, 40, 48, 56, 64, 80, 96, 128, 160, 192, 256]
    /// One cell: the icon, two lines of its name, and a gap before the
    /// next icon, so a name never touches the icon under it.
    readonly property var size: {
        var icon = Math.max(iconSizes[0], Math.min(iconSizes[iconSizes.length - 1],
                                                   Settings.desktopIconSize));
        return {
            icon: icon,
            width: Math.max(78, icon + 32),
            height: 6 + icon + 4 + 2 * Math.ceil(nameMetrics.height) + 12
        };
    }

    FontMetrics {
        id: nameMetrics
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
    }

    /// One size up (`step` 1) or down (-1), as Ctrl+wheel and Ctrl+plus or
    /// minus do on Windows.
    function zoom(step) {
        var index = 0;
        for (var i = 0; i < iconSizes.length; i++) {
            if (Math.abs(iconSizes[i] - size.icon) < Math.abs(iconSizes[index] - size.icon))
                index = i;
        }
        Settings.desktopIconSize = iconSizes[Math.max(0, Math.min(iconSizes.length - 1, index + step))];
    }

    // ----- what is on the desktop ---------------------------------------------

    property var items: []
    /// The icon cells on screen, by key, for the box and for renaming.
    property var cells: ({})
    /// The picked icons, by key.
    property var picked: ({})

    FolderListModel {
        id: files

        folder: "file://" + desktop.folder
        showDirsFirst: true
        showHidden: false
        showDotAndDotDot: false
        sortField: FolderListModel.Unsorted
    }

    Connections {
        target: files
        function onCountChanged() { Qt.callLater(desktop.rebuild); }
        function onStatusChanged() { Qt.callLater(desktop.rebuild); }
        function onRowsInserted() { Qt.callLater(desktop.rebuild); }
        function onRowsRemoved() { Qt.callLater(desktop.rebuild); }
        function onModelReset() { Qt.callLater(desktop.rebuild); }
        function onDataChanged() { Qt.callLater(desktop.rebuild); }
    }

    Connections {
        target: Settings
        function onDesktopSortChanged() { desktop.rebuild(); }
        function onDesktopSystemIconsChanged() { desktop.rebuild(); }
    }

    function rebuild() {
        var list = [];
        for (var i = 0; i < files.count; i++) {
            var path = String(files.get(i, "filePath"));
            var name = String(files.get(i, "fileName"));
            var dir = files.get(i, "fileIsDir") === true;
            var dot = name.lastIndexOf(".");
            list.push({
                key: path,
                kind: dir ? "folder"
                    : name.toLowerCase().endsWith(".desktop") ? "shortcut" : "file",
                path: path,
                name: name,
                suffix: dir || dot <= 0 ? "" : name.substring(dot + 1).toLowerCase(),
                size: Number(files.get(i, "fileSize")) || 0,
                modified: files.get(i, "fileModified"),
                dir: dir
            });
        }
        var sort = Settings.desktopSort;
        list.sort(function (a, b) {
            if (a.dir !== b.dir)
                return a.dir ? -1 : 1;
            if (sort === "size" && a.size !== b.size)
                return b.size - a.size;
            if (sort === "type" && a.suffix !== b.suffix)
                return a.suffix < b.suffix ? -1 : 1;
            if (sort === "date" && a.modified && b.modified
                && a.modified.getTime() !== b.modified.getTime())
                return b.modified.getTime() - a.modified.getTime();
            return a.name.localeCompare(b.name, undefined, { sensitivity: "base" });
        });
        // The system icons first, the Recycle Bin on top, as on Windows 7.
        var system = desktop.systemIcons.filter(function (icon) {
            return Settings.desktopSystemIcons.indexOf(icon.kind) >= 0;
        }).map(function (icon) {
            return { key: icon.kind + ":", kind: icon.kind, path: icon.path, name: icon.name };
        });
        list = [{ key: "trash:", kind: "trash", path: "trash:///", name: "Recycle Bin" }]
            .concat(system, list);
        desktop.items = list;
    }

    // ----- where the icons are ------------------------------------------------
    //
    // Each icon has a place in Settings.desktopPlaces: a screen and the top
    // left of its cell. An icon without one, or whose screen is not
    // connected, shows on the main screen (Screens) in the first free cell.
    // Its place is kept, so it goes back when its screen comes back.

    readonly property var screenNames: Quickshell.screens.map(function (screen) {
        return screen.name;
    })
    readonly property string firstScreen: Screens.primaryName
    /// The cells: the screen less the taskbar, filled in columns.
    readonly property int rows: Math.max(1, Math.floor((height - Theme.taskbarHeight - 4) / size.height))
    readonly property int columns: Math.max(1, Math.floor((width - 4) / size.width))

    function screenOf(item) {
        var place = Settings.desktopPlaces[item.key];
        return place && screenNames.indexOf(place.screen) >= 0 ? place.screen : firstScreen;
    }

    /// The icons on this screen, in sort order.
    readonly property var shown: items.filter(function (item) {
        return desktop.screenOf(item) === desktop.screenName;
    })

    function cellPoint(cell) {
        return Qt.point(2 + Math.floor(cell / rows) * size.width, 2 + (cell % rows) * size.height);
    }

    /// The cell nearest to a point.
    function cellAt(x, y) {
        var column = Math.max(0, Math.min(columns - 1, Math.round((x - 2) / size.width)));
        var row = Math.max(0, Math.min(rows - 1, Math.round((y - 2) / size.height)));
        return column * rows + row;
    }

    /// A point kept on the screen, so an icon is never out of reach.
    function onScreen(x, y) {
        return Qt.point(Math.max(2, Math.min(width - 2 - size.width, x)),
            Math.max(2, Math.min(height - Theme.taskbarHeight - 2 - size.height, y)));
    }

    /// The top left of each shown icon, by key.
    readonly property var positions: {
        var result = {};
        var taken = {};
        var waiting = [];
        for (var i = 0; i < shown.length; i++) {
            var item = shown[i];
            var place = Settings.desktopPlaces[item.key];
            if (Settings.desktopAutoArrange || !place || place.screen !== screenName) {
                waiting.push(item);
                continue;
            }
            var point = onScreen(place.x, place.y);
            var cell = cellAt(point.x, point.y);
            if (Settings.desktopAlignToGrid) {
                if (taken[cell]) {
                    waiting.push(item);
                    continue;
                }
                point = cellPoint(cell);
            }
            taken[cell] = true;
            result[item.key] = point;
        }
        var next = 0;
        for (var j = 0; j < waiting.length; j++) {
            while (taken[next])
                next++;
            taken[next] = true;
            result[waiting[j].key] = cellPoint(next);
        }
        return result;
    }

    /// Change the places of some icons; the others keep theirs.
    function setPlaces(changes) {
        var next = Object.assign({}, Settings.desktopPlaces);
        for (var key in changes) {
            if (changes[key])
                next[key] = changes[key];
            else
                delete next[key];
        }
        Settings.desktopPlaces = next;
    }

    /// Every icon of this screen gets the place it shows at now. Moving one
    /// icon then leaves the others where they are, instead of letting the
    /// ones without a place fill the cell it left.
    function keepPlaces(changes) {
        for (var i = 0; i < shown.length; i++) {
            var key = shown[i].key;
            var place = Settings.desktopPlaces[key];
            if (changes[key] === undefined && !(place && place.screen === screenName) && positions[key])
                changes[key] = { screen: screenName, x: positions[key].x, y: positions[key].y };
        }
        return changes;
    }

    /// "Sort by" without "Auto arrange icons": this screen's icons go to
    /// the first cells, in the new order, once.
    function arrange() {
        var changes = {};
        for (var i = 0; i < shown.length; i++) {
            var point = cellPoint(i);
            changes[shown[i].key] = { screen: screenName, x: point.x, y: point.y };
        }
        setPlaces(changes);
    }

    /// The data of a drag of the picked icons, started on `anchorKey` at a
    /// point of its cell: what a desktop needs to place them where they
    /// are dropped, on any screen.
    function dragData(anchorKey, grab) {
        var keys = [];
        var offsets = {};
        var anchor = positions[anchorKey] || Qt.point(0, 0);
        var list = pickedItems();
        for (var i = 0; i < list.length; i++) {
            var at = positions[list[i].key];
            if (!at)
                continue;
            keys.push(list[i].key);
            offsets[list[i].key] = { x: at.x - anchor.x, y: at.y - anchor.y };
        }
        if (!Settings.desktopAutoArrange)
            setPlaces(keepPlaces({}));
        return { urls: pickedUrls(), keys: keys, offsets: offsets, grab: { x: grab.x, y: grab.y } };
    }

    /// Dragged icons dropped at a point of this screen: the anchor's cell
    /// goes under the pointer where it was grabbed, the others keep their
    /// distance from it. On the grid, each takes the nearest free cell.
    function placeDragged(moving, x, y) {
        var taken = {};
        for (var key in positions) {
            if (moving.keys.indexOf(key) < 0)
                taken[cellAt(positions[key].x, positions[key].y)] = true;
        }
        var changes = {};
        for (var i = 0; i < moving.keys.length; i++) {
            var offset = moving.offsets[moving.keys[i]];
            var point = onScreen(x - moving.grab.x + offset.x, y - moving.grab.y + offset.y);
            if (Settings.desktopAlignToGrid) {
                var cell = cellAt(point.x, point.y);
                while (taken[cell])
                    cell++;
                taken[cell] = true;
                point = cellPoint(cell);
            }
            changes[moving.keys[i]] = { screen: screenName, x: Math.round(point.x), y: Math.round(point.y) };
        }
        setPlaces(Settings.desktopAutoArrange ? changes : keepPlaces(changes));
    }

    /// The icon a drag would drop into: the Recycle Bin or a folder under
    /// the pointer. It lights up as a picked icon does, as on Windows 7.
    property string dropTargetKey: ""

    /// The key of the trash or folder icon under (x, y) that a drag can drop
    /// into, or "": not one of the icons being dragged.
    function dropTargetAt(x, y) {
        var target = itemAt(x, y);
        if (!target || (target.kind !== "trash" && target.kind !== "folder"))
            return "";
        if (DragState.kind === "desktop" && DragState.data.keys.indexOf(target.key) >= 0)
            return "";
        return target.key;
    }

    /// Where a drag over empty desktop would put its (first) icon, for the
    /// pale place drawn there; null when it would drop into an icon.
    property var landing: null

    /// The top left the drop at (x, y) gives the icon under the pointer, as
    /// placeDragged and dropFiles place it: the shell's own icons keep where
    /// they were taken, files come centred under the pointer.
    function landingAt(x, y) {
        var point = DragState.kind === "desktop"
            ? onScreen(x - DragState.data.grab.x, y - DragState.data.grab.y)
            : onScreen(x - size.width / 2, y - size.height / 3);
        return Settings.desktopAlignToGrid ? cellPoint(cellAt(point.x, point.y)) : point;
    }

    // Whether the Recycle Bin has anything in it, for its icon. Files are
    // trashed where their mount allows it: ~/.local/share/Trash for $HOME's
    // own device, a .Trash-<uid> at the top of another mount. trash:///
    // (gvfs) counts them all, and its monitor says when that changes.
    property bool trashFull: false
    property bool trashRecount: false

    function countTrash() {
        if (trashCount.running)
            desktop.trashRecount = true;
        else
            trashCount.running = true;
    }

    Process {
        id: trashCount

        command: [Tools.gio, "info", "-a", "trash::item-count", "trash:///"]
        running: true
        stdout: StdioCollector {
            onStreamFinished: {
                var count = /trash::item-count:\s*(\d+)/.exec(text);
                desktop.trashFull = count !== null && Number(count[1]) > 0;
                if (desktop.trashRecount) {
                    desktop.trashRecount = false;
                    trashCount.running = true;
                }
            }
        }
    }

    Process {
        command: [Tools.gio, "monitor", "trash:///"]
        running: true
        stdout: SplitParser {
            onRead: desktop.countTrash()
        }
    }

    // ----- doing things -------------------------------------------------------

    function itemByKey(key) {
        for (var i = 0; i < items.length; i++) {
            if (items[i].key === key)
                return items[i];
        }
        return null;
    }

    function pickedItems() {
        var list = [];
        for (var i = 0; i < items.length; i++) {
            if (picked[items[i].key])
                list.push(items[i]);
        }
        return list;
    }

    function pick(key, add) {
        var next = add ? Object.assign({}, picked) : ({});
        if (add && picked[key])
            delete next[key];
        else
            next[key] = true;
        desktop.picked = next;
    }

    /// The system icons, as Windows 7's Desktop Icon Settings lists them.
    /// They are no files: they are not renamed, trashed or dragged out.
    readonly property var systemIcons: [
        { kind: "computer", name: "Computer", path: "/" },
        { kind: "userfiles", name: Apps.userName, path: Apps.home },
        { kind: "controlpanel", name: "Control Panel", path: "" }
    ]

    function isSystem(item) {
        return item.kind === "trash" || item.kind === "computer"
            || item.kind === "userfiles" || item.kind === "controlpanel";
    }

    function open(item) {
        if (!item)
            return;
        if (item.kind === "controlpanel")
            ControlPanelState.open("");
        else if (item.kind === "trash" || item.kind === "folder"
                 || item.kind === "computer" || item.kind === "userfiles")
            Apps.openPath(item.path);
        else if (item.kind === "shortcut")
            Quickshell.execDetached([Tools.gio, "launch", item.path]);
        else
            Quickshell.execDetached([Tools.gio, "open", item.path]);
    }

    function trash(list) {
        var paths = [];
        for (var i = 0; i < list.length; i++) {
            if (!desktop.isSystem(list[i]))
                paths.push(list[i].path);
        }
        if (paths.length > 0)
            Quickshell.execDetached([Tools.gio, "trash"].concat(paths));
        desktop.picked = ({});
    }

    function rename(item, name) {
        desktop.renameKey = "";
        desktop.forceActiveFocus();
        name = String(name).trim();
        if (!item || name.length === 0 || name === item.name || name.indexOf("/") >= 0)
            return;
        // The icon keeps its place under its new name.
        var changes = {};
        changes[parentOf(item.path) + "/" + name] = Settings.desktopPlaces[item.key];
        setPlaces(changes);
        Quickshell.execDetached([Tools.gio, "rename", item.path, name]);
    }

    /// The first "New folder", "New folder (2)" and so on that is free.
    function freeName(base, extension) {
        var taken = {};
        for (var i = 0; i < items.length; i++)
            taken[items[i].name] = true;
        var name = base + extension;
        for (var n = 2; taken[name]; n++)
            name = base + " (" + n + ")" + extension;
        return name;
    }

    // ----- dragging files ----------------------------------------------------------

    /// The icon under a point of the desktop, or null.
    function itemAt(x, y) {
        for (var i = 0; i < items.length; i++) {
            var cell = cells[items[i].key];
            if (!cell)
                continue;
            var r = cell.plateRect();
            if (x >= r.x && x < r.x + r.width && y >= r.y && y < r.y + r.height)
                return items[i];
        }
        return null;
    }

    function localPath(url) {
        return decodeURIComponent(String(url).replace(/^file:\/\//, ""));
    }

    function parentOf(path) {
        return path.substring(0, path.lastIndexOf("/")) || "/";
    }

    /// Files dropped at a point: into the Recycle Bin or a folder icon under
    /// it, else onto the desktop. Moved unless the drag asks for a copy;
    /// files already where they are dropped stay as they are.
    function dropFiles(urls, x, y, copy) {
        var target = itemAt(x, y);
        var paths = [];
        for (var i = 0; i < urls.length; i++) {
            if (String(urls[i]).startsWith("file://"))
                paths.push(localPath(urls[i]));
        }
        if (target && target.kind === "trash") {
            if (paths.length > 0)
                Quickshell.execDetached([Tools.gio, "trash"].concat(paths));
            return;
        }
        var folder = target && target.kind === "folder" ? target.path : desktop.folder;
        var moving = paths.filter(function (path) {
            return path !== folder && desktop.parentOf(path) !== folder;
        });
        if (moving.length === 0)
            return;
        // Files that come to the desktop show where they are dropped.
        if (folder === desktop.folder) {
            var changes = {};
            for (var j = 0; j < moving.length; j++) {
                var point = onScreen(x - size.width / 2 + j * size.width, y - size.height / 3);
                changes[folder + "/" + moving[j].split("/").pop()] = {
                    screen: screenName, x: Math.round(point.x), y: Math.round(point.y)
                };
            }
            setPlaces(changes);
        }
        Quickshell.execDetached([Tools.gio, copy ? "copy" : "move"].concat(moving, [folder + "/"]));
    }

    /// The picked icons as file URLs, for a drag to another program.
    function pickedUrls() {
        var list = pickedItems();
        var urls = [];
        for (var i = 0; i < list.length; i++) {
            if (!desktop.isSystem(list[i]))
                urls.push("file://" + encodeURI(list[i].path));
        }
        return urls;
    }

    function newFolder() {
        Quickshell.execDetached([Tools.gio, "mkdir", folder + "/" + freeName("New folder", "")]);
    }

    function newTextDocument() {
        var path = folder + "/" + freeName("New Text Document", ".txt");
        Quickshell.execDetached(["sh", "-c", ": >> \"$1\"", "sh", path]);
    }

    function openTerminal() {
        Quickshell.execDetached({ command: ["sh", "-c", Apps.terminal], workingDirectory: folder });
    }

    function refresh() {
        files.folder = "";
        files.folder = Qt.binding(function () { return "file://" + desktop.folder; });
        desktop.rebuild();
    }

    /// The icon of a shortcut: its own if the theme has it, else that of the
    /// program it starts, else a generic one.
    function shortcutIconFor(icon, exec) {
        if (icon && icon.startsWith("/"))
            return "file://" + icon;
        var path = icon ? Quickshell.iconPath(icon, true) : "";
        if (path)
            return path;
        var command = exec ? String(exec).trim().split(/\s+/)[0] : "";
        var entry = command ? DesktopEntries.heuristicLookup(command.split("/").pop()) : null;
        path = entry && entry.icon ? Quickshell.iconPath(entry.icon, true) : "";
        return path || Qt.resolvedUrl("../assets/icons/run.png");
    }

    // ----- the menus ------------------------------------------------------------

    function radio(label, setting, value) {
        return {
            label: label,
            checked: Settings[setting] === value,
            run: function () { Settings[setting] = value; }
        };
    }

    /// A "Sort by" line: the order, then, unless the icons arrange
    /// themselves, this screen's icons put in that order once.
    function sortBy(label, value) {
        return {
            label: label,
            checked: Settings.desktopSort === value,
            run: function () {
                Settings.desktopSort = value;
                desktop.rebuild();
                if (!Settings.desktopAutoArrange)
                    desktop.arrange();
            }
        };
    }

    function desktopEntries() {
        return [
            { label: "View", children: [
                radio("Large icons", "desktopIconSize", 64),
                radio("Medium icons", "desktopIconSize", 48),
                radio("Small icons", "desktopIconSize", 32),
                { separator: true },
                { label: "Auto arrange icons", checked: Settings.desktopAutoArrange,
                  run: function () { Settings.desktopAutoArrange = !Settings.desktopAutoArrange; } },
                { label: "Align icons to grid", checked: Settings.desktopAlignToGrid,
                  run: function () { Settings.desktopAlignToGrid = !Settings.desktopAlignToGrid; } },
                { separator: true },
                { label: "Show desktop icons", checked: Settings.showDesktopIcons,
                  run: function () { Settings.showDesktopIcons = !Settings.showDesktopIcons; } }
            ] },
            { label: "Sort by", children: [
                sortBy("Name", "name"),
                sortBy("Size", "size"),
                sortBy("Item type", "type"),
                sortBy("Date modified", "date")
            ] },
            { label: "Refresh", run: desktop.refresh },
            { separator: true },
            { label: "New", children: [
                { label: "Folder", icon: Qt.resolvedUrl("../assets/icons/folder.png"),
                  run: desktop.newFolder },
                { separator: true },
                { label: "Text Document", icon: Qt.resolvedUrl("../assets/icons/file.png"),
                  run: desktop.newTextDocument },
                { label: "Sticky Note",
                  run: function () {
                      StickyNotes.add(desktop.screenName,
                          Math.min(desktop.width - 230, desktop.menuAt.x),
                          Math.min(desktop.height - 210, desktop.menuAt.y));
                  } }
            ] },
            { label: "Open in Terminal", icon: Qt.resolvedUrl("../assets/icons/run.png"),
              run: desktop.openTerminal },
            { separator: true },
            { label: "Screen resolution", icon: Qt.resolvedUrl("../assets/icons/computer.png"),
              run: function () { ControlPanelState.open("display"); } },
            { label: "Gadgets", children: [
                { label: "CPU Meter", checked: Settings.showCpuMeter,
                  run: function () { Settings.showCpuMeter = !Settings.showCpuMeter; } },
                { label: "Clock", checked: Settings.showClockGadget,
                  run: function () { Settings.showClockGadget = !Settings.showClockGadget; } }
            ] },
            { label: "Personalize", icon: Qt.resolvedUrl("../assets/icons/control-panel.png"),
              run: function () { ControlPanelState.open("background"); } }
        ];
    }

    function itemEntries(item) {
        var list = pickedItems();
        if (item.kind === "computer" || item.kind === "userfiles" || item.kind === "controlpanel") {
            return [
                { label: "Open", run: function () { desktop.open(item); } },
                { separator: true },
                { label: "Remove from the desktop", run: function () {
                    Settings.desktopSystemIcons = Settings.desktopSystemIcons.filter(function (kind) {
                        return kind !== item.kind;
                    });
                } }
            ];
        }
        if (item.kind === "trash") {
            return [
                { label: "Open", run: function () { desktop.open(item); } },
                { separator: true },
                { label: "Empty Recycle Bin", enabled: desktop.trashFull,
                  run: function () { Quickshell.execDetached([Tools.gio, "trash", "--empty"]); } }
            ];
        }
        return [
            { label: item.kind === "shortcut" ? "Open" : "Open", run: function () {
                for (var i = 0; i < list.length; i++)
                    desktop.open(list[i]);
            } },
            { label: "Open file location", run: function () { Apps.openPath(desktop.folder); } },
            { separator: true },
            { label: "Delete", run: function () { desktop.trash(list); } },
            { label: "Rename", enabled: list.length === 1,
              run: function () { desktop.renameKey = item.key; } }
        ];
    }

    /// The open menu's lines, or [], and where it was asked for. The shell
    /// draws it in a surface of its own above everything (shell.qml).
    property var menuEntries: []
    property rect menuAt: Qt.rect(0, 0, 1, 1)
    /// The screen the desktop is on, for the notes made on it.
    property string screenName: ""
    readonly property bool menuOpen: menuEntries.length > 0

    function showMenu(entries, x, y) {
        // A fresh list each time, so a check mark is up to date.
        desktop.menuEntries = [];
        desktop.menuAt = Qt.rect(x, y, 1, 1);
        desktop.menuEntries = entries;
    }

    function closeMenu() {
        desktop.menuEntries = [];
    }

    // ----- the empty desktop: picks, the box, the menu ----------------------------

    MouseArea {
        id: background

        property point from: Qt.point(0, 0)
        property bool boxing: false

        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton

        onPressed: function (event) {
            desktop.closeMenu();
            if (desktop.renaming)
                renameCommit();
            desktop.forceActiveFocus();
            if (event.button === Qt.RightButton) {
                desktop.picked = ({});
                desktop.showMenu(desktop.desktopEntries(), event.x, event.y);
                return;
            }
            if (!(event.modifiers & Qt.ControlModifier))
                desktop.picked = ({});
            background.from = Qt.point(event.x, event.y);
            background.boxing = true;
        }
        onPositionChanged: function (event) {
            if (!background.boxing)
                return;
            box.x = Math.min(background.from.x, event.x);
            box.y = Math.min(background.from.y, event.y);
            box.width = Math.abs(event.x - background.from.x);
            box.height = Math.abs(event.y - background.from.y);
            box.visible = box.width > 2 || box.height > 2;
            if (!box.visible)
                return;
            var next = ({});
            for (var i = 0; i < desktop.items.length; i++) {
                var cell = desktop.cells[desktop.items[i].key];
                if (!cell)
                    continue;
                var r = cell.plateRect();
                if (r.x < box.x + box.width && r.x + r.width > box.x
                    && r.y < box.y + box.height && r.y + r.height > box.y)
                    next[desktop.items[i].key] = true;
            }
            desktop.picked = next;
        }
        onReleased: {
            background.boxing = false;
            box.visible = false;
        }
        onWheel: function (event) {
            if (!(event.modifiers & Qt.ControlModifier) || event.angleDelta.y === 0) {
                event.accepted = false;
                return;
            }
            desktop.zoom(event.angleDelta.y > 0 ? 1 : -1);
        }

        function renameCommit() {
            var cell = desktop.cells[desktop.renameKey];
            if (cell)
                cell.commitRename();
            else
                desktop.renameKey = "";
        }
    }

    DropArea {
        id: drop

        anchors.fill: parent
        keys: ["text/uri-list", "application/x-shojiwm-desktop-icons"]

        onPositionChanged: function (event) {
            desktop.dropTargetKey = desktop.dropTargetAt(event.x, event.y);
            desktop.landing = desktop.dropTargetKey.length > 0
                ? null : desktop.landingAt(event.x, event.y);
        }
        onExited: {
            desktop.dropTargetKey = "";
            desktop.landing = null;
        }

        onDropped: function (event) {
            desktop.dropTargetKey = "";
            desktop.landing = null;
            var copy = event.proposedAction === Qt.CopyAction;
            // The shell's own icons are not read back through Wayland: see
            // DragState. Dropped on the Recycle Bin or a folder, their files
            // go there; anywhere else, the icons move there, on this screen.
            if (DragState.kind === "desktop") {
                var moving = DragState.data;
                var target = desktop.itemAt(event.x, event.y);
                if (target && moving.keys.indexOf(target.key) < 0
                    && (target.kind === "trash" || target.kind === "folder")) {
                    desktop.dropFiles(moving.urls, event.x, event.y, copy);
                    event.accept(copy ? Qt.CopyAction : Qt.MoveAction);
                    return;
                }
                desktop.placeDragged(moving, event.x, event.y);
                event.accept(Qt.MoveAction);
                return;
            }
            var urls = event.hasUrls ? event.urls : [];
            if (urls.length === 0)
                return;
            desktop.dropFiles(urls, event.x, event.y, copy);
            event.accept(copy ? Qt.CopyAction : Qt.MoveAction);
        }
    }

    // ----- the icons ------------------------------------------------------------

    // The place a drag over empty desktop would drop at: a pale plate the
    // size of an icon's, under the icons.
    Rectangle {
        visible: desktop.landing !== null && Settings.showDesktopIcons
        x: (desktop.landing ? desktop.landing.x : 0) + 2
        y: (desktop.landing ? desktop.landing.y : 0) + 2
        width: desktop.size.width - 4
        height: desktop.size.height - 4
        radius: Scheme.corner(3)
        color: "#1affffff"
        border.width: 1
        border.color: "#80ffffff"
    }

    Item {
        anchors.fill: parent
        visible: Settings.showDesktopIcons

        Repeater {
            model: desktop.shown

            delegate: Item {
                id: cell

                required property var modelData
                readonly property var item: modelData
                readonly property bool isPicked: desktop.picked[modelData.key] === true
                /// A drag is over this trash or folder: dropping puts it there.
                readonly property bool isDropTarget: desktop.dropTargetKey === modelData.key
                readonly property bool editing: desktop.renameKey === modelData.key
                readonly property point at: desktop.positions[modelData.key] || Qt.point(2, 2)
                /// Where the pointer took the icon, in the cell, for a drag.
                property point grab: Qt.point(0, 0)

                x: at.x
                y: at.y
                width: desktop.size.width
                height: desktop.size.height
                z: isPicked || editing ? 2 : 0

                Component.onCompleted: desktop.cells[modelData.key] = cell
                Component.onDestruction: {
                    if (desktop.cells[modelData.key] === cell)
                        delete desktop.cells[modelData.key];
                }

                function plateRect() {
                    var at = cell.mapToItem(desktop, plate.x, plate.y);
                    return Qt.rect(at.x, at.y, plate.width, plate.height);
                }

                function commitRename() {
                    desktop.rename(cell.item, field.text);
                }

                readonly property string iconSource: {
                    var item = cell.item;
                    if (item.kind === "trash")
                        return Qt.resolvedUrl(desktop.trashFull
                            ? "../assets/icons/recycle-bin-full.png"
                            : "../assets/icons/recycle-bin.png");
                    if (item.kind === "folder")
                        return Qt.resolvedUrl("../assets/icons/folder.png");
                    if (item.kind === "computer")
                        return Qt.resolvedUrl("../assets/icons/computer.png");
                    if (item.kind === "userfiles")
                        return Qt.resolvedUrl("../assets/icons/home.png");
                    if (item.kind === "controlpanel")
                        return Qt.resolvedUrl("../assets/icons/control-panel.png");
                    if (item.kind === "shortcut")
                        return shortcutIcon;
                    if (["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg"].indexOf(item.suffix) >= 0)
                        return "file://" + item.path;
                    return Apps.fileIcon(item.name);
                }

                // The name and the icon of a shortcut, read from its .desktop file.
                property string shortcutName: ""
                property string shortcutIcon: Qt.resolvedUrl("../assets/icons/run.png")

                FileView {
                    path: cell.item.kind === "shortcut" ? cell.item.path : ""
                    printErrors: false
                    watchChanges: true
                    onFileChanged: reload()
                    onLoaded: {
                        var lines = text().split("\n");
                        var section = "";
                        var fields = {};
                        for (var i = 0; i < lines.length; i++) {
                            var line = lines[i].trim();
                            if (line.startsWith("[")) {
                                section = line;
                                continue;
                            }
                            if (section !== "[Desktop Entry]")
                                continue;
                            var match = /^(Name|Icon|Exec)\s*=\s*(.*)$/.exec(line);
                            if (match)
                                fields[match[1]] = match[2];
                        }
                        cell.shortcutName = fields.Name || "";
                        cell.shortcutIcon = desktop.shortcutIconFor(fields.Icon, fields.Exec);
                    }
                }

                readonly property string label: item.kind === "shortcut"
                    ? (shortcutName || item.name.replace(/\.desktop$/i, ""))
                    : item.name

                // The plate under the pointer or the pick: pale glass.
                Rectangle {
                    id: plate

                    x: 2
                    y: 2
                    width: parent.width - 4
                    height: Math.min(cell.height + (cell.isPicked ? 40 : 0),
                        picture.height + 8 + (cell.editing ? field.height : name.height) + 6)
                    radius: Scheme.corner(3)
                    visible: cell.isPicked || cell.isDropTarget || iconArea.containsMouse
                    color: cell.isPicked || cell.isDropTarget
                        ? (iconArea.containsMouse || cell.isDropTarget ? "#5c9fd2f7" : "#4d86c0ec")
                        : "#26ffffff"
                    border.width: 1
                    border.color: cell.isPicked || cell.isDropTarget ? "#a6cfeaff" : "#59ffffff"
                }

                Image {
                    id: picture

                    anchors.horizontalCenter: parent.horizontalCenter
                    y: 6
                    width: desktop.size.icon
                    height: desktop.size.icon
                    // The screen's pixels, so a big icon stays sharp.
                    sourceSize.width: Math.ceil(desktop.size.icon * Screen.devicePixelRatio)
                    sourceSize.height: Math.ceil(desktop.size.icon * Screen.devicePixelRatio)
                    fillMode: Image.PreserveAspectFit
                    asynchronous: true
                    smooth: true
                    source: cell.iconSource
                    // It grows a little under a drag it would take.
                    scale: cell.isDropTarget ? 1.12 : 1

                    Behavior on scale {
                        NumberAnimation { duration: Settings.motion(120); easing.type: Easing.OutCubic }
                    }
                }

                // The shortcut arrow, in the bottom left corner of the icon.
                Rectangle {
                    visible: cell.item.kind === "shortcut"
                    x: picture.x
                    y: picture.y + picture.height - height
                    width: Math.round(desktop.size.icon / 3)
                    height: width
                    radius: Scheme.corner(1)
                    color: "#ffffff"
                    border.width: 1
                    border.color: "#8c9aa6"

                    Text {
                        anchors.centerIn: parent
                        text: "↗"
                        color: "#1f5fa8"
                        font.pixelSize: parent.width - 3
                        font.bold: true
                    }
                }

                // The name: white, with a dark shadow so it reads on any
                // wallpaper; two lines at most until the icon is picked.
                Item {
                    id: name

                    visible: !cell.editing
                    anchors.horizontalCenter: parent.horizontalCenter
                    y: picture.y + picture.height + 4
                    width: cell.width - 6
                    height: nameText.implicitHeight

                    Text {
                        x: 1
                        y: 1
                        width: parent.width
                        text: nameText.text
                        font: nameText.font
                        horizontalAlignment: Text.AlignHCenter
                        wrapMode: nameText.wrapMode
                        maximumLineCount: nameText.maximumLineCount
                        elide: Text.ElideRight
                        color: "#d9000000"
                    }

                    Text {
                        id: nameText

                        width: parent.width
                        text: cell.label
                        color: "#ffffff"
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSize
                        horizontalAlignment: Text.AlignHCenter
                        wrapMode: Text.WrapAtWordBoundaryOrAnywhere
                        maximumLineCount: cell.isPicked ? 6 : 2
                        elide: Text.ElideRight
                    }
                }

                // Renaming: a plain white field round the name.
                Rectangle {
                    visible: cell.editing
                    anchors.horizontalCenter: parent.horizontalCenter
                    y: picture.y + picture.height + 3
                    width: cell.width - 4
                    height: field.height + 2
                    color: "#ffffff"
                    border.width: 1
                    border.color: "#000000"

                    TextInput {
                        id: field

                        x: 2
                        y: 1
                        width: parent.width - 4
                        text: cell.item.name
                        color: "#000000"
                        selectionColor: "#3399ff"
                        selectedTextColor: "#ffffff"
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSize
                        horizontalAlignment: TextInput.AlignHCenter
                        wrapMode: TextInput.WrapAnywhere
                        selectByMouse: true

                        onAccepted: cell.commitRename()
                        Keys.onEscapePressed: {
                            desktop.renameKey = "";
                            desktop.forceActiveFocus();
                        }
                    }
                }

                onEditingChanged: {
                    if (!editing)
                        return;
                    field.text = cell.item.name;
                    field.forceActiveFocus();
                    // The name without its extension is picked, as Windows does.
                    var dot = cell.item.dir ? -1 : cell.item.name.lastIndexOf(".");
                    field.select(0, dot > 0 ? dot : cell.item.name.length);
                }

                // What the pointer drags: invisible, the drag shows a picture of
                // the icon. It carries the picked files to whatever takes them,
                // and the icons themselves to a desktop (DragState): the system
                // icons, which are no files, only there.
                Item {
                    id: dragProxy

                    readonly property var urls: cell.isPicked ? desktop.pickedUrls() : []

                    width: plate.width
                    height: plate.height
                    Drag.active: iconArea.drag.active
                    Drag.dragType: Drag.Automatic
                    Drag.supportedActions: Qt.MoveAction | Qt.CopyAction | Qt.LinkAction
                    Drag.hotSpot.x: cell.grab.x - plate.x
                    Drag.hotSpot.y: cell.grab.y - plate.y
                    Drag.keys: dragProxy.urls.length > 0
                        ? ["text/uri-list", "application/x-shojiwm-desktop-icons"]
                        : ["application/x-shojiwm-desktop-icons"]
                    Drag.mimeData: dragProxy.urls.length > 0
                        ? ({ "text/uri-list": dragProxy.urls.join("\r\n"),
                             "application/x-shojiwm-desktop-icons": cell.item.key })
                        : ({ "application/x-shojiwm-desktop-icons": cell.item.key })
                    Drag.onDragStarted: DragState.start("desktop", desktop.dragData(cell.item.key, cell.grab))
                    Drag.onDragFinished: {
                        DragState.finish();
                        dragProxy.x = 0;
                        dragProxy.y = 0;
                    }
                }

                MouseArea {
                    id: iconArea

                    x: plate.x
                    y: plate.y
                    width: plate.width
                    height: plate.height
                    enabled: !cell.editing
                    hoverEnabled: true
                    acceptedButtons: Qt.LeftButton | Qt.RightButton
                    drag.target: dragProxy

                    onReleased: {
                        dragProxy.x = 0;
                        dragProxy.y = 0;
                    }

                    onPressed: function (event) {
                        desktop.closeMenu();
                        if (desktop.renaming)
                            background.renameCommit();
                        desktop.forceActiveFocus();
                        var add = (event.modifiers & Qt.ControlModifier) !== 0;
                        if (event.button === Qt.RightButton) {
                            if (!cell.isPicked)
                                desktop.pick(cell.item.key, false);
                            var at = iconArea.mapToItem(desktop, event.x, event.y);
                            desktop.showMenu(desktop.itemEntries(cell.item), at.x, at.y);
                            return;
                        }
                        if (add || !cell.isPicked)
                            desktop.pick(cell.item.key, add);
                        cell.grab = iconArea.mapToItem(cell, event.x, event.y);
                        picture.grabToImage(function (result) {
                            dragProxy.Drag.imageSource = result.url;
                        }, Qt.size(picture.width, picture.height));
                    }
                    onClicked: function (event) {
                        // A click on one of several picked icons picks it alone.
                        if (event.button === Qt.LeftButton
                            && !(event.modifiers & Qt.ControlModifier))
                            desktop.pick(cell.item.key, false);
                    }
                    onDoubleClicked: function (event) {
                        if (event.button === Qt.LeftButton)
                            desktop.open(cell.item);
                    }
                }
            }
        }
    }

    // The box a drag draws on the empty desktop.
    Rectangle {
        id: box

        visible: false
        color: "#4d3399ff"
        border.width: 1
        border.color: "#cc3399ff"
    }
}
