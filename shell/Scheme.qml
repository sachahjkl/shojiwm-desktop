pragma Singleton

import QtQuick
import Quickshell
import "theme.js" as Theme

// The colours and shapes that change with the style and with light or dark:
// the taskbar, the menus, the Start menu, the calendar, the Control Panel
// and the lock screen.
//
// Aero is Windows 7 as it was. Modern is the same Aero, squared: the glass,
// the gloss, the bevels and the colours stay; every corner is a right
// angle.
//
// Light and dark: the taskbar and the window frames are dark glass either
// way, as on Windows 7.
Singleton {
    readonly property bool dark: Settings.dark
    readonly property bool modern: Settings.modern

    // ----- shapes ---------------------------------------------------------------

    readonly property int popupRadius: modern ? 0 : 4
    readonly property int menuRadius: modern ? 0 : 5
    readonly property int itemRadius: modern ? 0 : 3
    readonly property int fieldRadius: modern ? 0 : 2
    readonly property int buttonRadius: modern ? 0 : 3
    /// The frame round a picked item.
    readonly property int selectionBorderWidth: 1

    // ----- popups: jump lists, tray menus, the power menu, the calendar --------------

    readonly property color popupTop: (dark ? "#f02a3644" : Theme.popupTop)
    readonly property color popupBottom: (dark ? "#f01d2731" : Theme.popupBottom)
    readonly property color popupOutline: (dark ? "#cc6b8aa6" : Theme.popupOutline)

    // ----- the Start menu ----------------------------------------------------------

    readonly property color menuTop: (dark ? "#f2273340" : Theme.menuTop)
    readonly property color menuBottom: (dark ? "#f21b242e" : Theme.menuBottom)
    readonly property color menuOutline: (dark ? "#cc6b8aa6" : Theme.menuOutline)
    readonly property color menuDivider: (dark ? "#59a8c4dc" : Theme.menuDivider)
    readonly property color menuHeading: dark ? "#9cc8f0" : Theme.menuHeading

    /// The places pane: dark blue glass.
    readonly property color placesTop: Theme.menuPlacesTop
    readonly property color placesBottom: Theme.menuPlacesBottom
    readonly property color placesText: Theme.textOnPlaces

    // ----- text ------------------------------------------------------------------

    readonly property color text: dark ? "#e8f1f9" : Theme.textOnMenu
    readonly property color textMuted: dark ? "#9fb3c4" : "#4d6578"
    readonly property color placeholder: dark ? "#7f94a6" : "#7d95a9"

    // ----- picks -------------------------------------------------------------------

    readonly property color selection: (dark ? "#664b8fd0" : Theme.selection)
    readonly property color selectionBorder: dark ? "#b37fb6e6" : Theme.selectionBorder

    // ----- fields -----------------------------------------------------------------

    readonly property color field: (dark ? "#1a232c" : "#ffffff")
    readonly property color fieldBorder: (dark ? "#5d7489" : "#8fa9bf")
    readonly property color fieldFocus: "#3f82b4"

    // ----- buttons ----------------------------------------------------------------

    readonly property color buttonTop: (dark ? "#3b4a58" : "#eef6ff")
    readonly property color buttonBottom: (dark ? "#2a3642" : "#c3d7e8")
    readonly property color buttonHotTop: (dark ? "#4c5f71" : "#ffffff")
    readonly property color buttonHotBottom: (dark ? "#364554" : "#d4e6f5")
    readonly property color buttonPressedTop: (dark ? "#2a3642" : "#c6dcef")
    readonly property color buttonPressedBottom: (dark ? "#22303c" : "#9fc0dc")

    // ----- the calendar -------------------------------------------------------------

    readonly property color link: dark ? "#7fbfff" : Theme.calendarLink
    readonly property color today: Theme.calendarToday
    readonly property color hover: (dark ? "#334b8fd0" : Theme.calendarHover)
    readonly property color otherMonth: dark ? "#66808f" : Theme.calendarOtherMonth

    // ----- the Control Panel ----------------------------------------------------------

    readonly property color page: dark ? "#1e262e" : "#ffffff"
    readonly property color sideTop: (dark ? "#24313d" : "#eaf3fb")
    readonly property color sideBottom: (dark ? "#1a242d" : "#c9def0")
    readonly property color pageHeading: dark ? "#9cc8f0" : "#1e3287"
    readonly property color link2: dark ? "#7fbfff" : "#0066cc"
}
