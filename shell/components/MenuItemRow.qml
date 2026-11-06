import QtQuick
import Quickshell.Widgets
import "../theme.js" as Theme

// One line of a popup menu, in the manner of a Windows 7 menu: a pale blue
// band under the pointer, a thin frame around it.
Item {
    id: row

    property string label: ""
    property string icon: ""
    property bool separator: false
    property bool itemEnabled: true
    signal activated

    implicitHeight: separator ? 7 : Theme.popupItemHeight

    Rectangle {
        anchors.verticalCenter: parent.verticalCenter
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: Theme.popupPadX / 2
        anchors.rightMargin: Theme.popupPadX / 2
        height: 1
        color: Theme.menuDivider
        visible: row.separator
    }

    Rectangle {
        anchors.fill: parent
        anchors.leftMargin: 2
        anchors.rightMargin: 2
        radius: 3
        visible: !row.separator && area.containsMouse && row.itemEnabled
        color: Theme.selection
        border.width: 1
        border.color: Theme.selectionBorder
    }

    Row {
        anchors.fill: parent
        anchors.leftMargin: Theme.popupPadX
        anchors.rightMargin: Theme.popupPadX
        spacing: 7
        visible: !row.separator

        IconImage {
            anchors.verticalCenter: parent.verticalCenter
            implicitSize: 16
            source: row.icon
            visible: row.icon.length > 0
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: row.label
            color: Theme.textOnMenu
            opacity: row.itemEnabled ? 1 : 0.45
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSize
            elide: Text.ElideRight
        }
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: !row.separator
        enabled: !row.separator && row.itemEnabled
        onClicked: row.activated()
    }
}
