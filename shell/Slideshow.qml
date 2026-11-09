pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// The desktop slide show of Windows 7's Desktop Background: the pictures of
// one folder become the wallpaper in turn, every few minutes, in name order
// or shuffled. The folder is read again at each change, so pictures added
// to it join the show.
Singleton {
    id: root

    readonly property bool running: Settings.slideshowFolder.length > 0

    /// Show the next picture now.
    function next() {
        if (!root.running)
            return;
        finder.running = false;
        finder.running = true;
    }

    function pick(pictures) {
        if (pictures.length === 0)
            return;
        var current = pictures.indexOf(Settings.wallpaper);
        var index;
        if (Settings.slideshowShuffle && pictures.length > 1) {
            do {
                index = Math.floor(Math.random() * pictures.length);
            } while (index === current);
        } else {
            index = (current + 1) % pictures.length;
        }
        Settings.wallpaper = pictures[index];
    }

    Process {
        id: finder

        command: ["sh", "-c",
            "find \"$1\" -maxdepth 1 -type f \\( -iname '*.jpg' -o -iname '*.jpeg'"
            + " -o -iname '*.png' -o -iname '*.webp' \\) 2>/dev/null | sort",
            "find", Settings.slideshowFolder]
        stdout: StdioCollector {
            onStreamFinished: root.pick(String(text).split("\n").filter(function (line) {
                return line.length > 0;
            }))
        }
    }

    Timer {
        interval: Math.max(1, Settings.slideshowMinutes) * 60000
        repeat: true
        running: root.running
        onTriggered: root.next()
    }

    // Starting a show, or changing its folder, shows a picture of it at once.
    Connections {
        target: Settings
        function onSlideshowFolderChanged() { root.next(); }
    }
}
