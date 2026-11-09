pragma ComponentBehavior: Bound

import QtQuick
import ".."
import "../theme.js" as Theme

// "Shut Down Windows", as Windows 7 shows it after Alt+F4 on the desktop: the
// screen dims, and a small window asks what the computer should do, from a
// list of the power actions. Return confirms, Escape cancels, Up and Down
// change the choice.
Item {
    id: dialog

    signal dismissed

    /// The power actions, without the separator; the one the dialog selects
    /// first (Apps.shutdownDialogAction).
    readonly property var actions: {
        var list = Apps.powerActions.filter(function (action) {
            return action.id !== "separator";
        });
        list.sort(function (a, b) {
            return (a.id === Apps.shutdownDialogAction ? 0 : 1) - (b.id === Apps.shutdownDialogAction ? 0 : 1);
        });
        return list;
    }

    readonly property var descriptions: ({
        "shutdown": "Closes all open programs and turns off the computer.",
        "restart": "Closes all open programs, turns off the computer, and then turns it on again.",
        "sleep": "Keeps your session in memory and puts the computer in a low-power state.",
        "hibernate": "Saves your session to the disk and turns off the computer.",
        "logoff": "Closes all open programs and logs you off.",
        "lock": "Locks the computer. Your programs keep running."
    })

    /// The index of the chosen action.
    property int current: 0
    /// The list of actions is open under the field.
    property bool listOpen: false
    /// What has the keyboard: 0 the list, 1 OK, 2 Cancel. Tab moves it.
    property int focusAt: 0

    /// Cleared each time the dialog opens.
    function reset() {
        dialog.current = 0;
        dialog.listOpen = false;
        dialog.focusAt = 0;
        keys.forceActiveFocus();
    }

    function accept() {
        var action = dialog.actions[dialog.current];
        dialog.dismissed();
        if (action)
            Apps.runPower(action.id);
    }

    /// Return: picks in the open list, else does what has the keyboard:
    /// the list and OK confirm, Cancel closes.
    function confirm() {
        if (dialog.listOpen)
            dialog.listOpen = false;
        else if (dialog.focusAt === 2)
            dialog.dismissed();
        else
            dialog.accept();
    }

    function tab(by) {
        dialog.listOpen = false;
        dialog.focusAt = (dialog.focusAt + by + 3) % 3;
    }

    function move(by) {
        dialog.current = Math.max(0, Math.min(dialog.actions.length - 1, dialog.current + by));
    }

    // The keys: Tab and Shift+Tab move between the list, OK and Cancel.
    // On the list, Up and Down change the choice, and Alt+Down, F4 and Space
    // open it, as in a Windows combo box. Return does what has the keyboard
    // (or picks in the open list); Space presses a button. Escape closes the
    // list, then the dialog.
    Item {
        id: keys
        focus: true
        Keys.onTabPressed: dialog.tab(1)
        Keys.onBacktabPressed: dialog.tab(-1)
        Keys.onUpPressed: {
            if (dialog.focusAt === 0)
                dialog.move(-1);
        }
        Keys.onDownPressed: function (event) {
            if (dialog.focusAt !== 0)
                return;
            if (event.modifiers & Qt.AltModifier)
                dialog.listOpen = !dialog.listOpen;
            else
                dialog.move(1);
        }
        Keys.onReturnPressed: dialog.confirm()
        Keys.onEnterPressed: dialog.confirm()
        Keys.onSpacePressed: {
            if (dialog.focusAt === 0)
                dialog.listOpen = !dialog.listOpen;
            else
                dialog.confirm();
        }
        Keys.onPressed: function (event) {
            if (dialog.focusAt === 0 && event.key === Qt.Key_F4 && !(event.modifiers & Qt.AltModifier)) {
                dialog.listOpen = !dialog.listOpen;
                event.accepted = true;
            }
        }
        Keys.onEscapePressed: {
            if (dialog.listOpen)
                dialog.listOpen = false;
            else
                dialog.dismissed();
        }
    }

    // The dimmed screen. The compositor blurs what is under it.
    Rectangle {
        anchors.fill: parent
        color: "#73000000"

        MouseArea {
            anchors.fill: parent
            onPressed: dialog.dismissed()
        }
    }

    // The window: a glass frame round the page, as Windows 7 draws a
    // dialog: the caption on the glass, a banner, the question, and the
    // buttons on a band at the bottom.
    Rectangle {
        id: frame

        anchors.centerIn: parent
        width: 440
        height: page.height + caption.height + 9
        radius: Scheme.corner(7)
        border.width: 1
        border.color: "#b3000000"
        gradient: Gradient {
            GradientStop { position: 0.0; color: Scheme.dark ? "#f0566f88" : "#e6b9d1ea" }
            GradientStop { position: 1.0; color: Scheme.dark ? "#f0324a60" : "#e68fb3d6" }
        }

        // The light line inside the outline.
        Rectangle {
            anchors.fill: parent
            anchors.margins: 1
            radius: Scheme.corner(6)
            color: "transparent"
            border.width: 1
            border.color: "#59ffffff"
        }

        // Swallow clicks, so a click in the window does not close it, and
        // close the list.
        MouseArea {
            anchors.fill: parent
            onPressed: dialog.listOpen = false
        }

        Text {
            id: caption

            x: 10
            height: 28
            verticalAlignment: Text.AlignVCenter
            text: "Shut Down Windows"
            color: Scheme.dark ? "#ffffff" : "#1e395b"
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSize
        }

        Rectangle {
            id: page

            x: 8
            y: caption.height
            width: parent.width - 16
            height: banner.height + content.implicitHeight + footer.height + 28
            color: Scheme.page
            border.width: 1
            border.color: "#80000000"

            // The banner: deep blue glass with a light sweep, your picture and
            // your name.
            Rectangle {
                id: banner

                x: 1
                y: 1
                width: parent.width - 2
                height: 64
                // The sweep is wider than the banner.
                clip: true
                gradient: Gradient {
                    GradientStop { position: 0.0; color: "#1c4f86" }
                    GradientStop { position: 0.55; color: "#0f3768" }
                    GradientStop { position: 1.0; color: "#0a2850" }
                }

                // The sweep of light across the banner.
                Rectangle {
                    x: -40
                    y: 30
                    width: parent.width * 1.4
                    height: 90
                    radius: 45
                    rotation: -6
                    // Soft at both edges, so the sweep has no hard line.
                    gradient: Gradient {
                        GradientStop { position: 0.0; color: "#007fc0ff" }
                        GradientStop { position: 0.12; color: "#407fc0ff" }
                        GradientStop { position: 0.55; color: "#00ffffff" }
                    }
                }

                // Your account picture, in the frame of the Start menu's.
                UserTile {
                    anchors.verticalCenter: parent.verticalCenter
                    x: 16
                    width: 48
                    height: 48
                    source: Settings.userPictureSource
                    dark: true
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    x: 76
                    text: Apps.userName.length > 0 ? Apps.userName : "Shut Down"
                    color: "#ffffff"
                    style: Text.Raised
                    styleColor: "#40000000"
                    font.family: Theme.fontFamily
                    font.pixelSize: 22
                    font.weight: Font.Light
                }
            }

            Column {
                id: content

                x: 20
                y: banner.height + 16
                width: parent.width - 40
                spacing: 10

                Text {
                    text: "What do you want the computer to do?"
                    color: Scheme.pageHeading
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSize + 2
                }

                // The combo box: the choice in a field, and the drop button
                // at its right end.
                Rectangle {
                    id: field

                    width: parent.width
                    height: 24
                    radius: Scheme.corner(2)
                    color: Scheme.field
                    border.width: 1
                    border.color: fieldArea.containsMouse || dialog.listOpen
                        ? Scheme.fieldFocus : Scheme.fieldBorder

                    // The choice, selected as Windows shows a focused combo.
                    Rectangle {
                        x: 3
                        y: 3
                        width: parent.width - dropButton.width - 8
                        height: parent.height - 6
                        radius: Scheme.corner(1)
                        color: dialog.listOpen || dialog.focusAt !== 0 ? "transparent" : "#3399ff"

                        Text {
                            anchors.left: parent.left
                            anchors.leftMargin: 4
                            anchors.verticalCenter: parent.verticalCenter
                            text: {
                                var action = dialog.actions[dialog.current];
                                return action ? action.label : "";
                            }
                            color: dialog.listOpen || dialog.focusAt !== 0 ? Scheme.text : "#ffffff"
                            font.family: Theme.fontFamily
                            font.pixelSize: Theme.fontSize
                        }
                    }

                    Rectangle {
                        id: dropButton

                        anchors.right: parent.right
                        anchors.top: parent.top
                        anchors.bottom: parent.bottom
                        anchors.margins: 1
                        width: 18
                        border.width: fieldArea.containsMouse || dialog.listOpen ? 1 : 0
                        border.color: Scheme.selectionBorder
                        gradient: Gradient {
                            GradientStop {
                                position: 0.0
                                color: dialog.listOpen ? Scheme.buttonPressedTop
                                    : fieldArea.containsMouse ? Scheme.buttonHotTop : Scheme.field
                            }
                            GradientStop {
                                position: 1.0
                                color: dialog.listOpen ? Scheme.buttonPressedBottom
                                    : fieldArea.containsMouse ? Scheme.buttonHotBottom : Scheme.field
                            }
                        }

                        Image {
                            anchors.centerIn: parent
                            width: 8
                            height: 5
                            sourceSize: Qt.size(16, 10)
                            source: "../assets/" + (Scheme.dark ? "" : "ink-") + "combo-arrow.svg"
                        }
                    }

                    MouseArea {
                        id: fieldArea
                        anchors.fill: parent
                        hoverEnabled: true
                        onClicked: {
                            dialog.listOpen = !dialog.listOpen;
                            keys.forceActiveFocus();
                        }
                        onWheel: function (event) {
                            dialog.move(event.angleDelta.y > 0 ? -1 : 1);
                        }
                    }
                }

                Text {
                    width: parent.width
                    height: Math.max(implicitHeight, 34)
                    text: {
                        var action = dialog.actions[dialog.current];
                        return action ? dialog.descriptions[action.id] || "" : "";
                    }
                    color: Scheme.text
                    wrapMode: Text.WordWrap
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSize
                }
            }

            // The band with the buttons.
            Rectangle {
                id: footer

                x: 1
                width: parent.width - 2
                height: 44
                anchors.bottom: parent.bottom
                anchors.bottomMargin: 1
                color: Scheme.dark ? "#1a232c" : "#f0f0f0"

                Rectangle {
                    width: parent.width
                    height: 1
                    color: Scheme.dark ? "#33ffffff" : "#dfdfdf"
                }

                Row {
                    anchors.right: parent.right
                    anchors.rightMargin: 12
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 8

                    CpButton {
                        label: "OK"
                        focused: dialog.focusAt === 1
                        onClicked: dialog.accept()
                    }

                    CpButton {
                        label: "Cancel"
                        focused: dialog.focusAt === 2
                        onClicked: dialog.dismissed()
                    }
                }
            }

            // The open list, over the description and the band.
            Rectangle {
                id: list

                visible: dialog.listOpen
                x: content.x
                y: content.y + field.y + field.height
                width: field.width
                height: column.height + 2
                color: Scheme.field
                border.width: 1
                border.color: Scheme.fieldFocus

                Column {
                    id: column

                    x: 1
                    y: 1
                    width: parent.width - 2

                    Repeater {
                        model: dialog.actions

                        delegate: Rectangle {
                            id: row

                            required property var modelData
                            required property int index

                            readonly property bool lit: rowArea.containsMouse
                                || (!list.hovered && row.index === dialog.current)

                            width: column.width
                            height: 22
                            color: row.lit ? "#3399ff" : "transparent"

                            Text {
                                anchors.left: parent.left
                                anchors.leftMargin: 6
                                anchors.verticalCenter: parent.verticalCenter
                                text: row.modelData.label
                                color: row.lit ? "#ffffff" : Scheme.text
                                font.family: Theme.fontFamily
                                font.pixelSize: Theme.fontSize
                            }

                            MouseArea {
                                id: rowArea
                                anchors.fill: parent
                                hoverEnabled: true
                                onClicked: {
                                    dialog.current = row.index;
                                    dialog.listOpen = false;
                                    keys.forceActiveFocus();
                                }
                            }
                        }
                    }
                }

                readonly property bool hovered: listHover.hovered

                HoverHandler {
                    id: listHover
                }
            }
        }
    }
}
