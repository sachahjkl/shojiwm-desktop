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
    /// Pinned programs and "All Programs" are set in bold, as on Windows.
    property bool bold: false
    /// A section title: smaller, coloured, not a choice.
    property bool heading: false
    /// Extra room on the left, for the programs inside a folder.
    property int indent: 0
    signal activated

    height: Theme.menuRowHeight
    radius: 3
    readonly property bool lit: !heading && (selected || area.containsMouse)
    color: lit ? Theme.selection : "transparent"
    border.width: lit ? 1 : 0
    border.color: Theme.selectionBorder

    Row {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: 6 + row.indent
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
            width: parent.width - (row.icon.length > 0 ? 20 + 8 : 0)
            text: row.label
            color: row.heading ? Theme.menuHeading : row.labelColor
            font.family: Theme.fontFamily
            font.pixelSize: row.heading ? Theme.fontSizeSmall : Theme.fontSize
            font.bold: row.bold || row.heading
            elide: Text.ElideRight
        }
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: !row.heading
        enabled: !row.heading
        onClicked: row.activated()
    }
}
