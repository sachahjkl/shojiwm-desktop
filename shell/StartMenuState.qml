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

    /// The sizes of the menu, large or compact (Settings.startMenuLargeIcons).
    readonly property int rowHeight: Settings.startMenuLargeIcons ? Theme.menuRowHeight : Theme.menuRowHeightSmall
    readonly property int iconSize: Settings.startMenuLargeIcons ? Theme.menuIconSize : Theme.menuIconSizeSmall
    readonly property int menuHeight: Settings.startMenuLargeIcons ? Theme.menuHeight : Theme.menuHeightSmall

    function firstScreen() {
        var screens = Quickshell.screens;
        return screens.length > 0 ? screens[0].name : "";
    }

    function open(screen) {
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
