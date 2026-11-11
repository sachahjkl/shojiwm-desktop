import QtQuick
import Quickshell
import ".."

// The speaker of the notification area: its waves show the level, a red
// mark shows a muted output. A click opens the volume panel, the wheel
// changes the volume, a middle click mutes, and a right click opens its
// menu (Taskbar.qml).
Item {
    id: icon

    /// The volume panel is open, so the speaker stays lit.
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
        anchors.centerIn: parent
        width: Scheme.theme.trayIconSize + 2
        height: Scheme.theme.trayIconSize + 2
        sourceSize: Qt.size(width * 2, height * 2)
        source: Qt.resolvedUrl("../assets/speaker-" + Audio.level + ".svg")
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        // The tooltip waits for the next rest of the pointer.
        onContainsMouseChanged: if (!containsMouse) tipTimer.shown = false
        acceptedButtons: Qt.LeftButton | Qt.MiddleButton | Qt.RightButton
        onClicked: function (event) {
            if (event.button === Qt.MiddleButton)
                Audio.toggleMute();
            else if (event.button === Qt.RightButton)
                icon.menuRequested(Qt.rect(icon.mapToItem(null, 0, 0).x, icon.mapToItem(null, 0, 0).y, icon.width, icon.height));
            else
                icon.clicked();
        }
        // One notch is 2 %, as Windows 7 steps it.
        onWheel: function (event) {
            Audio.step(event.angleDelta.y / 120 * 2);
        }
    }

    // The tooltip of Windows 7: the device and its volume.
    PopupWindow {
        anchor.item: icon
        anchor.rect: Qt.rect(0, 0, icon.width, icon.height)
        anchor.edges: Scheme.awayFromBar
        anchor.gravity: Scheme.awayFromBar
        anchor.margins.bottom: Scheme.popupMargin(Scheme.theme.popupGap + 2)
        visible: tipTimer.shown && area.containsMouse && !icon.active
        color: "transparent"
        implicitWidth: tip.implicitWidth + 14
        implicitHeight: tip.implicitHeight + 8

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
            text: Audio.deviceName + ": " + (Audio.muted ? "Muted" : Audio.percent + "%")
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
}
