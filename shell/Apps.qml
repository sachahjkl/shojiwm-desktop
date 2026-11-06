pragma Singleton

import QtQuick
import Quickshell
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
    /// `{ key, name, icon, entry, windows }`, in the order the first window
    /// of the program opened.
    property var groups: []

    /// The key of the group a window belongs to. With `combine` off every
    /// window gets its own.
    function groupKey(toplevel, position) {
        if (!root.combine)
            return "window:" + position;
        var appId = toplevel ? toplevel.appId : "";
        var entry = entryFor(appId);
        if (entry)
            return "entry:" + entry.id;
        return "appid:" + (appId ? String(appId).toLowerCase() : "<none>");
    }

    function rebuildGroups() {
        var list = [];
        var index = ({});
        var open = windows;

        for (var i = 0; i < open.length; i++) {
            var toplevel = open[i];
            if (!toplevel)
                continue;
            var key = groupKey(toplevel, i);
            var group = index[key];
            if (!group) {
                var entry = entryFor(toplevel.appId);
                group = {
                    key: key,
                    entry: entry,
                    name: entry && entry.name ? entry.name : titleOf(toplevel),
                    icon: iconFor(toplevel.appId),
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

    /// What a left click on a group does. One window: show it, or put it away
    /// when it is the one you are looking at. Several: the caller shows the
    /// thumbnails instead.
    function activateGroup(group) {
        if (!group || group.windows.length === 0)
            return;
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

    /// The pinned programs, in the order of `pinnedIds`, skipping any that
    /// are not installed.
    readonly property var pinned: {
        var list = [];
        for (var i = 0; i < pinnedIds.length; i++) {
            var entry = DesktopEntries.byId(pinnedIds[i]);
            if (entry)
                list.push(entry);
        }
        return list;
    }

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

    readonly property string home: Quickshell.env("HOME") || "/"
}
