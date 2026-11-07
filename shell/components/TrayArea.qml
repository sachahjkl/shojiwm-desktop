pragma ComponentBehavior: Bound

import QtQuick
import Quickshell.Services.SystemTray
import ".."
import "../theme.js" as Theme

// The notification area, as Windows 7 has it: the icons chosen to show on
// the taskbar, and left of them the arrow that opens the popover of the
// hidden ones. Which is which is set in the Control Panel; a new icon is
// hidden.
Row {
    id: tray

    readonly property var items: SystemTray.items ? SystemTray.items.values : []
    readonly property var shown: items.filter(function (item) {
        return item && Settings.trayVisible(item.id);
    })
    readonly property var hidden: items.filter(function (item) {
        return item && !Settings.trayVisible(item.id);
    })

    /// Whether the popover of hidden icons is open, so the arrow turns.
    property bool overflowOpen: false

    /// The taskbar owns the menus and the popover, so it is told which item
    /// and where.
    signal menuRequested(var item, rect at)
    signal overflowRequested(rect at)
    /// The pointer entered or left one of the icons or the arrow.
    signal hoverChanged(bool inside)

    spacing: 2

    // "Show hidden icons".
    Item {
        id: arrow

        visible: tray.hidden.length > 0
        width: 18
        height: Theme.trayIconSize + 8
        anchors.verticalCenter: parent.verticalCenter

        function arrowRect() {
            var point = arrow.mapToItem(null, 0, 0);
            return Qt.rect(point.x, point.y, arrow.width, arrow.height);
        }

        Rectangle {
            anchors.fill: parent
            radius: Scheme.itemRadius
            color: "#2effffff"
            opacity: arrowArea.containsMouse || tray.overflowOpen ? 1 : 0
            Behavior on opacity {
                NumberAnimation { duration: Theme.fade }
            }
        }

        Text {
            anchors.centerIn: parent
            text: tray.overflowOpen ? "▼" : "▲"
            color: Theme.textOnGlass
            font.pixelSize: 8
        }

        MouseArea {
            id: arrowArea
            anchors.fill: parent
            hoverEnabled: true
            onEntered: tray.hoverChanged(true)
            onExited: tray.hoverChanged(false)
            onClicked: tray.overflowRequested(arrow.arrowRect())
        }
    }

    Repeater {
        model: tray.shown

        delegate: TrayIcon {
            required property var modelData

            anchors.verticalCenter: parent ? parent.verticalCenter : undefined
            item: modelData
            onMenuRequested: function (item, at) { tray.menuRequested(item, at); }
            onHoverChanged: function (inside) { tray.hoverChanged(inside); }
        }
    }
}
