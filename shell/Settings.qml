pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import "tools.js" as Tools

// What the Control Panel changes, kept in ~/.config/shojiwm/settings.json:
// beside the installed configuration, which an update replaces, so the
// settings survive it. The file is watched, so editing it by hand works too.
//
// The light or dark choice is not kept here: it is the system's own
// preference (see `colorScheme`), which the programs read as well.
Singleton {
    id: root

    readonly property string configHome: Quickshell.env("XDG_CONFIG_HOME")
        || (Quickshell.env("HOME") + "/.config")
    readonly property string home: Quickshell.env("HOME") || "/"

    property alias wallpaper: adapter.wallpaper
    property alias lockBackground: adapter.lockBackground
    property alias userPicture: adapter.userPicture
    property alias combineWindows: adapter.combineWindows
    property alias showFrequent: adapter.showFrequent
    property alias powerButtonAction: adapter.powerButtonAction
    property alias peek: adapter.peek
    property alias clockFormat: adapter.clockFormat
    property alias showDate: adapter.showDate
    property alias lockAfterMinutes: adapter.lockAfterMinutes
    property alias lockOnSuspend: adapter.lockOnSuspend

    // ----- pictures -----------------------------------------------------------

    readonly property url builtInWallpaper: Qt.resolvedUrl("assets/wallpaper.svg")

    function fileUrl(path) {
        return path && path.length > 0 ? "file://" + path : "";
    }

    /// The desktop's picture.
    readonly property url wallpaperSource: adapter.wallpaper.length > 0
        ? fileUrl(adapter.wallpaper) : builtInWallpaper
    /// The lock screen's picture: its own ("builtin" for the built-in one),
    /// else the desktop's.
    readonly property url lockSource: adapter.lockBackground === "builtin"
        ? builtInWallpaper
        : adapter.lockBackground.length > 0
            ? fileUrl(adapter.lockBackground) : wallpaperSource

    /// The account picture: the one chosen here, else ~/.face, the file
    /// display managers read, else the drawn one.
    readonly property url userPictureSource: adapter.userPicture.length > 0
        ? fileUrl(adapter.userPicture)
        : faceExists ? fileUrl(home + "/.face") : Qt.resolvedUrl("assets/user.svg")

    property bool faceExists: false

    FileView {
        path: root.home + "/.face"
        printErrors: false
        blockLoading: false
        onLoaded: root.faceExists = true
        onLoadFailed: root.faceExists = false
    }

    // ----- the file -----------------------------------------------------------

    FileView {
        id: file

        path: root.configHome + "/shojiwm/settings.json"
        watchChanges: true
        printErrors: false
        onFileChanged: reload()
        onAdapterUpdated: writeAdapter()
        // The first run: write the defaults out, so there is a file to edit.
        onLoadFailed: function (error) {
            if (error === FileViewError.FileNotFound)
                writeAdapter();
        }

        JsonAdapter {
            id: adapter

            /// Absolute paths; "" for the built-in picture. For the lock
            /// screen, "" is the desktop's picture and "builtin" the built-in.
            property string wallpaper: ""
            property string lockBackground: ""
            property string userPicture: ""

            property bool combineWindows: true
            property bool showFrequent: true
            /// "shutdown", "restart", "sleep", "hibernate", "lock" or "logoff".
            property string powerButtonAction: "shutdown"
            property bool peek: true

            /// "locale", "24h" or "12h".
            property string clockFormat: "locale"
            property bool showDate: true

            /// 0 for never.
            property int lockAfterMinutes: 10
            property bool lockOnSuspend: true
        }
    }

    // ----- light or dark ------------------------------------------------------

    /// The system's preference, as GNOME stores it and the settings portal
    /// passes it on: "default", "prefer-light" or "prefer-dark".
    property string colorScheme: "default"
    readonly property bool dark: colorScheme === "prefer-dark"
    /// `false` when dconf cannot be reached: the choice then holds for this
    /// shell only, until it restarts.
    property bool systemSchemeAvailable: false

    readonly property string schemeKey: "/org/gnome/desktop/interface/color-scheme"
    readonly property string gtkThemeKey: "/org/gnome/desktop/interface/gtk-theme"

    function parseScheme(text) {
        var match = /'([a-z-]+)'/.exec(String(text));
        return match ? match[1] : "";
    }

    // The status is echoed after the value: an unset key reads as nothing,
    // which is not a failure.
    Process {
        command: ["sh", "-c", "\"$0\" read \"$1\"; echo \"status=$?\"",
                  Tools.dconf, root.schemeKey]
        running: true
        stdout: StdioCollector {
            onStreamFinished: {
                var scheme = root.parseScheme(text);
                if (scheme.length > 0)
                    root.colorScheme = scheme;
                root.systemSchemeAvailable = String(text).indexOf("status=0") >= 0;
            }
        }
    }

    // Another program can change it too: the shell follows.
    Process {
        command: [Tools.dconf, "watch", root.schemeKey]
        running: true
        stdout: SplitParser {
            onRead: function (line) {
                var scheme = root.parseScheme(line);
                if (scheme.length > 0)
                    root.colorScheme = scheme;
            }
        }
    }

    /// Set the preference for every program: the colour scheme, and the GTK 3
    /// theme, which GTK 3 programs read instead.
    function setDark(on) {
        root.colorScheme = on ? "prefer-dark" : "default";
        Quickshell.execDetached([Tools.dconf, "write", root.schemeKey,
                                 on ? "'prefer-dark'" : "'default'"]);
        Quickshell.execDetached([Tools.dconf, "write", root.gtkThemeKey,
                                 on ? "'Adwaita-dark'" : "'Adwaita'"]);
    }
}
