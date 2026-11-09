pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import Quickshell.Services.Greetd

// The log-on screen's side of greetd (greeter.qml): it logs one account on,
// into the session chosen in the bottom-left corner. It offers what
// LockScreen.qml asks of LockState: authenticate(), busy, message,
// failures and typed; and `prompt`, when PAM asks more than the password
// (a one-time code): the field then shows the question, and what is
// typed next answers it.
//
// Set by the system (the nixconfig module that starts greetd):
//
//   - SHOJIWM_GREETER_USER: the account to log on;
//   - SHOJIWM_GREETER_SESSIONS: the folder of the sessions' .desktop files
//     (share/wayland-sessions);
//   - SHOJIWM_GREETER_DEFAULT_SESSION: the session chosen the first time,
//     by file name without .desktop. Then the last one used is chosen.
Singleton {
    id: root

    readonly property string user: Quickshell.env("SHOJIWM_GREETER_USER") || ""
    /// The full name from the password database, else the login name.
    property string userName: root.user

    Process {
        command: ["getent", "passwd", root.user]
        running: root.user.length > 0
        stdout: StdioCollector {
            onStreamFinished: {
                var fields = String(text).trim().split(":");
                var full = fields.length > 4 ? fields[4].split(",")[0].trim() : "";
                if (full.length > 0)
                    root.userName = full;
            }
        }
    }

    /// The account picture: AccountsService's icon, which the greeter can
    /// read (not ~/.face, in a private home), else the drawn one.
    property url picture: Qt.resolvedUrl("assets/user.svg")

    FileView {
        path: root.user.length > 0 ? "/var/lib/AccountsService/icons/" + root.user : ""
        printErrors: false
        onLoaded: root.picture = "file://" + path
    }

    // ----- the sessions -------------------------------------------------------

    /// Each session: `id` (the file name without .desktop), `name` and `exec`
    /// from the .desktop file, `desktopNames` (its DesktopNames, as a list).
    property var sessions: []
    property string sessionId: ""

    readonly property var session: {
        for (var i = 0; i < root.sessions.length; i++) {
            if (root.sessions[i].id === root.sessionId)
                return root.sessions[i];
        }
        return root.sessions.length > 0 ? root.sessions[0] : null;
    }

    // Every .desktop file of the folder, each after a line "\x1e<file name>".
    Process {
        command: ["sh", "-c",
            "for f in \"$1\"/*.desktop; do printf '\\036%s\\n' \"${f##*/}\"; cat \"$f\"; done",
            "sh", Quickshell.env("SHOJIWM_GREETER_SESSIONS") || ""]
        running: true
        stdout: StdioCollector {
            onStreamFinished: root.sessions = root.parseSessions(String(text))
        }
    }

    function parseSessions(text) {
        var list = [];
        var files = text.split("\x1e");
        for (var i = 0; i < files.length; i++) {
            var lines = files[i].split("\n");
            var file = lines[0].trim();
            if (!file.endsWith(".desktop"))
                continue;
            var entry = { id: file.slice(0, -8), name: "", exec: "", desktopNames: [] };
            var inEntry = false;
            var hidden = false;
            for (var j = 1; j < lines.length; j++) {
                var line = lines[j].trim();
                if (line.startsWith("[")) {
                    inEntry = line === "[Desktop Entry]";
                    continue;
                }
                var eq = line.indexOf("=");
                if (!inEntry || eq < 0)
                    continue;
                var key = line.slice(0, eq);
                var value = line.slice(eq + 1);
                if (key === "Name")
                    entry.name = value;
                else if (key === "Exec")
                    entry.exec = value;
                else if (key === "DesktopNames")
                    entry.desktopNames = value.split(";").filter(function (name) {
                        return name.length > 0;
                    });
                else if ((key === "Hidden" || key === "NoDisplay") && value === "true")
                    hidden = true;
            }
            if (!hidden && entry.exec.length > 0)
                list.push(entry);
        }
        list.sort(function (a, b) {
            return a.name.localeCompare(b.name);
        });
        return list;
    }

    // The last session used, kept in the greeter's state folder.
    FileView {
        id: lastSession

        path: Quickshell.statePath("last-session")
        printErrors: false
        onLoaded: root.sessionId = lastSession.text().trim()
        onLoadFailed: root.sessionId = Quickshell.env("SHOJIWM_GREETER_DEFAULT_SESSION") || ""
    }

    // ----- logging on -----------------------------------------------------------

    property bool busy: false
    property string message: ""
    property int failures: 0
    property string typed: ""
    property string pendingPassword: ""
    /// A question from PAM after the password, or "": the field asks it.
    property string prompt: ""
    /// The answer to `prompt` shows as typed (not as dots).
    property bool promptEcho: false

    function authenticate(password) {
        if (root.busy || password.length === 0)
            return;
        if (root.prompt.length > 0) {
            root.prompt = "";
            root.typed = "";
            root.busy = true;
            Greetd.respond(password);
            return;
        }
        if (!Greetd.available || root.user.length === 0 || !root.session) {
            root.fail("Cannot log on: no greetd, account or session.");
            return;
        }
        root.pendingPassword = password;
        root.busy = true;
        root.message = "";
        Greetd.createSession(root.user);
    }

    function fail(text) {
        root.busy = false;
        root.pendingPassword = "";
        root.prompt = "";
        root.message = text;
        root.failures += 1;
    }

    Connections {
        target: Greetd

        // PAM asks for the password: give it the one typed. A question
        // after it (a one-time code) goes to the field.
        function onAuthMessage(message, error, responseRequired, echoResponse) {
            if (!responseRequired)
                return;
            if (root.pendingPassword.length > 0 && !echoResponse) {
                Greetd.respond(root.pendingPassword);
                root.pendingPassword = "";
            } else {
                root.busy = false;
                root.typed = "";
                root.promptEcho = echoResponse;
                root.prompt = message.trim().replace(/:$/, "");
            }
        }

        function onAuthFailure(message) {
            Greetd.cancelSession();
            root.fail("The user name or password is incorrect.");
        }

        function onReadyToLaunch() {
            var session = root.session;
            lastSession.setText(session.id + "\n");
            var desktops = session.desktopNames.length > 0
                ? session.desktopNames.join(":") : session.id;
            // greetd joins the command's words with spaces and gives the
            // line to a shell, after /etc/profile (the system's PATH and
            // XDG_DATA_DIRS). So Exec goes as one word: the shell splits it
            // as the .desktop file writes it. ["/bin/sh", "-c", exec] would
            // be split again, and run the first word of Exec alone.
            Greetd.launch([session.exec], [
                "XDG_SESSION_TYPE=wayland",
                "XDG_SESSION_DESKTOP=" + session.id,
                "XDG_CURRENT_DESKTOP=" + desktops
            ], true);
        }

        function onError(error) {
            Greetd.cancelSession();
            root.fail("Cannot log on: " + error);
        }
    }
}
