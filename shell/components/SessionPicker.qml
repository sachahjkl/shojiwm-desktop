pragma ComponentBehavior: Bound

import QtQuick
import ".."
import "../theme.js" as Theme

// The session to log on to, in the bottom-left corner of the log-on screen
// (greeter.qml): its name and an arrow; a click opens the list of sessions
// above it, styled as the power button's menu.
Item {
    id: picker

    property bool open: false

    FontMetrics {
        id: metrics
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
    }

    readonly property real longestName: {
        var longest = 0;
        for (var i = 0; i < GreeterState.sessions.length; i++)
            longest = Math.max(longest, metrics.advanceWidth(GreeterState.sessions[i].name));
        return Math.ceil(longest);
    }

    implicitWidth: button.width
    implicitHeight: button.height

    Rectangle {
        id: button

        width: label.implicitWidth + 40
        height: 28
        radius: 3
        color: area.containsMouse || picker.open ? "#40ffffff" : "#1fffffff"
        border.width: 1
        border.color: area.containsMouse || picker.open ? "#99ffffff" : "#59ffffff"

        ShadowText {
            id: label

            anchors.verticalCenter: parent.verticalCenter
            anchors.left: parent.left
            anchors.leftMargin: 12
            text: GreeterState.session ? GreeterState.session.name : "No session"
            font.pixelSize: Theme.fontSize + 1
        }

        // The arrow, pointing up to the list.
        Text {
            anchors.verticalCenter: parent.verticalCenter
            anchors.right: parent.right
            anchors.rightMargin: 10
            text: "▴"
            color: "#ffffff"
            font.pixelSize: Theme.fontSize
        }

        MouseArea {
            id: area
            anchors.fill: parent
            hoverEnabled: true
            onClicked: picker.open = !picker.open
        }
    }

    Rectangle {
        visible: picker.open
        anchors.left: button.left
        anchors.bottom: button.top
        anchors.bottomMargin: 4
        width: Math.max(button.width, picker.longestName + 40)
        height: list.implicitHeight + 8
        radius: Scheme.popupRadius
        color: "#e6f4f9ff"
        border.width: 1
        border.color: "#6e000000"

        Column {
            id: list

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: 4

            Repeater {
                model: GreeterState.sessions

                delegate: Rectangle {
                    id: row

                    required property var modelData

                    width: list.width
                    height: 26
                    radius: Scheme.itemRadius
                    color: rowArea.containsMouse ? "#669ad0f7" : "transparent"
                    border.width: rowArea.containsMouse ? 1 : 0
                    border.color: "#b3cfe9ff"

                    // The chosen session, with a dot, as a radio item.
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.left: parent.left
                        anchors.leftMargin: 8
                        text: "●"
                        visible: !!GreeterState.session
                            && GreeterState.session.id === row.modelData.id
                        color: "#10283c"
                        font.pixelSize: Theme.fontSize - 3
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.left: parent.left
                        anchors.leftMargin: 24
                        text: row.modelData.name
                        color: "#10283c"
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSize
                    }

                    MouseArea {
                        id: rowArea
                        anchors.fill: parent
                        hoverEnabled: true
                        onClicked: {
                            GreeterState.sessionId = row.modelData.id;
                            picker.open = false;
                        }
                    }
                }
            }
        }
    }
}
