pragma ComponentBehavior: Bound

import QtQuick
import ".."
import "../theme.js" as Theme

// The Run dialog (Super+R), as Windows 7 shows it above the Start button:
// the glass frame with its caption, the page with the run icon and the
// sentence, the Open field with the history under its arrow, then OK and
// Cancel. RunState runs what is typed and keeps the history.
//
// The field starts with the last command, selected. Typing completes from
// the history; Down, F4 or the arrow opens the history; Return runs,
// Escape closes the list, then the dialog. A command whose program cannot
// be found stays in the field, with the message under it.
Item {
    id: dialog

    signal dismissed

    property bool listOpen: false
    /// The history row picked with the keys while the list is open.
    property int listPick: 0

    /// Called each time the dialog opens.
    function reset() {
        dialog.listOpen = false;
        RunState.notFound = "";
        field.text = RunState.history.length > 0 ? RunState.history[0] : "";
        field.selectAll();
        field.forceActiveFocus();
    }

    function accept() {
        dialog.listOpen = false;
        RunState.run(field.text);
    }

    function choose(index) {
        field.text = RunState.history[index];
        dialog.listOpen = false;
        field.forceActiveFocus();
        field.selectAll();
    }

    // A click outside closes it.
    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.MiddleButton
        onPressed: dialog.dismissed()
    }

    Rectangle {
        id: frame

        x: 12
        y: parent.height - Theme.taskbarHeight - height - 12
        width: 420
        height: page.height + caption.height + 9
        radius: 7
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
            radius: 6
            color: "transparent"
            border.width: 1
            border.color: "#59ffffff"
        }

        // A click in the window keeps it, and closes the list.
        MouseArea {
            anchors.fill: parent
            onPressed: dialog.listOpen = false
        }

        Text {
            id: caption

            x: 10
            height: 28
            verticalAlignment: Text.AlignVCenter
            text: "Run"
            color: Scheme.dark ? "#ffffff" : "#1e395b"
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSize
        }

        Rectangle {
            id: page

            x: 8
            y: caption.height
            width: parent.width - 16
            height: content.implicitHeight + footer.height + 30
            color: Scheme.page
            border.width: 1
            border.color: "#80000000"

            Column {
                id: content

                x: 14
                y: 16
                width: parent.width - 28
                spacing: 14

                Row {
                    spacing: 14

                    Image {
                        width: 32
                        height: 32
                        source: "../assets/icons/run.png"
                        sourceSize.width: 64
                        sourceSize.height: 64
                    }

                    Text {
                        width: content.width - 46
                        text: "Type the name of a program, folder, document, or Internet resource, and it will be opened for you."
                        wrapMode: Text.WordWrap
                        color: Scheme.text
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSize
                    }
                }

                Row {
                    spacing: 10

                    Text {
                        height: 24
                        verticalAlignment: Text.AlignVCenter
                        text: "Open:"
                        color: Scheme.text
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSize
                    }

                    // The field and its arrow, as a Windows combo box.
                    Rectangle {
                        id: combo

                        width: content.width - 50
                        height: 24
                        color: Scheme.field
                        border.width: 1
                        border.color: field.activeFocus ? Scheme.fieldFocus : Scheme.fieldBorder

                        TextInput {
                            id: field

                            /// The length before the last edit: completion only
                            /// follows typing, not deleting.
                            property int lastLength: 0

                            anchors.left: parent.left
                            anchors.right: arrow.left
                            anchors.leftMargin: 5
                            anchors.rightMargin: 4
                            anchors.verticalCenter: parent.verticalCenter
                            color: Scheme.text
                            selectionColor: "#3399ff"
                            selectedTextColor: "#ffffff"
                            font.family: Theme.fontFamily
                            font.pixelSize: Theme.fontSize
                            selectByMouse: true
                            clip: true
                            focus: true

                            onTextEdited: {
                                RunState.notFound = "";
                                var typed = field.text;
                                var grew = typed.length > field.lastLength;
                                field.lastLength = typed.length;
                                if (!grew || field.cursorPosition !== typed.length)
                                    return;
                                var match = RunState.completion(typed);
                                if (match.length === 0)
                                    return;
                                field.text = match;
                                field.select(typed.length, match.length);
                            }
                            onTextChanged: {
                                if (!field.activeFocus)
                                    field.lastLength = field.text.length;
                            }

                            Keys.onReturnPressed: {
                                if (dialog.listOpen)
                                    dialog.choose(dialog.listPick);
                                else
                                    dialog.accept();
                            }
                            Keys.onEnterPressed: {
                                if (dialog.listOpen)
                                    dialog.choose(dialog.listPick);
                                else
                                    dialog.accept();
                            }
                            Keys.onEscapePressed: {
                                if (dialog.listOpen)
                                    dialog.listOpen = false;
                                else
                                    dialog.dismissed();
                            }
                            Keys.onUpPressed: {
                                if (dialog.listOpen)
                                    dialog.listPick = Math.max(0, dialog.listPick - 1);
                            }
                            Keys.onDownPressed: {
                                if (!dialog.listOpen) {
                                    dialog.listPick = 0;
                                    dialog.listOpen = RunState.history.length > 0;
                                } else {
                                    dialog.listPick = Math.min(RunState.history.length - 1,
                                                               dialog.listPick + 1);
                                }
                            }
                            Keys.onPressed: function (event) {
                                if (event.key === Qt.Key_F4) {
                                    dialog.listPick = 0;
                                    dialog.listOpen = !dialog.listOpen && RunState.history.length > 0;
                                    event.accepted = true;
                                }
                            }
                        }

                        // The arrow: opens the history.
                        Rectangle {
                            id: arrow

                            anchors.right: parent.right
                            anchors.top: parent.top
                            anchors.bottom: parent.bottom
                            anchors.margins: 1
                            width: 18
                            color: arrowArea.containsMouse || dialog.listOpen
                                ? (Scheme.dark ? "#2a4660" : "#e3f4fc") : "transparent"

                            Image {
                                anchors.centerIn: parent
                                width: 7
                                height: 4
                                source: "../assets/combo-arrow.svg"
                                sourceSize.width: 14
                                sourceSize.height: 8
                            }

                            MouseArea {
                                id: arrowArea
                                anchors.fill: parent
                                hoverEnabled: true
                                onClicked: {
                                    dialog.listPick = 0;
                                    dialog.listOpen = !dialog.listOpen && RunState.history.length > 0;
                                    field.forceActiveFocus();
                                }
                            }
                        }
                    }
                }

                // Its program could not be found: the command stays, to fix.
                Text {
                    x: 50
                    width: content.width - 50
                    visible: RunState.notFound.length > 0 && RunState.notFound === field.text.trim()
                    text: "Cannot find “" + RunState.notFound.split(/\s+/)[0]
                        + "”. Check the name, then try again."
                    wrapMode: Text.WordWrap
                    color: Scheme.dark ? "#ff8a80" : "#c42b1c"
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSize
                }
            }

            // The band of buttons at the bottom.
            Rectangle {
                id: footer

                anchors.left: parent.left
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                anchors.margins: 1
                height: 44
                color: Scheme.dark ? "#1b242d" : "#f0f0f0"

                Row {
                    anchors.right: parent.right
                    anchors.rightMargin: 12
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 8

                    CpButton {
                        label: "OK"
                        onClicked: dialog.accept()
                    }

                    CpButton {
                        label: "Cancel"
                        onClicked: dialog.dismissed()
                    }
                }
            }

            // The history, under the field, over the rest of the page.
            Rectangle {
                visible: dialog.listOpen
                z: 10
                x: content.x + combo.parent.x + combo.x
                y: content.y + combo.parent.y + combo.height
                width: combo.width
                height: historyColumn.implicitHeight + 2
                color: Scheme.field
                border.width: 1
                border.color: Scheme.fieldBorder

                Column {
                    id: historyColumn

                    x: 1
                    y: 1
                    width: parent.width - 2

                    Repeater {
                        model: RunState.history

                        delegate: Rectangle {
                            id: row

                            required property string modelData
                            required property int index

                            width: historyColumn.width
                            height: 20
                            color: row.index === dialog.listPick ? "#3399ff" : "transparent"

                            Text {
                                anchors.left: parent.left
                                anchors.leftMargin: 5
                                anchors.right: parent.right
                                anchors.verticalCenter: parent.verticalCenter
                                text: row.modelData
                                elide: Text.ElideRight
                                color: row.index === dialog.listPick ? "#ffffff" : Scheme.text
                                font.family: Theme.fontFamily
                                font.pixelSize: Theme.fontSize
                            }

                            MouseArea {
                                anchors.fill: parent
                                hoverEnabled: true
                                onEntered: dialog.listPick = row.index
                                onClicked: dialog.choose(row.index)
                            }
                        }
                    }
                }
            }
        }
    }
}
