import QtQuick
import ".."
import "../theme.js" as Theme

// The bell of the notification area: the notifications of this session. A
// red count shows the ones not seen yet; a slashed bell shows Do not
// disturb. A click opens the notification panel, a middle click switches
// Do not disturb.
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
        source: Qt.resolvedUrl("../assets/" + (NotificationHistory.doNotDisturb ? "bell-off" : "bell") + ".svg")
    }

    Rectangle {
        visible: NotificationHistory.unread > 0 && !NotificationHistory.doNotDisturb
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.topMargin: 2
        height: 12
        width: Math.max(12, count.implicitWidth + 6)
        radius: 6
        border.width: 1
        border.color: "#80000000"
        gradient: Gradient {
            GradientStop { position: 0.0; color: "#f46a5e" }
            GradientStop { position: 1.0; color: "#c4271b" }
        }

        Text {
            id: count
            anchors.centerIn: parent
            text: NotificationHistory.unread > 9 ? "9+" : NotificationHistory.unread
            color: "#ffffff"
            font.family: Theme.fontFamily
            font.pixelSize: 9
            font.bold: true
        }
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.MiddleButton
        onClicked: function (event) {
            if (event.button === Qt.MiddleButton)
                NotificationHistory.setDoNotDisturb(!NotificationHistory.doNotDisturb);
            else
                icon.clicked();
        }
    }
}
