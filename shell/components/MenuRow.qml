import QtQuick
import Quickshell.Widgets
import "../theme.js" as Theme

// One line of the Start menu: an icon and a label, lit under the pointer.
Rectangle {
    id: row

    property string label: ""
    property string icon: ""
    property color labelColor: Theme.textOnMenu
    property bool selected: false
    signal activated

    height: Theme.menuRowHeight
    radius: 3
    color: selected || area.containsMouse ? Theme.selection : "transparent"
    border.width: selected || area.containsMouse ? 1 : 0
    border.color: Theme.selectionBorder

    Row {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: 6
        anchors.rightMargin: 6
        anchors.verticalCenter: parent.verticalCenter
        spacing: 8

        IconImage {
            anchors.verticalCenter: parent.verticalCenter
            implicitSize: 20
            source: row.icon
            visible: row.icon.length > 0
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width - 20 - 8 - 12
            text: row.label
            color: row.labelColor
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSize
            elide: Text.ElideRight
        }
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onClicked: row.activated()
    }
}
