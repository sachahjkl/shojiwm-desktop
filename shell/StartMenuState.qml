pragma Singleton

import QtQuick
import Quickshell
import "theme.js" as Theme

// Which screen shows the Start menu. The empty string means none of them.
//
// One piece of state for the whole shell: the menu is built per screen, but
// only one of them can be open at a time.
Singleton {
    id: root

    property string openOn: ""

    /// The launcher (Super+Shift+D), on the screen named by
    /// `launcherScreen` (the focused one). It and the Start menu are never
    /// open together.
    property bool launcherOpen: false
    property string launcherScreen: ""

    /// Opens the launcher on a screen, by name; "" for the main screen.
    function openLauncher(screen) {
        root.openOn = "";
        root.launcherScreen = screen.length > 0 ? screen : firstScreen();
        root.launcherOpen = true;
    }

    function closeLauncher() {
        root.launcherOpen = false;
    }

    function toggleLauncher(screen) {
        if (root.launcherOpen)
            root.closeLauncher();
        else
            root.openLauncher(screen);
    }

    /// The sizes of the menu, large or compact (Settings.startMenuLargeIcons).
    readonly property int rowHeight: Settings.startMenuLargeIcons ? Theme.menuRowHeight : Theme.menuRowHeightSmall
    readonly property int iconSize: Settings.startMenuLargeIcons ? Theme.menuIconSize : Theme.menuIconSizeSmall
    readonly property int menuHeight: Settings.startMenuLargeIcons ? Theme.menuHeight : Theme.menuHeightSmall

    function firstScreen() {
        return Screens.primaryName;
    }

    function open(screen) {
        root.launcherOpen = false;
        root.openOn = screen.length > 0 ? screen : firstScreen();
    }

    function close() {
        root.openOn = "";
    }

    function toggle(screen) {
        var name = screen.length > 0 ? screen : firstScreen();
        if (root.openOn === name)
            root.close();
        else
            root.open(name);
    }
}
