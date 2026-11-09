import QtQuick
import Quickshell.Widgets
import ".."
import "../theme.js" as Theme

// One line of the Start menu: an icon and a label, lit under the pointer.
Rectangle {
    id: row

    property string label: ""
    property string icon: ""
    property color labelColor: Scheme.text
    property bool selected: false
    /// Pinned programs and "All Programs" are set in bold, as on Windows.
    property bool bold: false
    /// A section title: smaller, coloured, not a choice.
    property bool heading: false
    /// Extra room on the left, for the programs inside a folder.
    property int indent: 0
    readonly property bool hovered: area.containsMouse
    signal activated
    /// A right click, at this point of the row.
    signal menuRequested(real x, real y)

    height: Theme.menuRowHeight
    radius: Scheme.itemRadius
    readonly property bool lit: !heading && (selected || area.containsMouse)
    color: lit ? Scheme.selection : "transparent"
    border.width: lit ? Scheme.selectionBorderWidth : 0
    border.color: Scheme.selectionBorder

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
            color: row.heading ? Scheme.menuHeading : row.labelColor
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
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: function (event) {
            if (event.button === Qt.RightButton)
                row.menuRequested(event.x, event.y);
            else
                row.activated();
        }
    }
}
