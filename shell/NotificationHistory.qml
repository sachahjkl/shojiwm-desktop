pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// The notifications of this session, newest first, for the notification
// panel; and Hokum's Do not disturb.
//
// Hokum draws the notifications and keeps no history. busctl watches the
// Notify calls programs make on the session bus and prints each one as a
// line of JSON. Notifications that only show a level (volume, brightness:
// the x-canonical-private-synchronous or x-dunst-stack-tag hints) and the
// transient ones stay out of the list.
Singleton {
    id: root

    /// { app, icon, summary, body, time (ms since the epoch) }, newest first.
    property var entries: []
    /// How many came since the panel was last opened.
    property int unread: 0
    property bool doNotDisturb: false
    readonly property int keep: 100

    function add(line) {
        var message;
        try {
            message = JSON.parse(line);
        } catch (error) {
            return;
        }
        var data = message.payload ? message.payload.data : null;
        if (!data || data.length < 7)
            return;
        var hints = data[6] || {};
        if (hints.transient && hints.transient.data === true)
            return;
        if (hints["x-canonical-private-synchronous"] || hints["x-dunst-stack-tag"])
            return;
        var entry = {
            app: String(data[0] || ""),
            icon: String(data[2] || ""),
            summary: String(data[3] || ""),
            // The body may hold the markup the specification allows.
            body: String(data[4] || "").replace(/<[^>]*>/g, ""),
            time: Date.now()
        };
        if (entry.summary.length === 0 && entry.body.length === 0)
            return;
        root.entries = [entry].concat(root.entries).slice(0, root.keep);
        root.unread += 1;
    }

    function remove(index) {
        var list = root.entries.slice();
        list.splice(index, 1);
        root.entries = list;
    }

    function clear() {
        root.entries = [];
        root.unread = 0;
    }

    function markRead() {
        root.unread = 0;
    }

    function setDoNotDisturb(on) {
        Quickshell.execDetached(["hokum", "dnd", on ? "on" : "off"]);
    }

    /// "Just now", "5 min ago", "14:32", or the date for an older one.
    function age(time, now) {
        var minutes = Math.floor((now - time) / 60000);
        if (minutes < 1)
            return "Just now";
        if (minutes < 60)
            return minutes + " min ago";
        var date = new Date(time);
        var today = new Date(now);
        if (date.toDateString() === today.toDateString())
            return Qt.formatTime(date, "HH:mm");
        return Qt.formatDate(date, Qt.locale().dateFormat(Locale.ShortFormat));
    }

    Process {
        command: ["busctl", "--user", "monitor", "--json=short", "--match",
                  "type='method_call',interface='org.freedesktop.Notifications',member='Notify'"]
        running: true
        stdout: SplitParser {
            onRead: function (line) { root.add(line); }
        }
    }

    // Hokum prints its Do not disturb at start and at each change.
    Process {
        command: ["hokum", "dnd", "--watch", "--json"]
        running: true
        stdout: SplitParser {
            onRead: function (line) {
                try {
                    root.doNotDisturb = JSON.parse(line).dnd === true;
                } catch (error) {
                }
            }
        }
    }
}
