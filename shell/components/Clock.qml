import QtQuick
import Quickshell
import ".."

// The clock at the right end of the taskbar: the time over the date. A click
// opens the calendar, a right click the clock's menu; the taskbar owns both.
Item {
    id: clock

    /// The calendar is open, so the clock stays lit.
    property bool active: false
    readonly property bool hovered: area.containsMouse
    signal clicked
    /// A right click; `at` is where the clock is in its window.
    signal menuRequested(rect at)

    implicitWidth: Scheme.theme.clockWidth
    implicitHeight: Scheme.theme.taskbarHeight - Scheme.theme.taskbarPadding * 2

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
            NumberAnimation { duration: Settings.motion(Scheme.theme.fade) }
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
            color: Scheme.theme.textOnGlass
            font.family: Scheme.theme.fontFamily
            font.pixelSize: Scheme.theme.fontSize
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
            color: Scheme.theme.textOnGlass
            font.family: Scheme.theme.fontFamily
            font.pixelSize: Scheme.theme.fontSize
        }
    }

    // The tooltip of Windows 7: the long date, then the additional clocks.
    PopupWindow {
        anchor.item: clock
        anchor.rect: Qt.rect(0, 0, clock.width, clock.height)
        anchor.edges: Scheme.awayFromBar
        anchor.gravity: Scheme.awayFromBar
        anchor.margins.bottom: Scheme.popupMargin(Scheme.theme.popupGap + 2)
        visible: tipTimer.shown && area.containsMouse && !clock.active
        color: "transparent"
        implicitWidth: tip.implicitWidth + 16
        implicitHeight: tip.implicitHeight + 10

        Rectangle {
            anchors.fill: parent
            radius: Scheme.itemRadius
            border.width: 1
            border.color: Scheme.popupOutline
            gradient: Gradient {
                GradientStop { position: 0.0; color: Scheme.popupTop }
                GradientStop { position: 1.0; color: Scheme.popupBottom }
            }
        }

        Text {
            id: tip
            anchors.centerIn: parent
            text: [source.date.toLocaleDateString(Qt.locale(), Locale.LongFormat)].concat(
                WorldClocks.clocks.map(function (other) {
                    return other.label + ": " + other.time
                        + (other.day === "Today" ? "" : " (" + other.day.toLowerCase() + ")");
                })).join("\n")
            color: Scheme.text
            font.family: Scheme.theme.fontFamily
            font.pixelSize: Scheme.theme.fontSizeSmall
        }
    }

    Timer {
        id: tipTimer
        property bool shown: false
        interval: 600
        running: area.containsMouse && !shown
        onTriggered: shown = true
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        // The tooltip waits for the next rest of the pointer.
        onContainsMouseChanged: if (!containsMouse) tipTimer.shown = false
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: function (event) {
            if (event.button === Qt.RightButton) {
                var point = clock.mapToItem(null, 0, 0);
                clock.menuRequested(Qt.rect(point.x, point.y, clock.width, clock.height));
            } else {
                clock.clicked();
            }
        }
    }
}
