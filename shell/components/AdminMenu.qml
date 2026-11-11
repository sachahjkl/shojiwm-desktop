pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import ".."

// The menu of the Start orb, on a right click or Super+X: the tools a power
// user wants at hand, as Windows' Win+X menu has them. A terminal, a root
// one, the task manager, the Control Panel, the files, the system's pages,
// the network and disk tools, the system log, Search and Run, the power
// actions and the desktop.
//
// It fills its screen's window (shell.qml), with the keyboard: Up and Down
// pick an entry, Return starts it, Escape or a click outside closes it.
Item {
    id: popup

    signal dismissed
    signal launcherRequested
    signal runRequested

    /// The entry the keys picked, as an index into `entries`; -1 for none.
    property int pick: -1

    /// Called each time it opens.
    function reset() {
        popup.pick = -1;
        keys.forceActiveFocus();
    }

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
            popup.runRequested();
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

    /// The next entry from `from` in the direction `by`, past separators.
    function step(from, by) {
        var count = popup.entries.length;
        var at = from;
        for (var i = 0; i < count; i++) {
            at = (at + by + count) % count;
            if (!popup.entries[at].separator)
                return at;
        }
        return from;
    }

    readonly property real longestLabel: {
        var longest = 0;
        for (var i = 0; i < popup.entries.length; i++) {
            if (popup.entries[i].label)
                longest = Math.max(longest, metrics.advanceWidth(popup.entries[i].label));
        }
        return Math.ceil(longest);
    }

    FontMetrics {
        id: metrics
        font.family: Scheme.theme.fontFamily
        font.pixelSize: Scheme.theme.fontSize
    }

    Item {
        id: keys

        focus: true
        Keys.onUpPressed: popup.pick = popup.step(popup.pick < 0 ? 0 : popup.pick, -1)
        Keys.onDownPressed: popup.pick = popup.step(popup.pick, 1)
        Keys.onReturnPressed: {
            if (popup.pick >= 0)
                popup.entries[popup.pick].run();
        }
        Keys.onEnterPressed: {
            if (popup.pick >= 0)
                popup.entries[popup.pick].run();
        }
        Keys.onEscapePressed: popup.dismissed()
    }

    // A click outside closes it.
    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.MiddleButton
        onPressed: popup.dismissed()
    }

    // The menu, above the Start orb.
    Rectangle {
        id: menu

        x: Scheme.theme.orbLeft
        y: parent.height - Scheme.theme.taskbarHeight - height - Scheme.theme.popupGap
        width: Math.min(Scheme.theme.popupMaxWidth,
            Math.max(Scheme.theme.popupMinWidth, popup.longestLabel + Scheme.theme.popupPadX * 2 + 24))
        height: column.implicitHeight + Scheme.theme.popupPadY * 2 + 2
        radius: Scheme.popupRadius
        border.width: 1
        border.color: Scheme.popupOutline
        gradient: Gradient {
            GradientStop { position: 0.0; color: Scheme.popupTop }
            GradientStop { position: 1.0; color: Scheme.popupBottom }
        }

        // A click on the menu stays here.
        MouseArea {
            anchors.fill: parent
        }

        // The light line inside the outline, as every Windows 7 menu has.
        Rectangle {
            anchors.fill: parent
            anchors.margins: 1
            radius: Scheme.corner(Math.max(0, Scheme.popupRadius - 1))
            color: "transparent"
            border.width: 1
            border.color: Scheme.popupInnerLine
        }

        Column {
            id: column

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.topMargin: Scheme.theme.popupPadY
            anchors.leftMargin: 1
            anchors.rightMargin: 1

            Repeater {
                model: popup.entries

                delegate: MenuItemRow {
                    required property var modelData
                    required property int index

                    width: column.width
                    label: modelData.label || ""
                    separator: !!modelData.separator
                    highlighted: index === popup.pick
                    onActivated: modelData.run()
                }
            }
        }
    }
}
