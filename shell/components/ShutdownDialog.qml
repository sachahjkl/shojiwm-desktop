pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls.Basic
import ".."
import "../theme.js" as Theme

// "Shut Down Windows", as Windows 7 shows it after Alt+F4 on the desktop: the
// screen dims, and a small window asks what the computer should do, from a
// list of the power actions. Return confirms, Escape cancels, Up and Down
// change the choice.
Item {
    id: dialog

    signal dismissed

    /// The power actions, without the separator; the power button's own
    /// action first, as Windows 7 picks it.
    readonly property var actions: {
        var list = Apps.powerActions.filter(function (action) {
            return action.id !== "separator";
        });
        list.sort(function (a, b) {
            return (a.id === Apps.powerButtonAction ? 0 : 1) - (b.id === Apps.powerButtonAction ? 0 : 1);
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

    /// Cleared each time the dialog opens.
    function reset() {
        choice.currentIndex = 0;
        choice.forceActiveFocus();
    }

    function accept() {
        var action = dialog.actions[choice.currentIndex];
        dialog.dismissed();
        if (action)
            Apps.runPower(action.id);
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

    // The window: a glass frame round a white page.
    Rectangle {
        id: frame

        anchors.centerIn: parent
        width: 420
        height: page.height + caption.height + 9
        radius: 7
        border.width: 1
        border.color: "#b3000000"
        gradient: Gradient {
            GradientStop { position: 0.0; color: "#e6b9d1ea" }
            GradientStop { position: 1.0; color: "#e68fb3d6" }
        }

        // Swallow clicks, so a click in the window does not close it.
        MouseArea {
            anchors.fill: parent
        }

        Text {
            id: caption

            x: 10
            height: 28
            verticalAlignment: Text.AlignVCenter
            text: "Shut Down Windows"
            color: "#1e395b"
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSize
        }

        Rectangle {
            id: page

            x: 8
            y: caption.height
            width: parent.width - 16
            height: content.implicitHeight + 28
            color: Scheme.page
            border.width: 1
            border.color: "#80000000"

            Column {
                id: content

                x: 16
                y: 14
                width: parent.width - 32
                spacing: 12

                Row {
                    spacing: 12

                    Image {
                        width: 48
                        height: 48
                        source: "../assets/start-orb.svg"
                        sourceSize.width: 96
                        sourceSize.height: 96
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: "What do you want the computer to do?"
                        color: Scheme.pageHeading
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSize + 2
                    }
                }

                ComboBox {
                    id: choice

                    width: parent.width
                    model: dialog.actions.map(function (action) { return action.label; })
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSize

                    Keys.onReturnPressed: dialog.accept()
                    Keys.onEnterPressed: dialog.accept()
                    Keys.onEscapePressed: dialog.dismissed()
                }

                Text {
                    width: parent.width
                    text: {
                        var action = dialog.actions[choice.currentIndex];
                        return action ? dialog.descriptions[action.id] || "" : "";
                    }
                    color: Scheme.text
                    wrapMode: Text.WordWrap
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSize
                }

                Row {
                    anchors.right: parent.right
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
        }
    }
}
