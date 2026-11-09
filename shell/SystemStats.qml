pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// The processor and memory use, for the CPU Meter gadget: from /proc/stat
// (the time the processors spent busy since the last reading) and
// /proc/meminfo (the memory not available to programs). It reads only
// while something shows it.
Singleton {
    id: root

    /// Set by the gadget while it shows.
    property bool active: false
    /// 0 to 1.
    property real cpu: 0
    property real memory: 0
    /// Used and total memory in bytes.
    property real memoryUsed: 0
    property real memoryTotal: 0

    property var lastBusy: 0
    property var lastTotal: 0

    function parse(text) {
        var lines = text.split("\n");
        var available = 0;
        var total = 0;
        for (var i = 0; i < lines.length; i++) {
            var line = lines[i];
            if (line.indexOf("cpu ") === 0) {
                var f = line.trim().split(/\s+/).slice(1).map(Number);
                // user nice system idle iowait irq softirq steal
                var idle = f[3] + (f[4] || 0);
                var all = 0;
                for (var k = 0; k < Math.min(f.length, 8); k++)
                    all += f[k];
                var busy = all - idle;
                if (root.lastTotal > 0 && all > root.lastTotal)
                    root.cpu = Math.max(0, Math.min(1, (busy - root.lastBusy) / (all - root.lastTotal)));
                root.lastBusy = busy;
                root.lastTotal = all;
            } else if (line.indexOf("MemTotal:") === 0) {
                total = Number(line.split(/\s+/)[1]) * 1024;
            } else if (line.indexOf("MemAvailable:") === 0) {
                available = Number(line.split(/\s+/)[1]) * 1024;
            }
        }
        if (total > 0) {
            root.memoryTotal = total;
            root.memoryUsed = total - available;
            root.memory = root.memoryUsed / total;
        }
    }

    Process {
        id: reader
        command: ["cat", "/proc/stat", "/proc/meminfo"]
        stdout: StdioCollector {
            onStreamFinished: root.parse(text)
        }
    }

    Timer {
        interval: 1500
        repeat: true
        running: root.active
        triggeredOnStart: true
        onTriggered: reader.running = true
    }
}
