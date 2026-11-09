pragma Singleton

import QtQuick
import Quickshell

// The screens and their settings, for the Control Panel's Screen Resolution
// page. The compositor lists the screens and applies the choices
// (src/display.ts). A change asks to be kept: as on Windows, it goes back on
// its own after 15 seconds, so a mode the screen cannot show undoes itself.
Singleton {
    id: root

    /// The screens as the compositor reports them: name, description,
    /// width, height, refreshRate, scale, x, y and modes.
    property var screens: []
    /// The choices before the change waiting to be kept.
    property var previous: null
    readonly property bool confirming: root.previous !== null
    property int secondsLeft: 0
    readonly property int confirmSeconds: 15

    function refresh() {
        CompositorIpc.request("screens", {}, function (result) {
            if (result)
                root.screens = result;
        });
    }

    /// Apply new choices for every screen, and wait for them to be kept.
    function apply(choices) {
        if (!root.confirming)
            root.previous = Settings.displayChoices;
        Settings.displayChoices = choices;
        root.secondsLeft = root.confirmSeconds;
        countdown.restart();
        later.restart();
    }

    function keep() {
        countdown.stop();
        root.previous = null;
    }

    function revert() {
        countdown.stop();
        if (root.previous !== null)
            Settings.displayChoices = root.previous;
        root.previous = null;
        later.restart();
    }

    /// Back to the session's own layout.
    function reset() {
        root.apply({});
    }

    Timer {
        id: countdown
        interval: 1000
        repeat: true
        onTriggered: {
            root.secondsLeft -= 1;
            if (root.secondsLeft <= 0)
                root.revert();
        }
    }

    // The screens report their new modes once the compositor has set them.
    Timer {
        id: later
        interval: 700
        onTriggered: root.refresh()
    }
}
