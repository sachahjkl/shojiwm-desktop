pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// The progress and counts programs show on their taskbar button, as
// Windows 7 lets them: a download in the browser fills its button green.
//
// Programs announce them with the com.canonical.Unity.LauncherEntry
// `Update` signal on the session bus (Chromium, Firefox, KDE and GTK
// programs do), naming their .desktop file. busctl watches that signal and
// prints each one as a line of JSON.
Singleton {
    id: root

    /// By .desktop file name: { progress (0 to 1) or -1 for none, count or
    /// -1 for none, urgent }.
    property var entries: ({})

    /// What the program of a .desktop file name shows, or null.
    function of(fileId) {
        return root.entries[fileId] || null;
    }

    function update(line) {
        var message;
        try {
            message = JSON.parse(line);
        } catch (error) {
            return;
        }
        if (!message.payload || !message.payload.data || message.payload.data.length < 2)
            return;
        var id = String(message.payload.data[0]).replace(/^application:\/\//, "");
        var props = message.payload.data[1] || {};
        var value = function (key) {
            return props[key] !== undefined ? props[key].data : undefined;
        };
        var old = root.entries[id] || { progress: -1, count: -1, urgent: false };
        var next = Object.assign({}, old);
        if (value("progress-visible") !== undefined || value("progress") !== undefined) {
            var shown = value("progress-visible") !== undefined
                ? value("progress-visible") : old.progress >= 0;
            var progress = value("progress") !== undefined
                ? Number(value("progress")) : Math.max(0, old.progress);
            next.progress = shown ? Math.max(0, Math.min(1, progress)) : -1;
        }
        if (value("count-visible") !== undefined || value("count") !== undefined) {
            var counted = value("count-visible") !== undefined
                ? value("count-visible") : old.count >= 0;
            var count = value("count") !== undefined ? Number(value("count")) : Math.max(0, old.count);
            next.count = counted ? count : -1;
        }
        if (value("urgent") !== undefined)
            next.urgent = value("urgent") === true;
        var all = Object.assign({}, root.entries);
        all[id] = next;
        root.entries = all;
    }

    Process {
        command: ["busctl", "--user", "monitor", "--json=short",
                  "--match", "type='signal',interface='com.canonical.Unity.LauncherEntry'"]
        running: true
        stdout: SplitParser {
            onRead: function (line) { root.update(line); }
        }
    }
}
