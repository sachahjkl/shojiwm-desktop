pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import ".."
import "../theme.js" as Theme

// The lock screen, as the Windows 7 log-on screen is laid out: the picture
// of your choice behind, the time over it, your account picture in its glass
// frame, your name, and the password field with the round blue arrow. The
// power button sits in the bottom right corner.
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
            GradientStop { position: 0.0; color: "#59000000" }
            GradientStop { position: 0.35; color: "#14000000" }
            GradientStop { position: 0.7; color: "#14000000" }
            GradientStop { position: 1.0; color: "#73000000" }
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

    // The account picture in its glass frame: a light rim, a soft glow.
    Item {
        id: tile

        anchors.horizontalCenter: parent.horizontalCenter
        anchors.verticalCenter: parent.verticalCenter
        anchors.verticalCenterOffset: -40
        width: 146
        height: 146

        Rectangle {
            anchors.fill: parent
            anchors.margins: -6
            radius: 14
            color: "transparent"
            border.width: 6
            border.color: "#26ffffff"
        }

        Rectangle {
            anchors.fill: parent
            radius: 9
            border.width: 1
            border.color: "#e6ffffff"
            gradient: Gradient {
                GradientStop { position: 0.0; color: "#8cffffff" }
                GradientStop { position: 1.0; color: "#40ffffff" }
            }
        }

        Rectangle {
            anchors.fill: parent
            anchors.margins: 10
            radius: 3
            color: "#ffffff"
            clip: true

            Image {
                anchors.fill: parent
                source: Settings.userPictureSource
                sourceSize.width: 252
                sourceSize.height: 252
                fillMode: Image.PreserveAspectCrop
                smooth: true
            }
        }
    }

    ShadowText {
        id: name

        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: tile.bottom
        anchors.topMargin: 16
        text: Apps.userName
        font.pixelSize: 26
    }

    // ----- the password ---------------------------------------------------------

    Item {
        id: entry

        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: name.bottom
        anchors.topMargin: 16
        width: field.width + 8 + go.width
        height: 30

        Rectangle {
            id: field

            width: 240
            height: 28
            anchors.verticalCenter: parent.verticalCenter
            radius: 3
            color: "#ffffff"
            border.width: 1
            border.color: password.activeFocus ? "#3f82b4" : "#7a8fa3"

            // A faint inner shade at the top, as a Windows text box has.
            Rectangle {
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.top: parent.top
                anchors.margins: 1
                height: 3
                radius: 2
                gradient: Gradient {
                    GradientStop { position: 0.0; color: "#1f000000" }
                    GradientStop { position: 1.0; color: "#00000000" }
                }
            }

            TextInput {
                id: password

                anchors.fill: parent
                anchors.leftMargin: 8
                anchors.rightMargin: 8
                verticalAlignment: TextInput.AlignVCenter
                echoMode: TextInput.Password
                passwordCharacter: "●"
                color: "#10283c"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSize + 1
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
                    color: "#8a9aa8"
                    font: password.font
                    visible: password.text.length === 0
                }
            }
        }

        // The round blue arrow.
        Item {
            id: go

            anchors.left: field.right
            anchors.leftMargin: 8
            anchors.verticalCenter: parent.verticalCenter
            width: 30
            height: 30

            Rectangle {
                anchors.fill: parent
                radius: width / 2
                border.width: 1
                border.color: "#1d4f86"
                gradient: Gradient {
                    GradientStop {
                        position: 0.0
                        color: goArea.pressed ? "#2a6db8" : goArea.containsMouse ? "#7cc0ff" : "#5aa9f0"
                    }
                    GradientStop { position: 0.5; color: goArea.pressed ? "#1f5a9e" : "#2a7fd4" }
                    GradientStop { position: 1.0; color: goArea.pressed ? "#174a85" : "#1c63ae" }
                }
            }

            // The gloss on the upper half.
            Rectangle {
                anchors.horizontalCenter: parent.horizontalCenter
                y: 2
                width: parent.width - 6
                height: parent.height / 2 - 2
                radius: height / 2
                color: "#40ffffff"
            }

            Image {
                anchors.centerIn: parent
                width: 16
                height: 16
                source: "../assets/arrow-right.svg"
                sourceSize.width: 32
                sourceSize.height: 32
            }

            MouseArea {
                id: goArea
                anchors.fill: parent
                hoverEnabled: true
                onClicked: screen.submit()
            }
        }
    }

    ShadowText {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: entry.bottom
        anchors.topMargin: 14
        text: LockState.busy ? "Welcome" : LockState.message
        visible: text.length > 0
        font.pixelSize: Theme.fontSize + 2
    }

    // ----- the power button -----------------------------------------------------

    Item {
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: 28
        width: 40
        height: 30

        Rectangle {
            anchors.fill: parent
            radius: 3
            border.width: 1
            border.color: "#80200b06"
            gradient: Gradient {
                GradientStop { position: 0.0; color: powerArea.containsMouse ? "#f07a62" : "#e0644b" }
                GradientStop { position: 0.5; color: powerArea.containsMouse ? "#d24a30" : "#c23d24" }
                GradientStop { position: 1.0; color: powerArea.containsMouse ? "#b8381f" : "#a53019" }
            }
        }

        Image {
            anchors.centerIn: parent
            width: 16
            height: 16
            source: "../assets/power-white.svg"
            sourceSize.width: 32
            sourceSize.height: 32
        }

        MouseArea {
            id: powerArea
            anchors.fill: parent
            hoverEnabled: true
            onClicked: Apps.runPower("shutdown")
        }
    }

    function submit() {
        if (password.text.length === 0 || LockState.busy)
            return;
        LockState.authenticate(password.text);
    }

    // A wrong password: empty the field and keep typing.
    Connections {
        target: LockState
        function onFailuresChanged() {
            password.text = "";
            password.forceActiveFocus();
        }
        function onLockedChanged() {
            password.text = "";
        }
    }

    Component.onCompleted: password.forceActiveFocus()
}
