pragma Singleton

import Quickshell
import Quickshell.Io

// How the compositor opens the Start menu:
//
//   quickshell -p ~/.config/shojiwm/shell ipc call startmenu toggle
//   quickshell -p ~/.config/shojiwm/shell ipc call startmenu toggleOn eDP-1
//
// A singleton, because the shell is built once per screen and one handler per
// screen would be several handlers fighting over the same target name. This
// one exists once and passes the request on as a signal; every screen hears
// it and only the addressed one answers.
Singleton {
    id: root

    /// `screen` is a connector name (eDP-1, DP-3). Empty means every screen.
    signal requested(string action, string screen)
    /// The Bluetooth panel, asked over IPC (bluetooth open, close).
    signal bluetoothRequested(bool open)

    IpcHandler {
        target: "startmenu"

        function open(): void { root.requested("open", ""); }
        function close(): void { root.requested("close", ""); }
        function toggle(): void { root.requested("toggle", ""); }

        function openOn(screen: string): void { root.requested("open", screen); }
        function closeOn(screen: string): void { root.requested("close", screen); }
        function toggleOn(screen: string): void { root.requested("toggle", screen); }
    }

    IpcHandler {
        target: "bluetooth"

        function open(): void { root.bluetoothRequested(true); }
        function close(): void { root.bluetoothRequested(false); }
    }

    // The Start orb's right-click menu (Super+X).
    IpcHandler {
        target: "adminmenu"

        function toggle(): void { StartMenuState.toggleAdminMenu(""); }
        function toggleOn(screen: string): void { StartMenuState.toggleAdminMenu(screen); }
        function close(): void { StartMenuState.adminMenuOn = ""; }
    }
}
