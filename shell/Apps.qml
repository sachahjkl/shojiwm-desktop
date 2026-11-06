pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Wayland

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
