pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// Whether the Control Panel is open, and on which page.
//
//   quickshell -p ~/.config/shojiwm/shell ipc call controlpanel open
//   quickshell -p ~/.config/shojiwm/shell ipc call controlpanel openPage appearance
Singleton {
    id: root

    property bool shown: false
    /// "home", "appearance", "background", "lockscreen", "account",
    /// "taskbar", "clock" or "about".
    property string page: "home"

    // Every task the pages hold, with the words it is found by: the
    // Control Panel's search box and the Start menu's find them.
    readonly property var tasks: [
        { text: "Choose light or dark", page: "appearance", words: "dark light theme colour color scheme night appearance" },
        { text: "Change the colour of the taskbar and the Start menu", page: "appearance", words: "window color colour glass tint taskbar start menu intensity personalize" },
        { text: "Change how much the glass blurs", page: "appearance", words: "glass blur transparency aero frosted taskbar window colour color" },
        { text: "Change the desktop background on its own (slide show)", page: "background", words: "slide show slideshow wallpaper change rotate shuffle background folder" },
        { text: "Change desktop background", page: "background", words: "wallpaper picture image desktop background" },
        { text: "Change the size of desktop icons", page: "taskbar", words: "desktop icons size large small medium zoom" },
        { text: "Lock screen and its picture", page: "lockscreen", words: "lock screen picture password idle delay suspend sleep" },
        { text: "Change the screen saver", page: "lockscreen", words: "screen saver screensaver mystify idle wait preview" },
        { text: "Lock the computer after a delay", page: "lockscreen", words: "lock idle minutes delay timeout screensaver" },
        { text: "Change your account picture", page: "account", words: "account user picture avatar face photo profile" },
        { text: "Show or hide common icons on the desktop", page: "taskbar", words: "desktop icons computer user files control panel recycle bin show hide" },
        { text: "Combine taskbar buttons", page: "taskbar", words: "taskbar combine group buttons windows" },
        { text: "Show labels on taskbar buttons", page: "taskbar", words: "taskbar labels compact icons text" },
        { text: "Use Aero Peek to preview the desktop", page: "taskbar", words: "aero peek preview desktop show" },
        { text: "Show the programs used most in the Start menu", page: "taskbar", words: "start menu frequent recent programs" },
        { text: "Change what the power button does", page: "taskbar", words: "power button shut down shutdown restart sleep hibernate log off lock" },
        { text: "Select which icons appear on the taskbar", page: "notifications", words: "notification area tray icons hidden show" },
        { text: "Show clocks for other time zones", page: "clock", words: "additional clocks time zone world clock other city" },
        { text: "Change the clock format", page: "clock", words: "clock time date format 24 12 hour" },
        { text: "See the time zone and time synchronization", page: "clock", words: "time zone timezone ntp synchronization sync server internet" },
        { text: "Turn off all unnecessary animations", page: "ease", words: "animations motion reduce reduced effects ease access accessibility" },
        { text: "Change the animation speed", page: "ease", words: "animation speed fast faster slow slower motion duration" },
        { text: "Change the screen resolution and scale", page: "display", words: "display screen monitor resolution scale size dpi refresh rate hz mode text bigger" },
        { text: "View devices and printers", page: "devices", words: "devices printers usb hardware keyboard mouse webcam printer monitor" },
        { text: "Manage audio devices", page: "sound", words: "sound audio speakers headphones microphone recording playback default device output input" },
        { text: "Choose the programs that open your files and links", page: "defaults", words: "default programs apps browser web mail email open with file association mime pdf pictures music video folder" },
        { text: "Choose the programs that start when you log on", page: "startup", words: "startup start log on login boot autostart programs run session" },
        { text: "About this desktop", page: "about", words: "about system version information" }
    ]

    /// The tasks that hold every word of `query`, for the Start menu.
    function findTasks(query) {
        var words = String(query).toLowerCase().split(/\s+/).filter(function (word) {
            return word.length > 0;
        });
        if (words.length === 0)
            return [];
        return root.tasks.filter(function (task) {
            var haystack = (task.text + " " + task.words).toLowerCase();
            return words.every(function (word) { return haystack.indexOf(word) >= 0; });
        });
    }

    function open(page) {
        root.page = page && page.length > 0 ? page : "home";
        root.shown = true;
    }

    function close() {
        root.shown = false;
    }

    IpcHandler {
        target: "controlpanel"

        function open(): void { root.open(""); }
        function openPage(page: string): void { root.open(page); }
        function close(): void { root.close(); }
    }
}
