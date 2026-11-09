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

    function peek(on) {
        return send("peek", { on: on });
    }

    function showDesktop() {
        return send("show-desktop", {});
    }

    function sendAnimations() {
        return send("animations", { on: Settings.animations });
    }

    function sendGlass() {
        return send("glass", { level: Settings.glassBlur });
    }

    Connections {
        target: Settings
        function onAnimationsChanged() { root.sendAnimations(); }
        function onGlassBlurChanged() { root.sendGlass(); }
    }


    Socket {
        id: socket

        path: root.path
        connected: root.path.length > 0
        // The compositor can be reloaded, which closes its end.
        onConnectedChanged: {
            if (connected) {
                root.sendAnimations();
                root.sendGlass();
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
