pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// Whether the Control Panel is open, and on which page.
//
//   quickshell -p ~/.config/shojiwm/shell ipc call controlpanel open
//   quickshell -p ~/.config/shojiwm/shell ipc call controlpanel openPage appearance
Singleton {
    id: root

    property bool shown: false
    /// "home", "appearance", "background", "lockscreen", "account",
    /// "taskbar", "clock" or "about".
    property string page: "home"

    function open(page) {
        root.page = page && page.length > 0 ? page : "home";
        root.shown = true;
    }

    function close() {
        root.shown = false;
    }

    IpcHandler {
        target: "controlpanel"

        function open(): void { root.open(""); }
        function openPage(page: string): void { root.open(page); }
        function close(): void { root.close(); }
    }
}
