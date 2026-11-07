pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import ".."
import "../theme.js" as Theme

// The jump list of a taskbar button, on a right click.
//
// From the top: the files the program opened last, from
// `recently-used.xbel`; its tasks, which are the actions of its .desktop file
// (`DesktopEntry.actions`, each with `name`, `icon` and `execute()`); then
// the program itself and "Close window".
PopupWindow {
    id: popup

    property var group: null
    property var anchorWindow: null
    property rect anchorRect: Qt.rect(0, 0, 1, 1)

    signal dismissed

    readonly property var entry: group ? group.entry : null
    readonly property var actions: entry && entry.actions ? entry.actions : []
    readonly property int windowCount: group ? group.windows.length : 0
    readonly property var recent: popup.visible ? Apps.recentFor(entry) : []

    color: "transparent"
    visible: group !== null

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

    FontMetrics {
        id: metrics
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
    }

    readonly property real widest: {
        var texts = [popup.closeLabel, popup.openLabel];
        for (var i = 0; i < actions.length; i++)
            texts.push(String(actions[i].name));
        for (var k = 0; k < recent.length; k++)
            texts.push(String(recent[k].name));
        var width = 0;
        for (var j = 0; j < texts.length; j++)
            width = Math.max(width, metrics.advanceWidth(texts[j]));
        return width;
    }

    readonly property string openLabel: popup.group && popup.group.name
        ? popup.group.name : "Open"
    readonly property string closeLabel: popup.windowCount > 1
        ? "Close all windows" : "Close window"

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

        MenuItemRow {
            width: column.width
            heading: true
            label: "Recent"
            visible: popup.recent.length > 0
        }

        Repeater {
            model: popup.recent

            delegate: MenuItemRow {
                required property var modelData

                width: column.width
                label: modelData ? modelData.name : ""
                icon: Quickshell.iconPath("text-x-generic", true)
                onActivated: {
                    Apps.openWith(popup.entry, modelData.path);
                    popup.dismissed();
                }
            }
        }

        MenuItemRow {
            width: column.width
            heading: true
            label: "Tasks"
            visible: popup.actions.length > 0
        }

        Repeater {
            model: popup.actions

            delegate: MenuItemRow {
                required property var modelData

                width: column.width
                label: modelData ? String(modelData.name) : ""
                icon: modelData && modelData.icon
                    ? Quickshell.iconPath(modelData.icon, true) : ""
                onActivated: {
                    if (modelData)
                        modelData.execute();
                    popup.dismissed();
                }
            }
        }

        MenuItemRow {
            width: column.width
            separator: true
            visible: popup.actions.length > 0 || popup.recent.length > 0
        }

        MenuItemRow {
            width: column.width
            label: popup.openLabel
            icon: popup.group ? popup.group.icon : ""
            itemEnabled: popup.entry !== null
            onActivated: {
                Apps.launch(popup.entry);
                popup.dismissed();
            }
        }

        MenuItemRow {
            width: column.width
            separator: true
            visible: popup.windowCount > 0
        }

        MenuItemRow {
            width: column.width
            label: popup.closeLabel
            visible: popup.windowCount > 0
            onActivated: {
                var open = popup.group ? popup.group.windows.slice() : [];
                for (var i = 0; i < open.length; i++)
                    open[i].close();
                popup.dismissed();
            }
        }
    }
}
