pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import ".."

// The taskbar: the orb, the pinned programs and one button per running
// program, the notification area, the clock, and the show-desktop strip at
// the far right.
//
// It owns the three popups the buttons and the tray ask for, so there is one
// of each rather than one per button.
Item {
    id: bar

    property bool menuOpen: false
    /// A right click on the Start orb: its menu (AdminMenu, shell.qml).
    signal adminMenuRequested
    /// The layer-shell window this bar is drawn in. Popups anchor to it.
    property var window: null

    signal startRequested


    // What each popup is showing, or null.
    property var previewGroup: null
    property var jumpListGroup: null
    property var trayMenuItem: null
    property bool calendarOpen: false
    property bool volumeOpen: false
    property bool networkOpen: false
    property bool bluetoothOpen: false
    property bool notificationsOpen: false
    property bool actionCenterOpen: false
    /// One of the bar's menus or panels is open: a click outside closes it
    /// (shell.qml).
    readonly property bool popupOpen: bar.jumpListGroup !== null || bar.trayMenuItem !== null
        || bar.calendarOpen || bar.contextMenuKey.length > 0 || bar.trayOverflowOpen
        || bar.volumeOpen || bar.networkOpen || bar.bluetoothOpen || bar.notificationsOpen
        || bar.actionCenterOpen
    /// A menu, a panel or the thumbnails of the bar are open: an auto-hidden
    /// bar stays up meanwhile.
    readonly property bool busy: bar.menuOpen || bar.previewGroup !== null || bar.popupOpen
    /// Which right-click menu is open (contextEntries): "bell", "volume",
    /// "network", "flag", "bluetooth", "clock", "desktop", "taskbar"; "" for
    /// none.
    property string contextMenuKey: ""
    property bool trayOverflowOpen: false

    // A notification-area icon dropped anywhere on the taskbar shows there:
    // on the icons, at that place among them, which reorders them; elsewhere,
    // at the end. Under everything else: the arrow's own drop area, which
    // hides an icon, sits above it.
    DropArea {
        z: -1
        anchors.fill: parent
        keys: [Settings.trayDragType]

        /// Where a drop at this point falls in the tray area: left of the
        /// icons (the arrow has its own drop area) is the first place, right
        /// of them the last.
        function trayX(event) {
            var at = trayArea.mapFromItem(bar, event.x, event.y);
            return Math.max(0, Math.min(trayArea.width, at.x));
        }

        function preview(event) {
            trayArea.previewId = DragState.kind === "tray" ? String(DragState.data) : "";
            trayArea.previewBefore = trayArea.indexAt(trayX(event));
        }

        onEntered: function (event) { preview(event); }
        onPositionChanged: function (event) { preview(event); }
        onExited: trayArea.previewBefore = -1
        onDropped: function (event) {
            trayArea.previewBefore = -1;
            var id = DragState.kind === "tray" ? String(DragState.data) : "";
            if (id.length === 0)
                return;
            var x = trayX(event);
            event.acceptProposedAction();
            // After the drag has ended: the change removes its source.
            Qt.callLater(function () { trayArea.placeAt(id, x); });
        }
    }

    function closePopups() {
        bar.previewGroup = null;
        bar.jumpListGroup = null;
        bar.trayMenuItem = null;
        bar.calendarOpen = false;
        bar.volumeOpen = false;
        bar.networkOpen = false;
        bar.bluetoothOpen = false;
        bar.notificationsOpen = false;
        bar.actionCenterOpen = false;
        bar.contextMenuKey = "";
        bar.trayOverflowOpen = false;
    }

    function showTheDesktop() {
        if (!CompositorIpc.showDesktop())
            Apps.minimizeAll();
    }

    /// Opens the right-click menu `key` above `at`, or closes it when it is
    /// the one open.
    function openContextMenu(key, at) {
        var again = bar.contextMenuKey === key;
        bar.closePopups();
        if (again)
            return;
        contextMenu.entries = bar.contextEntries(key);
        contextMenu.anchorRect = at;
        bar.contextMenuKey = key;
    }

    /// The lines of each right-click menu (ContextMenu), as Windows 7 has
    /// them where it can, read when the menu opens.
    function contextEntries(key) {
        var separator = { separator: true };
        var showDesktop = { label: "Show the desktop", run: bar.showTheDesktop };
        var taskManager = { label: "Start Task Manager", run: function () {
            Quickshell.execDetached(["sh", "-c", Apps.taskManager]);
        } };
        var iconsPage = { label: "Customize notification icons", run: function () {
            ControlPanelState.open("notifications");
        } };
        switch (key) {
        case "bell":
            return [
                { label: "Open notifications", run: function () { bar.notificationsOpen = true; } },
                { label: "Clear all", enabled: NotificationHistory.entries.length > 0,
                  run: NotificationHistory.clear },
                separator,
                { label: "Do not disturb", checked: NotificationHistory.doNotDisturb,
                  run: function () { NotificationHistory.setDoNotDisturb(!NotificationHistory.doNotDisturb); } },
                separator,
                iconsPage
            ];
        case "volume":
            return [
                { label: "Open Volume Mixer", run: Audio.openMixer },
                { label: "Playback devices", run: function () {
                    bar.volumeOpen = true;
                    volumePopup.devicesOpen = true;
                } },
                { label: "Mute", checked: Audio.muted, enabled: Audio.ready, run: Audio.toggleMute },
                separator,
                { label: "Sound settings", run: function () { ControlPanelState.open("sound"); } }
            ];
        case "network":
            return [
                { label: "Open the network panel", run: function () { bar.networkOpen = true; } },
                { label: "Wi-Fi", checked: Network.wifiEnabled, enabled: Network.hasWifi,
                  run: function () { Network.setWifi(!Network.wifiEnabled); } },
                separator,
                { label: "Network connections", run: Network.openEditor }
            ];
        case "flag":
            return [
                { label: "Open Action Center", run: function () { bar.actionCenterOpen = true; } },
                separator,
                { label: "Control Panel", run: function () { ControlPanelState.open("home"); } },
                iconsPage
            ];
        case "bluetooth":
            return [
                { label: "Open the Bluetooth panel", run: function () { bar.bluetoothOpen = true; } },
                { label: "Bluetooth settings", run: function () {
                    Quickshell.execDetached(["blueman-manager"]);
                } }
            ];
        case "clock":
            return [
                { label: "Open the calendar", icon: Qt.resolvedUrl("../assets/icons/calendar.png"),
                  run: function () { bar.calendarOpen = true; } },
                { label: "Adjust date/time", run: function () { ControlPanelState.open("clock"); } },
                separator,
                showDesktop,
                taskManager
            ];
        case "desktop":
            return [
                showDesktop,
                { label: "Peek at desktop", checked: Settings.peek,
                  run: function () { Settings.peek = !Settings.peek; } }
            ];
        case "taskbar":
            return [
                showDesktop,
                taskManager,
                separator,
                { label: "Properties", run: function () { ControlPanelState.open("taskbar"); } }
            ];
        }
        return [];
    }

    // ----- glass ------------------------------------------------------------

    Rectangle {
        anchors.fill: parent
        // Light at the top, darkest a little under half way, a slight lift
        // in the lower third: the shape a Windows 7 taskbar has.
        gradient: Gradient {
            GradientStop { position: 0.0; color: Scheme.theme.barTop }
            GradientStop { position: 0.1; color: Scheme.theme.barUpper }
            GradientStop { position: 0.45; color: Scheme.theme.barMiddle }
            GradientStop { position: 0.78; color: Scheme.theme.barLower }
            GradientStop { position: 1.0; color: Scheme.theme.barBottom }
        }
    }

    // The colour of the glass chosen in the Control Panel, over it.
    Rectangle {
        anchors.fill: parent
        visible: Settings.glassColor.length > 0
        color: Settings.glassColor.length > 0 ? Settings.glassColor : "transparent"
        opacity: Settings.glassColorStrength
    }

    // The edge of the bar towards the desktop is a dark hairline with a lit
    // one inside it, in that order (on Windows 7's taskbar, measured on the
    // reference desktop: #313030 then #a6a7a4). Its edge at the screen's
    // side has a dark one. A bar at the top has them the other way up.
    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        y: Scheme.barAtTop ? parent.height - 1 : 0
        height: 1
        color: Scheme.theme.barEdge
    }
    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        y: Scheme.barAtTop ? parent.height - 2 : 1
        height: 1
        color: Scheme.theme.barHighlight
    }
    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        y: Scheme.barAtTop ? 0 : parent.height - 1
        height: 1
        color: Scheme.theme.barBottomEdge
    }

    // ----- the right hand end -----------------------------------------------

    Row {
        id: status

        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        spacing: Scheme.theme.trayGap

        TrayArea {
            id: trayArea

            anchors.verticalCenter: parent.verticalCenter
            overflowOpen: bar.trayOverflowOpen

            onOverflowRequested: function (at) {
                var open = !bar.trayOverflowOpen;
                bar.closePopups();
                trayOverflow.anchorRect = at;
                bar.trayOverflowOpen = open;
            }
            onMenuRequested: function (item, at) {
                bar.previewGroup = null;
                bar.jumpListGroup = null;
                trayMenu.anchorWindow = bar.window;
                trayMenu.anchorRect = at;
                bar.trayMenuItem = bar.trayMenuItem === item ? null : item;
            }
        }

        ActionCenterIcon {
            id: actionCenterIcon

            anchors.verticalCenter: parent.verticalCenter
            onMenuRequested: function (at) { bar.openContextMenu("flag", at); }
            active: bar.actionCenterOpen
            onClicked: {
                var open = !bar.actionCenterOpen;
                bar.closePopups();
                bar.actionCenterOpen = open;
            }
        }

        NotificationIcon {
            id: notificationIcon

            anchors.verticalCenter: parent.verticalCenter
            onMenuRequested: function (at) { bar.openContextMenu("bell", at); }
            active: bar.notificationsOpen
            onClicked: {
                var open = !bar.notificationsOpen;
                bar.closePopups();
                bar.notificationsOpen = open;
            }
        }

        BluetoothIcon {
            id: bluetoothIcon

            anchors.verticalCenter: parent.verticalCenter
            onMenuRequested: function (at) { bar.openContextMenu("bluetooth", at); }
            active: bar.bluetoothOpen
            onClicked: {
                var open = !bar.bluetoothOpen;
                bar.closePopups();
                bar.bluetoothOpen = open;
            }
        }

        NetworkIcon {
            id: networkIcon

            anchors.verticalCenter: parent.verticalCenter
            onMenuRequested: function (at) { bar.openContextMenu("network", at); }
            active: bar.networkOpen
            onClicked: {
                var open = !bar.networkOpen;
                bar.closePopups();
                bar.networkOpen = open;
            }
        }

        VolumeIcon {
            id: volumeIcon

            anchors.verticalCenter: parent.verticalCenter
            onMenuRequested: function (at) { bar.openContextMenu("volume", at); }
            active: bar.volumeOpen
            onClicked: {
                var open = !bar.volumeOpen;
                bar.closePopups();
                bar.volumeOpen = open;
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
            onMenuRequested: function (at) { bar.openContextMenu("clock", at); }
        }

        // Show desktop: the glass strip Windows puts past the clock, a
        // dark line and a light one at its left, lit under the pointer.
        Item {
            id: showDesktop

            width: Scheme.theme.showDesktopWidth
            height: Scheme.theme.taskbarHeight

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
                color: Scheme.theme.buttonOutline
            }
            Rectangle {
                x: 1
                width: 1
                height: parent.height
                color: Scheme.theme.buttonRestBorder
            }

            // Aero Peek: resting here turns every window to glass, through
            // the compositor. A click puts them away, and a second one brings
            // them back.
            Timer {
                id: peekTimer
                interval: Scheme.theme.peekDelay
                onTriggered: CompositorIpc.peek(true)
            }

            MouseArea {
                id: showDesktopArea
                anchors.fill: parent
                hoverEnabled: true
                acceptedButtons: Qt.LeftButton | Qt.RightButton
                onEntered: {
                    if (Settings.peek)
                        peekTimer.restart();
                }
                onExited: {
                    peekTimer.stop();
                    CompositorIpc.peek(false);
                }
                onClicked: function (event) {
                    peekTimer.stop();
                    if (event.button === Qt.RightButton) {
                        var point = showDesktop.mapToItem(null, 0, 0);
                        bar.openContextMenu("desktop",
                            Qt.rect(point.x, point.y, showDesktop.width, showDesktop.height));
                        return;
                    }
                    bar.closePopups();
                    bar.showTheDesktop();
                }
            }
        }
    }

    // A right click on the bar where nothing else is: its menu. Under
    // everything else on the bar (z), so the orb, the buttons and the
    // notification area keep their own clicks.
    MouseArea {
        z: -1
        anchors.fill: parent
        acceptedButtons: Qt.RightButton
        onClicked: function (event) {
            bar.openContextMenu("taskbar", Qt.rect(event.x, 0, 1, bar.height));
        }
    }

    // ----- the orb and the programs -----------------------------------------

    // The Start button: the theme's own (Aero's orb, Kernel's launcher). Both
    // have the same interface: menuOpen, activated, menuRequested.
    Loader {
        id: orb

        anchors.left: parent.left
        anchors.leftMargin: Scheme.theme.orbLeft
        anchors.verticalCenter: parent.verticalCenter
        sourceComponent: Scheme.kernel ? kernelLauncher : aeroOrb
    }

    Component {
        id: aeroOrb
        StartOrb {}
    }

    Component {
        id: kernelLauncher
        KernelLauncher {}
    }

    Binding {
        target: orb.item
        property: "menuOpen"
        value: bar.menuOpen
        when: orb.item !== null
    }

    Connections {
        target: orb.item
        function onActivated() {
            bar.closePopups();
            bar.startRequested();
        }
        function onMenuRequested() {
            bar.closePopups();
            bar.adminMenuRequested();
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
        status.x - (orb.x + orb.width) - Scheme.theme.taskButtonGap * 2)
    readonly property real buttonWidth: openCount === 0 ? 0 : Math.min(
        Scheme.theme.taskButtonMaxWidth,
        Math.max(
            Scheme.theme.taskButtonMinWidth,
            (buttonsAvailable - idleCount * Scheme.theme.pinnedButtonWidth
                - (groupCount - 1) * Scheme.theme.taskButtonGap)
                / openCount))

    Row {
        id: buttons

        anchors.left: orb.right
        anchors.leftMargin: Scheme.theme.taskButtonGap * 2
        anchors.bottom: parent.bottom
        anchors.bottomMargin: (Scheme.theme.taskbarHeight - Scheme.theme.taskButtonHeight) / 2
        // Sized so a very long row is cut off instead of running under the
        // clock.
        width: bar.buttonsAvailable
        height: Scheme.theme.taskButtonHeight
        clip: true
        spacing: Scheme.theme.taskButtonGap

        Repeater {
            id: buttonsRepeater

            model: Apps.groups

            delegate: TaskButton {
                required property var modelData
                required property int index

                group: modelData
                shift: bar.shiftOf(index)
                dragMin: -x
                dragMax: buttonsRepeater.count > 0 && buttonsRepeater.itemAt(buttonsRepeater.count - 1)
                    ? buttonsRepeater.itemAt(buttonsRepeater.count - 1).x
                        + buttonsRepeater.itemAt(buttonsRepeater.count - 1).width - x - width
                    : 0
                onReorderMoved: function (centre) {
                    bar.dragFrom = index;
                    programDrop.place(centre);
                }
                onReorderEnded: {
                    var key = modelData.key;
                    var before = bar.dropBefore;
                    bar.dragFrom = -1;
                    bar.dropBefore = -1;
                    // Once this button has finished with the release: the
                    // change rebuilds every button.
                    if (before >= 0)
                        Qt.callLater(function () { Apps.moveButton(key, before); });
                }
                width: Settings.taskbarCompact ? Scheme.theme.compactButtonWidth
                    : modelData.windows.length === 0
                        ? Scheme.theme.pinnedButtonWidth : bar.buttonWidth
                height: buttons.height

                onPreviewRequested: function (at) {
                    bar.jumpListGroup = null;
                    bar.trayMenuItem = null;
                    thumbnails.anchorRect = at;
                    bar.previewGroup = modelData;
                    hideThumbnails.stop();
                }
                onPreviewCancelled: hideThumbnails.restart()
                onMenuRequested: function (at) {
                    bar.previewGroup = null;
                    bar.trayMenuItem = null;
                    // The place first: a popup shown before it would open at
                    // its last place and then move.
                    jumpList.anchorRect = at;
                    bar.jumpListGroup =
                        bar.jumpListGroup === modelData ? null : modelData;
                }
            }
        }
    }

    // A button dragged along the bar to a new place, or a program dropped on
    // it from the Start menu or a .desktop file, pinned where it lands. The
    // other buttons stand aside meanwhile, as Windows 7 shows it.

    /// The index of the button being dragged, or -1.
    property int dragFrom: -1
    /// The button the drop goes before (the count of buttons for the end),
    /// or -1 while nothing is dragged over the buttons.
    property int dropBefore: -1
    /// The room a dragged button takes.
    readonly property real dragSlot: (bar.dragFrom >= 0 && buttonsRepeater.itemAt(bar.dragFrom)
        ? buttonsRepeater.itemAt(bar.dragFrom).width : Scheme.theme.pinnedButtonWidth) + Scheme.theme.taskButtonGap

    /// How far the button at `index` stands aside.
    function shiftOf(index) {
        var to = bar.dropBefore;
        var from = bar.dragFrom;
        if (to < 0)
            return 0;
        if (from < 0)
            return index >= to ? bar.dragSlot : 0;
        if (from < to && index > from && index < to)
            return -bar.dragSlot;
        if (to <= index && index < from)
            return bar.dragSlot;
        return 0;
    }

    DropArea {
        id: programDrop

        /// The button the pointer is before: the first whose middle is right
        /// of it, from the places the buttons had before anything moved.
        function place(x) {
            // A button dragged left goes before a button whose middle it
            // reaches; dragged right, after it. At the ends of the bar the
            // middles meet exactly, so the tie decides.
            var from = bar.dragFrom >= 0 ? buttonsRepeater.itemAt(bar.dragFrom) : null;
            var leftward = from !== null && x < from.x + from.width / 2;
            var before = buttonsRepeater.count;
            for (var i = 0; i < buttonsRepeater.count; i++) {
                var item = buttonsRepeater.itemAt(i);
                if (!item)
                    continue;
                var middle = item.x + item.width / 2;
                if (leftward ? x <= middle : x < middle) {
                    before = i;
                    break;
                }
            }
            bar.dropBefore = before;
        }

        anchors.fill: buttons
        keys: [Settings.programDragType, "text/uri-list"]

        onEntered: function (event) { programDrop.place(event.x); }
        onPositionChanged: function (event) { programDrop.place(event.x); }
        onExited: bar.dropBefore = -1
        onDropped: function (event) {
            var before = bar.dropBefore;
            bar.dropBefore = -1;
            var id = Apps.droppedProgramId(event);
            if (id.length === 0)
                return;
            event.accept(Qt.CopyAction);
            Qt.callLater(function () { Apps.pinDroppedAt(id, before); });
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

    ContextMenu {
        id: contextMenu

        anchorWindow: bar.window
        open: bar.contextMenuKey.length > 0
        onDismissed: bar.contextMenuKey = ""
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
            trayMenu.anchorWindow = trayOverflow;
            trayMenu.anchorRect = at;
            bar.trayMenuItem = bar.trayMenuItem === item ? null : item;
        }
    }

    ActionCenterPanel {
        id: actionCenterPanel

        anchorItem: actionCenterIcon
        open: bar.actionCenterOpen
        onDismissed: bar.actionCenterOpen = false
    }

    Connections {
        target: Maintenance
        function onPanelRequested(open) {
            bar.closePopups();
            bar.actionCenterOpen = open;
        }
    }

    NotificationPanel {
        id: notificationPanel

        anchorItem: notificationIcon
        open: bar.notificationsOpen
        onDismissed: bar.notificationsOpen = false
    }

    BluetoothPopup {
        id: bluetoothPopup

        anchorItem: bluetoothIcon
        open: bar.bluetoothOpen
        onDismissed: bar.bluetoothOpen = false
    }

    NetworkPopup {
        id: networkPopup

        anchorItem: networkIcon
        open: bar.networkOpen
        onDismissed: bar.networkOpen = false
    }

    // The Bluetooth panel asked over IPC (quickshell ipc call bluetooth …).
    Connections {
        target: StartMenuIpc
        function onBluetoothRequested(screen, open) {
            var mine = bar.window && bar.window.screen
                && (screen.length > 0 ? bar.window.screen.name === screen
                                      : Screens.isPrimary(bar.window.screen));
            if (!open)
                bar.bluetoothOpen = false;
            else if (mine) {
                bar.closePopups();
                bar.bluetoothOpen = true;
            }
        }
    }

    // The volume panel asked over IPC (quickshell ipc call volume …).
    Connections {
        target: Audio
        function onPanelRequested(screen, devices) {
            if (screen === "-") {
                bar.volumeOpen = false;
                return;
            }
            bar.closePopups();
            bar.volumeOpen = true;
            volumePopup.devicesOpen = devices;
        }
    }

    VolumePopup {
        id: volumePopup

        anchorItem: volumeIcon
        open: bar.volumeOpen
        onDismissed: bar.volumeOpen = false
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
