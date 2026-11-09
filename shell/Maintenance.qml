pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// What Windows 7's Action Center reports, for this system: the system's and
// your services that failed (systemd), and the disks almost full. It looks
// again every minute and whenever its panel opens.
Singleton {
    id: root

    /// { level: "error" | "warning", title, detail, unit, user, mount },
    /// errors first.
    property var issues: []
    readonly property bool hasErrors: root.issues.some(function (issue) { return issue.level === "error"; })

    /// A disk this full or more is reported.
    readonly property int fullPercent: 90

    /// Asked over IPC: open (true) or close (false) the panel.
    signal panelRequested(bool open)

    IpcHandler {
        target: "actioncenter"

        function open(): void { root.panelRequested(true); }
        function close(): void { root.panelRequested(false); }
    }

    function refresh() {
        reader.running = false;
        reader.running = true;
    }

    /// Clear a failed service of yours, so it is not reported again.
    function resetUnit(issue) {
        Quickshell.execDetached(issue.user
            ? ["systemctl", "--user", "reset-failed", issue.unit]
            : ["systemctl", "reset-failed", issue.unit]);
        later.restart();
    }

    function openMount(issue) {
        Apps.openPath(issue.mount);
    }

    Process {
        id: reader

        command: ["sh", "-c",
            "echo '#system'; systemctl --failed --no-legend --plain 2>/dev/null | awk '{print $1}';"
            + " echo '#user'; systemctl --user --failed --no-legend --plain 2>/dev/null | awk '{print $1}';"
            + " echo '#disks'; df -P -x tmpfs -x devtmpfs -x efivarfs -x overlay -x squashfs 2>/dev/null"
            + " | awk 'NR > 1 {sub(\"%\", \"\", $5); print $5, $6}'"]
        running: true

        stdout: StdioCollector {
            onStreamFinished: {
                var section = "";
                var list = [];
                String(text).split("\n").forEach(function (line) {
                    if (line.length === 0)
                        return;
                    if (line[0] === "#") {
                        section = line.substring(1);
                        return;
                    }
                    if (section === "system" || section === "user") {
                        list.push({
                            level: "error",
                            title: line.replace(/\.service$/, "") + " failed",
                            detail: (section === "user" ? "A service of yours" : "A system service")
                                + " stopped with an error (" + line + ").",
                            unit: line,
                            user: section === "user",
                            mount: ""
                        });
                    } else if (section === "disks") {
                        var parts = line.split(" ");
                        var used = Number(parts[0]);
                        if (used >= root.fullPercent) {
                            list.push({
                                level: used >= 97 ? "error" : "warning",
                                title: "Disk almost full: " + parts.slice(1).join(" "),
                                detail: used + " % of its space is used.",
                                unit: "",
                                user: false,
                                mount: parts.slice(1).join(" ")
                            });
                        }
                    }
                });
                list.sort(function (a, b) {
                    return (a.level === "error" ? 0 : 1) - (b.level === "error" ? 0 : 1);
                });
                root.issues = list;
            }
        }
    }

    Timer {
        interval: 60000
        repeat: true
        running: true
        onTriggered: root.refresh()
    }

    Timer {
        id: later
        interval: 800
        onTriggered: root.refresh()
    }
}
