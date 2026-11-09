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
    /// The icon alone: a pinned program that is not running, or any button
    /// of the compact taskbar.
    readonly property bool iconOnly: idle || Settings.taskbarCompact
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

    // The stack: the edges of the other windows, behind the button at its
    // right, as Windows 7 draws a combined button.
    Repeater {
        model: button.stacked ? Math.min(2, button.windows.length - 1) : 0

        delegate: Rectangle {
            required property int index

            x: (index + 1) * 2
            y: body.y
            z: -1 - index
            width: body.width
            height: body.height
            radius: Scheme.itemRadius
            color: Theme.buttonStack
            border.width: 1
            border.color: Theme.buttonStackBorder
        }
    }

    // The plate: a dark outline, a light line inside it, and glass that is
    // lighter in its upper half.
    Rectangle {
        id: body

        width: button.width - Theme.stackRoom
        height: Theme.taskButtonHeight
        anchors.bottom: parent.bottom
        radius: Scheme.itemRadius
        antialiasing: true
        color: "transparent"
        border.width: 1
        border.color: Theme.buttonOutline
        opacity: button.idle && !button.hovered && !area.pressed ? 0 : 1

        Behavior on opacity {
            NumberAnimation { duration: Settings.motion(Theme.fade) }
        }

        // The looks are layers, as Windows 7 paints them: the resting glass,
        // the active plate over it, the pointer's glow over that, and the
        // pressed plate on top. Each fades in quickly and out slowly, so the
        // glow is added to an active button rather than replacing it.
        Repeater {
            model: ["Rest", "Active", "Hover", "Pressed"]

            delegate: Rectangle {
                id: plate

                required property string modelData
                readonly property bool shown: modelData === "Rest" ? true
                    : modelData === "Active" ? button.current
                    : modelData === "Hover" ? button.hovered && !area.pressed
                    : area.pressed

                anchors.fill: parent
                anchors.margins: 1
                radius: Scheme.itemRadius - 1
                antialiasing: true
                opacity: shown ? 1 : 0
                border.width: 1
                border.color: modelData === "Pressed" ? Theme.buttonRestBorder
                    : Theme["button" + modelData + "Border"]
                gradient: Gradient {
                    GradientStop { position: 0.0; color: Theme["button" + plate.modelData + "Top"] }
                    GradientStop { position: 0.46; color: Theme["button" + plate.modelData + "Upper"] }
                    GradientStop { position: 0.5; color: Theme["button" + plate.modelData + "Lower"] }
                    GradientStop { position: 1.0; color: Theme["button" + plate.modelData + "Bottom"] }
                }

                Behavior on opacity {
                    NumberAnimation {
                        duration: Settings.motion(plate.shown ? Theme.hoverIn : Theme.hoverOut)
                    }
                }
            }
        }
    }

    // The icon alone, for a pinned program that is not running.
    IconImage {
        anchors.centerIn: body
        anchors.verticalCenterOffset: area.pressed ? 1 : 0
        implicitSize: Theme.taskIconSize
        source: button.group ? button.group.icon : ""
        visible: button.iconOnly
    }

    Item {
        anchors.fill: body
        visible: !button.iconOnly

        Row {
            anchors.fill: parent
            anchors.leftMargin: 6
            anchors.rightMargin: 6
            spacing: 6

            IconImage {
                anchors.verticalCenter: parent.verticalCenter
                implicitSize: Theme.taskIconSize
                source: button.group ? button.group.icon : ""
            }

            // White, with a dark shadow a pixel down, as Windows 7 writes on
            // glass.
            Item {
                anchors.verticalCenter: parent.verticalCenter
                width: Math.max(0, body.width - 12 - Theme.taskIconSize - 6)
                height: label.implicitHeight
                visible: body.width > 70

                Text {
                    x: 0
                    y: 1
                    width: parent.width
                    text: label.text
                    font: label.font
                    elide: Text.ElideRight
                    color: "#b3000000"
                }

                Text {
                    id: label

                    width: parent.width
                    text: button.stacked
                        ? button.group.name
                        : Apps.titleOf(button.windows[0])
                    color: "#ffffff"
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSize
                    elide: Text.ElideRight
                }
            }
        }
    }

    Timer {
        id: restTimer
        interval: Theme.thumbDelay
        onTriggered: button.previewRequested(button.buttonRect())
    }

    /// The button can be dragged to another place on the taskbar: a pinned
    /// program.
    readonly property bool draggable: button.group !== null && button.group.pinned === true
        && button.group.entry !== null

    // What the pointer drags: invisible, the drag shows a picture of the
    // button and carries the program's .desktop file name.
    Item {
        id: dragProxy

        width: body.width
        height: body.height
        Drag.active: area.drag.active
        Drag.dragType: Drag.Automatic
        Drag.supportedActions: Qt.MoveAction
        Drag.hotSpot.x: width / 2
        Drag.hotSpot.y: height / 2
        Drag.keys: [Settings.programDragType]
        Drag.mimeData: ({ [Settings.programDragType]: Apps.entryFileId(button.group ? button.group.entry : null) })
        Drag.onDragFinished: {
            dragProxy.x = 0;
            dragProxy.y = 0;
        }
    }

    MouseArea {
        id: area

        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.MiddleButton
        drag.target: button.draggable ? dragProxy : null

        onPressed: function (event) {
            if (event.button === Qt.LeftButton && button.draggable)
                body.grabToImage(function (result) { dragProxy.Drag.imageSource = result.url; },
                                 Qt.size(body.width, body.height));
        }

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
