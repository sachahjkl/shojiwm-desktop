import QtQuick
import Quickshell
import ".."
import "../theme.js" as Theme

// The clock at the right end of the taskbar: the time over the date. A click
// opens the calendar, which the taskbar owns.
Item {
    id: clock

    /// The calendar is open, so the clock stays lit.
    property bool active: false
    readonly property bool hovered: area.containsMouse
    signal clicked

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
        opacity: clock.active ? 0.22 : area.containsMouse ? 0.14 : 0
        Behavior on opacity {
            NumberAnimation { duration: Theme.fade }
        }
    }

    Column {
        anchors.centerIn: parent
        spacing: 0

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: Settings.clockFormat === "24h"
                ? Qt.formatTime(source.date, "HH:mm")
                : Settings.clockFormat === "12h"
                    ? Qt.formatTime(source.date, "h:mm AP")
                    : source.date.toLocaleTimeString(Qt.locale(), Locale.ShortFormat)
            color: Theme.textOnGlass
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSize
        }

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            visible: Settings.showDate
            text: source.date.toLocaleDateString(Qt.locale(), Locale.ShortFormat)
            color: Theme.textMuted
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSizeSmall
        }
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onClicked: clock.clicked()
    }
}
