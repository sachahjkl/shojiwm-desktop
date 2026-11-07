import QtQuick
import ".."
import "../theme.js" as Theme

// A push button, as Windows 7 draws one: a pale gradient that turns blue
// under the pointer.
Rectangle {
    id: button

    property string label: ""
    signal clicked

    implicitWidth: Math.max(86, text.implicitWidth + 24)
    implicitHeight: 24
    radius: 3
    border.width: 1
    border.color: area.containsMouse ? "#3c7fb1" : "#707070"
    gradient: Gradient {
        GradientStop {
            position: 0.0
            color: area.pressed ? Scheme.buttonPressedTop
                : area.containsMouse ? "#eaf6fd" : Scheme.buttonTop
        }
        GradientStop {
            position: 0.5
            color: area.pressed ? Scheme.buttonPressedBottom
                : area.containsMouse ? "#d9f0fc" : Scheme.buttonBottom
        }
        GradientStop {
            position: 1.0
            color: area.pressed ? Scheme.buttonPressedBottom
                : area.containsMouse ? "#a7d9f5" : Scheme.buttonBottom
        }
    }

    Text {
        id: text
        anchors.centerIn: parent
        text: button.label
        color: area.containsMouse && !Scheme.dark ? "#10283c" : Scheme.text
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onClicked: button.clicked()
    }
}
