pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// The Run dialog (Super+R): which screen shows it, the commands run from
// it, last first, and how a command runs.
//
// As Windows' Run, it keeps the last 26 commands; the field starts with the
// last one. A folder, a file or an address opens with xdg-open; anything
// else runs in a shell from the home folder, unless its program cannot be
// found, which the dialog says instead.
Singleton {
    id: root

    /// The screen showing the dialog, by name; "" when it is closed.
    property string openOn: ""

    function open(screen) {
        StartMenuState.close();
        StartMenuState.adminMenuOn = "";
        root.openOn = screen.length > 0 ? screen : Screens.primaryName;
    }

    function close() {
        root.openOn = "";
    }

    function toggle(screen) {
        if (root.openOn.length > 0)
            root.close();
        else
            root.open(screen);
    }

    IpcHandler {
        target: "run"

        function open(): void { root.open(""); }
        function openOn(screen: string): void { root.open(screen); }
        function toggleOn(screen: string): void { root.toggle(screen); }
        function close(): void { root.close(); }
    }

    // ----- the history --------------------------------------------------------

    /// How many commands the history keeps, as Windows' RunMRU.
    readonly property int historyKept: 26

    /// The commands run from the dialog, last first.
    property var history: []

    FileView {
        id: historyFile

        path: Quickshell.statePath("run-history.json")
        printErrors: false
        onLoaded: {
            try {
                var parsed = JSON.parse(historyFile.text());
                root.history = Array.isArray(parsed) ? parsed.map(String) : [];
            } catch (error) {
                root.history = [];
            }
        }
    }

    function remember(command) {
        var list = root.history.filter(function (entry) {
            return entry !== command;
        });
        list.unshift(command);
        root.history = list.slice(0, root.historyKept);
        historyFile.setText(JSON.stringify(root.history));
    }

    /// The most recent command that starts with `prefix` and is longer, or "".
    function completion(prefix) {
        if (prefix.length === 0)
            return "";
        var lower = prefix.toLowerCase();
        for (var i = 0; i < root.history.length; i++) {
            var entry = root.history[i];
            if (entry.length > prefix.length && entry.toLowerCase().startsWith(lower))
                return entry;
        }
        return "";
    }

    // ----- running ------------------------------------------------------------

    /// The last command that could not be found, for the dialog to say so.
    property string notFound: ""

    /// `~` and `~/...` in the user's home.
    function expandHome(text) {
        if (text === "~")
            return Apps.home;
        return text.startsWith("~/") ? Apps.home + text.slice(1) : text;
    }

    /// An address or a path: opened, not run.
    function isTarget(text) {
        return /^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(text) || text.startsWith("/")
            || text.startsWith("~") || text.startsWith("./");
    }

    /// Run what was typed: `true` when it started (the dialog closes),
    /// `false` when its program cannot be found (the dialog says so).
    function run(text) {
        var command = String(text).trim();
        if (command.length === 0)
            return;
        root.notFound = "";
        if (root.isTarget(command)) {
            root.remember(command);
            Quickshell.execDetached(["xdg-open", root.expandHome(command)]);
            root.close();
            return;
        }
        // The program, the first word, must exist before the line runs.
        check.command = ["sh", "-c",
            "if command -v -- \"$1\" >/dev/null; then echo found; else echo missing; fi",
            "sh", command.split(/\s+/)[0]];
        check.line = command;
        check.running = true;
    }

    Process {
        id: check

        property string line: ""

        stdout: StdioCollector {
            onStreamFinished: {
                if (String(text).trim() !== "found") {
                    root.notFound = check.line;
                    return;
                }
                root.remember(check.line);
                Apps.runTyped(check.line);
                root.close();
            }
        }
    }
}
