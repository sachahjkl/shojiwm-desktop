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
    property alias taskbarCompact: adapter.taskbarCompact
    property alias desktopIconSize: adapter.desktopIconSize
    property alias showDesktopIcons: adapter.showDesktopIcons
    property alias desktopSort: adapter.desktopSort
    property alias showFrequent: adapter.showFrequent
    property alias powerButtonAction: adapter.powerButtonAction
    property alias peek: adapter.peek
    property alias clockFormat: adapter.clockFormat
    property alias showDate: adapter.showDate
    property alias lockAfterMinutes: adapter.lockAfterMinutes
    property alias lockOnSuspend: adapter.lockOnSuspend
    property alias trayShowAll: adapter.trayShowAll
    property alias trayShown: adapter.trayShown
    property alias startPinned: adapter.startPinned
    property alias taskbarPinned: adapter.taskbarPinned

    /// `list` without `id`, and with it at the end when `on`.
    function withMember(list, id, on) {
        var next = [];
        for (var i = 0; i < list.length; i++) {
            if (list[i] !== String(id))
                next.push(list[i]);
        }
        if (on)
            next.push(String(id));
        return next;
    }

    /// The MIME type of a notification-area icon being dragged: its item id.
    readonly property string trayDragType: "application/x-shojiwm-win7-tray-item"

    /// Whether a notification-area item shows on the taskbar, or only in the
    /// popover of hidden icons. New items are hidden, as on Windows 7.
    function trayVisible(id) {
        return adapter.trayShowAll || adapter.trayShown.indexOf(String(id)) >= 0;
    }

    function setTrayVisible(id, on) {
        adapter.trayShown = withMember(adapter.trayShown, id, on);
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

    /// The account picture: the one chosen here, else the system's, else the
    /// drawn one.
    readonly property url userPictureSource: adapter.userPicture.length > 0
        ? fileUrl(adapter.userPicture)
        : systemPicture.length > 0 ? fileUrl(systemPicture) : Qt.resolvedUrl("assets/user.svg")

    /// The first readable of the places display managers and desktops read
    /// the account picture from: AccountsService's icon, ~/.face, then
    /// ~/.face.icon. "" when there is none.
    property string systemPicture: ""

    Process {
        command: ["sh", "-c",
            "for f in \"$@\"; do [ -r \"$f\" ] && [ -s \"$f\" ] && { echo \"$f\"; exit; }; done", "sh",
            "/var/lib/AccountsService/icons/" + (Quickshell.env("USER") || ""),
            root.home + "/.face",
            root.home + "/.face.icon"]
        running: true
        stdout: StdioCollector {
            onStreamFinished: root.systemPicture = String(text).trim()
        }
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
            /// Taskbar buttons show the icon alone, as Windows 7's "hide
            /// labels", which is its default too.
            property bool taskbarCompact: true
            property bool showFrequent: true

            /// The desktop: "large", "medium" or "small" icons, whether they
            /// show at all, and how they are sorted: "name", "size", "type"
            /// or "date".
            property string desktopIconSize: "medium"
            property bool showDesktopIcons: true
            property string desktopSort: "name"
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

            /// The programs pinned to the Start menu and to the taskbar, as
            /// .desktop file names, in order.
            property list<string> startPinned: ["kitty.desktop", "thunar.desktop", "chromium.desktop"]
            property list<string> taskbarPinned: ["thunar.desktop", "chromium.desktop", "kitty.desktop"]
        }
    }

    // ----- light or dark ------------------------------------------------------

    //   quickshell -p ~/.config/shojiwm/shell ipc call appearance setDark true
    IpcHandler {
        target: "appearance"

        function setDark(on: bool): void { root.setDark(on); }
        function toggle(): void { root.setDark(!root.dark); }
        function isDark(): bool { return root.dark; }
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
