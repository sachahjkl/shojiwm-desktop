import QtQuick
import ".."
import "../theme.js" as Theme

// A check box with its label, drawn as Windows 7 draws one.
Item {
    id: check

    property string label: ""
    property bool checked: false
    signal toggled(bool checked)

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
            border.width: 1
            border.color: area.containsMouse ? "#3c7fb1" : "#8e8f8f"
            gradient: Gradient {
                GradientStop { position: 0.0; color: area.containsMouse ? "#dff2fd" : Scheme.field }
                GradientStop { position: 1.0; color: area.containsMouse ? "#b0dbf5" : Scheme.field }
            }

            Text {
                anchors.centerIn: parent
                text: check.checked ? "✔" : ""
                color: "#1e4f7c"
                font.pixelSize: 11
            }
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: check.label
            color: Scheme.text
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSize
        }
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onClicked: check.toggled(!check.checked)
    }
}
