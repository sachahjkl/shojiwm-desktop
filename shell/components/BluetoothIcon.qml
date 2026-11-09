import QtQuick
import Quickshell.Bluetooth
import ".."
import "../theme.js" as Theme

// The Bluetooth icon of the notification area: a rune on blue glass, grey
// while Bluetooth is off. It shows only when there is an adapter. A click
// opens the Bluetooth panel.
Item {
    id: icon

    /// The Bluetooth panel is open, so the icon stays lit.
    property bool active: false
    readonly property bool hovered: area.containsMouse
    signal clicked

    readonly property var adapter: Bluetooth.defaultAdapter

    visible: icon.adapter !== null
    implicitWidth: visible ? Theme.trayIconSize + 8 : 0
    implicitHeight: Theme.taskbarHeight - Theme.taskbarPadding * 2

    Rectangle {
        anchors.fill: parent
        anchors.topMargin: 2
        anchors.bottomMargin: 2
        radius: Scheme.itemRadius
        color: "#ffffff"
        opacity: icon.active ? 0.22 : area.containsMouse ? 0.14 : 0
        Behavior on opacity {
            NumberAnimation { duration: Settings.motion(Theme.fade) }
        }
    }

    Image {
        anchors.centerIn: parent
        width: Theme.trayIconSize
        height: Theme.trayIconSize
        sourceSize: Qt.size(width * 2, height * 2)
        source: Qt.resolvedUrl("../assets/bluetooth"
            + (icon.adapter && icon.adapter.enabled ? "" : "-off") + ".svg")
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onClicked: icon.clicked()
    }
}
