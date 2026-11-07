import QtQuick
import "../theme.js" as Theme

// One half of the Start menu's power button: the button itself, or the
// arrow beside it. The two halves share an edge and round only their outer
// corners.
Item {
    id: part

    property bool leftRounded: false
    property bool rightRounded: false
    property bool hot: false
    property bool pressed: false

    height: parent ? parent.height : 28

    readonly property color fillTop: part.pressed ? "#c6dcef"
        : part.hot ? "#ffffff" : "#eef6ff"
    readonly property color fillBottom: part.pressed ? "#9fc0dc"
        : part.hot ? "#d4e6f5" : "#c3d7e8"

    // Rounded all round, then squared off on the inner side by a second
    // rectangle that covers those corners.
    Rectangle {
        anchors.fill: parent
        radius: 3
        border.width: 1
        border.color: Theme.menuOutline
        gradient: Gradient {
            GradientStop { position: 0.0; color: part.fillTop }
            GradientStop { position: 1.0; color: part.fillBottom }
        }
    }

    Rectangle {
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        x: part.leftRounded ? parent.width - width : 0
        width: 4
        visible: part.leftRounded !== part.rightRounded
        border.width: 0
        gradient: Gradient {
            GradientStop { position: 0.0; color: part.fillTop }
            GradientStop { position: 1.0; color: part.fillBottom }
        }

        // The outline along the top and the bottom of the squared side.
        Rectangle { anchors.top: parent.top; width: parent.width; height: 1; color: Theme.menuOutline }
        Rectangle { anchors.bottom: parent.bottom; width: parent.width; height: 1; color: Theme.menuOutline }
    }

    // The line between the two halves: on the arrow's left edge.
    Rectangle {
        visible: part.rightRounded && !part.leftRounded
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: 1
        color: Theme.menuOutline
    }
}
