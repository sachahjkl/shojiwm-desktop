pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// The shell's line to the compositor, for what the shell alone cannot do:
// Aero Peek, the show-desktop toggle that brings the windows back, and
// whether windows animate.
//
// src/peek.tsx listens, and hands the path to the shell in
// SHOJIWM_WIN7_SOCKET when it starts it. The wire format is ShojiWM's: one
// JSON object per line, `{ "method": ..., "params": ... }`.
Singleton {
    id: root

    readonly property string path: Quickshell.env("SHOJIWM_WIN7_SOCKET") || ""
    /// `false` when the shell was started some other way: the callers then
    /// fall back to what the shell can do on its own.
    readonly property bool available: root.socket !== null && root.socket.connected
    /// The current try's Socket (see `line`).
    readonly property Socket socket: line.item as Socket

    // Once the line is up, the compositor gets the settings it draws with.
    // Not from the Socket's own connected change: a local socket can connect
    // while the Loader still builds it, before `socket` names it, and what
    // is sent then is lost.
    onAvailableChanged: {
        if (available) {
            root.sendAnimations();
            root.sendGlass();
            root.sendCorners();
            root.sendPrograms();
            root.sendDisplay();
        }
    }

    function send(method, params) {
        if (!root.available)
            return false;
        root.socket.write(JSON.stringify({ method: method, params: params }) + "\n");
        root.socket.flush();
        return true;
    }

    /// Ask the compositor and get its answer: `callback(result)`, with
    /// `null` when it fails or the line is down.
    property int lastId: 0
    property var pending: ({})

    function request(method, params, callback) {
        if (!root.available) {
            callback(null);
            return;
        }
        root.lastId += 1;
        root.pending[root.lastId] = callback;
        root.socket.write(JSON.stringify({ id: root.lastId, method: method, params: params }) + "\n");
        root.socket.flush();
    }

    /// The Alt+Tab panel while it is open: { output, selected, titleHeight,
    /// cells: [{ x, y, width, title }] } in the output's logical pixels; null
    /// when it is closed. AltTabTitles.qml draws the titles.
    property var altTab: null

    function answer(line) {
        var message;
        try {
            message = JSON.parse(line);
        } catch (error) {
            return;
        }
        // The compositor's own events: the Alt+Tab panel's titles (src/alt-tab.tsx).
        if (message.event === "alt-tab") {
            root.altTab = message.payload || null;
            return;
        }
        if (message.id === undefined || !root.pending[message.id])
            return;
        var callback = root.pending[message.id];
        delete root.pending[message.id];
        callback(message.error !== undefined ? null : message.result);
    }

    function sendDisplay() {
        return send("display", { choices: Settings.displayChoices });
    }

    function peek(on) {
        return send("peek", { on: on });
    }

    function showDesktop() {
        return send("show-desktop", {});
    }

    function sendAnimations() {
        return send("animations", {
            on: Settings.animations,
            speed: Settings.animationSpeed
        });
    }

    /// The programs the compositor's keys start (src/programs.ts).
    function sendPrograms() {
        return send("programs", {
            terminal: Apps.program("terminal"),
            taskManager: Apps.program("taskManager"),
            clipboard: Apps.program("clipboard"),
            screenshot: Apps.program("screenshot")
        });
    }

    function sendCorners() {
        return send("corners", { square: Settings.squareCorners });
    }

    function sendGlass() {
        return send("glass", {
            level: Settings.glassBlur,
            colour: Settings.glassColor,
            strength: Settings.glassColorStrength
        });
    }

    Connections {
        target: Settings
        function onAnimationsChanged() { root.sendAnimations(); }
        function onAnimationSpeedChanged() { root.sendAnimations(); }
        function onGlassBlurChanged() { root.sendGlass(); }
        function onSquareCornersChanged() { root.sendCorners(); }
        function onProgramsChanged() { root.sendPrograms(); }
        function onGlassColorChanged() { root.sendGlass(); }
        function onGlassColorStrengthChanged() { root.sendGlass(); }
        function onDisplayChoicesChanged() { root.sendDisplay(); }
    }


    // The line to the compositor. A Socket whose connection failed keeps
    // its failed QLocalSocket, and setting `connected` again does not try
    // again (Quickshell 0.3), so each try is a new Socket. The compositor can
    // be reloaded, which closes its end, and its new socket may not listen
    // yet: a lost or failed line is tried again each second.
    Loader {
        id: line

        active: root.path.length > 0
        sourceComponent: Socket {
            path: root.path
            connected: true
            parser: SplitParser {
                onRead: function (line) { root.answer(line); }
            }
        }
    }

    // While the line is down, a new try each second.
    Timer {
        interval: 1000
        repeat: true
        running: root.path.length > 0 && !root.available
        onTriggered: {
            line.active = false;
            line.active = true;
        }
    }
}
