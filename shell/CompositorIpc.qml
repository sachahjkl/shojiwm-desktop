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
    readonly property bool available: socket.connected

    function send(method, params) {
        if (!socket.connected)
            return false;
        socket.write(JSON.stringify({ method: method, params: params }) + "\n");
        socket.flush();
        return true;
    }

    /// Ask the compositor and get its answer: `callback(result)`, with
    /// `null` when it fails or the line is down.
    property int lastId: 0
    property var pending: ({})

    function request(method, params, callback) {
        if (!socket.connected) {
            callback(null);
            return;
        }
        root.lastId += 1;
        root.pending[root.lastId] = callback;
        socket.write(JSON.stringify({ id: root.lastId, method: method, params: params }) + "\n");
        socket.flush();
    }

    function answer(line) {
        var message;
        try {
            message = JSON.parse(line);
        } catch (error) {
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
        function onGlassColorChanged() { root.sendGlass(); }
        function onGlassColorStrengthChanged() { root.sendGlass(); }
        function onDisplayChoicesChanged() { root.sendDisplay(); }
    }


    Socket {
        id: socket

        path: root.path
        connected: root.path.length > 0
        parser: SplitParser {
            onRead: function (line) { root.answer(line); }
        }
        // The compositor can be reloaded, which closes its end.
        onConnectedChanged: {
            if (connected) {
                root.sendAnimations();
                root.sendGlass();
                root.sendCorners();
                root.sendDisplay();
            }
            else if (root.path.length > 0)
                reconnect.restart();
        }
    }

    Timer {
        id: reconnect
        interval: 1000
        onTriggered: socket.connected = true
    }
}
