import QtQuick
import ".."

// A push button, as Windows 7 draws one: a pale gradient that turns blue
// under the pointer. The dark scheme lightens it a little instead, so
// the light text stays readable.
Rectangle {
    id: button

    property string label: ""
    /// It has the keyboard: a dotted line inside, as Windows draws a focused
    /// button.
    property bool focused: false
    signal clicked

    implicitWidth: Math.max(86, text.implicitWidth + 24)
    implicitHeight: 24
    radius: Scheme.buttonRadius
    border.width: 1
    border.color: area.containsMouse || button.focused ? (Scheme.dark ? Scheme.selectionBorder : "#3c7fb1")
        : (Scheme.dark ? Scheme.fieldBorder : "#707070")
    gradient: Gradient {
        GradientStop {
            position: 0.0
            color: area.pressed ? Scheme.buttonPressedTop
                : area.containsMouse ? (Scheme.dark ? Scheme.buttonHotTop : "#eaf6fd") : Scheme.buttonTop
        }
        GradientStop {
            position: 0.5
            color: area.pressed ? Scheme.buttonPressedBottom
                : area.containsMouse ? (Scheme.dark ? Scheme.buttonHotBottom : "#d9f0fc") : Scheme.buttonBottom
        }
        GradientStop {
            position: 1.0
            color: area.pressed ? Scheme.buttonPressedBottom
                : area.containsMouse ? (Scheme.dark ? Scheme.buttonHotBottom : "#a7d9f5") : Scheme.buttonBottom
        }
    }

    Text {
        id: text
        anchors.centerIn: parent
        text: button.label
        color: area.containsMouse && !Scheme.dark ? "#10283c" : Scheme.text
        font.family: Scheme.theme.fontFamily
        font.pixelSize: Scheme.theme.fontSize
    }

    Rectangle {
        anchors.fill: parent
        anchors.margins: 3
        visible: button.focused
        color: "transparent"
        radius: Scheme.corner(1)
        border.width: 1
        border.color: Scheme.dark ? "#99ffffff" : "#99000000"
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onClicked: button.clicked()
    }
}
