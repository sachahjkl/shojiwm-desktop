//@ pragma Env QSG_RENDER_LOOP=threaded

import QtQuick
import Quickshell
import Quickshell.Wayland
import "components"
import "theme.js" as Theme

// The desktop shell: a wallpaper, a taskbar and a Start menu, one of each per
// screen.
//
// Every surface is a wlr-layer-shell surface. The compositor blurs what is
// behind the taskbar and the menu, which is why their fills are translucent;
// the wallpaper asks for no blur at all through the `no_blur` namespace.
ShellRoot {
    // The Start menu is reached from a keybinding, so its IPC entry point has
    // to exist before anything asks for it.
    property var ipc: StartMenuIpc

    // ----- wallpaper --------------------------------------------------------

    Variants {
        model: Quickshell.screens

        PanelWindow {
            required property var modelData
            screen: modelData

            color: "transparent"
            anchors.top: true
            anchors.bottom: true
            anchors.left: true
            anchors.right: true

            exclusionMode: ExclusionMode.Ignore
            WlrLayershell.layer: WlrLayer.Background
            WlrLayershell.namespace: "no_blur"
            WlrLayershell.keyboardFocus: WlrKeyboardFocus.None
            // A wallpaper must not take a click away from the compositor.
            mask: Region {}

            Image {
                anchors.fill: parent
                source: "assets/wallpaper.svg"
                sourceSize.width: parent.width * 1.0
                sourceSize.height: parent.height * 1.0
                fillMode: Image.PreserveAspectCrop
                smooth: true
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
            implicitHeight: Theme.menuHeight

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
