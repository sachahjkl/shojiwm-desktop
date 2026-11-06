pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import "../theme.js" as Theme

// The menu of a notification-area item, drawn like a Windows 7 menu.
//
// `QsMenuOpener` opens the item's DBusMenu and lists its entries
// (Quickshell/QsMenuOpener, Quickshell/QsMenuEntry). Triggering an entry is
// what the item expects, submenus included.
PopupWindow {
    id: popup

    property var item: null
    property var anchorWindow: null
    property rect anchorRect: Qt.rect(0, 0, 1, 1)

    signal dismissed

    readonly property var entries: opener.children ? opener.children.values : []

    color: "transparent"
    visible: item !== null && entries.length > 0

    anchor.window: popup.anchorWindow
    anchor.rect: popup.anchorRect
    anchor.edges: Edges.Top
    anchor.gravity: Edges.Top
    anchor.margins.bottom: Theme.popupGap
    anchor.adjustment: PopupAdjustment.SlideX | PopupAdjustment.FlipY

    implicitWidth: Math.min(
        Theme.popupMaxWidth,
        Math.max(Theme.popupMinWidth, popup.widest + Theme.popupPadX * 2 + 24))
    implicitHeight: column.implicitHeight + Theme.popupPadY * 2 + 2

    QsMenuOpener {
        id: opener
        menu: popup.item ? popup.item.menu : null
    }

    FontMetrics {
        id: metrics
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
    }

    // The popup is sized before it is shown, so the width comes from the
    // entry strings rather than from laid-out rows.
    readonly property real widest: {
        var width = 0;
        for (var i = 0; i < entries.length; i++) {
            var text = entries[i].text || "";
            width = Math.max(width, metrics.advanceWidth(text));
        }
        return width;
    }

    Rectangle {
        anchors.fill: parent
        radius: 4
        border.width: 1
        border.color: Theme.popupOutline
        gradient: Gradient {
            GradientStop { position: 0.0; color: Theme.popupTop }
            GradientStop { position: 1.0; color: Theme.popupBottom }
        }
    }

    /// `true` while the pointer is over the menu. The taskbar closes a menu
    /// the pointer has left.
    readonly property bool pointerInside: hoverArea.containsMouse

    MouseArea {
        id: hoverArea

        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.NoButton
    }

    Column {
        id: column

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.topMargin: Theme.popupPadY
        anchors.leftMargin: 1
        anchors.rightMargin: 1

        Repeater {
            model: popup.entries

            delegate: MenuItemRow {
                required property var modelData

                width: column.width
                separator: modelData ? modelData.isSeparator : false
                itemEnabled: modelData ? modelData.enabled : false
                label: modelData && modelData.text ? modelData.text : ""
                icon: modelData && modelData.icon ? modelData.icon : ""
                onActivated: {
                    if (modelData)
                        modelData.triggered();
                    popup.dismissed();
                }
            }
        }
    }
}
