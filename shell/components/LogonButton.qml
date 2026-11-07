import QtQuick
import "../theme.js" as Theme

// A button of the Windows 7 log-on screen (OK, Cancel): dark blue glass, a
// light rim, a gloss over the top half, white text.
Item {
    id: button

    property string label: ""
    signal clicked

    width: Math.max(94, text.implicitWidth + 30)
    height: 28

    Rectangle {
        anchors.fill: parent
        radius: 3
        border.width: 1
        border.color: "#0b2238"
        gradient: Gradient {
            GradientStop {
                position: 0.0
                color: area.pressed ? "#cc163a5e" : area.containsMouse ? "#cc3f7bb4" : "#cc2a5d8f"
            }
            GradientStop {
                position: 0.5
                color: area.pressed ? "#cc12304f" : area.containsMouse ? "#cc2c6aa6" : "#cc1d4a78"
            }
            GradientStop {
                position: 0.51
                color: area.pressed ? "#cc0e2741" : area.containsMouse ? "#cc205a93" : "#cc143b63"
            }
            GradientStop {
                position: 1.0
                color: area.pressed ? "#cc163a5e" : area.containsMouse ? "#cc2f74b5" : "#cc1f4f80"
            }
        }
    }

    // The light rim inside the dark line.
    Rectangle {
        anchors.fill: parent
        anchors.margins: 1
        radius: 2
        color: "transparent"
        border.width: 1
        border.color: area.containsMouse ? "#b3d9eeff" : "#7fa8cdee"
    }

    Text {
        id: text
        anchors.centerIn: parent
        text: button.label
        color: "#ffffff"
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize + 1
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onClicked: button.clicked()
    }
}
