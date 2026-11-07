pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import ".."
import "../theme.js" as Theme

// The lock screen, as the Windows 7 log-on screen is laid out: the picture
// of your choice behind, the time over it, your account picture in its glass
// frame, your name, and the password field with the round arrow. A wrong
// password replaces the field with the message and OK, as Windows does. The
// red power button and its arrow sit in the bottom right corner.
//
// One per screen. The password is checked once, by LockState.
Item {
    id: screen

    // ----- the picture behind ---------------------------------------------------

    Rectangle {
        anchors.fill: parent
        color: "#0e3a63"
    }

    Image {
        anchors.fill: parent
        source: Settings.lockSource
        sourceSize.width: width
        sourceSize.height: height
        fillMode: Image.PreserveAspectCrop
        asynchronous: true
        smooth: true
    }

    // A shade from the top and the bottom, so white text reads on any picture.
    Rectangle {
        anchors.fill: parent
        gradient: Gradient {
            GradientStop { position: 0.0; color: Settings.dark ? "#80000000" : "#59000000" }
            GradientStop { position: 0.35; color: Settings.dark ? "#33000000" : "#14000000" }
            GradientStop { position: 0.7; color: Settings.dark ? "#33000000" : "#14000000" }
            GradientStop { position: 1.0; color: Settings.dark ? "#99000000" : "#73000000" }
        }
    }

    SystemClock {
        id: clock
        precision: SystemClock.Seconds
    }

    // ----- the time -------------------------------------------------------------

    Column {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: tile.top
        anchors.bottomMargin: Math.round(screen.height * 0.06)
        spacing: 2

        ShadowText {
            anchors.horizontalCenter: parent.horizontalCenter
            text: Settings.clockFormat === "24h"
                ? Qt.formatTime(clock.date, "HH:mm")
                : Settings.clockFormat === "12h"
                    ? Qt.formatTime(clock.date, "h:mm AP")
                    : clock.date.toLocaleTimeString(Qt.locale(), Locale.ShortFormat)
            font.pixelSize: 76
            font.weight: Font.Light
        }

        ShadowText {
            anchors.horizontalCenter: parent.horizontalCenter
            text: clock.date.toLocaleDateString(Qt.locale(), Locale.LongFormat)
            font.pixelSize: 20
        }
    }

    // ----- you ------------------------------------------------------------------
    //
    // Measured on Windows 7 log-on screenshots: a frame of about 116 px round
    // a 98 px picture at 800 x 600, a 175 x 20 field, a 22 px round arrow.
    // Here at the size a 1080p screen shows them.

    UserTile {
        id: tile

        dark: Settings.dark
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.verticalCenter: parent.verticalCenter
        anchors.verticalCenterOffset: -50
        source: Settings.userPictureSource
    }

    ShadowText {
        id: name

        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: tile.bottom
        anchors.topMargin: 14
        text: Apps.userName
        font.pixelSize: 22
    }

    // The password field and its round arrow, or, after a wrong password,
    // the message and OK, as Windows 7 swaps them.
    Item {
        id: entry

        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: name.bottom
        anchors.topMargin: 12
        width: field.width + 6 + go.width
        height: 28
        visible: !screen.showingError

        Rectangle {
            id: field

            width: 225
            height: 26
            anchors.verticalCenter: parent.verticalCenter
            color: Settings.dark ? "#141c24" : "#ffffff"
            border.width: 1
            border.color: password.activeFocus
                ? (Settings.dark ? "#4b8fd0" : "#3d7bad")
                : (Settings.dark ? "#3c5164" : "#5a7f9c")

            // The inner shade a Windows text box has along its top and left.
            Rectangle {
                x: 1
                y: 1
                width: parent.width - 2
                height: 1
                color: Settings.dark ? "#0a1016" : "#c4d3de"
            }
            Rectangle {
                x: 1
                y: 1
                width: 1
                height: parent.height - 2
                color: Settings.dark ? "#0d141b" : "#dde6ec"
            }

            TextInput {
                id: password

                anchors.fill: parent
                anchors.leftMargin: 6
                anchors.rightMargin: 6
                verticalAlignment: TextInput.AlignVCenter
                echoMode: TextInput.Password
                passwordCharacter: "\u25CF"
                color: Settings.dark ? "#e8f1f9" : "#000000"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSize
                clip: true
                focus: true
                enabled: !LockState.busy

                Keys.onReturnPressed: screen.submit()
                Keys.onEnterPressed: screen.submit()
                Keys.onEscapePressed: password.text = ""

                Text {
                    anchors.fill: parent
                    verticalAlignment: Text.AlignVCenter
                    text: "Password"
                    color: Settings.dark ? "#7f94a6" : "#6d6d6d"
                    font: password.font
                    visible: password.text.length === 0
                }
            }
        }

        // The round arrow: a bead of glass, drawn in arrow-bead.svg, and
        // brighter under the pointer (arrow-bead-hot.svg).
        Item {
            id: go

            anchors.left: field.right
            anchors.leftMargin: 4
            anchors.verticalCenter: parent.verticalCenter
            width: 32
            height: 32

            Image {
                anchors.fill: parent
                source: "../assets/arrow-bead"
                    + (Settings.dark ? "-dark" : "")
                    + (goArea.containsMouse && !goArea.pressed ? "-hot" : "")
                    + ".svg"
                sourceSize.width: 64
                sourceSize.height: 64
                smooth: true
                // Pressed: the glass dims, as if pushed in.
                opacity: goArea.pressed ? 0.8 : 1
            }

            MouseArea {
                id: goArea
                anchors.fill: parent
                hoverEnabled: true
                onClicked: screen.submit()
            }
        }
    }

    // "Welcome", while PAM checks.
    ShadowText {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: entry.bottom
        anchors.topMargin: 14
        text: "Welcome"
        visible: LockState.busy
        font.pixelSize: Theme.fontSize + 4
    }

    // A wrong password: the field gives way to the message and OK.
    Column {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: name.bottom
        anchors.topMargin: 14
        spacing: 16
        visible: screen.showingError

        Row {
            anchors.horizontalCenter: parent.horizontalCenter
            spacing: 10

            Image {
                anchors.verticalCenter: parent.verticalCenter
                width: 24
                height: 24
                source: "../assets/error-shield.svg"
                sourceSize.width: 48
                sourceSize.height: 48
            }

            ShadowText {
                anchors.verticalCenter: parent.verticalCenter
                text: LockState.message
                font.pixelSize: Theme.fontSize + 4
            }
        }

        LogonButton {
            id: okButton
            anchors.horizontalCenter: parent.horizontalCenter
            label: "OK"
            onClicked: screen.dismissError()
        }
    }

    // ----- the power button -------------------------------------------------------

    PowerSplitButton {
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: 26
        onPower: Apps.runPower(Apps.powerButtonAction === "lock"
            ? "shutdown" : Apps.powerButtonAction)
        onMenu: powerMenuOpen = !powerMenuOpen
    }

    property bool powerMenuOpen: false

    // Restart, sleep, hibernate, shut down, above the button.
    Rectangle {
        visible: screen.powerMenuOpen
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.rightMargin: 26
        anchors.bottomMargin: 26 + 26
        width: 150
        height: powerColumn.implicitHeight + 8
        radius: Scheme.popupRadius
        color: "#e6f4f9ff"
        border.width: 1
        border.color: "#2c5779"

        Column {
            id: powerColumn

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: 4

            Repeater {
                model: Apps.powerActions.filter(function (action) {
                    return ["restart", "sleep", "hibernate", "shutdown"].indexOf(action.id) >= 0;
                })

                delegate: Rectangle {
                    id: powerRow

                    required property var modelData

                    width: powerColumn.width
                    height: 26
                    radius: Scheme.itemRadius
                    color: powerRowArea.containsMouse ? "#669ad0f7" : "transparent"
                    border.width: powerRowArea.containsMouse ? 1 : 0
                    border.color: "#b3cfe9ff"

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.left: parent.left
                        anchors.leftMargin: 10
                        text: powerRow.modelData.label
                        color: "#10283c"
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSize
                    }

                    MouseArea {
                        id: powerRowArea
                        anchors.fill: parent
                        hoverEnabled: true
                        onClicked: {
                            screen.powerMenuOpen = false;
                            Apps.runPower(powerRow.modelData.id);
                        }
                    }
                }
            }
        }
    }

    // ----- behaviour --------------------------------------------------------------

    /// A wrong password is on show, until OK.
    property bool showingError: false

    function dismissError() {
        screen.showingError = false;
        password.text = "";
        password.forceActiveFocus();
    }

    // Return or Escape on the message is OK.
    Keys.onPressed: function (event) {
        if (screen.showingError && (event.key === Qt.Key_Return
            || event.key === Qt.Key_Enter || event.key === Qt.Key_Escape
            || event.key === Qt.Key_Space)) {
            screen.dismissError();
            event.accepted = true;
        }
    }

    function submit() {
        if (password.text.length === 0 || LockState.busy)
            return;
        LockState.authenticate(password.text);
    }

    Connections {
        target: LockState
        function onFailuresChanged() {
            password.text = "";
            screen.showingError = true;
            screen.forceActiveFocus();
        }
        function onLockedChanged() {
            password.text = "";
            screen.showingError = false;
            screen.powerMenuOpen = false;
        }
    }

    Component.onCompleted: password.forceActiveFocus()
}
