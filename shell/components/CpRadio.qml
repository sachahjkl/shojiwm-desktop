import QtQuick
import ".."
import "../theme.js" as Theme

// A radio button with its label, drawn as Windows 7 draws one.
Item {
    id: radio

    property string label: ""
    property bool checked: false
    signal clicked

    implicitWidth: row.implicitWidth
    implicitHeight: 22

    Row {
        id: row
        anchors.verticalCenter: parent.verticalCenter
        spacing: 7

        Rectangle {
            anchors.verticalCenter: parent.verticalCenter
            width: 13
            height: 13
            radius: 6.5
            border.width: 1
            border.color: area.containsMouse ? "#3c7fb1" : "#8e8f8f"
            gradient: Gradient {
                GradientStop { position: 0.0; color: area.containsMouse ? "#dff2fd" : Scheme.field }
                GradientStop { position: 1.0; color: area.containsMouse ? "#b0dbf5" : Scheme.field }
            }

            Rectangle {
                anchors.centerIn: parent
                width: 7
                height: 7
                radius: 3.5
                visible: radio.checked
                gradient: Gradient {
                    GradientStop { position: 0.0; color: "#5aa9f0" }
                    GradientStop { position: 1.0; color: "#1c63ae" }
                }
            }
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: radio.label
            color: Scheme.text
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSize
        }
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onClicked: radio.clicked()
    }
}
