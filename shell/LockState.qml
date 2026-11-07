pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import Quickshell.Services.Pam
import Quickshell.Wayland
import "tools.js" as Tools

// Whether the session is locked, and everything that locks it:
//
//   - the Lock entry of the Start menu's power button, and Super+L
//     (`quickshell -p ~/.config/shojiwm/shell ipc call lock lock`);
//   - `loginctl lock-session`, and anything else that asks logind, such as
//     an idle daemon;
//   - going to sleep, when the Control Panel says so;
//   - the pointer and the keyboard left alone for the Control Panel's delay.
//
// Only a right password unlocks it: see LockScreen.qml.
Singleton {
    id: root

    property bool locked: false

    function lock() {
        root.locked = true;
    }

    // ----- the password ---------------------------------------------------------

    /// PAM is checking a password.
    property bool busy: false
    /// What to tell the user under the field, or "".
    property string message: ""
    property bool messageIsError: false
    /// Counts failures, so each screen clears its field.
    property int failures: 0

    property string pendingPassword: ""

    /// Check a password with PAM, as the "login" service of /etc/pam.d does.
    /// Only its success unlocks.
    function authenticate(password) {
        if (root.busy || !root.locked)
            return;
        root.pendingPassword = password;
        root.busy = true;
        root.message = "";
        if (!pam.start()) {
            root.busy = false;
            root.fail("The password could not be checked.");
        }
    }

    function fail(text) {
        root.message = text;
        root.messageIsError = true;
        root.failures += 1;
    }

    PamContext {
        id: pam

        // PAM asks for the password: give it the one typed.
        onResponseRequiredChanged: {
            if (responseRequired) {
                respond(root.pendingPassword);
                root.pendingPassword = "";
            }
        }

        onCompleted: function (result) {
            root.busy = false;
            root.pendingPassword = "";
            if (result === PamResult.Success) {
                root.message = "";
                root.locked = false;
            } else {
                root.fail("The password is incorrect. Try again.");
            }
        }

        onError: function (error) {
            root.busy = false;
            root.pendingPassword = "";
            root.fail("The password could not be checked.");
        }
    }

    onLockedChanged: {
        if (locked) {
            root.message = "";
            root.busy = false;
        }
    }

    IpcHandler {
        target: "lock"

        function lock(): void { root.lock(); }
        function isLocked(): bool { return root.locked; }
    }

    // ----- logind ---------------------------------------------------------------

    // The session's object path: logind escapes the session id, a digit
    // first, so "2" is "_32".
    readonly property string sessionId: Quickshell.env("XDG_SESSION_ID") || ""
    readonly property string sessionPath: {
        var escaped = "";
        for (var i = 0; i < sessionId.length; i++) {
            var c = sessionId.charAt(i);
            if (/[A-Za-z]/.test(c) || (/[0-9]/.test(c) && i > 0))
                escaped += c;
            else
                escaped += "_" + c.charCodeAt(0).toString(16);
        }
        return "/org/freedesktop/login1/session/" + escaped;
    }

    // `loginctl lock-session` makes logind send Lock to the session.
    Process {
        running: root.sessionId.length > 0
        command: [Tools.gdbus, "monitor", "--system",
                  "--dest", "org.freedesktop.login1",
                  "--object-path", root.sessionPath]
        stdout: SplitParser {
            onRead: function (line) {
                // e.g. "/org/freedesktop/login1/session/_32:
                // org.freedesktop.login1.Session.Lock ()"
                if (line.indexOf(".Session.Lock (") >= 0)
                    root.lock();
            }
        }
    }

    // Going to sleep, from the lid or a key as much as from the Start menu.
    Process {
        running: true
        command: [Tools.gdbus, "monitor", "--system",
                  "--dest", "org.freedesktop.login1",
                  "--object-path", "/org/freedesktop/login1"]
        stdout: SplitParser {
            onRead: function (line) {
                if (Settings.lockOnSuspend
                    && line.indexOf("PrepareForSleep (true") >= 0)
                    root.lock();
            }
        }
    }

    // ----- idle -----------------------------------------------------------------

    IdleMonitor {
        enabled: Settings.lockAfterMinutes > 0 && !root.locked
        timeout: Math.max(1, Settings.lockAfterMinutes) * 60
        // Leave a video playing alone.
        respectInhibitors: true
        onIsIdleChanged: {
            if (isIdle)
                root.lock();
        }
    }
}
