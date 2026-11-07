import QtQuick
import Quickshell.Widgets
import ".."
import "../theme.js" as Theme

// One line of a popup menu, in the manner of a Windows 7 menu: a pale blue
// band under the pointer, a thin frame around it.
Item {
    id: row

    property string label: ""
    property string icon: ""
    property bool separator: false
    /// A section title, such as "Recent" or "Tasks": not a choice.
    property bool heading: false
    property bool itemEnabled: true
    /// A submenu opens from this line; an arrow says so.
    property bool hasChildren: false
    /// The submenu of this line is open.
    property bool highlighted: false
    /// A check box or a radio button in front of the label.
    property bool checkable: false
    property bool checked: false
    readonly property bool hovered: area.containsMouse

    signal activated

    implicitHeight: separator ? 7 : Theme.popupItemHeight

    /// Where the line is in its window, which is what a submenu anchors to.
    function rowRect() {
        var point = row.mapToItem(null, 0, 0);
        return Qt.rect(point.x, point.y, row.width, row.height);
    }

    Rectangle {
        anchors.verticalCenter: parent.verticalCenter
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: Theme.popupPadX / 2
        anchors.rightMargin: Theme.popupPadX / 2
        height: 1
        color: Scheme.menuDivider
        visible: row.separator
    }

    Rectangle {
        anchors.fill: parent
        anchors.leftMargin: 2
        anchors.rightMargin: 2
        radius: 3
        visible: !row.separator && !row.heading && row.itemEnabled
            && (area.containsMouse || row.highlighted)
        color: Scheme.selection
        border.width: 1
        border.color: Scheme.selectionBorder
    }

    Row {
        id: content

        anchors.fill: parent
        anchors.leftMargin: Theme.popupPadX
        anchors.rightMargin: Theme.popupPadX + (row.hasChildren ? 12 : 0)
        spacing: 7
        visible: !row.separator

        // The check mark or the dot, drawn in the icon's place.
        Text {
            anchors.verticalCenter: parent.verticalCenter
            width: 16
            horizontalAlignment: Text.AlignHCenter
            visible: row.checkable && row.icon.length === 0
            text: row.checked ? "\u2714" : ""
            color: Scheme.text
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSizeSmall
        }

        IconImage {
            id: iconImage

            anchors.verticalCenter: parent.verticalCenter
            implicitSize: 16
            source: row.icon
            visible: row.icon.length > 0
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            width: Math.max(0, content.width
                - (iconImage.visible || row.checkable ? 16 + content.spacing : 0))
            text: row.label
            color: row.heading ? Scheme.menuHeading : Scheme.text
            opacity: row.itemEnabled ? 1 : 0.45
            font.family: Theme.fontFamily
            font.pixelSize: row.heading ? Theme.fontSizeSmall : Theme.fontSize
            font.bold: row.heading
            elide: Text.ElideRight
        }
    }

    // The arrow of a line that opens a submenu.
    Text {
        anchors.verticalCenter: parent.verticalCenter
        anchors.right: parent.right
        anchors.rightMargin: Theme.popupPadX
        visible: row.hasChildren && !row.separator
        text: "\u25B8"
        color: Scheme.text
        opacity: row.itemEnabled ? 1 : 0.45
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: !row.separator && !row.heading
        enabled: !row.separator && !row.heading && row.itemEnabled
        onClicked: row.activated()
    }
}
