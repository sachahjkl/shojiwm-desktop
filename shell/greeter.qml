//@ pragma Env QSG_RENDER_LOOP=threaded
//@ pragma IconTheme Aero
//@ pragma Env QSG_DISTANCEFIELD_ANTIALIASING=gray

import QtQuick
import Quickshell
import Quickshell.Wayland
import "components"

// The log-on screen: the lock screen's picture, clock, account and password
// field on the main screen, black on the others, and the session to log on
// to in the bottom-left corner. greetd checks the password and starts the
// session (GreeterState). Run it in a compositor that greetd starts, with
// `quickshell -p greeter.qml`.
ShellRoot {
    Variants {
        model: Quickshell.screens

        PanelWindow {
            id: surface

            required property var modelData
            readonly property bool primary: Screens.isPrimary(surface.modelData)

            screen: surface.modelData
            color: surface.primary ? "#0e3a63" : "#000000"
            anchors.top: true
            anchors.bottom: true
            anchors.left: true
            anchors.right: true
            exclusionMode: ExclusionMode.Ignore
            WlrLayershell.layer: WlrLayer.Overlay
            WlrLayershell.namespace: "shojiwm-win7-greeter"
            WlrLayershell.keyboardFocus: surface.primary
                ? WlrKeyboardFocus.Exclusive : WlrKeyboardFocus.None

            LockScreen {
                anchors.fill: parent
                visible: surface.primary
                auth: GreeterState
                userName: GreeterState.userName
                picture: GreeterState.picture
                runPower: function (id) {
                    var action = Apps.powerAction(id);
                    if (action && action.command)
                        Quickshell.execDetached(action.command);
                }
            }

            SessionPicker {
                visible: surface.primary
                anchors.left: parent.left
                anchors.bottom: parent.bottom
                anchors.margins: 26
            }
        }
    }
}
