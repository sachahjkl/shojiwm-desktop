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
        radius: Scheme.itemRadius
        color: "#ffffff"
        opacity: clock.active ? 0.22 : area.containsMouse ? 0.14 : 0
        Behavior on opacity {
            NumberAnimation { duration: Settings.motion(Theme.fade) }
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
            // The date in full, day first, as Windows 7 shows it; the
            // locale's own form when it has one.
            text: Qt.locale().name === "C"
                ? Qt.formatDate(source.date, "dd/MM/yyyy")
                : Qt.formatDate(source.date, Qt.locale().dateFormat(Locale.ShortFormat)
                    .replace(/\bd\b/, "dd").replace(/\bM\b/, "MM")
                    .replace(/\byy\b/, "yyyy").replace(/ /g, "/"))
            color: Theme.textOnGlass
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSize
        }
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onClicked: clock.clicked()
    }
}
