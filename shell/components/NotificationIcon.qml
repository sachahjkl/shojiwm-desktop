import QtQuick
import ".."

// The bell of the notification area: the notifications of this session. A
// red count shows the ones not seen yet; a slashed bell shows Do not
// disturb. A click opens the notification panel, a middle click switches
// Do not disturb, a right click opens its menu (Taskbar.qml).
Item {
    id: icon

    property bool active: false
    readonly property bool hovered: area.containsMouse
    signal clicked
    /// A right click; `at` is where the icon is in its window.
    signal menuRequested(rect at)

    implicitWidth: Scheme.theme.trayIconSize + 8
    implicitHeight: Scheme.theme.taskbarHeight - Scheme.theme.taskbarPadding * 2

    Rectangle {
        anchors.fill: parent
        anchors.topMargin: 2
        anchors.bottomMargin: 2
        radius: Scheme.itemRadius
        color: "#ffffff"
        opacity: icon.active ? 0.22 : area.containsMouse ? 0.14 : 0
        Behavior on opacity {
            NumberAnimation { duration: Settings.motion(Scheme.theme.fade) }
        }
    }

    Image {
        id: picture
        anchors.centerIn: parent
        width: Scheme.theme.trayIconSize + 2
        height: Scheme.theme.trayIconSize + 2
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
            font.family: Scheme.theme.fontFamily
            font.pixelSize: 9
            font.bold: true
        }
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.MiddleButton | Qt.RightButton
        onClicked: function (event) {
            if (event.button === Qt.MiddleButton)
                NotificationHistory.setDoNotDisturb(!NotificationHistory.doNotDisturb);
            else if (event.button === Qt.RightButton)
                icon.menuRequested(Qt.rect(icon.mapToItem(null, 0, 0).x, icon.mapToItem(null, 0, 0).y, icon.width, icon.height));
            else
                icon.clicked();
        }
    }
}
