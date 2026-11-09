pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// What the Start menu's search box and the launcher (Super+Shift+D) find:
// the programs, the Control Panel's tasks and the files, as rows ready to
// show. One search serves both.
//
// It answers at once: the programs' words are put in lower case when the
// list of programs changes, not at each key, and only the few best rows are
// made. A program whose name starts with what is typed comes first, then
// one with a word of its name that starts with it, then the others; among
// equals, the one started most often (Apps.launchCounts).
Singleton {
    id: root

    /// What is typed. The search box that has the keyboard sets it.
    property string query: ""
    readonly property string needle: query.trim().toLowerCase()
    readonly property bool active: needle.length > 0

    /// The most programs a search lists.
    readonly property int programLimit: 8

    /// Each program with its name and its other words in lower case.
    readonly property var index: Apps.applications.map(function (entry) {
        return {
            entry: entry,
            name: String(entry.name).toLowerCase(),
            other: (String(entry.genericName || "") + " " + String(entry.comment || "")
                + " " + String(entry.id || "")).toLowerCase()
        };
    })

    /// 0: the name starts with the needle; 1: a word of the name does;
    /// 2: the name holds it; 3: another word does; -1: no match.
    function rank(item, needle) {
        var at = item.name.indexOf(needle);
        if (at === 0)
            return 0;
        if (at > 0) {
            var before = item.name.charAt(at - 1);
            return before === " " || before === "-" || before === "." ? 1 : 2;
        }
        return item.other.indexOf(needle) >= 0 ? 3 : -1;
    }

    readonly property var programs: {
        if (!root.active)
            return [];
        var counts = Apps.launchCounts;
        var found = [];
        for (var i = 0; i < root.index.length; i++) {
            var rank = root.rank(root.index[i], root.needle);
            if (rank >= 0)
                found.push({ rank: rank, uses: counts[root.index[i].entry.id] || 0, item: root.index[i] });
        }
        // The index is in name order already: a stable sort keeps it so
        // among programs started as often.
        found.sort(function (a, b) { return a.rank - b.rank || b.uses - a.uses; });
        return found.slice(0, root.programLimit).map(function (match) {
            return match.item.entry;
        });
    }

    readonly property var tasks: root.active
        ? ControlPanelState.findTasks(root.query).slice(0, 5) : []
    property var files: []

    /// The results as rows: a heading over each group that has any, then
    /// its rows. `pick` numbers the rows that can be chosen; -1 is a heading.
    readonly property var rows: {
        var out = [];
        var pick = 0;
        var group = function (title, kind, list) {
            if (list.length === 0)
                return;
            out.push({ kind: "heading", label: title, pick: -1 });
            for (var i = 0; i < list.length; i++)
                out.push({ kind: kind, value: list[i], pick: pick++ });
        };
        group("Programs", "program", root.programs);
        group("Control Panel", "task", root.tasks);
        group("Files", "file", root.files);
        return out;
    }
    readonly property int pickCount: {
        var count = 0;
        for (var i = 0; i < root.rows.length; i++) {
            if (root.rows[i].pick >= 0)
                count++;
        }
        return count;
    }

    /// The row with that pick number, or null.
    function rowAt(pick) {
        for (var i = 0; i < root.rows.length; i++) {
            if (root.rows[i].pick === pick)
                return root.rows[i];
        }
        return null;
    }

    /// Do what a row stands for: start the program, open the task's page,
    /// or open the file (a folder in the file manager).
    function choose(row) {
        if (!row)
            return;
        if (row.kind === "program")
            Apps.launch(row.value);
        else if (row.kind === "task")
            ControlPanelState.open(row.value.page);
        else if (row.kind === "file")
            Quickshell.execDetached(["xdg-open", row.value]);
    }

    /// Return in a search box: the row picked, else the text run as a
    /// command, as Run... does. `true` when something was done.
    function accept(pick) {
        if (!root.active)
            return false;
        var row = root.rowAt(Math.min(pick, root.pickCount - 1));
        if (row) {
            root.choose(row);
            return true;
        }
        return Apps.runTyped(root.query);
    }

    // The files: fd in your home folder, a moment after the last key, for
    // names that hold what is typed. Hidden folders stay out.
    onQueryChanged: {
        root.files = [];
        findFiles.restart();
    }

    Timer {
        id: findFiles
        interval: 250
        onTriggered: {
            var query = root.query.trim();
            if (query.length < 2)
                return;
            fileFinder.running = false;
            fileFinder.query = query;
            fileFinder.command = ["fd", "--max-results", "8", "--ignore-case",
                                  "--fixed-strings", "--", query, Apps.home];
            fileFinder.running = true;
        }
    }

    Process {
        id: fileFinder

        /// The query it looks for: an answer to an older one is dropped.
        property string query: ""

        stdout: StdioCollector {
            onStreamFinished: {
                if (fileFinder.query !== root.query.trim())
                    return;
                root.files = String(text).split("\n").filter(function (line) {
                    return line.length > 0;
                }).map(function (line) { return line.replace(/\/$/, ""); });
            }
        }
    }
}
