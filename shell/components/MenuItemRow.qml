import QtQuick
import ".."

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
    /// Keep the column of the icon and the check mark, empty on this line,
    /// so its label lines up with lines that have one.
    property bool gutter: false
    readonly property bool hovered: area.containsMouse

    signal activated
    /// "pin" or "unpin" for a jump list file, "" for any other line.
    property string pinState: ""
    signal pinClicked

    implicitHeight: separator ? 7 : Scheme.theme.popupItemHeight

    /// Where the line is in its window, which is what a submenu anchors to.
    function rowRect() {
        var point = row.mapToItem(null, 0, 0);
        return Qt.rect(point.x, point.y, row.width, row.height);
    }

    Rectangle {
        anchors.verticalCenter: parent.verticalCenter
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: Scheme.theme.popupPadX / 2
        anchors.rightMargin: Scheme.theme.popupPadX / 2
        height: 1
        color: Scheme.menuDivider
        visible: row.separator
    }

    Rectangle {
        anchors.fill: parent
        anchors.leftMargin: 2
        anchors.rightMargin: 2
        radius: Scheme.itemRadius
        visible: !row.separator && !row.heading && row.itemEnabled
            && (area.containsMouse || row.highlighted)
        color: Scheme.selection
        border.width: Scheme.selectionBorderWidth
        border.color: Scheme.selectionBorder
    }

    Row {
        id: content

        anchors.fill: parent
        anchors.leftMargin: Scheme.theme.popupPadX
        anchors.rightMargin: Scheme.theme.popupPadX + (row.hasChildren ? 12 : 0)
        spacing: 7
        visible: !row.separator

        // The check mark or the dot, drawn in the icon's place.
        Text {
            anchors.verticalCenter: parent.verticalCenter
            width: 16
            horizontalAlignment: Text.AlignHCenter
            visible: (row.checkable || row.gutter) && row.icon.length === 0
            text: row.checked ? "\u2714" : ""
            color: Scheme.text
            font.family: Scheme.theme.fontFamily
            font.pixelSize: Scheme.theme.fontSizeSmall
        }

        ShellIcon {
            id: iconImage

            anchors.verticalCenter: parent.verticalCenter
            implicitSize: 16
            source: row.icon
            visible: row.icon.length > 0
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            width: Math.max(0, content.width
                - (iconImage.visible || row.checkable || row.gutter ? 16 + content.spacing : 0))
            text: row.label
            color: row.heading ? Scheme.menuHeading : Scheme.text
            opacity: row.itemEnabled ? 1 : 0.45
            font.family: Scheme.theme.fontFamily
            font.pixelSize: row.heading ? Scheme.theme.fontSizeSmall : Scheme.theme.fontSize
            font.bold: row.heading
            elide: Text.ElideRight
        }
    }

    // The arrow of a line that opens a submenu.
    Text {
        anchors.verticalCenter: parent.verticalCenter
        anchors.right: parent.right
        anchors.rightMargin: Scheme.theme.popupPadX
        visible: row.hasChildren && !row.separator
        text: "\u25B8"
        color: Scheme.text
        opacity: row.itemEnabled ? 1 : 0.45
        font.family: Scheme.theme.fontFamily
        font.pixelSize: Scheme.theme.fontSize
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: !row.separator && !row.heading
        enabled: !row.separator && !row.heading && row.itemEnabled
        onClicked: row.activated()
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
