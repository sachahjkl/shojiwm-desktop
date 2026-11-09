pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// The programs that start when you log on: the XDG autostart entries, which
// systemd's xdg-autostart generator starts in this session.
//
// An entry is a .desktop file in an autostart folder: the system's
// ($XDG_CONFIG_DIRS/autostart, the first folder wins) or yours
// (~/.config/autostart). A file of yours replaces the system's of the same
// name, and `Hidden=true` in it turns the entry off. Changes apply from the
// next log-on.
Singleton {
    id: root

    readonly property string userDir: (Quickshell.env("XDG_CONFIG_HOME")
        || (Quickshell.env("HOME") + "/.config")) + "/autostart"
    readonly property var systemDirs: (Quickshell.env("XDG_CONFIG_DIRS") || "/etc/xdg")
        .split(":").filter(function (dir) { return dir.length > 0; })
        .map(function (dir) { return dir + "/autostart"; })
    readonly property var desktops: (Quickshell.env("XDG_CURRENT_DESKTOP") || "")
        .split(":")

    /// One object per entry, by name: id (the file name), name, comment,
    /// exec, icon, enabled, here (shown in this desktop), systemPath and
    /// userPath ("" when there is none).
    property var entries: []

    // Every autostart file, as lines: `FILE <tab> system|user <tab> path`,
    // then `KEY <tab> key <tab> value` for the keys of its [Desktop Entry]
    // group that matter here.
    readonly property string listScript: "
        keys() {
            awk '/^\\[/ { group = $0; next }
                 group == \"[Desktop Entry]\" && /^(Name|Comment|Exec|Icon|Hidden|OnlyShowIn|NotShowIn|X-GNOME-Autostart-enabled)=/ {
                     i = index($0, \"=\");
                     print \"KEY\\t\" substr($0, 1, i - 1) \"\\t\" substr($0, i + 1)
                 }' \"$1\"
        }
        user=$1; shift
        seen=' '
        for dir in \"$@\"; do
            for file in \"$dir\"/*.desktop; do
                [ -f \"$file\" ] || continue
                id=${file##*/}
                case \"$seen\" in *\" $id \"*) continue ;; esac
                seen=\"$seen$id \"
                printf 'FILE\\tsystem\\t%s\\n' \"$file\"
                keys \"$file\"
            done
        done
        for file in \"$user\"/*.desktop; do
            [ -f \"$file\" ] || continue
            printf 'FILE\\tuser\\t%s\\n' \"$file\"
            keys \"$file\"
        done"

    function refresh() {
        lister.running = false;
        lister.running = true;
    }

    function parse(text) {
        var byId = {};
        var current = null;
        var lines = String(text).split("\n");
        for (var i = 0; i < lines.length; i++) {
            var parts = lines[i].split("\t");
            if (parts[0] === "FILE" && parts.length >= 3) {
                var path = parts.slice(2).join("\t");
                var id = path.substring(path.lastIndexOf("/") + 1);
                var item = byId[id] || { id: id, systemPath: "", userPath: "", keys: {} };
                if (parts[1] === "user") {
                    item.userPath = path;
                    // Your file replaces the system's as a whole.
                    item.keys = {};
                } else {
                    item.systemPath = path;
                }
                byId[id] = item;
                current = item;
            } else if (parts[0] === "KEY" && current && parts.length >= 3) {
                current.keys[parts[1]] = parts.slice(2).join("\t");
            }
        }
        var list = [];
        for (var key in byId) {
            var entry = byId[key];
            var keys = entry.keys;
            list.push({
                id: entry.id,
                name: keys.Name || entry.id.replace(/\.desktop$/, ""),
                comment: keys.Comment || "",
                exec: keys.Exec || "",
                icon: keys.Icon || "",
                enabled: keys.Hidden !== "true"
                    && keys["X-GNOME-Autostart-enabled"] !== "false",
                here: root.showsHere(keys),
                systemPath: entry.systemPath,
                userPath: entry.userPath
            });
        }
        list.sort(function (a, b) { return a.name.localeCompare(b.name); });
        root.entries = list;
    }

    /// Whether OnlyShowIn and NotShowIn let the entry start in this desktop.
    function showsHere(keys) {
        var only = (keys.OnlyShowIn || "").split(";").filter(function (name) { return name.length > 0; });
        var not = (keys.NotShowIn || "").split(";").filter(function (name) { return name.length > 0; });
        var inList = function (names) {
            return root.desktops.some(function (desktop) { return names.indexOf(desktop) >= 0; });
        };
        if (only.length > 0 && !inList(only))
            return false;
        return !inList(not);
    }

    /// Turn an entry on or off: your copy of it, without the keys that turn
    /// it off, and with `Hidden=true` when it must not start.
    function setEnabled(entry, on) {
        var source = entry.userPath || entry.systemPath;
        var target = root.userDir + "/" + entry.id;
        root.run("mkdir -p \"$1\" && sed -e '/^Hidden=/d' -e '/^X-GNOME-Autostart-enabled=/d'"
            + (on ? "" : " -e '/^\\[Desktop Entry\\]/a Hidden=true'")
            + " \"$2\" > \"$3.new\" && mv \"$3.new\" \"$3\"",
            [root.userDir, source, target]);
    }

    /// Remove your file: an entry you added goes, a system entry you
    /// changed is back as the system has it.
    function removeUserFile(entry) {
        if (entry.userPath.length > 0)
            root.run("rm -f \"$1\"", [entry.userPath]);
    }

    /// Start a program at log-on: a copy of its .desktop file in your
    /// autostart folder.
    function add(fileId) {
        var dataDirs = [(Quickshell.env("XDG_DATA_HOME")
            || (Quickshell.env("HOME") + "/.local/share"))]
            .concat((Quickshell.env("XDG_DATA_DIRS") || "/usr/share").split(":"));
        root.run("target=$1; id=$2; shift 2; mkdir -p \"$target\"; "
            + "for dir in \"$@\"; do [ -f \"$dir/applications/$id\" ] && "
            + "{ cp -L \"$dir/applications/$id\" \"$target/$id\"; chmod u+w \"$target/$id\"; exit 0; }; done; exit 1",
            [root.userDir, fileId].concat(dataDirs));
    }

    function run(script, args) {
        writer.command = ["sh", "-c", script, "sh"].concat(args);
        writer.running = true;
    }

    Process {
        id: lister
        command: ["sh", "-c", root.listScript, "sh", root.userDir].concat(root.systemDirs)
        running: true
        stdout: StdioCollector {
            onStreamFinished: root.parse(text)
        }
    }

    Process {
        id: writer
        // Read the folders again once the change is written.
        onRunningChanged: {
            if (!running)
                root.refresh();
        }
    }
}
