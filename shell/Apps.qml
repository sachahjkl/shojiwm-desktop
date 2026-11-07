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

    // The programs the Start menu pins and the places open. Change these to
    // the programs you use.
    readonly property string terminal: "kitty"
    readonly property string fileManager: "thunar"
    readonly property string browser: "chromium"
    /// What the "Control Panel" place starts. Change it to your own.
    readonly property var controlPanel: ["xfce4-settings-manager"]
    readonly property var pinnedIds: ["kitty.desktop", "thunar.desktop", "chromium.desktop"]
    /// The programs that keep a button on the taskbar while they are closed,
    /// left to right.
    readonly property var taskbarPinnedIds: ["thunar.desktop", "chromium.desktop", "kitty.desktop"]

    // ----- windows ----------------------------------------------------------

    /// Every open window, oldest first.
    readonly property var windows: ToplevelManager.toplevels
        ? ToplevelManager.toplevels.values : []
    /// The window that has the keyboard.
    readonly property var activeWindow: ToplevelManager.activeToplevel

    // ----- grouping ---------------------------------------------------------

    /// Combine the windows of one program into one button.
    readonly property bool combine: Theme.combineWindows

    /// The row of the taskbar, left to right. Each entry is
    /// `{ key, name, icon, entry, pinned, windows }`: the pinned programs
    /// first, in the order of `taskbarPinnedIds`, then the others in the
    /// order their first window opened. A pinned program with no window open
    /// has an empty `windows`.
    property var groups: []

    /// The key of the program a window belongs to, whatever `combine` says.
    function programKey(toplevel) {
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
                group = {
                    key: root.combine ? key : "window:" + i,
                    entry: entry,
                    name: entry && entry.name ? entry.name : titleOf(toplevel),
                    icon: iconFor(toplevel.appId),
                    pinned: false,
                    windows: []
                };
                index[key] = group;
                list.push(group);
            }
            group.windows.push(toplevel);
        }

        groups = list;
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
    function activateGroup(group) {
        if (!group)
            return;
        if (group.windows.length === 0) {
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
        if (entry)
            entry.execute();
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
