pragma ComponentBehavior: Bound

import QtQuick
import Quickshell.Services.SystemTray
import Quickshell.Widgets
import ".."
import "../theme.js" as Theme

// The notification area: one icon per status-notifier item. A left click
// activates the item, a right click opens its menu. An item that offers
// nothing but a menu opens it on either button.
Row {
    id: tray

    readonly property var items: SystemTray.items ? SystemTray.items.values : []

    /// The taskbar owns the menu, so it is told which item and where.
    signal menuRequested(var item, rect at)
    /// The pointer entered or left one of the icons.
    signal hoverChanged(bool inside)

    spacing: Theme.trayGap

    Repeater {
        model: tray.items

        delegate: Item {
            id: trayItem

            required property var modelData

            width: Theme.trayIconSize + 6
            height: Theme.trayIconSize + 6

            function iconRect() {
                var point = trayItem.mapToItem(null, 0, 0);
                return Qt.rect(point.x, point.y, trayItem.width, trayItem.height);
            }

            Rectangle {
                anchors.fill: parent
                radius: Scheme.itemRadius
                color: "#ffffff"
                opacity: area.containsMouse ? 0.18 : 0
                Behavior on opacity {
                    NumberAnimation { duration: Theme.fade }
                }
            }

            IconImage {
                anchors.centerIn: parent
                implicitSize: Theme.trayIconSize
                source: trayItem.modelData && trayItem.modelData.icon
                    ? trayItem.modelData.icon : ""
            }

            MouseArea {
                id: area
                anchors.fill: parent
                hoverEnabled: true
                acceptedButtons: Qt.LeftButton | Qt.RightButton

                onEntered: tray.hoverChanged(true)
                onExited: tray.hoverChanged(false)

                onClicked: function (event) {
                    var item = trayItem.modelData;
                    if (!item)
                        return;
                    var wantsMenu = event.button === Qt.RightButton
                        || item.onlyMenu;
                    if (wantsMenu) {
                        if (item.hasMenu)
                            tray.menuRequested(item, trayItem.iconRect());
                        else if (!item.onlyMenu)
                            item.secondaryActivate();
                        return;
                    }
                    item.activate();
                }
            }
        }
    }
}
