pragma ComponentBehavior: Bound

import QtQuick
import Qt.labs.folderlistmodel
import Quickshell
import Quickshell.Io
import ".."
import "../theme.js" as Theme
import "../tools.js" as Tools

// The desktop icons, over the wallpaper: the Recycle Bin, then whatever is
// in the desktop folder (XDG_DESKTOP_DIR, ~/Desktop by default), in columns
// from the top left as Windows 7 arranges them.
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
            if (list.length === 1 && list[0].kind !== "trash")
                desktop.renameKey = list[0].key;
        } else if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
            var open = desktop.pickedItems();
            for (var i = 0; i < open.length; i++)
                desktop.open(open[i]);
        } else if (ctrl && event.key === Qt.Key_A) {
            var all = ({});
            for (var k = 0; k < desktop.items.length; k++)
                all[desktop.items[k].key] = true;
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

    readonly property var sizes: ({
        "large": { icon: 64, width: 96, height: 100 },
        "medium": { icon: 48, width: 78, height: 84 },
        "small": { icon: 32, width: 78, height: 66 }
    })
    readonly property var size: sizes[Settings.desktopIconSize] || sizes.medium

    /// One size up (`step` 1) or down (-1), as Ctrl+wheel and Ctrl+plus or
    /// minus do on Windows.
    function zoom(step) {
        var order = ["small", "medium", "large"];
        var index = order.indexOf(Settings.desktopIconSize);
        if (index < 0)
            index = 1;
        Settings.desktopIconSize = order[Math.max(0, Math.min(order.length - 1, index + step))];
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
        list.unshift({ key: "trash:", kind: "trash", path: "trash:///", name: "Recycle Bin" });
        desktop.items = list;
    }

    // Whether the Recycle Bin has anything in it, for its icon.
    property bool trashFull: false

    FolderListModel {
        id: trashFiles
        folder: "file://" + Apps.dataHome + "/Trash/files"
        showHidden: true
        showDotAndDotDot: false
        onCountChanged: desktop.trashFull = trashFiles.count > 0
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

    function open(item) {
        if (!item)
            return;
        if (item.kind === "trash" || item.kind === "folder")
            Apps.openPath(item.path);
        else if (item.kind === "shortcut")
            Quickshell.execDetached([Tools.gio, "launch", item.path]);
        else
            Quickshell.execDetached([Tools.gio, "open", item.path]);
    }

    function trash(list) {
        var paths = [];
        for (var i = 0; i < list.length; i++) {
            if (list[i].kind !== "trash")
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
        if (moving.length > 0)
            Quickshell.execDetached([Tools.gio, copy ? "copy" : "move"].concat(moving, [folder + "/"]));
    }

    /// The picked icons as file URLs, for a drag to another program.
    function pickedUrls() {
        var list = pickedItems();
        var urls = [];
        for (var i = 0; i < list.length; i++) {
            if (list[i].kind !== "trash")
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

    function desktopEntries() {
        return [
            { label: "View", children: [
                radio("Large icons", "desktopIconSize", "large"),
                radio("Medium icons", "desktopIconSize", "medium"),
                radio("Small icons", "desktopIconSize", "small"),
                { separator: true },
                { label: "Show desktop icons", checked: Settings.showDesktopIcons,
                  run: function () { Settings.showDesktopIcons = !Settings.showDesktopIcons; } }
            ] },
            { label: "Sort by", children: [
                radio("Name", "desktopSort", "name"),
                radio("Size", "desktopSort", "size"),
                radio("Item type", "desktopSort", "type"),
                radio("Date modified", "desktopSort", "date")
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
            { label: "Gadgets", checked: Settings.showCpuMeter,
              run: function () { Settings.showCpuMeter = !Settings.showCpuMeter; } },
            { label: "Personalize", icon: Qt.resolvedUrl("../assets/icons/control-panel.png"),
              run: function () { ControlPanelState.open("background"); } }
        ];
    }

    function itemEntries(item) {
        var list = pickedItems();
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
        keys: ["text/uri-list"]

        onDropped: function (event) {
            // The shell's own icons are not read back through Wayland: see
            // DragState.
            var urls = DragState.kind === "files" ? DragState.data
                : event.hasUrls ? event.urls : [];
            if (urls.length === 0)
                return;
            var copy = event.proposedAction === Qt.CopyAction;
            desktop.dropFiles(urls, event.x, event.y, copy);
            event.accept(copy ? Qt.CopyAction : Qt.MoveAction);
        }
    }

    // ----- the icons ------------------------------------------------------------

    GridView {
        id: grid

        x: 2
        y: 2
        width: parent.width - 4
        height: parent.height - Theme.taskbarHeight - 4
        visible: Settings.showDesktopIcons
        interactive: false
        flow: GridView.FlowTopToBottom
        cellWidth: desktop.size.width
        cellHeight: desktop.size.height
        model: desktop.items
        cacheBuffer: 10000

        delegate: Item {
            id: cell

            required property var modelData
            required property int index
            readonly property var item: modelData
            readonly property bool isPicked: desktop.picked[modelData.key] === true
            readonly property bool editing: desktop.renameKey === modelData.key

            width: grid.cellWidth
            height: grid.cellHeight
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
                radius: 3
                visible: cell.isPicked || iconArea.containsMouse
                color: cell.isPicked
                    ? (iconArea.containsMouse ? "#5c9fd2f7" : "#4d86c0ec")
                    : "#26ffffff"
                border.width: 1
                border.color: cell.isPicked ? "#a6cfeaff" : "#59ffffff"
            }

            Image {
                id: picture

                anchors.horizontalCenter: parent.horizontalCenter
                y: 6
                width: desktop.size.icon
                height: desktop.size.icon
                sourceSize.width: desktop.size.icon * 2
                sourceSize.height: desktop.size.icon * 2
                fillMode: Image.PreserveAspectFit
                asynchronous: true
                smooth: true
                source: cell.iconSource
            }

            // The shortcut arrow, in the bottom left corner of the icon.
            Rectangle {
                visible: cell.item.kind === "shortcut"
                x: picture.x
                y: picture.y + picture.height - height
                width: Math.round(desktop.size.icon / 3)
                height: width
                radius: 1
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
            // the icon and carries the picked files to whatever takes them.
            Item {
                id: dragProxy

                width: plate.width
                height: plate.height
                Drag.active: iconArea.drag.active
                Drag.dragType: Drag.Automatic
                Drag.supportedActions: Qt.MoveAction | Qt.CopyAction | Qt.LinkAction
                Drag.hotSpot.x: width / 2
                Drag.hotSpot.y: height / 2
                Drag.keys: ["text/uri-list"]
                Drag.mimeData: ({ "text/uri-list": desktop.pickedUrls().join("\r\n") })
                Drag.onDragStarted: DragState.start("files", desktop.pickedUrls())
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
                drag.target: cell.item.kind === "trash" ? null : dragProxy

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

    // The box a drag draws on the empty desktop.
    Rectangle {
        id: box

        visible: false
        color: "#4d3399ff"
        border.width: 1
        border.color: "#cc3399ff"
    }
}
