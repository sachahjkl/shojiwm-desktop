pragma ComponentBehavior: Bound

import QtQuick
import ".."

// One program on the taskbar, with every window of it behind one button.
//
// Left click: show the window, or put it away when it is the one you are
// looking at. With several windows it shows the thumbnails instead.
// Right click: the jump list. Middle click, Ctrl+click or Shift+click:
// start another instance. Resting
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
    /// What the program shows on its button: progress, a count, urgency.
    readonly property var launcherEntry: button.group
        ? LauncherEntries.of(Apps.entryFileId(button.group.entry)) : null
    readonly property real progress: button.launcherEntry ? button.launcherEntry.progress : -1
    readonly property int count: button.launcherEntry ? button.launcherEntry.count : -1
    readonly property bool urgent: !!button.launcherEntry && button.launcherEntry.urgent

    /// The thumbnails or the jump list are asked for by the taskbar, which
    /// owns one of each. `rect` is in the coordinates of the bar.
    signal previewRequested(rect at)
    signal previewCancelled
    signal menuRequested(rect at)
    /// The pointer entered or left the button.
    signal hoverChanged(bool inside)

    implicitHeight: Scheme.theme.taskButtonHeight

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
            color: Scheme.theme.buttonStack
            border.width: 1
            border.color: Scheme.theme.buttonStackBorder
        }
    }

    // The plate: a dark outline, a light line inside it, and glass that is
    // lighter in its upper half.
    Rectangle {
        id: body

        width: button.width - Scheme.theme.stackRoom
        height: Scheme.theme.taskButtonHeight
        anchors.bottom: parent.bottom
        radius: Scheme.itemRadius
        antialiasing: true
        color: "transparent"
        border.width: 1
        border.color: Scheme.theme.buttonOutline
        opacity: button.idle && !button.hovered && !area.pressed ? 0 : 1

        Behavior on opacity {
            NumberAnimation { duration: Settings.motion(Scheme.theme.fade) }
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
                radius: Scheme.corner(Scheme.itemRadius - 1)
                antialiasing: true
                opacity: shown ? 1 : 0
                border.width: 1
                border.color: modelData === "Pressed" ? Scheme.theme.buttonRestBorder
                    : Scheme.theme["button" + modelData + "Border"]
                gradient: Gradient {
                    GradientStop { position: 0.0; color: Scheme.theme["button" + plate.modelData + "Top"] }
                    GradientStop { position: 0.46; color: Scheme.theme["button" + plate.modelData + "Upper"] }
                    GradientStop { position: 0.5; color: Scheme.theme["button" + plate.modelData + "Lower"] }
                    GradientStop { position: 1.0; color: Scheme.theme["button" + plate.modelData + "Bottom"] }
                }

                Behavior on opacity {
                    NumberAnimation {
                        duration: Settings.motion(plate.shown ? Scheme.theme.hoverIn : Scheme.theme.hoverOut)
                    }
                }
            }
        }
    }

    // A program's progress fills its button green from the left, glossy in
    // the upper half, as Windows 7 shows a download or a copy. An urgent
    // program turns its button orange.
    Rectangle {
        x: body.x + 2
        y: body.y + 2
        width: Math.max(0, (body.width - 4) * (button.urgent ? 1 : button.progress))
        height: body.height - 4
        radius: Scheme.corner(Scheme.itemRadius - 1)
        visible: button.urgent || button.progress >= 0
        opacity: 0.85
        gradient: Gradient {
            GradientStop { position: 0.0; color: button.urgent ? "#ffd08a" : "#8fe08a" }
            GradientStop { position: 0.46; color: button.urgent ? "#f6a43c" : "#3fbf3a" }
            GradientStop { position: 0.5; color: button.urgent ? "#e98a12" : "#25a51f" }
            GradientStop { position: 1.0; color: button.urgent ? "#f3a640" : "#4cc443" }
        }

        Behavior on width {
            NumberAnimation { duration: Settings.motion(Scheme.theme.fade * 2) }
        }
    }

    // The icon alone, for a pinned program that is not running.
    ShellIcon {
        anchors.centerIn: body
        anchors.verticalCenterOffset: area.pressed ? 1 : 0
        implicitSize: Scheme.theme.taskIconSize
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

            ShellIcon {
                anchors.verticalCenter: parent.verticalCenter
                implicitSize: Scheme.theme.taskIconSize
                source: button.group ? button.group.icon : ""
            }

            // White, with a dark shadow a pixel down, as Windows 7 writes on
            // glass.
            Item {
                anchors.verticalCenter: parent.verticalCenter
                width: Math.max(0, body.width - 12 - Scheme.theme.taskIconSize - 6)
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
                    font.family: Scheme.theme.fontFamily
                    font.pixelSize: Scheme.theme.fontSize
                    elide: Text.ElideRight
                }
            }
        }
    }

    Timer {
        id: restTimer
        interval: Scheme.theme.thumbDelay
        onTriggered: button.previewRequested(button.buttonRect())
    }

    /// How far the button stands aside while another one is dragged over the
    /// taskbar.
    // A count (unread mail, messages) in a small red badge on the icon's
    // upper right corner.
    Rectangle {
        visible: button.count > 0
        x: body.x + (button.iconOnly ? body.width / 2 + 2 : 6 + Scheme.theme.taskIconSize - 8)
        y: body.y + 2
        z: 2
        height: 14
        width: Math.max(14, badgeText.implicitWidth + 8)
        radius: Scheme.corner(7)
        border.width: 1
        border.color: "#80000000"
        gradient: Gradient {
            GradientStop { position: 0.0; color: "#f46a5e" }
            GradientStop { position: 1.0; color: "#c4271b" }
        }

        Text {
            id: badgeText
            anchors.centerIn: parent
            text: button.count > 99 ? "99+" : button.count
            color: "#ffffff"
            font.family: Scheme.theme.fontFamily
            font.pixelSize: 10
            font.bold: true
        }
    }

    property real shift: 0
    /// How far the button may be dragged left and right: to the ends of the
    /// row of buttons. The taskbar sets both.
    property real dragMin: 0
    property real dragMax: 0

    /// The button is being dragged along the taskbar, this far from its place.
    property bool reordering: false
    property real dragOffset: 0
    property real pressX: 0
    /// `centre` is where the middle of the button is in the row.
    signal reorderMoved(real centre)
    signal reorderEnded

    z: button.reordering ? 10 : 0
    transform: Translate {
        x: button.reordering ? button.dragOffset : button.shift
        Behavior on x {
            enabled: !button.reordering
            NumberAnimation { duration: Settings.motion(Scheme.theme.hoverIn); easing.type: Easing.OutCubic }
        }
    }

    MouseArea {
        id: area

        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.MiddleButton

        // A drag along the taskbar moves the button; it stays on the bar.
        onPressed: function (event) {
            button.pressX = area.mapToItem(null, event.x, 0).x;
        }
        onPositionChanged: function (event) {
            if (!area.pressed || !(area.pressedButtons & Qt.LeftButton))
                return;
            var dx = area.mapToItem(null, event.x, 0).x - button.pressX;
            if (!button.reordering && Math.abs(dx) < 6)
                return;
            if (!button.reordering) {
                restTimer.stop();
                button.previewCancelled();
                button.reordering = true;
            }
            button.dragOffset = Math.max(button.dragMin, Math.min(button.dragMax, dx));
            button.reorderMoved(button.x + button.width / 2 + button.dragOffset);
        }
        onReleased: {
            if (!button.reordering)
                return;
            button.reorderEnded();
            button.reordering = false;
            button.dragOffset = 0;
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
            // The release that ends a drag along the bar is not a click.
            if (Math.abs(area.mapToItem(null, event.x, 0).x - button.pressX) >= 6)
                return;
            if (event.button === Qt.RightButton) {
                button.menuRequested(button.buttonRect());
                return;
            }
            // A middle click, Ctrl+click or Shift+click starts another
            // instance.
            if (event.button === Qt.MiddleButton
                || (event.modifiers & (Qt.ControlModifier | Qt.ShiftModifier))) {
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
