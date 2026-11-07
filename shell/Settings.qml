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

    /// "aero", Windows 7 as it was, or "modern", the same glass made quieter.
    property alias style: adapter.style
    readonly property bool modern: adapter.style === "modern"
    property alias wallpaper: adapter.wallpaper
    property alias lockBackground: adapter.lockBackground
    property alias userPicture: adapter.userPicture
    property alias combineWindows: adapter.combineWindows
    property alias taskbarCompact: adapter.taskbarCompact
    property alias showFrequent: adapter.showFrequent
    property alias powerButtonAction: adapter.powerButtonAction
    property alias peek: adapter.peek
    property alias clockFormat: adapter.clockFormat
    property alias showDate: adapter.showDate
    property alias lockAfterMinutes: adapter.lockAfterMinutes
    property alias lockOnSuspend: adapter.lockOnSuspend
    property alias trayShowAll: adapter.trayShowAll
    property alias trayShown: adapter.trayShown

    /// Whether a notification-area item shows on the taskbar, or only in the
    /// popover of hidden icons. New items are hidden, as on Windows 7.
    function trayVisible(id) {
        return adapter.trayShowAll || adapter.trayShown.indexOf(String(id)) >= 0;
    }

    function setTrayVisible(id, on) {
        var list = [];
        for (var i = 0; i < adapter.trayShown.length; i++) {
            if (adapter.trayShown[i] !== String(id))
                list.push(adapter.trayShown[i]);
        }
        if (on)
            list.push(String(id));
        adapter.trayShown = list;
    }

    // ----- pictures -----------------------------------------------------------

    /// The built-in picture, in its night version while the system prefers
    /// dark.
    readonly property url builtInWallpaper: Qt.resolvedUrl(root.dark
        ? "assets/wallpaper-dark.svg" : "assets/wallpaper.svg")

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

            property string style: "aero"

            /// Absolute paths; "" for the built-in picture. For the lock
            /// screen, "" is the desktop's picture and "builtin" the built-in.
            property string wallpaper: ""
            property string lockBackground: ""
            property string userPicture: ""

            property bool combineWindows: true
            /// Taskbar buttons show the icon alone, as Windows 7's "hide
            /// labels".
            property bool taskbarCompact: false
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

            /// Show every notification-area icon on the taskbar.
            property bool trayShowAll: false
            /// The ids of the icons shown on the taskbar; the rest wait in
            /// the popover of hidden icons.
            property list<string> trayShown: []
        }
    }

    // ----- light or dark ------------------------------------------------------

    //   quickshell -p ~/.config/shojiwm/shell ipc call appearance setDark true
    IpcHandler {
        target: "appearance"

        function setDark(on: bool): void { root.setDark(on); }
        function toggle(): void { root.setDark(!root.dark); }
        function isDark(): bool { return root.dark; }
        /// "aero" or "modern".
        function setStyle(style: string): void {
            if (style === "aero" || style === "modern")
                adapter.style = style;
        }
    }

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

    /// Set the preference for every program. The colour scheme is what GTK 4,
    /// libadwaita, Qt, the browsers and Vicinae read. GTK 3 programs read the
    /// theme instead: it moves to the light or dark variant of the theme in
    /// use, when that variant is installed (Arc and Arc-Dark, Adwaita and
    /// Adwaita-dark, Foo-Light and Foo-Dark), and is left alone otherwise.
    function setDark(on) {
        root.colorScheme = on ? "prefer-dark" : "default";
        Quickshell.execDetached([Tools.dconf, "write", root.schemeKey,
                                 on ? "'prefer-dark'" : "'default'"]);
        Quickshell.execDetached(["sh", root.gtkSwitch,
                                 Tools.dconf, root.gtkThemeKey, on ? "dark" : "light"]);
    }

    readonly property string gtkSwitch: String(Qt.resolvedUrl("gtk-switch.sh")).replace(/^file:\/\//, "")
}
