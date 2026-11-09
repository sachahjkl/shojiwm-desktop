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

    /// The launcher (Super+Shift+D), on the main screen. It and the Start
    /// menu are never open together.
    property bool launcherOpen: false

    function openLauncher() {
        root.openOn = "";
        root.launcherOpen = true;
    }

    function closeLauncher() {
        root.launcherOpen = false;
    }

    function toggleLauncher() {
        if (root.launcherOpen)
            root.closeLauncher();
        else
            root.openLauncher();
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
