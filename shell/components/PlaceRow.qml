import QtQuick
import ".."
import "../theme.js" as Theme

// One line of the Start menu's places: white text on the glass, lit under
// the pointer by a pale band of glass with a light rim, as Windows 7 draws
// them. No icon: the picture above the menu shows the place instead.
Item {
    id: row

    property string label: ""
    readonly property bool hovered: area.containsMouse
    signal activated

    implicitHeight: Theme.menuRowHeight

    Rectangle {
        anchors.fill: parent
        radius: Scheme.itemRadius
        visible: area.containsMouse
        border.width: 1
        border.color: "#80ffffff"
        gradient: Gradient {
            GradientStop { position: 0.0; color: area.pressed ? "#1affffff" : "#4dffffff" }
            GradientStop { position: 0.5; color: area.pressed ? "#14ffffff" : "#2effffff" }
            GradientStop { position: 1.0; color: area.pressed ? "#26ffffff" : "#40ffffff" }
        }

        // The darker line outside the rim.
        Rectangle {
            anchors.fill: parent
            anchors.margins: -1
            radius: parent.radius + 1
            color: "transparent"
            border.width: 1
            border.color: "#40000000"
        }
    }

    Text {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: 10
        anchors.rightMargin: 8
        anchors.verticalCenter: parent.verticalCenter
        text: row.label
        color: "#ffffff"
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
        elide: Text.ElideRight
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onClicked: row.activated()
    }
}
