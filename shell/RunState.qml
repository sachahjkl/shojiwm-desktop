pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// The Run dialog (Super+R): which screen shows it, the commands run from
// it, last first, and how a command runs.
//
// As Windows' Run, it keeps the last 26 commands; the field starts with the
// last one. What a line does follows from its first word (see run()).
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

    /// Run what was typed, as Windows' Run does with its first word: an
    /// address opens; a path to an executable file runs, with the rest of the
    /// line; another path that exists opens; a program found in PATH runs.
    /// Anything else stays in the field, with Cannot find under it. A second
    /// Run while the first is still checked waits for it.
    function run(text) {
        var command = String(text).trim();
        if (command.length === 0 || check.running)
            return;
        root.notFound = "";
        check.line = command;
        check.command = ["sh", "-c", root.decide, "sh", command.split(/\s+/)[0]];
        check.running = true;
    }

    // Prints what to do with the first word: open, run or missing.
    readonly property string decide: [
        "w=$1",
        "case $w in \"~\") w=$HOME ;; \"~/\"*) w=$HOME/${w#\"~/\"} ;; esac",
        "case $w in",
        "  [a-zA-Z]*:*) echo open ;;",
        "  /*|./*|../*|\"$HOME\"*)",
        "    if [ -f \"$w\" ] && [ -x \"$w\" ]; then echo run",
        "    elif [ -e \"$w\" ]; then echo open",
        "    else echo missing; fi ;;",
        "  *) if command -v -- \"$w\" >/dev/null; then echo run; else echo missing; fi ;;",
        "esac"
    ].join("\n")

    /// `~` and `~/...` in the user's home, for xdg-open.
    function expandHome(text) {
        if (text === "~")
            return Apps.home;
        return text.startsWith("~/") ? Apps.home + text.slice(1) : text;
    }

    // From the home folder, as the line runs (Apps.runTyped).
    Process {
        id: check

        property string line: ""

        workingDirectory: Apps.home

        stdout: StdioCollector {
            onStreamFinished: {
                var verdict = String(text).trim();
                if (verdict === "missing") {
                    root.notFound = check.line;
                    return;
                }
                root.remember(check.line);
                if (verdict === "open")
                    Quickshell.execDetached({
                        command: ["xdg-open", root.expandHome(check.line)],
                        workingDirectory: Apps.home
                    });
                else
                    Apps.runTyped(check.line);
                root.close();
            }
        }
    }
}
