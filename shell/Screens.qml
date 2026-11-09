pragma Singleton

import QtQuick
import Quickshell

// The main screen: the one Settings.primaryScreen (else
// SHOJIWM_WIN7_PRIMARY_SCREEN) names, when it is
// connected, else the first screen Quickshell lists. It is not the screen
// with the focus: it is where the shell puts what has no screen of its own.
// The lock screen shows there alone; the Start menu and the Shut Down
// dialog open there when asked without a screen; the gadgets stand there;
// desktop icons without a place, or whose screen is gone, show there.
Singleton {
    id: root

    /// The main screen asked for: the Control Panel's choice, else
    /// SHOJIWM_WIN7_PRIMARY_SCREEN (the log-on screen has no settings of its
    /// own), by connector name.
    readonly property string wanted: Settings.primaryScreen
        || Quickshell.env("SHOJIWM_WIN7_PRIMARY_SCREEN") || ""

    readonly property string primaryName: {
        var screens = Quickshell.screens;
        for (var i = 0; i < screens.length; i++) {
            if (screens[i].name === root.wanted)
                return root.wanted;
        }
        return screens.length > 0 ? screens[0].name : "";
    }

    /// The main screen itself, or null when there is none.
    readonly property var primary: {
        var screens = Quickshell.screens;
        for (var i = 0; i < screens.length; i++) {
            if (screens[i].name === root.primaryName)
                return screens[i];
        }
        return null;
    }

    function isPrimary(screen) {
        return !!screen && screen.name === root.primaryName;
    }
}
