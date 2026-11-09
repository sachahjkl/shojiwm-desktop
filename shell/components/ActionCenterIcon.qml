import QtQuick
import ".."
import "../theme.js" as Theme

// The flag of the notification area, as Windows 7's Action Center: a red
// mark when something failed, a yellow one for a warning. A click opens its
// panel.
Item {
    id: icon

    property bool active: false
    readonly property bool hovered: area.containsMouse
    signal clicked

    implicitWidth: Theme.trayIconSize + 8
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
        id: picture
        anchors.centerIn: parent
        width: Theme.trayIconSize + 2
        height: Theme.trayIconSize + 2
        sourceSize: Qt.size(width * 2, height * 2)
        source: Qt.resolvedUrl("../assets/action-center.svg")
    }

    Image {
        anchors.right: picture.right
        anchors.bottom: picture.bottom
        anchors.rightMargin: -2
        anchors.bottomMargin: -1
        width: 10
        height: 10
        sourceSize: Qt.size(20, 20)
        visible: Maintenance.issues.length > 0
        source: Qt.resolvedUrl("../assets/network-badge-"
            + (Maintenance.hasErrors ? "error" : "warning") + ".svg")
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onClicked: icon.clicked()
    }
}
