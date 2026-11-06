pragma ComponentBehavior: Bound

import QtQuick
import ".."
import "../theme.js" as Theme

// The taskbar: the orb, one button per window, the notification area, the
// clock, and the show-desktop strip at the far right.
Item {
    id: bar

    property bool menuOpen: false
    signal startRequested

    // ----- glass ------------------------------------------------------------

    Rectangle {
        anchors.fill: parent
        gradient: Gradient {
            GradientStop { position: 0.0; color: Theme.barTop }
            GradientStop { position: 0.12; color: Theme.barMiddle }
            GradientStop { position: 0.92; color: Theme.barMiddle }
            GradientStop { position: 1.0; color: Theme.barBottom }
        }
    }

    // The lit top edge, and the hairline under it.
    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        height: 1
        color: Theme.barHighlight
    }
    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        height: 1
        color: Theme.barEdge
    }

    // ----- the right hand end -----------------------------------------------

    Row {
        id: status

        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        anchors.rightMargin: 0
        spacing: Theme.trayGap

        TrayArea {
            anchors.verticalCenter: parent.verticalCenter
        }

        Rectangle {
            width: 1
            height: Theme.taskbarHeight - 12
            color: Theme.barHighlight
            opacity: 0.5
            anchors.verticalCenter: parent.verticalCenter
        }

        Clock {
            anchors.verticalCenter: parent.verticalCenter
        }

        // Show desktop: the strip Windows puts past the clock.
        Rectangle {
            id: showDesktop

            width: Theme.showDesktopWidth
            height: Theme.taskbarHeight
            color: "#ffffff"
            opacity: showDesktopArea.containsMouse ? 0.22 : 0.06

            Behavior on opacity {
                NumberAnimation { duration: Theme.fade }
            }

            Rectangle {
                anchors.left: parent.left
                width: 1
                height: parent.height
                color: Theme.barHighlight
                opacity: 0.6
            }

            MouseArea {
                id: showDesktopArea
                anchors.fill: parent
                hoverEnabled: true
                onClicked: Apps.minimizeAll()
            }
        }
    }

    // ----- the orb and the windows ------------------------------------------

    StartOrb {
        id: orb

        anchors.left: parent.left
        anchors.leftMargin: Theme.taskbarPadding
        anchors.verticalCenter: parent.verticalCenter
        menuOpen: bar.menuOpen
        onActivated: bar.startRequested()
    }

    readonly property int windowCount: Apps.windows.length
    readonly property real buttonsAvailable: Math.max(
        0,
        status.x - (orb.x + orb.width) - Theme.taskButtonGap * 2)
    readonly property real buttonWidth: windowCount === 0 ? 0 : Math.min(
        Theme.taskButtonMaxWidth,
        Math.max(
            Theme.taskButtonMinWidth,
            (buttonsAvailable - (windowCount - 1) * Theme.taskButtonGap)
                / windowCount))

    Row {
        id: buttons

        anchors.left: orb.right
        anchors.leftMargin: Theme.taskButtonGap * 2
        anchors.verticalCenter: parent.verticalCenter
        spacing: Theme.taskButtonGap

        Repeater {
            model: Apps.windows

            delegate: TaskButton {
                required property var modelData
                toplevel: modelData
                width: bar.buttonWidth
            }
        }
    }
}
