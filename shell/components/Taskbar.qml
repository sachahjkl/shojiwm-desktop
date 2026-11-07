pragma ComponentBehavior: Bound

import QtQuick
import ".."
import "../theme.js" as Theme

// The taskbar: the orb, the pinned programs and one button per running
// program, the notification area, the clock, and the show-desktop strip at
// the far right.
//
// It owns the three popups the buttons and the tray ask for, so there is one
// of each rather than one per button.
Item {
    id: bar

    property bool menuOpen: false
    /// The layer-shell window this bar is drawn in. Popups anchor to it.
    property var window: null

    signal startRequested

    // Whether the pointer is on the button or the icon a menu belongs to. A
    // layer-shell popup is told nothing about presses outside itself, so the
    // menus close on their own once the pointer is away from both.
    property bool pointerOnButton: false
    property bool pointerOnTray: false
    property bool pointerOnClock: false

    // What each popup is showing, or null.
    property var previewGroup: null
    property var jumpListGroup: null
    property var trayMenuItem: null
    property bool calendarOpen: false
    property bool trayOverflowOpen: false

    function closePopups() {
        bar.previewGroup = null;
        bar.jumpListGroup = null;
        bar.trayMenuItem = null;
        bar.calendarOpen = false;
        bar.trayOverflowOpen = false;
    }

    // ----- glass ------------------------------------------------------------

    Rectangle {
        anchors.fill: parent
        // Light at the top, darkest a little under half way, a slight lift
        // in the lower third: the shape a Windows 7 taskbar has.
        gradient: Gradient {
            GradientStop { position: 0.0; color: Theme.barTop }
            GradientStop { position: 0.1; color: Theme.barUpper }
            GradientStop { position: 0.45; color: Theme.barMiddle }
            GradientStop { position: 0.78; color: Theme.barLower }
            GradientStop { position: 1.0; color: Theme.barBottom }
        }
    }

    // The top edge of a Windows 7 taskbar is a dark hairline with a lit one
    // under it, in that order (measured on the reference desktop: #313030
    // then #a6a7a4).
    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        height: 1
        color: Theme.barEdge
    }
    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.topMargin: 1
        height: 1
        color: Theme.barHighlight
    }
    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        height: 1
        color: Theme.barBottomEdge
    }

    // ----- the right hand end -----------------------------------------------

    Row {
        id: status

        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        spacing: Theme.trayGap

        TrayArea {
            id: trayArea

            anchors.verticalCenter: parent.verticalCenter
            overflowOpen: bar.trayOverflowOpen

            onOverflowRequested: function (at) {
                var open = !bar.trayOverflowOpen;
                bar.closePopups();
                bar.trayOverflowOpen = open;
                trayOverflow.anchorRect = at;
            }
            onMenuRequested: function (item, at) {
                bar.previewGroup = null;
                bar.jumpListGroup = null;
                bar.trayMenuItem = bar.trayMenuItem === item ? null : item;
                trayMenu.anchorWindow = bar.window;
                trayMenu.anchorRect = at;
            }
            onHoverChanged: function (inside) {
                bar.pointerOnTray = inside;
            }
        }

        Clock {
            id: clock

            anchors.verticalCenter: parent.verticalCenter
            active: bar.calendarOpen
            onClicked: {
                var open = !bar.calendarOpen;
                bar.closePopups();
                bar.calendarOpen = open;
            }
            onHoveredChanged: bar.pointerOnClock = clock.hovered
        }

        // Show desktop: the glass strip Windows puts past the clock, a
        // dark line and a light one at its left, lit under the pointer.
        Item {
            id: showDesktop

            width: Theme.showDesktopWidth
            height: Theme.taskbarHeight

            Rectangle {
                anchors.fill: parent
                anchors.leftMargin: 2
                anchors.topMargin: 2
                gradient: Gradient {
                    GradientStop { position: 0.0; color: showDesktopArea.containsMouse ? "#80ffffff" : "#40ffffff" }
                    GradientStop { position: 0.5; color: showDesktopArea.containsMouse ? "#4dffffff" : "#1affffff" }
                    GradientStop { position: 1.0; color: showDesktopArea.containsMouse ? "#66ffffff" : "#26ffffff" }
                }
            }
            Rectangle {
                width: 1
                height: parent.height
                color: Theme.buttonOutline
            }
            Rectangle {
                x: 1
                width: 1
                height: parent.height
                color: Theme.buttonRestBorder
            }

            // Aero Peek: resting here turns every window to glass, through
            // the compositor. A click puts them away, and a second one brings
            // them back.
            Timer {
                id: peekTimer
                interval: Theme.peekDelay
                onTriggered: CompositorIpc.peek(true)
            }

            MouseArea {
                id: showDesktopArea
                anchors.fill: parent
                hoverEnabled: true
                onEntered: {
                    if (Settings.peek)
                        peekTimer.restart();
                }
                onExited: {
                    peekTimer.stop();
                    CompositorIpc.peek(false);
                }
                onClicked: {
                    peekTimer.stop();
                    bar.closePopups();
                    if (!CompositorIpc.showDesktop())
                        Apps.minimizeAll();
                }
            }
        }
    }

    // ----- the orb and the programs -----------------------------------------

    StartOrb {
        id: orb

        anchors.left: parent.left
        anchors.leftMargin: Theme.orbLeft
        anchors.verticalCenter: parent.verticalCenter
        menuOpen: bar.menuOpen
        onActivated: {
            bar.closePopups();
            bar.startRequested();
        }
    }

    // A pinned program with no window open keeps its narrow icon button; the
    // others share what is left.
    readonly property int idleCount: {
        var count = 0;
        for (var i = 0; i < Apps.groups.length; i++) {
            if (Apps.groups[i].windows.length === 0)
                count++;
        }
        return count;
    }
    readonly property int groupCount: Apps.groups.length
    readonly property int openCount: groupCount - idleCount
    readonly property real buttonsAvailable: Math.max(
        0,
        status.x - (orb.x + orb.width) - Theme.taskButtonGap * 2)
    readonly property real buttonWidth: openCount === 0 ? 0 : Math.min(
        Theme.taskButtonMaxWidth,
        Math.max(
            Theme.taskButtonMinWidth,
            (buttonsAvailable - idleCount * Theme.pinnedButtonWidth
                - (groupCount - 1) * Theme.taskButtonGap)
                / openCount))

    Row {
        id: buttons

        anchors.left: orb.right
        anchors.leftMargin: Theme.taskButtonGap * 2
        anchors.bottom: parent.bottom
        anchors.bottomMargin: (Theme.taskbarHeight - Theme.taskButtonHeight) / 2
        // Sized so a very long row is cut off instead of running under the
        // clock.
        width: bar.buttonsAvailable
        height: Theme.taskButtonHeight
        clip: true
        spacing: Theme.taskButtonGap

        Repeater {
            model: Apps.groups

            delegate: TaskButton {
                required property var modelData

                group: modelData
                width: Settings.taskbarCompact ? Theme.compactButtonWidth
                    : modelData.windows.length === 0
                        ? Theme.pinnedButtonWidth : bar.buttonWidth
                height: buttons.height

                onPreviewRequested: function (at) {
                    bar.jumpListGroup = null;
                    bar.trayMenuItem = null;
                    bar.previewGroup = modelData;
                    thumbnails.anchorRect = at;
                    hideThumbnails.stop();
                }
                onPreviewCancelled: hideThumbnails.restart()
                onHoverChanged: function (inside) {
                    bar.pointerOnButton = inside;
                }
                onMenuRequested: function (at) {
                    bar.previewGroup = null;
                    bar.trayMenuItem = null;
                    bar.jumpListGroup =
                        bar.jumpListGroup === modelData ? null : modelData;
                    jumpList.anchorRect = at;
                }
            }
        }
    }

    // ----- popups -----------------------------------------------------------

    // The thumbnails stay up while the pointer moves from the button into
    // them.
    Timer {
        id: hideThumbnails
        interval: 180
        onTriggered: {
            if (!thumbnails.pointerInside)
                bar.previewGroup = null;
            else
                hideThumbnails.restart();
        }
    }

    // A menu the pointer has left closes on its own: a layer-shell popup gets
    // no press from outside itself.
    Timer {
        id: hideMenus
        interval: 900
        running: bar.jumpListGroup !== null || bar.trayMenuItem !== null
            || bar.calendarOpen || bar.trayOverflowOpen
        repeat: true
        onTriggered: {
            if (bar.jumpListGroup !== null
                && !jumpList.pointerInside && !bar.pointerOnButton)
                bar.jumpListGroup = null;
            if (bar.trayMenuItem !== null && !trayMenu.pointerInside
                && !bar.pointerOnTray && !trayOverflow.pointerInside)
                bar.trayMenuItem = null;
            if (bar.calendarOpen
                && !calendar.pointerInside && !bar.pointerOnClock)
                bar.calendarOpen = false;
            if (bar.trayOverflowOpen && bar.trayMenuItem === null
                && !trayOverflow.pointerInside && !bar.pointerOnTray)
                bar.trayOverflowOpen = false;
        }
    }

    ThumbnailPopup {
        id: thumbnails

        anchorWindow: bar.window
        windows: bar.previewGroup ? bar.previewGroup.windows : []
        onDismissed: bar.previewGroup = null
    }

    JumpList {
        id: jumpList

        anchorWindow: bar.window
        group: bar.jumpListGroup
        onDismissed: bar.jumpListGroup = null
    }

    TrayOverflow {
        id: trayOverflow

        anchorWindow: bar.window
        items: trayArea.hidden
        open: bar.trayOverflowOpen
        onDismissed: bar.trayOverflowOpen = false
        // An icon's menu opens on the icon, in the popover, which stays
        // open under it.
        onMenuRequested: function (item, at) {
            bar.trayMenuItem = bar.trayMenuItem === item ? null : item;
            trayMenu.anchorWindow = trayOverflow;
            trayMenu.anchorRect = at;
        }
    }

    CalendarPopup {
        id: calendar

        anchorItem: clock
        open: bar.calendarOpen
    }

    TrayMenu {
        id: trayMenu

        anchorWindow: bar.window
        item: bar.trayMenuItem
        // A choice made in the menu of a hidden icon closes the popover too.
        onDismissed: {
            if (trayMenu.anchorWindow === trayOverflow)
                bar.trayOverflowOpen = false;
            bar.trayMenuItem = null;
        }
    }
}
