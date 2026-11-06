pragma Singleton

import QtQuick
import Quickshell

// Which screen shows the Start menu. The empty string means none of them.
//
// One piece of state for the whole shell: the menu is built per screen, but
// only one of them can be open at a time.
Singleton {
    id: root

    property string openOn: ""

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
