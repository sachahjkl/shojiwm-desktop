pragma Singleton

import Quickshell
import Quickshell.Io

// The "Shut Down Windows" dialog: which screen shows it, or none. The
// compositor opens it with Alt+F4 on the desktop and Super+Shift+E:
//
//   quickshell -p ~/.config/shojiwm/shell ipc call shutdown openOn DP-2
Singleton {
    id: root

    /// The screen showing the dialog, by connector name; "" for none.
    property string openOn: ""

    function open(screen) {
        var screens = Quickshell.screens;
        root.openOn = screen.length > 0 ? screen
            : screens.length > 0 ? screens[0].name : "";
    }

    function close() {
        root.openOn = "";
    }

    IpcHandler {
        target: "shutdown"

        function openOn(screen: string): void { root.open(screen); }
        function close(): void { root.close(); }
    }
}
