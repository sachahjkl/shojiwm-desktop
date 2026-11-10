pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// The default programs, as Windows 7's Default Programs sets them: for each
// kind of file or link, the program that opens it, and the programs that
// say in their .desktop file that they can. xdg-mime reads and writes the
// choices (~/.config/mimeapps.list).
Singleton {
    id: root

    /// The kinds the page offers; the first type of each decides, and a
    /// choice sets all of them.
    readonly property var kinds: [
        { label: "Web browser", types: ["x-scheme-handler/https", "x-scheme-handler/http", "text/html"] },
        { label: "E-mail", types: ["x-scheme-handler/mailto"] },
        { label: "Folders", types: ["inode/directory"] },
        { label: "Text files", types: ["text/plain"] },
        { label: "Pictures", types: ["image/png", "image/jpeg", "image/webp", "image/gif"] },
        { label: "Music", types: ["audio/mpeg", "audio/flac", "audio/ogg"] },
        { label: "Videos", types: ["video/mp4", "video/x-matroska", "video/webm"] },
        { label: "PDF documents", types: ["application/pdf"] }
    ]

    /// The programs the desktop starts (Settings.programs, Apps.program): the
    /// installed programs offered for each are the ones whose .desktop file
    /// has one of `categories`; clipboard and screenshot have none to offer,
    /// only a command.
    readonly property var programKinds: [
        { key: "terminal", label: "Terminal", categories: ["TerminalEmulator"],
          note: "Super+Enter" },
        { key: "fileManager", label: "File manager", categories: ["FileManager"],
          note: "Super+E, and the places" },
        { key: "taskManager", label: "Task manager", categories: ["Monitor"],
          note: "Ctrl+Shift+Escape" },
        { key: "systemMonitor", label: "System monitor", categories: ["Monitor"],
          note: "CPU Meter's button" },
        { key: "clipboard", label: "Clipboard history", categories: [],
          note: "Super+V" },
        { key: "screenshot", label: "Screenshot", categories: [],
          note: "Print Screen" }
    ]

    /// The command line that starts an installed program, as a shell reads
    /// it: its .desktop file's command, in the terminal when it says so.
    function commandOf(entry) {
        var words = (entry.command || []).map(function (word) {
            return /^[A-Za-z0-9_\/.:=+,@%-]+$/.test(word) ? word : "'" + String(word).replace(/'/g, "'\\''") + "'";
        });
        var line = words.join(" ");
        return entry.runInTerminal ? Apps.sessionPrograms.terminal + " -e " + line : line;
    }

    /// The installed programs a kind offers: { name, command }, by name.
    function programChoices(kind) {
        var all = DesktopEntries.applications ? DesktopEntries.applications.values : [];
        var out = [];
        for (var i = 0; i < all.length; i++) {
            var entry = all[i];
            if (entry.noDisplay || !entry.command || entry.command.length === 0)
                continue;
            var categories = entry.categories || [];
            if (!kind.categories.some(function (c) { return categories.indexOf(c) >= 0; }))
                continue;
            out.push({ name: String(entry.name), command: root.commandOf(entry) });
        }
        out.sort(function (a, b) { return a.name.localeCompare(b.name); });
        return out;
    }

    /// Chooses a program's command; "" goes back to the session's.
    function setProgram(key, command) {
        var programs = Object.assign({}, Settings.programs);
        if (command.length > 0)
            programs[key] = command;
        else
            delete programs[key];
        Settings.programs = programs;
    }

    /// By the first type of each kind: { current: "app.desktop", candidates: [...] }.
    property var state: ({})

    function refresh() {
        reader.running = false;
        reader.running = true;
    }

    /// Make a program the default for every type of a kind.
    function setDefault(kind, desktopId) {
        writer.command = ["sh", "-c", "app=$1; shift; for type; do xdg-mime default \"$app\" \"$type\"; done",
                          "sh", desktopId].concat(kind.types);
        writer.running = true;
    }

    Process {
        id: reader

        // For each kind: its default, then the .desktop files that list its
        // first type, from your applications and the system's.
        command: ["sh", "-c",
            "dirs=\"${XDG_DATA_HOME:-$HOME/.local/share}:${XDG_DATA_DIRS:-/usr/share}\";"
            + " for type; do echo \"#$type\"; echo \"default=$(xdg-mime query default \"$type\")\";"
            + " IFS=:; for dir in $dirs; do"
            + " grep -l -s \"^MimeType=.*$type;\" \"$dir\"/applications/*.desktop 2>/dev/null; done; unset IFS;"
            + " done", "sh"].concat(root.kinds.map(function (kind) { return kind.types[0]; }))
        running: true

        stdout: StdioCollector {
            onStreamFinished: {
                var next = {};
                var current = null;
                String(text).split("\n").forEach(function (line) {
                    if (line.length === 0)
                        return;
                    if (line[0] === "#") {
                        current = { current: "", candidates: [] };
                        next[line.substring(1)] = current;
                    } else if (current && line.indexOf("default=") === 0) {
                        current.current = line.substring(8);
                    } else if (current) {
                        var id = line.substring(line.lastIndexOf("/") + 1);
                        if (current.candidates.indexOf(id) < 0)
                            current.candidates.push(id);
                    }
                });
                root.state = next;
            }
        }
    }

    Process {
        id: writer
        onRunningChanged: {
            if (!running)
                root.refresh();
        }
    }
}
