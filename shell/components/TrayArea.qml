pragma ComponentBehavior: Bound

import QtQuick
import Quickshell.Services.SystemTray
import ".."
import "../theme.js" as Theme

// The notification area, as Windows 7 has it: the icons chosen to show on
// the taskbar, and left of them the arrow that opens the popover of the
// hidden ones. Which is which is set in the Control Panel, or by dragging an
// icon: anywhere onto the taskbar to show it, onto the arrow, into the
// popover or off the taskbar to hide it. An icon with no choice made shows while there are three icons or
// fewer, and waits in the popover beyond.
Item {
    id: tray

    readonly property var items: SystemTray.items ? SystemTray.items.values : []
    /// The icons on the taskbar, left to right: in the order they were put
    /// there, then the ones shown by default.
    readonly property var shown: {
        var order = Settings.trayShown;
        return items.filter(function (item) {
            return item && Settings.trayVisible(item.id);
        }).map(function (item, natural) {
            var at = order.indexOf(String(item.id));
            return { item: item, rank: at >= 0 ? at : order.length + natural };
        }).sort(function (a, b) {
            return a.rank - b.rank;
        }).map(function (entry) {
            return entry.item;
        });
    }

    /// Where among the shown icons a drop at `x` (in this item) goes.
    function indexAt(x) {
        for (var i = 0; i < iconRepeater.count; i++) {
            var icon = iconRepeater.itemAt(i);
            if (icon && x < icon.x + icon.width / 2)
                return i;
        }
        return iconRepeater.count;
    }

    /// Show `id` at the place a drop at `x` (in this item) points to.
    function placeAt(id, x) {
        var order = tray.shown.map(function (item) { return String(item.id); });
        Settings.placeTrayIcon(id, tray.indexAt(x), order);
    }
    readonly property var hidden: items.filter(function (item) {
        return item && !Settings.trayVisible(item.id);
    })

    /// Whether the popover of hidden icons is open, so the arrow turns.
    property bool overflowOpen: false
    /// One of the icons here is being dragged.
    property bool dragging: false

    /// The taskbar owns the menus and the popover, so it is told which item
    /// and where.
    signal menuRequested(var item, rect at)
    signal overflowRequested(rect at)
    /// The pointer entered or left one of the icons or the arrow.
    signal hoverChanged(bool inside)

    implicitWidth: row.implicitWidth
    implicitHeight: row.implicitHeight

    Row {
        id: row

        spacing: 2

        // "Show hidden icons".
        Item {
            id: arrow

            // Kept while an icon is dragged, so there is somewhere to drop
            // the last shown one.
            visible: tray.hidden.length > 0 || tray.dragging || arrowDrop.containsDrag
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
                opacity: arrowArea.containsMouse || tray.overflowOpen || arrowDrop.containsDrag ? 1 : 0
                Behavior on opacity {
                    NumberAnimation { duration: Settings.motion(Theme.fade) }
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

            // An icon dropped on the arrow goes to the popover. The taskbar
            // takes a drop anywhere else and shows the icon.
            DropArea {
                id: arrowDrop

                anchors.fill: parent
                keys: [Settings.trayDragType]

                onDropped: function (event) {
                    var id = String(event.getDataAsString(Settings.trayDragType));
                    if (id.length === 0)
                        return;
                    event.acceptProposedAction();
                    // After the drag has ended: the change can remove its source.
                    Qt.callLater(function () { Settings.setTrayVisible(id, false); });
                }
            }
        }

        Repeater {
            id: iconRepeater

            model: tray.shown

            delegate: TrayIcon {
                required property var modelData

                anchors.verticalCenter: parent ? parent.verticalCenter : undefined
                item: modelData
                inTaskbar: true
                onMenuRequested: function (item, at) { tray.menuRequested(item, at); }
                onHoverChanged: function (inside) { tray.hoverChanged(inside); }
                onDraggingChanged: tray.dragging = dragging
            }
        }
    }
}
