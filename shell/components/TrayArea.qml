pragma ComponentBehavior: Bound

import QtQuick
import Quickshell.Services.SystemTray
import Quickshell.Widgets
import "../theme.js" as Theme

// The notification area: one icon per status-notifier item. A left click
// activates the item, a right click asks it for its secondary action.
Row {
    id: tray

    readonly property var items: SystemTray.items ? SystemTray.items.values : []

    spacing: Theme.trayGap

    Repeater {
        model: tray.items

        delegate: Item {
            id: trayItem

            required property var modelData

            width: Theme.trayIconSize + 6
            height: Theme.trayIconSize + 6

            Rectangle {
                anchors.fill: parent
                radius: 3
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

                onClicked: function (event) {
                    if (!trayItem.modelData)
                        return;
                    if (event.button === Qt.RightButton)
                        trayItem.modelData.secondaryActivate();
                    else
                        trayItem.modelData.activate();
                }
            }
        }
    }
}
