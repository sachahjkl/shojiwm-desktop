pragma Singleton

import QtQuick
import Quickshell
import "theme.js" as Theme

// The colours that change with the light or dark preference: the menus, the
// Start menu, the calendar and the Control Panel. The taskbar and the window
// frames are dark glass either way, as on Windows 7, so they stay in
// theme.js.
//
// Light is the Windows 7 look. Dark keeps its shapes and its blue, on smoked
// glass.
Singleton {
    readonly property bool dark: Settings.dark

    // Popups: jump lists, tray menus, the power menu, the calendar.
    readonly property color popupTop: dark ? "#f02a3644" : Theme.popupTop
    readonly property color popupBottom: dark ? "#f01d2731" : Theme.popupBottom
    readonly property color popupOutline: dark ? "#cc6b8aa6" : Theme.popupOutline

    // The Start menu's programs pane, and its outline.
    readonly property color menuTop: dark ? "#f2273340" : Theme.menuTop
    readonly property color menuBottom: dark ? "#f21b242e" : Theme.menuBottom
    readonly property color menuOutline: dark ? "#cc6b8aa6" : Theme.menuOutline
    readonly property color menuDivider: dark ? "#59a8c4dc" : Theme.menuDivider
    readonly property color menuHeading: dark ? "#9cc8f0" : Theme.menuHeading

    // Text on those.
    readonly property color text: dark ? "#e8f1f9" : Theme.textOnMenu
    readonly property color textMuted: dark ? "#9fb3c4" : "#4d6578"
    readonly property color placeholder: dark ? "#7f94a6" : "#7d95a9"

    // The pale blue band with a thin frame that marks the item under the
    // pointer.
    readonly property color selection: dark ? "#664b8fd0" : Theme.selection
    readonly property color selectionBorder: dark ? "#b37fb6e6" : Theme.selectionBorder

    // Fields: the search box, the Control Panel's inputs.
    readonly property color field: dark ? "#1a232c" : "#ffffff"
    readonly property color fieldBorder: dark ? "#5d7489" : "#8fa9bf"
    readonly property color fieldFocus: "#3f82b4"

    // Buttons: the power button, the Control Panel's.
    readonly property color buttonTop: dark ? "#3b4a58" : "#eef6ff"
    readonly property color buttonBottom: dark ? "#2a3642" : "#c3d7e8"
    readonly property color buttonHotTop: dark ? "#4c5f71" : "#ffffff"
    readonly property color buttonHotBottom: dark ? "#364554" : "#d4e6f5"
    readonly property color buttonPressedTop: dark ? "#2a3642" : "#c6dcef"
    readonly property color buttonPressedBottom: dark ? "#22303c" : "#9fc0dc"

    // The calendar.
    readonly property color link: dark ? "#7fbfff" : Theme.calendarLink
    readonly property color today: Theme.calendarToday
    readonly property color hover: dark ? "#334b8fd0" : Theme.calendarHover
    readonly property color otherMonth: dark ? "#66808f" : Theme.calendarOtherMonth

    // The Control Panel: a window's white page, and its blue side pane.
    readonly property color page: dark ? "#1e262e" : "#ffffff"
    readonly property color sideTop: dark ? "#24313d" : "#eaf3fb"
    readonly property color sideBottom: dark ? "#1a242d" : "#c9def0"
    readonly property color pageHeading: dark ? "#9cc8f0" : "#1e3287"
    readonly property color link2: dark ? "#7fbfff" : "#0066cc"
}
