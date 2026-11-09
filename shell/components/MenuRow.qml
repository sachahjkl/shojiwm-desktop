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
    /// The .desktop file name of the program on this row, which a drag
    /// carries to the taskbar; "" for a row that does not drag.
    property string dragId: ""

    // What the pointer drags: invisible, the drag shows a picture of the row.
    Item {
        id: dragProxy

        width: row.width
        height: row.height
        Drag.active: area.drag.active
        Drag.dragType: Drag.Automatic
        Drag.supportedActions: Qt.CopyAction
        Drag.hotSpot.x: 16
        Drag.hotSpot.y: height / 2
        Drag.keys: [Settings.programDragType]
        Drag.mimeData: ({ [Settings.programDragType]: row.dragId })
        Drag.onDragFinished: {
            dragProxy.x = 0;
            dragProxy.y = 0;
        }
    }

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
        drag.target: row.dragId.length > 0 ? dragProxy : null

        onPressed: function (event) {
            if (event.button === Qt.LeftButton && row.dragId.length > 0)
                row.grabToImage(function (result) { dragProxy.Drag.imageSource = result.url; },
                                Qt.size(row.width, row.height));
        }
        onClicked: function (event) {
            if (event.button === Qt.RightButton)
                row.menuRequested(event.x, event.y);
            else
                row.activated();
        }
    }
}
