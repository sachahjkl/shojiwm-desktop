pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import ".."
import "../theme.js" as Theme

// What the arrow beside the Start menu's power button opens: log off, lock,
// restart, sleep, hibernate. The list and the commands are in Apps.qml.
PopupWindow {
    id: popup

    /// The arrow it opens from.
    property Item anchorItem: null
    property bool open: false

    signal dismissed

    // Every action but the one the button itself runs.
    readonly property var actions: Apps.powerActions.filter(function (action) {
        return action.id !== Apps.powerButtonAction;
    })

    color: "transparent"
    visible: popup.open && popup.anchorItem !== null

    anchor.item: popup.anchorItem
    anchor.rect: Qt.rect(0, 0,
                         popup.anchorItem ? popup.anchorItem.width : 1,
                         popup.anchorItem ? popup.anchorItem.height : 1)
    // To the right of the arrow, growing upwards from its bottom, as on
    // Windows; flipped when the screen ends.
    anchor.edges: Edges.Right | Edges.Bottom
    anchor.gravity: Edges.Right | Edges.Top
    anchor.margins.left: Theme.popupGap
    anchor.adjustment: PopupAdjustment.FlipX | PopupAdjustment.SlideY

    implicitWidth: Math.max(Theme.popupMinWidth - 30,
                            widest + Theme.popupPadX * 2 + 30)
    implicitHeight: column.implicitHeight + Theme.popupPadY * 2 + 2

    FontMetrics {
        id: metrics
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
    }

    readonly property real widest: {
        var width = 0;
        for (var i = 0; i < actions.length; i++)
            width = Math.max(width, metrics.advanceWidth(actions[i].label || ""));
        return width;
    }

    /// `true` while the pointer is over the menu.
    readonly property bool pointerInside: hoverArea.containsMouse

    Rectangle {
        anchors.fill: parent
        radius: Scheme.popupRadius
        border.width: 1
        border.color: Scheme.popupOutline
        gradient: Gradient {
            GradientStop { position: 0.0; color: Scheme.popupTop }
            GradientStop { position: 1.0; color: Scheme.popupBottom }
        }
    }

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
                separator: modelData.id === "separator"
                label: modelData.label || ""
                onActivated: {
                    popup.dismissed();
                    Apps.runPower(modelData.id);
                }
            }
        }
    }
}
