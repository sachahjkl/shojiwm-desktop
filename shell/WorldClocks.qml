pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// The additional clocks of the Date and Time settings, as Windows 7 shows
// them under the calendar and in the clock's tooltip: the time in other
// time zones. `date` reads each zone from the system's zone database.
Singleton {
    id: root

    /// One per clock of Settings.extraClocks: { label, zone, time, day },
    /// where day is "Today", "Tomorrow" or "Yesterday" from here.
    property var clocks: []
    /// Every time zone the system knows, for the settings page.
    property var zones: []

    function refresh() {
        var list = Settings.extraClocks || [];
        if (list.length === 0) {
            root.clocks = [];
            return;
        }
        var format = Settings.clockFormat === "12h" ? "%-I:%M %p" : "%H:%M";
        reader.command = ["sh", "-c",
            "format=$1; shift; date +%Y%m%d; for zone; do TZ=$zone date +\"%Y%m%d|$format\"; done",
            "sh", format].concat(list.map(function (clock) { return clock.zone; }));
        reader.running = true;
    }

    Process {
        id: reader

        stdout: StdioCollector {
            onStreamFinished: {
                var lines = text.trim().split("\n");
                var here = Number(lines[0]);
                var list = Settings.extraClocks || [];
                var out = [];
                for (var i = 0; i < list.length; i++) {
                    var parts = (lines[i + 1] || "").split("|");
                    var there = Number(parts[0]);
                    // Dates as yyyymmdd numbers: one day apart is ±1, or
                    // more across a month's end, so compare the order only.
                    var day = there === here ? "Today" : there > here ? "Tomorrow" : "Yesterday";
                    out.push({
                        label: list[i].label || list[i].zone,
                        zone: list[i].zone,
                        time: parts[1] || "",
                        day: day
                    });
                }
                root.clocks = out;
            }
        }
    }

    Process {
        command: ["timedatectl", "list-timezones"]
        running: true
        stdout: StdioCollector {
            onStreamFinished: root.zones = text.trim().split("\n")
        }
    }

    // On the minute, and at once when the clocks or the format change.
    Timer {
        interval: 30000
        repeat: true
        running: (Settings.extraClocks || []).length > 0
        triggeredOnStart: true
        onTriggered: root.refresh()
    }

    Connections {
        target: Settings
        function onExtraClocksChanged() { root.refresh(); }
        function onClockFormatChanged() { root.refresh(); }
    }
}
