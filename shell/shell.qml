//@ pragma Env QSG_RENDER_LOOP=threaded
//@ pragma Env QSG_DISTANCEFIELD_ANTIALIASING=gray

// Grey antialiasing, not subpixel: subpixel put red and blue fringes on
// every glyph of the bar and the menus, and a screenshot keeps them.

import QtQuick
import Quickshell
import Quickshell.Wayland
import "components"
import "theme.js" as Theme

// The desktop shell: a wallpaper with its icons, a taskbar and a Start menu, one of each per
// screen.
//
// Every surface is a wlr-layer-shell surface. The compositor blurs what is
// behind the taskbar and the menu, which is why their fills are translucent;
// the wallpaper asks for no blur at all through the `no_blur` namespace.
ShellRoot {
    // The Start menu is reached from a keybinding, so its IPC entry point has
    // to exist before anything asks for it.
    property var ipc: StartMenuIpc
    // Likewise the lock (its IPC, logind and the idle timer) and the Control
    // Panel's IPC: singletons are only built when something names them.
    property var lockState: LockState
    property var controlPanel: ControlPanelState
    property var shutdown: ShutdownState
    // The desktop slide show changes the wallpaper on its own.
    property var slideshow: Slideshow

    // ----- lock screen ------------------------------------------------------

    // ext-session-lock: while locked, the compositor shows these surfaces and
    // nothing else, and sends them every key. If the shell dies, the session
    // stays locked.
    WlSessionLock {
        locked: LockState.locked

        WlSessionLockSurface {
            color: "#0e3a63"

            LockScreen {
                anchors.fill: parent
            }
        }
    }

    // The Control Panel, a normal window with the Aero frame.
    ControlPanel {}

    // ----- wallpaper --------------------------------------------------------

    Variants {
        model: Quickshell.screens

        Scope {
            id: desktopScope

            // The screen saver of this screen.
            ScreenSaver {
                screen: desktopScope.modelData
            }

            required property var modelData

            PanelWindow {
                id: wallpaperWindow

                screen: desktopScope.modelData

                color: "transparent"
                anchors.top: true
                anchors.bottom: true
                anchors.left: true
                anchors.right: true

                exclusionMode: ExclusionMode.Ignore
                WlrLayershell.layer: WlrLayer.Background
                WlrLayershell.namespace: "no_blur"
                // The desktop takes the keyboard when clicked, for Delete, F2,
                // Return and Ctrl+A, and holds it while an icon is renamed.
                WlrLayershell.keyboardFocus: desktopIcons.renaming || StickyNotes.editing.length > 0
                    ? WlrKeyboardFocus.Exclusive : WlrKeyboardFocus.OnDemand

                // A sticky note keeps the keyboard until the pointer goes to
                // a window or a panel: the desktop is then under it no more.
                HoverHandler {
                    onHoveredChanged: {
                        if (!hovered)
                            StickyNotes.editing = "";
                    }
                }

                // Under a picture that does not cover the screen (Fit,
                // Center), the dark blue of Windows 7's background colour.
                Rectangle {
                    anchors.fill: parent
                    color: "#1d3550"
                }

                Image {
                    anchors.fill: parent
                    source: Settings.wallpaperSource
                    // Tile and Center keep the picture's own size (0).
                    sourceSize.width: Settings.wallpaperPosition === "tile"
                        || Settings.wallpaperPosition === "center" ? 0 : parent.width
                    sourceSize.height: Settings.wallpaperPosition === "tile"
                        || Settings.wallpaperPosition === "center" ? 0 : parent.height
                    fillMode: ({
                        fill: Image.PreserveAspectCrop,
                        fit: Image.PreserveAspectFit,
                        stretch: Image.Stretch,
                        tile: Image.Tile,
                        center: Image.Pad
                    })[Settings.wallpaperPosition] ?? Image.PreserveAspectCrop
                    smooth: true
                }

                // The icons.
                Desktop {
                    id: desktopIcons
                    anchors.fill: parent
                    screenName: desktopScope.modelData.name
                }

                // The sticky notes of this screen.
                Repeater {
                    model: StickyNotes.notes.filter(function (note) {
                        return note.screen === desktopScope.modelData.name;
                    })

                    delegate: StickyNote {
                        focusOnShow: modelData.id === StickyNotes.lastAdded
                    }
                }

                // The gadgets, on the first screen. A drag moves one; it
                // keeps its distance from the right and top edges.
                GadgetFrame {
                    placeSetting: "cpuMeterPlace"
                    active: Settings.showCpuMeter
                        && desktopScope.modelData === Quickshell.screens[0]
                    sourceComponent: CpuMeter {}
                }

                GadgetFrame {
                    placeSetting: "clockGadgetPlace"
                    active: Settings.showClockGadget
                        && desktopScope.modelData === Quickshell.screens[0]
                    sourceComponent: ClockGadget {}
                }
            }

            // The desktop's right-click menu, over everything while it is
            // open. A click anywhere else on the screen closes it, as on
            // Windows 7, and so does Escape.
            PanelWindow {
                screen: desktopScope.modelData
                visible: desktopIcons.menuOpen

                color: "transparent"
                anchors.top: true
                anchors.bottom: true
                anchors.left: true
                anchors.right: true

                exclusionMode: ExclusionMode.Ignore
                WlrLayershell.layer: WlrLayer.Overlay
                WlrLayershell.namespace: "no_blur"
                WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive

                Item {
                    id: menuLayer

                    anchors.fill: parent
                    focus: true
                    Keys.onEscapePressed: desktopIcons.closeMenu()

                    MouseArea {
                        anchors.fill: parent
                        acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.MiddleButton
                        onPressed: desktopIcons.closeMenu()
                    }

                    DesktopMenu {
                        entries: desktopIcons.menuEntries
                        atX: desktopIcons.menuAt.x
                        atY: desktopIcons.menuAt.y
                        boundsWidth: menuLayer.width
                        boundsHeight: menuLayer.height
                        onDismissed: desktopIcons.closeMenu()
                    }
                }
            }
        }
    }

    // ----- taskbar ----------------------------------------------------------

    Variants {
        model: Quickshell.screens

        PanelWindow {
            id: barWindow

            required property var modelData
            screen: modelData

            color: "transparent"
            anchors.bottom: true
            anchors.left: true
            anchors.right: true
            implicitHeight: Theme.taskbarHeight

            // Auto-hide, as Windows 7's: the bar slides down to a 2 px
            // line and comes back when the pointer reaches it; windows then
            // have the whole screen.
            readonly property bool autoHide: Settings.taskbarAutoHide
            readonly property bool raised: !barWindow.autoHide || barHover.hovered
                || taskbar.busy || stayUp.running

            exclusionMode: ExclusionMode.Normal
            exclusiveZone: barWindow.autoHide ? 0 : Theme.taskbarHeight

            WlrLayershell.layer: WlrLayer.Top
            WlrLayershell.namespace: "shojiwm-win7-taskbar"
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.None

            // Only the part of the bar on screen takes the pointer.
            mask: Region { item: taskbar }

            HoverHandler {
                id: barHover
                onHoveredChanged: {
                    if (!hovered)
                        stayUp.restart();
                }
            }

            Timer {
                id: stayUp
                interval: 700
            }

            Taskbar {
                id: taskbar
                width: parent.width
                height: parent.height
                y: barWindow.raised ? 0 : parent.height - 2
                Behavior on y {
                    NumberAnimation { duration: Settings.motion(160); easing.type: Easing.OutCubic }
                }
                window: barWindow
                menuOpen: StartMenuState.openOn === barWindow.modelData.name
                onStartRequested: StartMenuState.toggle(barWindow.modelData.name)
            }
        }
    }

    // ----- the click that closes the Start menu -----------------------------
    //
    // A surface the size of the screen, under the menu and over everything
    // else, that does nothing but close the menu. It asks for no blur: there
    // is nothing of it to see.
    //
    // It stays mapped and empties its input region instead of unmapping.
    // Unmapping a layer surface while another was unmapping in the same frame
    // ended the Wayland connection of the shell ("fatal error: Invalid
    // argument"), which took the whole bar down with it.

    Variants {
        model: Quickshell.screens

        PanelWindow {
            id: dismissWindow

            required property var modelData
            screen: modelData

            readonly property bool open: StartMenuState.openOn !== ""

            color: "transparent"
            anchors.top: true
            anchors.bottom: true
            anchors.left: true
            anchors.right: true

            exclusionMode: ExclusionMode.Ignore
            WlrLayershell.layer: WlrLayer.Top
            WlrLayershell.namespace: "no_blur"
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.None

            mask: Region {
                width: dismissWindow.open ? dismissWindow.width : 0
                height: dismissWindow.open ? dismissWindow.height : 0
            }

            MouseArea {
                anchors.fill: parent
                onPressed: StartMenuState.close()
            }
        }
    }

    // ----- Start menu -------------------------------------------------------

    Variants {
        model: Quickshell.screens

        PanelWindow {
            id: menuWindow

            required property var modelData
            screen: modelData

            readonly property bool open: StartMenuState.openOn === modelData.name

            color: "transparent"
            anchors.bottom: true
            anchors.left: true
            margins.bottom: Theme.taskbarHeight + Theme.menuMargin
            margins.left: Theme.menuMargin
            implicitWidth: Theme.menuWidth
            // Room above the glass for the account picture, which stands
            // over the menu's top edge.
            implicitHeight: StartMenuState.menuHeight + Theme.menuTileOverhang

            exclusionMode: ExclusionMode.Ignore
            WlrLayershell.layer: WlrLayer.Top
            WlrLayershell.namespace: "shojiwm-win7-start"
            WlrLayershell.keyboardFocus: menuWindow.open
                ? WlrKeyboardFocus.OnDemand : WlrKeyboardFocus.None

            // Mapped at all times, empty while closed. The blur behind it is
            // clipped to what it draws, so an empty menu shows nothing.
            mask: Region {
                width: menuWindow.open ? menuWindow.width : 0
                height: menuWindow.open ? menuWindow.height : 0
            }

            StartMenu {
                id: startMenu
                anchors.fill: parent
                visible: menuWindow.open
                onDismissed: StartMenuState.close()
            }

            onOpenChanged: {
                if (menuWindow.open)
                    startMenu.reset();
            }
        }
    }

    // ----- Shut Down Windows ------------------------------------------------
    //
    // Over everything, the screen dimmed and blurred behind the dialog.

    Variants {
        model: Quickshell.screens

        PanelWindow {
            id: shutdownWindow

            required property var modelData
            screen: modelData

            readonly property bool open: ShutdownState.openOn === modelData.name

            visible: open
            color: "transparent"
            anchors.top: true
            anchors.bottom: true
            anchors.left: true
            anchors.right: true

            exclusionMode: ExclusionMode.Ignore
            WlrLayershell.layer: WlrLayer.Overlay
            WlrLayershell.namespace: "shojiwm-win7-shutdown"
            WlrLayershell.keyboardFocus: open ? WlrKeyboardFocus.Exclusive : WlrKeyboardFocus.None

            ShutdownDialog {
                id: shutdownDialog
                anchors.fill: parent
                onDismissed: ShutdownState.close()
            }

            onOpenChanged: {
                if (open) {
                    StartMenuState.close();
                    shutdownDialog.reset();
                }
            }
        }
    }

    // ----- which screen shows the menu --------------------------------------

    Connections {
        target: StartMenuIpc

        function onRequested(action, screen) {
            var name = screen.length > 0 ? screen : StartMenuState.firstScreen();
            if (action === "open")
                StartMenuState.open(name);
            else if (action === "close")
                StartMenuState.close();
            else
                StartMenuState.toggle(name);
        }
    }
}
