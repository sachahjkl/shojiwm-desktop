pragma ComponentBehavior: Bound

import QtQuick
import Quickshell.Widgets
import ".."
import "../theme.js" as Theme

// One program on the taskbar, with every window of it behind one button.
//
// Left click: show the window, or put it away when it is the one you are
// looking at. With several windows it shows the thumbnails instead.
// Right click: the jump list. Middle click: start another instance. Resting
// on it shows the thumbnails.
//
// A pinned program with no window open is the icon alone, with no plate
// until the pointer is on it. A click starts it.
Item {
    id: button

    required property var group

    readonly property var windows: group ? group.windows : []
    readonly property bool stacked: windows.length > 1
    /// Pinned, and no window of it is open.
    readonly property bool idle: windows.length === 0
    readonly property bool current: Apps.groupIsCurrent(group)
    readonly property bool hovered: area.containsMouse

    /// The thumbnails or the jump list are asked for by the taskbar, which
    /// owns one of each. `rect` is in the coordinates of the bar.
    signal previewRequested(rect at)
    signal previewCancelled
    signal menuRequested(rect at)
    /// The pointer entered or left the button.
    signal hoverChanged(bool inside)

    implicitHeight: Theme.taskButtonHeight

    /// Where the button is in its window, which is what a popup anchors to.
    function buttonRect() {
        var point = button.mapToItem(null, 0, 0);
        return Qt.rect(point.x, point.y, button.width, body.height);
    }

    // The stack: two plates behind the button, as Windows 7 draws a combined
    // button.
    Repeater {
        model: button.stacked ? 2 : 0

        delegate: Rectangle {
            required property int index

            x: 0
            y: -(index + 1) * 2
            width: button.width - (index + 1) * 3
            height: body.height
            radius: 3
            color: Theme.buttonStack
            border.width: 1
            border.color: Theme.buttonStackBorder
        }
    }

    Rectangle {
        id: body

        width: button.width
        height: Theme.taskButtonHeight
        anchors.bottom: parent.bottom
        radius: 3
        antialiasing: true
        opacity: button.idle && !button.hovered && !area.pressed ? 0 : 1

        Behavior on opacity {
            NumberAnimation { duration: Theme.fade }
        }

        border.width: 1
        border.color: button.current ? Theme.buttonActiveBorder
            : button.hovered ? Theme.buttonHoverBorder : Theme.buttonRestBorder

        readonly property color fill: area.pressed || button.current
            ? Theme.buttonActive
            : button.hovered ? Theme.buttonHover : Theme.buttonRest

        gradient: Gradient {
            GradientStop { position: 0.0; color: Theme.buttonGloss }
            GradientStop { position: 0.48; color: body.fill }
            GradientStop { position: 0.52; color: Qt.darker(body.fill, 1.15) }
            GradientStop { position: 1.0; color: body.fill }
        }

        Behavior on border.color {
            ColorAnimation { duration: Theme.fade }
        }
    }

    // The icon alone, for a pinned program that is not running.
    IconImage {
        anchors.centerIn: body
        implicitSize: Theme.pinnedIconSize
        source: button.group ? button.group.icon : ""
        visible: button.idle
    }

    Item {
        anchors.fill: body
        visible: !button.idle

        Row {
            anchors.fill: parent
            anchors.leftMargin: 6
            anchors.rightMargin: 6
            spacing: 6

            IconImage {
                anchors.verticalCenter: parent.verticalCenter
                implicitSize: 16
                source: button.group ? button.group.icon : ""
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                width: Math.max(0, body.width - 12 - 16 - 6)
                text: button.stacked
                    ? button.group.name
                    : Apps.titleOf(button.windows[0])
                color: Theme.textOnGlass
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSize
                elide: Text.ElideRight
                visible: body.width > 70
            }
        }
    }

    Timer {
        id: restTimer
        interval: Theme.thumbDelay
        onTriggered: button.previewRequested(button.buttonRect())
    }

    MouseArea {
        id: area

        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.MiddleButton

        onEntered: {
            if (!button.idle)
                restTimer.restart();
            button.hoverChanged(true);
        }
        onExited: {
            restTimer.stop();
            button.previewCancelled();
            button.hoverChanged(false);
        }

        onClicked: function (event) {
            restTimer.stop();
            if (event.button === Qt.RightButton) {
                button.menuRequested(button.buttonRect());
                return;
            }
            if (event.button === Qt.MiddleButton) {
                Apps.launch(button.group ? button.group.entry : null);
                return;
            }
            if (button.stacked) {
                button.previewRequested(button.buttonRect());
                return;
            }
            Apps.activateGroup(button.group);
        }
    }
}
