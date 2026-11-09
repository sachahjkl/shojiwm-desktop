pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import ".."
import "../theme.js" as Theme

// The menu of the Start orb, on a right click or Super+X: the tools a power
// user wants at hand, as Windows' Win+X menu has them. A terminal, a root
// one, the task manager, the Control Panel, the files, the system's pages,
// the network and disk tools, the system log, the launcher, the power
// actions and the desktop.
PopupWindow {
    id: popup

    property bool open: false
    property var anchorWindow: null
    property rect anchorRect: Qt.rect(0, 0, 1, 1)

    signal dismissed
    signal launcherRequested

    color: "transparent"
    visible: popup.open

    anchor.window: popup.anchorWindow
    anchor.rect: popup.anchorRect
    anchor.edges: Edges.Top
    anchor.gravity: Edges.Top
    anchor.margins.bottom: Theme.popupGap
    anchor.adjustment: PopupAdjustment.SlideX | PopupAdjustment.FlipY

    /// A command line run by a shell, as the terminal and the task manager
    /// are given.
    function shell(command) {
        popup.dismissed();
        Quickshell.execDetached(["sh", "-c", command]);
    }

    /// In the terminal: `-e` runs a command in kitty, Ghostty, foot and
    /// Alacritty alike.
    function inTerminal(command) {
        popup.shell(Apps.terminal + " -e " + command);
    }

    function page(name) {
        popup.dismissed();
        ControlPanelState.open(name);
    }

    readonly property var entries: [
        { label: "Terminal", run: function () { popup.shell(Apps.terminal); } },
        { label: "Terminal (Administrator)", run: function () { popup.inTerminal("sudo -i"); } },
        { separator: true },
        { label: "Task Manager", run: function () { popup.shell(Apps.taskManager); } },
        { label: "Control Panel", run: function () { popup.page(""); } },
        { label: "File Explorer", run: function () {
            popup.dismissed();
            Apps.openPath(Apps.home);
        } },
        { label: "System", run: function () { popup.page("about"); } },
        { label: "Devices and Printers", run: function () { popup.page("devices"); } },
        { label: "Network Connections", run: function () { popup.shell("nm-connection-editor"); } },
        { label: "Disk Management", run: function () { popup.shell("gparted"); } },
        { label: "System Log", run: function () { popup.inTerminal("journalctl -b -e"); } },
        { separator: true },
        { label: "Search", run: function () {
            popup.dismissed();
            popup.launcherRequested();
        } },
        { label: "Run", run: function () {
            popup.dismissed();
            popup.launcherRequested();
        } },
        { separator: true },
        { label: "Lock", run: function () { popup.power("lock"); } },
        { label: "Log off", run: function () { popup.power("logoff"); } },
        { label: "Sleep", run: function () { popup.power("sleep"); } },
        { label: "Restart", run: function () { popup.power("restart"); } },
        { label: "Shut down", run: function () { popup.power("shutdown"); } },
        { separator: true },
        { label: "Desktop", run: function () {
            popup.dismissed();
            CompositorIpc.showDesktop();
        } }
    ]

    function power(id) {
        popup.dismissed();
        Apps.runPower(id);
    }

    readonly property real longestLabel: {
        var longest = 0;
        for (var i = 0; i < popup.entries.length; i++) {
            if (popup.entries[i].label)
                longest = Math.max(longest, metrics.advanceWidth(popup.entries[i].label));
        }
        return Math.ceil(longest);
    }

    implicitWidth: Math.min(Theme.popupMaxWidth,
        Math.max(Theme.popupMinWidth, popup.longestLabel + Theme.popupPadX * 2 + 24))
    implicitHeight: column.implicitHeight + Theme.popupPadY * 2 + 2

    FontMetrics {
        id: metrics
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
    }

    /// `true` while the pointer is over the menu. The taskbar closes a menu
    /// the pointer has left.
    readonly property bool pointerInside: hoverArea.hovered

    Rectangle {
        anchors.fill: parent
        radius: Scheme.popupRadius
        border.width: 1
        border.color: Scheme.popupOutline
        gradient: Gradient {
            GradientStop { position: 0.0; color: Scheme.popupTop }
            GradientStop { position: 1.0; color: Scheme.popupBottom }
        }
    }

    // The light line inside the outline, as every Windows 7 menu has.
    Rectangle {
        anchors.fill: parent
        anchors.margins: 1
        radius: Math.max(0, Scheme.popupRadius - 1)
        color: "transparent"
        border.width: 1
        border.color: Scheme.popupInnerLine
    }

    HoverHandler {
        id: hoverArea
    }

    Column {
        id: column

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.topMargin: Theme.popupPadY
        anchors.leftMargin: 1
        anchors.rightMargin: 1

        Repeater {
            model: popup.entries

            delegate: MenuItemRow {
                required property var modelData

                width: column.width
                label: modelData.label || ""
                separator: !!modelData.separator
                onActivated: modelData.run()
            }
        }
    }
}
