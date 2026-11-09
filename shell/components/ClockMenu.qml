pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import ".."
import "../theme.js" as Theme

// The menu of the clock, on a right click: the calendar, the date and time
// settings, the desktop and the task manager.
PopupWindow {
    id: popup

    property bool open: false
    property var anchorWindow: null
    property rect anchorRect: Qt.rect(0, 0, 1, 1)

    signal dismissed
    signal calendarRequested

    color: "transparent"
    visible: popup.open

    anchor.window: popup.anchorWindow
    anchor.rect: popup.anchorRect
    anchor.edges: Edges.Top
    anchor.gravity: Edges.Top
    anchor.margins.bottom: Theme.popupGap
    anchor.adjustment: PopupAdjustment.SlideX | PopupAdjustment.FlipY

    implicitWidth: Math.min(
        Theme.popupMaxWidth,
        Math.max(Theme.popupMinWidth, metrics.advanceWidth("Adjust date/time")
            + Theme.popupPadX * 2 + 24))
    implicitHeight: column.implicitHeight + Theme.popupPadY * 2 + 2

    FontMetrics {
        id: metrics
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
    }

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
        radius: Math.max(0, Scheme.popupRadius - 1)
        color: "transparent"
        border.width: 1
        border.color: Scheme.popupInnerLine
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
            label: "Open the calendar"
            icon: Qt.resolvedUrl("../assets/icons/calendar.png")
            onActivated: {
                popup.dismissed();
                popup.calendarRequested();
            }
        }

        MenuItemRow {
            width: column.width
            label: "Adjust date/time"
            onActivated: {
                popup.dismissed();
                ControlPanelState.open("clock");
            }
        }

        MenuItemRow {
            width: column.width
            separator: true
        }

        MenuItemRow {
            width: column.width
            label: "Show the desktop"
            onActivated: {
                popup.dismissed();
                CompositorIpc.showDesktop();
            }
        }

        MenuItemRow {
            width: column.width
            label: "Start Task Manager"
            onActivated: {
                popup.dismissed();
                Quickshell.execDetached(["sh", "-c", Apps.taskManager]);
            }
        }
    }
}
