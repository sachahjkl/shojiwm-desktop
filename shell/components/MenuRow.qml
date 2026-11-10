import QtQuick
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
    /// "pin" or "unpin" for a jump list file, "" for any other line.
    property string pinState: ""
    signal pinClicked
    /// A right click, at this point of the row.
    signal menuRequested(real x, real y)
    /// The .desktop file name of the program on this row, which a drag
    /// carries to the taskbar; "" for a row that does not drag.
    property string dragId: ""
    /// The program has a jump list: an arrow at the right end, as Windows 7
    /// draws one.
    property bool hasArrow: false

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
        Drag.onDragStarted: DragState.start("program", row.dragId)
        Drag.onDragFinished: {
            DragState.finish();
            dragProxy.x = 0;
            dragProxy.y = 0;
        }
    }

    height: StartMenuState.rowHeight
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

        ShellIcon {
            anchors.verticalCenter: parent.verticalCenter
            implicitSize: StartMenuState.iconSize
            source: row.icon
            visible: row.icon.length > 0
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width - (row.icon.length > 0 ? StartMenuState.iconSize + 8 : 0) - (row.hasArrow ? 14 : 0)
            text: row.label
            color: row.heading ? Scheme.menuHeading : row.labelColor
            font.family: Theme.fontFamily
            font.pixelSize: row.heading ? Theme.fontSizeSmall : Theme.fontSize
            font.bold: row.bold || row.heading
            elide: Text.ElideRight
        }
    }

    Text {
        anchors.right: parent.right
        anchors.rightMargin: 6
        anchors.verticalCenter: parent.verticalCenter
        visible: row.hasArrow
        text: "▸"
        color: Scheme.textMuted
        font.pixelSize: Theme.fontSize
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

    // The pin of a jump list file, at the right end under the pointer, as
    // Windows 7 shows it: leaning to pin the file, upright to unpin it.
    Item {
        anchors.right: parent.right
        anchors.rightMargin: 4
        anchors.verticalCenter: parent.verticalCenter
        width: 22
        height: parent.height - 4
        visible: row.pinState.length > 0 && (row.hovered || pinArea.containsMouse)

        Rectangle {
            anchors.fill: parent
            radius: Scheme.corner(2)
            color: pinArea.containsMouse ? Scheme.hover : "transparent"
            border.width: pinArea.containsMouse ? 1 : 0
            border.color: Scheme.selectionBorder
        }

        Image {
            anchors.centerIn: parent
            width: 14
            height: 14
            sourceSize: Qt.size(28, 28)
            source: Qt.resolvedUrl("../assets/" + (Scheme.dark ? "" : "ink-")
                + (row.pinState === "unpin" ? "pin-on" : "pin") + ".svg")
        }

        MouseArea {
            id: pinArea
            anchors.fill: parent
            hoverEnabled: true
            onClicked: row.pinClicked()
        }
    }
}
