pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// The network, from NetworkManager through nmcli: the connection in use,
// the wireless networks in reach, and the actions of the Windows 7 network
// panel. `nmcli monitor` reports each change; the state is then read again.
Singleton {
    id: root

    /// The connection in use: { name, type ("ethernet", "wifi", ...),
    /// device }, or null.
    property var primary: null
    /// "full", "limited", "portal", "none" or "unknown".
    property string connectivity: "unknown"
    property bool wifiEnabled: false
    /// Whether the computer has a wireless device at all.
    property bool hasWifi: false
    /// The wireless networks in reach, strongest first, one per name:
    /// { ssid, signal (0 to 100), secure, active, saved }.
    property var networks: []
    /// The names of the saved connections.
    property var saved: []
    /// The last error of an action, for the panel to show.
    property string error: ""
    property bool busy: false

    readonly property string editor: "nm-connection-editor"

    /// 0 to 4 bars for the signal of the wireless network in use.
    readonly property int bars: {
        for (var i = 0; i < root.networks.length; i++) {
            if (root.networks[i].active)
                return root.barsOf(root.networks[i].signal);
        }
        return 0;
    }

    function barsOf(signal) {
        return signal >= 80 ? 4 : signal >= 60 ? 3 : signal >= 40 ? 2 : signal >= 20 ? 1 : 0;
    }

    /// Split a line of `nmcli -t`, where `:` separates fields and `\:` is a
    /// colon inside one.
    function fields(line) {
        var out = [];
        var current = "";
        for (var i = 0; i < line.length; i++) {
            var c = line[i];
            if (c === "\\" && i + 1 < line.length) {
                current += line[i + 1];
                i++;
            } else if (c === ":") {
                out.push(current);
                current = "";
            } else {
                current += c;
            }
        }
        out.push(current);
        return out;
    }

    function refresh() {
        state.running = false;
        state.running = true;
    }

    /// Scan again for wireless networks; the list follows when it is done.
    function rescan() {
        root.run(["nmcli", "device", "wifi", "rescan"]);
    }

    function setWifi(on) {
        root.run(["nmcli", "radio", "wifi", on ? "on" : "off"]);
    }

    /// Join a wireless network: a saved one by its connection, a new one
    /// by its name. For a new secured network, NetworkManager asks its
    /// secret agent (nm-applet) for the key.
    function connect(network) {
        if (network.saved)
            root.run(["nmcli", "connection", "up", "id", network.ssid]);
        else
            root.run(["nmcli", "device", "wifi", "connect", network.ssid]);
    }

    function disconnect() {
        if (root.primary)
            root.run(["nmcli", "device", "disconnect", root.primary.device]);
    }

    function openEditor() {
        Quickshell.execDetached([root.editor]);
    }

    function run(command) {
        root.error = "";
        root.busy = true;
        action.command = command;
        action.running = true;
    }

    // Everything in one run of a shell, as sections the parser splits.
    Process {
        id: state

        command: ["sh", "-c",
            "echo '#general'; nmcli -t -f CONNECTIVITY,WIFI general;"
            + " echo '#devices'; nmcli -t -f DEVICE,TYPE,STATE,CONNECTION device;"
            + " echo '#saved'; nmcli -t -f NAME,TYPE connection show;"
            + " echo '#wifi'; nmcli -t -f IN-USE,SSID,SIGNAL,SECURITY device wifi list --rescan no"]
        running: true

        stdout: StdioCollector {
            onStreamFinished: {
                var section = "";
                var primary = null;
                var hasWifi = false;
                var saved = [];
                var byName = {};
                var lines = text.split("\n");
                for (var i = 0; i < lines.length; i++) {
                    var line = lines[i];
                    if (line.length === 0)
                        continue;
                    if (line[0] === "#") {
                        section = line.substring(1);
                        continue;
                    }
                    var f = root.fields(line);
                    if (section === "general") {
                        root.connectivity = f[0] || "unknown";
                        root.wifiEnabled = f[1] === "enabled";
                    } else if (section === "devices") {
                        if (f[1] === "wifi")
                            hasWifi = true;
                        // The first connected ethernet or wifi device, as
                        // nmcli lists the default route's first.
                        if (!primary && f[2] === "connected"
                            && (f[1] === "ethernet" || f[1] === "wifi"))
                            primary = { device: f[0], type: f[1], name: f[3] };
                    } else if (section === "saved") {
                        if (f[1] === "802-11-wireless")
                            saved.push(f[0]);
                    } else if (section === "wifi") {
                        var ssid = f[1];
                        if (!ssid)
                            continue;
                        var network = {
                            ssid: ssid,
                            signal: Number(f[2]) || 0,
                            secure: (f[3] || "").length > 0 && f[3] !== "--",
                            active: f[0] === "*"
                        };
                        var known = byName[ssid];
                        if (!known || network.active || (!known.active && network.signal > known.signal))
                            byName[ssid] = network;
                    }
                }
                var list = [];
                for (var name in byName) {
                    var item = byName[name];
                    item.saved = saved.indexOf(name) >= 0;
                    list.push(item);
                }
                list.sort(function (a, b) {
                    return (b.active ? 1 : 0) - (a.active ? 1 : 0) || b.signal - a.signal;
                });
                root.primary = primary;
                root.hasWifi = hasWifi;
                root.saved = saved;
                root.networks = list;
            }
        }
    }

    Process {
        id: action

        stderr: StdioCollector {
            onStreamFinished: {
                if (text.trim().length > 0)
                    root.error = text.trim().replace(/^Error: /, "");
            }
        }
        onRunningChanged: {
            if (!running) {
                root.busy = false;
                root.refresh();
            }
        }
    }

    // Each change NetworkManager reports: read the state again, once the
    // burst of lines is over.
    Process {
        command: ["nmcli", "monitor"]
        running: true
        stdout: SplitParser {
            onRead: settle.restart()
        }
    }

    Timer {
        id: settle
        interval: 400
        onTriggered: root.refresh()
    }
}
