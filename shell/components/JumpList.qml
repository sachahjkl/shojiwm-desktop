pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import ".."
import "../theme.js" as Theme

// The jump list of a taskbar button, on a right click.
//
// Its tasks are the actions of the program's .desktop file, which Quickshell
// exposes as `DesktopEntry.actions` (a list of `DesktopAction`, each with
// `name`, `icon` and `execute()`).
PopupWindow {
    id: popup

    property var group: null
    property var anchorWindow: null
    property rect anchorRect: Qt.rect(0, 0, 1, 1)

    signal dismissed

    readonly property var entry: group ? group.entry : null
    readonly property var actions: entry && entry.actions ? entry.actions : []
    readonly property int windowCount: group ? group.windows.length : 0

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
            visible: popup.actions.length > 0
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
        }

        MenuItemRow {
            width: column.width
            label: popup.closeLabel
            itemEnabled: popup.windowCount > 0
            onActivated: {
                var open = popup.group ? popup.group.windows.slice() : [];
                for (var i = 0; i < open.length; i++)
                    open[i].close();
                popup.dismissed();
            }
        }
    }
}
