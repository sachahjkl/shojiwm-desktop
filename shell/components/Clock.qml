import QtQuick
import Quickshell
import "../theme.js" as Theme

// The clock at the right end of the taskbar: the time over the date.
Item {
    id: clock

    implicitWidth: Theme.clockWidth
    implicitHeight: Theme.taskbarHeight - Theme.taskbarPadding * 2

    SystemClock {
        id: source
        precision: SystemClock.Minutes
    }

    Rectangle {
        anchors.fill: parent
        radius: 3
        color: "#ffffff"
        opacity: area.containsMouse ? 0.14 : 0
        Behavior on opacity {
            NumberAnimation { duration: Theme.fade }
        }
    }

    Column {
        anchors.centerIn: parent
        spacing: 0

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: Qt.formatDateTime(source.date, "HH:mm")
            color: Theme.textOnGlass
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSize
        }

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: Qt.formatDateTime(source.date, "dd/MM/yyyy")
            color: Theme.textMuted
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSizeSmall
        }
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
    }
}
