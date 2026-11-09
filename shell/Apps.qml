pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import Quickshell.Wayland
import "theme.js" as Theme

// What the taskbar and the Start menu know about programs and windows.
//
// Windows come from zwlr_foreign_toplevel_management_v1 through Quickshell's
// ToplevelManager, so nothing here is specific to ShojiWM. Programs come from
// the .desktop files of the system.
//
// A singleton: the shell is built once per screen, the lists are not.
Singleton {
    id: root

    // The programs the places open. Change these to the programs you use.
    /// The terminal: the command in SHOJIWM_WIN7_TERMINAL, run by a shell,
    /// else kitty.
    readonly property string terminal: Quickshell.env("SHOJIWM_WIN7_TERMINAL") || "kitty"
    /// The task manager: the command in SHOJIWM_WIN7_TASK_MANAGER, run by a
    /// shell, else Mission Center. Ctrl+Shift+Escape opens the same one.
    readonly property string taskManager: Quickshell.env("SHOJIWM_WIN7_TASK_MANAGER") || "missioncenter"
    readonly property string fileManager: "thunar"
    readonly property string browser: "chromium"
    /// The programs pinned to the Start menu, and the ones that keep a button
    /// on the taskbar while they are closed, left to right. Kept in
    /// settings.json; "Pin to ..." in the menus changes them.
    readonly property var pinnedIds: Settings.startPinned
    readonly property var taskbarPinnedIds: Settings.taskbarPinned

    /// The .desktop file name of an entry, as the pinned lists keep it.
    function entryFileId(entry) {
        return entry ? String(entry.id) + ".desktop" : "";
    }

    function isPinned(entry) {
        return pinnedIds.indexOf(entryFileId(entry)) >= 0;
    }

    function isTaskbarPinned(entry) {
        return taskbarPinnedIds.indexOf(entryFileId(entry)) >= 0;
    }

    function setPinned(entry, on) {
        if (entry)
            Settings.startPinned = Settings.withMember(Settings.startPinned, entryFileId(entry), on);
    }

    function setTaskbarPinned(entry, on) {
        if (entry)
            Settings.taskbarPinned = Settings.withMember(Settings.taskbarPinned, entryFileId(entry), on);
    }

    /// The .desktop file name a drop carries: a program dragged from the
    /// shell, or a .desktop file from the desktop or a file manager. "" for
    /// anything else.
    function droppedProgramId(event) {
        // The shell's own drags are not read back through Wayland: see
        // DragState.
        var urls = [];
        if (DragState.kind === "program")
            return String(DragState.data);
        if (DragState.kind === "files")
            urls = DragState.data;
        else if (DragState.kind === "" && event.hasUrls)
            urls = event.urls;
        for (var i = 0; i < urls.length; i++) {
            var name = decodeURIComponent(String(urls[i])).split("/").pop();
            if (name.endsWith(".desktop"))
                return name;
        }
        return "";
    }


    // ----- windows ----------------------------------------------------------

    /// Every open window, oldest first.
    readonly property var windows: ToplevelManager.toplevels
        ? ToplevelManager.toplevels.values : []
    /// The window that has the keyboard.
    readonly property var activeWindow: ToplevelManager.activeToplevel

    // ----- grouping ---------------------------------------------------------

    /// Combine the windows of one program into one button.
    readonly property bool combine: Settings.combineWindows

    /// The row of the taskbar, left to right. Each entry is
    /// `{ key, name, icon, entry, pinned, windows }`: the pinned programs
    /// first, in the order of `taskbarPinnedIds`, then the others in the
    /// order their first window opened. A pinned program with no window open
    /// has an empty `windows`.
    property var groups: []

    /// The order the buttons were dragged into, as group keys, for this
    /// session. Buttons not in it keep their place after the ones that are.
    property var buttonOrder: []

    function isControlPanel(toplevel) {
        return toplevel && toplevel.title === "Control Panel"
            && String(toplevel.appId).toLowerCase().indexOf("quickshell") >= 0;
    }

    /// The key of the program a window belongs to, whatever `combine` says.
    function programKey(toplevel) {
        if (isControlPanel(toplevel))
            return "controlpanel";
        var appId = toplevel ? toplevel.appId : "";
        var entry = entryFor(appId);
        if (entry)
            return "entry:" + entry.id;
        return "appid:" + (appId ? String(appId).toLowerCase() : "<none>");
    }

    function rebuildGroups() {
        var list = [];
        // The group each program's next window goes into, or after.
        var index = ({});
        var open = windows;

        var pins = taskbarPinned;
        for (var p = 0; p < pins.length; p++) {
            var pinnedKey = "entry:" + pins[p].id;
            if (index[pinnedKey])
                continue;
            var pinnedGroup = {
                key: pinnedKey,
                entry: pins[p],
                name: pins[p].name,
                icon: entryIcon(pins[p]),
                pinned: true,
                windows: []
            };
            index[pinnedKey] = pinnedGroup;
            list.push(pinnedGroup);
        }

        for (var i = 0; i < open.length; i++) {
            var toplevel = open[i];
            if (!toplevel)
                continue;
            var key = programKey(toplevel);
            var group = index[key];

            // Not combined: the first window takes the pinned button's place,
            // each later one gets its own button just after the last one of
            // the same program.
            if (group && !root.combine && group.windows.length > 0) {
                var own = {
                    key: "window:" + i,
                    entry: group.entry,
                    name: group.name,
                    icon: group.icon,
                    pinned: false,
                    windows: [toplevel]
                };
                list.splice(list.indexOf(group) + 1, 0, own);
                index[key] = own;
                continue;
            }

            if (!group) {
                var entry = entryFor(toplevel.appId);
                // The shell's own window: the Control Panel.
                var ours = isControlPanel(toplevel);
                group = {
                    key: root.combine ? key : "window:" + i,
                    entry: entry,
                    name: ours ? "Control Panel"
                        : entry && entry.name ? entry.name : titleOf(toplevel),
                    icon: ours ? Qt.resolvedUrl("assets/icons/control-panel.png")
                        : iconFor(toplevel.appId),
                    pinned: false,
                    windows: []
                };
                index[key] = group;
                list.push(group);
            }
            group.windows.push(toplevel);
        }

        var order = root.buttonOrder;
        var ranked = list.map(function (group, natural) {
            var at = order.indexOf(group.key);
            return { group: group, rank: at >= 0 ? at : order.length + natural };
        });
        ranked.sort(function (a, b) { return a.rank - b.rank; });
        groups = ranked.map(function (item) { return item.group; });
    }

    /// Put the button of `key` before the button now at `index`, or last for
    /// `groups.length`. The pinned programs keep that order in settings.json.
    function moveButton(key, index) {
        var keys = groups.map(function (group) { return group.key; });
        var from = keys.indexOf(key);
        if (from < 0)
            return;
        keys.splice(from, 1);
        keys.splice(from < index ? index - 1 : index, 0, key);
        root.buttonOrder = keys;
        var pins = [];
        for (var i = 0; i < keys.length; i++) {
            var group = groups.filter(function (g) { return g.key === keys[i]; })[0];
            if (group && group.pinned && group.entry)
                pins.push(entryFileId(group.entry));
        }
        Settings.taskbarPinned = pins;
        rebuildGroups();
    }

    /// Pin a program dropped on the taskbar before the button now at `index`.
    function pinDroppedAt(id, index) {
        var entry = entryById(id);
        if (!entry)
            return;
        var key = "entry:" + entry.id;
        var keys = groups.map(function (group) { return group.key; }).filter(function (k) {
            return k !== key;
        });
        keys.splice(Math.max(0, Math.min(keys.length, index)), 0, key);
        root.buttonOrder = keys;
        if (!isTaskbarPinned(entry))
            setTaskbarPinned(entry, true);
        rebuildGroups();
    }

    onWindowsChanged: rebuildGroups()
    onCombineChanged: rebuildGroups()
    onTaskbarPinnedChanged: rebuildGroups()
    Component.onCompleted: rebuildGroups()

    // A window's app id can arrive after the window itself, and the grouping
    // is built from it. The list changing does not catch that: the window is
    // already in it.
    Instantiator {
        model: ToplevelManager.toplevels

        delegate: QtObject {
            required property var modelData
            readonly property string appId: modelData ? modelData.appId : ""
            onAppIdChanged: root.rebuildGroups()
        }
    }

    /// What a left click on a group does. No window: start the program. One
    /// window: show it, or put it away when it is the one you are looking at.
    /// Several: the caller shows the thumbnails instead.
    /// When each pinned program was last started from its button, by
    /// .desktop file name: clicks on it while it starts are not new starts.
    property var starting: ({})
    readonly property int startingMs: 3000

    function activateGroup(group) {
        if (!group)
            return;
        if (group.windows.length === 0) {
            var id = entryFileId(group.entry);
            var now = Date.now();
            if (id.length > 0 && now - (root.starting[id] || 0) < root.startingMs)
                return;
            var next = Object.assign({}, root.starting);
            next[id] = now;
            root.starting = next;
            launch(group.entry);
            return;
        }
        toggleWindow(group.windows[0]);
    }

    /// `true` while any window of the group has the keyboard.
    function groupIsCurrent(group) {
        if (!group)
            return false;
        for (var i = 0; i < group.windows.length; i++) {
            if (group.windows[i].activated && !group.windows[i].minimized)
                return true;
        }
        return false;
    }

    function titleOf(toplevel) {
        if (!toplevel)
            return "";
        if (toplevel.title && toplevel.title.length > 0)
            return toplevel.title;
        return toplevel.appId || "Window";
    }

    /// The .desktop entry of an app id, or null.
    function entryFor(appId) {
        if (!appId)
            return null;
        return DesktopEntries.byId(appId) || DesktopEntries.heuristicLookup(appId);
    }

    function iconFor(appId) {
        var entry = entryFor(appId);
        if (entry && entry.icon)
            return Quickshell.iconPath(entry.icon, "application-x-executable");
        if (appId)
            return Quickshell.iconPath(String(appId).toLowerCase(),
                                       "application-x-executable");
        return Quickshell.iconPath("application-x-executable", true);
    }

    function entryIcon(entry) {
        if (entry && entry.icon)
            return Quickshell.iconPath(entry.icon, "application-x-executable");
        return Quickshell.iconPath("application-x-executable", true);
    }

    /// Super+1 to Super+9, as Windows 7: the Nth taskbar button (from 1).
    /// Not running: start it. One window: show it, or put it away when it
    /// is the active one. Several: the next one each time. `another` starts
    /// a new instance instead (Super+Shift+N).
    function activateButton(number, another) {
        var group = root.groups[number - 1];
        if (!group)
            return;
        if (another || group.windows.length === 0) {
            if (another)
                launch(group.entry);
            else
                activateGroup(group);
            return;
        }
        if (group.windows.length === 1) {
            toggleWindow(group.windows[0]);
            return;
        }
        var active = -1;
        for (var i = 0; i < group.windows.length; i++) {
            if (group.windows[i].activated)
                active = i;
        }
        var next = group.windows[(active + 1) % group.windows.length];
        next.minimized = false;
        next.activate();
    }

    IpcHandler {
        target: "taskbar"

        function activate(number: int): void { root.activateButton(number, false); }
        function launchNew(number: int): void { root.activateButton(number, true); }
    }

    /// A taskbar click: show the window, or put it away when it is the one
    /// you are already looking at.
    function toggleWindow(toplevel) {
        if (!toplevel)
            return;
        if (toplevel.activated && !toplevel.minimized) {
            toplevel.minimized = true;
            return;
        }
        toplevel.minimized = false;
        toplevel.activate();
    }

    /// Show the desktop: put every window away.
    function minimizeAll() {
        var open = windows.slice();
        for (var i = 0; i < open.length; i++)
            open[i].minimized = true;
    }

    // ----- programs ---------------------------------------------------------

    /// Every program with a .desktop file that asks to be listed, by name.
    readonly property var applications: {
        var all = DesktopEntries.applications
            ? DesktopEntries.applications.values : [];
        var listed = [];
        for (var i = 0; i < all.length; i++) {
            if (all[i] && !all[i].noDisplay)
                listed.push(all[i]);
        }
        listed.sort(function (a, b) {
            return String(a.name).localeCompare(String(b.name));
        });
        return listed;
    }

    /// The .desktop entry of a file name such as `kitty.desktop`, or null.
    /// Quickshell keys its entries by the name without the extension.
    function entryById(id) {
        var name = String(id).replace(/\.desktop$/, "");
        return DesktopEntries.byId(name);
    }

    /// The entries of a list of file names, in order, skipping any that are
    /// not installed.
    function entriesOf(ids) {
        // Read so the list is rebuilt once the entries have been scanned.
        void DesktopEntries.applications.values;
        var list = [];
        for (var i = 0; i < ids.length; i++) {
            var entry = entryById(ids[i]);
            if (entry)
                list.push(entry);
        }
        return list;
    }

    /// The programs the Start menu pins, in the order of `pinnedIds`.
    readonly property var pinned: entriesOf(pinnedIds)
    /// The programs the taskbar pins, in the order of `taskbarPinnedIds`.
    readonly property var taskbarPinned: entriesOf(taskbarPinnedIds)

    /// The programs whose name or description contains `query`.
    function search(query) {
        var needle = String(query).trim().toLowerCase();
        if (needle.length === 0)
            return applications;
        var found = [];
        for (var i = 0; i < applications.length; i++) {
            var entry = applications[i];
            var haystack = String(entry.name) + " "
                + String(entry.genericName || "") + " "
                + String(entry.comment || "");
            if (haystack.toLowerCase().indexOf(needle) >= 0)
                found.push(entry);
        }
        return found;
    }

    function launch(entry) {
        if (!entry)
            return;
        entry.execute();
        countLaunch(entry);
    }

    /// Run what was typed in the Start menu's search box, as the Run dialog
    /// does: through the shell, so arguments, pipes and `~` work.
    function runTyped(text) {
        var command = String(text).trim();
        if (command.length === 0)
            return false;
        Quickshell.execDetached({
            command: ["sh", "-c", command],
            workingDirectory: root.home
        });
        return true;
    }

    // ----- the programs used most -------------------------------------------

    /// How many programs the Start menu lists under the pinned ones.
    readonly property int frequentLimit: 7

    /// How often each program was started from the shell, by entry id.
    property var launchCounts: ({})

    // Kept in Quickshell's state directory for this shell, which it creates.
    FileView {
        id: countsFile

        path: Quickshell.statePath("launch-counts.json")
        printErrors: false
        onLoaded: {
            try {
                var parsed = JSON.parse(countsFile.text());
                root.launchCounts = parsed && typeof parsed === "object"
                    ? parsed : ({});
            } catch (error) {
                root.launchCounts = ({});
            }
        }
    }

    function countLaunch(entry) {
        var counts = Object.assign({}, root.launchCounts);
        counts[entry.id] = (counts[entry.id] || 0) + 1;
        root.launchCounts = counts;
        countsFile.setText(JSON.stringify(counts));
    }

    /// The programs started most often, most first, leaving out the pinned
    /// ones and any that are no longer installed.
    readonly property var frequent: {
        var counts = root.launchCounts;
        var pinnedIds = pinned.map(function (entry) {
            return entry.id;
        });
        var ids = Object.keys(counts).filter(function (id) {
            return pinnedIds.indexOf(id) < 0;
        });
        ids.sort(function (a, b) {
            return counts[b] - counts[a];
        });
        var list = [];
        for (var i = 0; i < ids.length && list.length < frequentLimit; i++) {
            var entry = DesktopEntries.byId(ids[i]);
            if (entry && !entry.noDisplay)
                list.push(entry);
        }
        return list;
    }

    // ----- All Programs -----------------------------------------------------

    // The main categories of the freedesktop menu specification, under the
    // names Windows gave its folders where it had one.
    readonly property var folderNames: ({
        "AudioVideo": "Multimedia",
        "Audio": "Multimedia",
        "Video": "Multimedia",
        "Development": "Development",
        "Education": "Education",
        "Game": "Games",
        "Graphics": "Graphics",
        "Network": "Internet",
        "Office": "Office",
        "Science": "Science",
        "Settings": "Settings",
        "System": "System Tools",
        "Utility": "Accessories"
    })

    function folderOf(entry) {
        var categories = entry.categories || [];
        for (var i = 0; i < categories.length; i++) {
            var name = folderNames[categories[i]];
            if (name)
                return name;
        }
        return "Other";
    }

    /// Every listed program, in folders: `[{ name, entries }]`, by name.
    readonly property var folders: {
        var byName = ({});
        var list = [];
        for (var i = 0; i < applications.length; i++) {
            var entry = applications[i];
            var name = folderOf(entry);
            if (!byName[name]) {
                byName[name] = { name: name, entries: [] };
                list.push(byName[name]);
            }
            byName[name].entries.push(entry);
        }
        list.sort(function (a, b) {
            return a.name.localeCompare(b.name);
        });
        return list;
    }

    // ----- the user ---------------------------------------------------------

    readonly property string loginName: Quickshell.env("USER")
        || home.split("/").pop()
    /// The full name from the password database, else the login name.
    property string userName: loginName

    Process {
        command: ["getent", "passwd", root.loginName]
        running: root.loginName.length > 0
        stdout: StdioCollector {
            onStreamFinished: {
                var fields = String(text).trim().split(":");
                // The GECOS field: full name first, then comma-separated
                // extras nobody fills in.
                var full = fields.length > 4 ? fields[4].split(",")[0].trim() : "";
                if (full.length > 0)
                    root.userName = full;
            }
        }
    }

    // ----- the user's folders -----------------------------------------------

    /// The folders of xdg-user-dirs, by key (DOCUMENTS, PICTURES, ...), with
    /// `$HOME` expanded. Missing keys fall back to the English names.
    property var userDirs: ({})

    FileView {
        id: userDirsFile

        path: (Quickshell.env("XDG_CONFIG_HOME") || (root.home + "/.config"))
            + "/user-dirs.dirs"
        watchChanges: true
        printErrors: false
        onFileChanged: reload()
        onLoaded: {
            var dirs = ({});
            var lines = userDirsFile.text().split("\n");
            for (var i = 0; i < lines.length; i++) {
                var match = /^\s*XDG_([A-Z]+)_DIR\s*=\s*"(.*)"\s*$/.exec(lines[i]);
                if (match)
                    dirs[match[1]] = match[2].replace(/^\$HOME/, root.home);
            }
            root.userDirs = dirs;
        }
    }

    function userDir(key, fallback) {
        return userDirs[key] || (root.home + "/" + fallback);
    }

    // ----- power ------------------------------------------------------------

    /// What the Start menu's power button does, and what its arrow offers.
    /// Each command runs as is; change them for a system without systemd.
    readonly property var powerActions: [
        { id: "logoff", label: "Log off",
          command: Quickshell.env("XDG_SESSION_ID")
              ? ["loginctl", "terminate-session", Quickshell.env("XDG_SESSION_ID")]
              : ["loginctl", "terminate-user", root.loginName] },
        // The shell's own lock screen.
        { id: "lock", label: "Lock" },
        { id: "separator" },
        { id: "restart", label: "Restart", command: ["systemctl", "reboot"] },
        { id: "sleep", label: "Sleep", command: ["systemctl", "suspend"] },
        { id: "hibernate", label: "Hibernate", command: ["systemctl", "hibernate"] },
        { id: "shutdown", label: "Shut down", command: ["systemctl", "poweroff"] }
    ]
    /// The action the button itself runs, by id. Set in the Control Panel.
    readonly property string powerButtonAction: powerAction(Settings.powerButtonAction)
        ? Settings.powerButtonAction : "logoff"

    function powerAction(id) {
        for (var i = 0; i < powerActions.length; i++) {
            if (powerActions[i].id === id)
                return powerActions[i];
        }
        return null;
    }

    function runPower(id) {
        if (id === "lock") {
            LockState.lock();
            return;
        }
        // Lock first, so the machine wakes up locked, as Windows does.
        if ((id === "sleep" || id === "hibernate") && Settings.lockOnSuspend)
            LockState.lock();
        var action = powerAction(id);
        if (action && action.command)
            Quickshell.execDetached(action.command);
    }

    /// Open one of the places of the Start menu in the file manager.
    function openPath(path) {
        Quickshell.execDetached([root.fileManager, path]);
    }

    function run(command) {
        Quickshell.execDetached(command);
    }

    // ----- recent files -----------------------------------------------------

    /// How many recent files a jump list shows.
    readonly property int recentLimit: 8

    /// Every file the desktop remembers opening, newest first. Each entry is
    /// `{ path, name, visited, apps }`, `apps` being the names and the first
    /// word of the commands the file was opened with.
    property var recentFiles: []

    readonly property string dataHome: Quickshell.env("XDG_DATA_HOME")
        || (home + "/.local/share")

    // The list GTK, Qt's KDE platform and most file managers write.
    FileView {
        id: recentFile

        path: root.dataHome + "/recently-used.xbel"
        watchChanges: true
        printErrors: false
        onFileChanged: reload()
        onLoaded: root.recentFiles = root.parseRecent(recentFile.text())
        onLoadFailed: root.recentFiles = []
    }

    function unescapeXml(text) {
        return String(text)
            .replace(/&lt;/g, "<").replace(/&gt;/g, ">")
            .replace(/&quot;/g, "\"").replace(/&apos;/g, "'")
            .replace(/&amp;/g, "&");
    }

    /// The first word of an exec string, without its quotes and its path.
    function commandName(exec) {
        var word = String(exec).trim().replace(/^['"]+/, "").split(/[\s'"]/)[0];
        return word.substring(word.lastIndexOf("/") + 1).toLowerCase();
    }

    function parseRecent(text) {
        var found = [];
        var bookmark = /<bookmark\b([^>]*)>([\s\S]*?)<\/bookmark>/g;
        var match;
        while ((match = bookmark.exec(text)) !== null) {
            var attributes = match[1];
            var href = /\bhref="([^"]*)"/.exec(attributes);
            if (!href)
                continue;
            var uri = unescapeXml(href[1]);
            // Only local files: the jump list hands a path to the program.
            if (uri.indexOf("file://") !== 0)
                continue;
            var path;
            try {
                path = decodeURIComponent(uri.substring(7));
            } catch (error) {
                continue;
            }
            var visited = /\b(?:visited|modified)="([^"]*)"/.exec(attributes);
            var apps = [];
            var application = /<bookmark:application\b([^>]*)\/?>/g;
            var app;
            while ((app = application.exec(match[2])) !== null) {
                var name = /\bname="([^"]*)"/.exec(app[1]);
                var exec = /\bexec="([^"]*)"/.exec(app[1]);
                if (name)
                    apps.push(unescapeXml(name[1]).toLowerCase());
                if (exec)
                    apps.push(commandName(unescapeXml(exec[1])));
            }
            found.push({
                path: path,
                name: path.substring(path.lastIndexOf("/") + 1),
                visited: visited ? Date.parse(visited[1]) || 0 : 0,
                apps: apps
            });
        }
        found.sort(function (a, b) {
            return b.visited - a.visited;
        });
        return found;
    }

    /// The names a program goes by in `recently-used.xbel`.
    function namesOf(entry) {
        var names = [String(entry.id).toLowerCase()];
        if (entry.name)
            names.push(String(entry.name).toLowerCase());
        if (entry.startupClass)
            names.push(String(entry.startupClass).toLowerCase());
        if (entry.command && entry.command.length > 0)
            names.push(commandName(entry.command[0]));
        // Reverse-DNS ids, such as org.gnome.TextEditor, also go by their
        // last part.
        var id = String(entry.id);
        if (id.indexOf(".") > 0)
            names.push(id.substring(id.lastIndexOf(".") + 1).toLowerCase());
        return names;
    }

    /// The files a program opened most recently, newest first.
    /// The files pinned to a program's jump list, in the order pinned.
    function pinnedFilesFor(entry) {
        var paths = (Settings.jumpPins || {})[entryFileId(entry)] || [];
        return paths.map(function (path) {
            return { name: path.substring(path.lastIndexOf("/") + 1), path: path };
        });
    }

    /// The recent files of a program, without the pinned ones.
    function recentUnpinned(entry) {
        var pinned = ((Settings.jumpPins || {})[entryFileId(entry)] || []);
        return recentFor(entry).filter(function (file) {
            return pinned.indexOf(file.path) < 0;
        });
    }

    function setFilePinned(entry, path, on) {
        var all = Object.assign({}, Settings.jumpPins || {});
        var id = entryFileId(entry);
        var list = (all[id] || []).filter(function (other) { return other !== path; });
        if (on)
            list.push(path);
        if (list.length > 0)
            all[id] = list;
        else
            delete all[id];
        Settings.jumpPins = all;
    }

    function recentFor(entry) {
        if (!entry)
            return [];
        var names = namesOf(entry);
        var list = [];
        for (var i = 0; i < recentFiles.length && list.length < recentLimit; i++) {
            var file = recentFiles[i];
            for (var j = 0; j < file.apps.length; j++) {
                if (names.indexOf(file.apps[j]) >= 0) {
                    list.push(file);
                    break;
                }
            }
        }
        return list;
    }

    /// The colour icon of a file, from its extension.
    function fileIcon(name) {
        var dot = String(name).lastIndexOf(".");
        var extension = dot > 0 ? String(name).substring(dot + 1).toLowerCase() : "";
        var kinds = {
            "file-image": ["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "tif", "tiff", "avif", "heic"],
            "file-audio": ["mp3", "flac", "ogg", "opus", "wav", "m4a", "aac"],
            "file-video": ["mp4", "mkv", "webm", "avi", "mov", "m4v"]
        };
        for (var kind in kinds) {
            if (kinds[kind].indexOf(extension) >= 0)
                return Qt.resolvedUrl("assets/icons/" + kind + ".png");
        }
        return Qt.resolvedUrl("assets/icons/file.png");
    }

    /// Open a file with a program, as its .desktop file says to start it.
    function openWith(entry, path) {
        if (!entry || !entry.command || entry.command.length === 0)
            return;
        // Quickshell drops the %f / %u field codes, so the file goes last.
        var command = entry.command.slice();
        command.push(path);
        Quickshell.execDetached({
            command: command,
            workingDirectory: entry.workingDirectory || root.home
        });
    }

    readonly property string home: Quickshell.env("HOME") || "/"
}
