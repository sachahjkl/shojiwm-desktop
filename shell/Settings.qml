pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import Quickshell.Services.SystemTray
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
    property alias glassBlur: adapter.glassBlur
    property alias animationSpeed: adapter.animationSpeed
    property alias displayChoices: adapter.displayChoices
    property alias extraClocks: adapter.extraClocks
    property alias showCpuMeter: adapter.showCpuMeter
    property alias slideshowFolder: adapter.slideshowFolder
    property alias slideshowMinutes: adapter.slideshowMinutes
    property alias slideshowShuffle: adapter.slideshowShuffle
    property alias jumpPins: adapter.jumpPins
    property alias desktopSystemIcons: adapter.desktopSystemIcons
    property alias glassColor: adapter.glassColor
    property alias wallpaperPosition: adapter.wallpaperPosition
    property alias glassColorStrength: adapter.glassColorStrength
    property alias cpuMeterPlace: adapter.cpuMeterPlace
    property alias showClockGadget: adapter.showClockGadget
    property alias clockGadgetPlace: adapter.clockGadgetPlace
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
    /// The MIME type of a program being dragged: its .desktop file name.
    readonly property string programDragType: "application/x-shojiwm-win7-program"

    /// Up to this many notification-area items, the ones with no choice made
    /// show on the taskbar; beyond it they wait in the popover.
    readonly property int trayShownByDefault: 3
    readonly property int trayCount: SystemTray.items ? SystemTray.items.values.length : 0

    /// Whether a notification-area item shows on the taskbar, or only in the
    /// popover of hidden icons: the choice made for it, else shown while there
    /// are few items.
    function trayVisible(id) {
        if (adapter.trayShowAll || adapter.trayShown.indexOf(String(id)) >= 0)
            return true;
        if (adapter.trayHidden.indexOf(String(id)) >= 0)
            return false;
        return trayCount <= trayShownByDefault;
    }

    /// Show the icon `id` at `index` among `order`, the ids shown now, left
    /// to right; the taskbar keeps that order.
    function placeTrayIcon(id, index, order) {
        var list = order.filter(function (other) { return other !== String(id); });
        index = Math.max(0, Math.min(list.length, index));
        list.splice(index, 0, String(id));
        adapter.trayShown = list;
        adapter.trayHidden = withMember(adapter.trayHidden, id, false);
    }

    function setTrayVisible(id, on) {
        // Hiding one icon while every icon is shown: the others stay shown,
        // now by choice, and this one goes to the popover.
        if (!on && adapter.trayShowAll) {
            var items = SystemTray.items ? SystemTray.items.values : [];
            var shown = [];
            for (var i = 0; i < items.length; i++) {
                if (items[i] && String(items[i].id) !== String(id))
                    shown.push(String(items[i].id));
            }
            adapter.trayShown = shown;
            adapter.trayShowAll = false;
        }
        adapter.trayShown = withMember(adapter.trayShown, id, on);
        adapter.trayHidden = withMember(adapter.trayHidden, id, !on);
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
            /// The blur behind the glass of the caption bars, the taskbar
            /// and the menus: 0 for none, up to 4.
            property int glassBlur: 2
            /// How fast every animation plays: 1 is normal, 2 twice as fast.
            property real animationSpeed: 1
            /// The Control Panel's mode and scale per screen, by connector:
            /// { "DP-2": { width, height, refreshRate, scale } }. Empty
            /// keeps the session's own layout (SHOJIWM_WIN7_OUTPUTS).
            property var displayChoices: ({})
            /// Up to two more clocks, as Windows 7's Additional Clocks:
            /// [{ zone: "America/New_York", label: "New York" }].
            property var extraClocks: []
            /// The CPU Meter gadget on the first screen's desktop, and where:
            /// its distance from the right and top edges.
            property bool showCpuMeter: true
            /// The desktop slide show: the folder its pictures come from
            /// ("" for none), how often the picture changes, and in which
            /// order.
            property string slideshowFolder: ""
            property int slideshowMinutes: 30
            property bool slideshowShuffle: false
            /// The files pinned to a program's jump list, by .desktop file
            /// name: { "org.gnome.TextEditor.desktop": ["/home/…/notes.txt"] }.
            property var jumpPins: ({})
            /// The system icons on the desktop beside the Recycle Bin:
            /// "computer", "userfiles", "controlpanel".
            property list<string> desktopSystemIcons: ["computer"]
            /// The colour of the taskbar's and the Start menu's glass, as
            /// Windows 7's Window Color: "" for the plain glass, else a
            /// colour laid over it at this strength (0 to 1).
            property string glassColor: ""
            /// How the wallpaper covers the screen, as Windows 7's Picture
            /// position: "fill", "fit", "stretch", "tile" or "center".
            property string wallpaperPosition: "fill"
            property real glassColorStrength: 0.35
            property var cpuMeterPlace: ({ right: 24, top: 24 })
            /// The Clock gadget, under the CPU Meter by default.
            property bool showClockGadget: true
            property var clockGadgetPlace: ({ right: 52, top: 168 })
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
            /// The ids of the icons chosen to show on the taskbar, and of the
            /// ones chosen to wait in the popover of hidden icons.
            property list<string> trayShown: []
            property list<string> trayHidden: []

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

    // ----- animations ---------------------------------------------------------

    /// The system's preference, as GNOME stores it: GTK, libadwaita and the
    /// settings portal pass it on to programs. `false` asks for no motion.
    property bool animations: true
    readonly property string animationsKey: "/org/gnome/desktop/interface/enable-animations"

    /// The length of an animation: `ms` at the chosen speed, or 0 when the
    /// system asks for none.
    function motion(ms) {
        return root.animations ? Math.round(ms / root.animationSpeed) : 0;
    }

    function setAnimations(on) {
        root.animations = on;
        Quickshell.execDetached([Tools.dconf, "write", root.animationsKey, on ? "true" : "false"]);
    }

    function parseBool(text) {
        var value = String(text).trim();
        return value === "true" ? true : value === "false" ? false : null;
    }

    Process {
        command: [Tools.dconf, "read", root.animationsKey]
        running: true
        stdout: StdioCollector {
            onStreamFinished: {
                var on = root.parseBool(text);
                if (on !== null)
                    root.animations = on;
            }
        }
    }

    // Another program can change it too: the shell follows. `dconf watch`
    // prints the key on one line and its value on the next.
    Process {
        command: [Tools.dconf, "watch", root.animationsKey]
        running: true
        stdout: SplitParser {
            onRead: function (line) {
                var on = root.parseBool(line);
                if (on !== null)
                    root.animations = on;
            }
        }
    }
}
