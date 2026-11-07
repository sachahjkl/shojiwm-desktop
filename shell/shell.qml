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
                // The desktop takes the keyboard only to rename an icon.
                WlrLayershell.keyboardFocus: desktopIcons.renaming
                    ? WlrKeyboardFocus.Exclusive : WlrKeyboardFocus.None

                Image {
                    anchors.fill: parent
                    source: Settings.wallpaperSource
                    sourceSize.width: parent.width * 1.0
                    sourceSize.height: parent.height * 1.0
                    fillMode: Image.PreserveAspectCrop
                    smooth: true
                }

                // The icons.
                Desktop {
                    id: desktopIcons
                    anchors.fill: parent
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

            exclusionMode: ExclusionMode.Normal
            exclusiveZone: Theme.taskbarHeight

            WlrLayershell.layer: WlrLayer.Top
            WlrLayershell.namespace: "shojiwm-win7-taskbar"
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.None

            Taskbar {
                anchors.fill: parent
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
            implicitHeight: Theme.menuHeight + Theme.menuTileOverhang

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
