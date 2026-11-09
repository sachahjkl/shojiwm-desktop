pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import ".."
import "../theme.js" as Theme

// The popover of hidden notification-area icons, over the arrow, as Windows 7
// shows it: the icons in a grid, and "Customize..." under them, which opens
// the Control Panel's page for them.
PopupWindow {
    id: popup

    property var items: []
    property bool open: false
    /// One of the icons here is being dragged.
    property bool dragging: false
    property var anchorWindow: null
    property rect anchorRect: Qt.rect(0, 0, 1, 1)

    signal dismissed
    /// An icon's menu, at the icon: `at` is in this popover's coordinates.
    signal menuRequested(var item, rect at)

    readonly property int columns: Math.max(1, Math.min(4, items.length))
    readonly property int cell: Theme.trayIconSize + 12

    color: "transparent"
    visible: popup.open && popup.items.length > 0

    anchor.window: popup.anchorWindow
    anchor.rect: popup.anchorRect
    anchor.edges: Edges.Top
    anchor.gravity: Edges.Top
    anchor.margins.bottom: Theme.popupGap + 2
    anchor.adjustment: PopupAdjustment.SlideX | PopupAdjustment.FlipY

    implicitWidth: Math.max(popup.columns * popup.cell + 16, customize.implicitWidth + 24)
    implicitHeight: column.implicitHeight + 16

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

    // The light line inside the outline, as every Windows 7 menu has.
    Rectangle {
        anchors.fill: parent
        anchors.margins: 1
        radius: Scheme.corner(Math.max(0, Scheme.popupRadius - 1))
        color: "transparent"
        border.width: 1
        border.color: Scheme.popupInnerLine
    }

    // A hover handler, not a MouseArea: it sees the pointer over the rows
    // and icons too, which take the hover from a MouseArea under them.

    // An icon dragged here from the taskbar is hidden.
    DropArea {
        anchors.fill: parent
        keys: [Settings.trayDragType]

        onDropped: function (event) {
            var id = DragState.kind === "tray" ? String(DragState.data) : "";
            if (id.length === 0)
                return;
            event.acceptProposedAction();
            // After the drag has ended: the change can remove its source.
            Qt.callLater(function () { Settings.setTrayVisible(id, false); });
        }
    }

    Column {
        id: column

        anchors.top: parent.top
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.topMargin: 8
        spacing: 6

        Grid {
            anchors.horizontalCenter: parent.horizontalCenter
            columns: popup.columns

            Repeater {
                model: popup.items

                delegate: Item {
                    id: slot

                    required property var modelData

                    width: popup.cell
                    height: popup.cell

                    TrayIcon {
                        anchors.centerIn: parent
                        item: slot.modelData
                        hoverColor: Scheme.selection
                        onMenuRequested: function (item, at) { popup.menuRequested(item, at); }
                        onUsed: popup.dismissed()
                        onDraggingChanged: popup.dragging = dragging
                    }
                }
            }
        }

        Rectangle {
            width: popup.implicitWidth - 16
            height: 1
            color: Scheme.menuDivider
        }

        CpLink {
            id: customize
            anchors.horizontalCenter: parent.horizontalCenter
            text: "Customize..."
            onClicked: {
                popup.dismissed();
                ControlPanelState.open("notifications");
            }
        }
    }
}
