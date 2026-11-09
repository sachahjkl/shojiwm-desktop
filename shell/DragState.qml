pragma Singleton

import Quickshell

// What the shell itself is dragging, for its own drop areas.
//
// A drag the shell starts is a Wayland drag, so it can leave for other
// programs. But the shell must not read its own drag's data back through
// Wayland: the source is this same process, blocked in its drag loop, so the
// read times out, and Quickshell crashes. The source puts the data here when
// the drag starts; a drop area reads it here, and reads the drag's data only
// for drags from other programs.
Singleton {
    id: root

    /// "tray" (a notification-area item id), "program" (a .desktop file
    /// name), "files" (file URLs), or "" when the shell drags nothing.
    property string kind: ""
    /// The id, the file name, or the list of URLs.
    property var data: null

    function start(kind, data) {
        root.kind = kind;
        root.data = data;
    }

    function finish() {
        root.kind = "";
        root.data = null;
    }
}
