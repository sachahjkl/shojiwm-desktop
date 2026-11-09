import QtQuick
import Quickshell
import ".."
import "../theme.js" as Theme

// The speaker of the notification area: its waves show the level, a red
// mark shows a muted output. A click opens the volume panel, the wheel
// changes the volume, a middle click mutes, and a right click opens the
// mixer.
Item {
    id: icon

    /// The volume panel is open, so the speaker stays lit.
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
        anchors.centerIn: parent
        width: Theme.trayIconSize + 2
        height: Theme.trayIconSize + 2
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
                Audio.openMixer();
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
        anchor.edges: Edges.Top
        anchor.gravity: Edges.Top
        anchor.margins.bottom: Theme.popupGap + 2
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
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSizeSmall
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
