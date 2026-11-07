pragma Singleton

import QtQuick
import Quickshell
import "theme.js" as Theme

// The colours and shapes that change with the style and with light or dark:
// the taskbar, the menus, the Start menu, the calendar, the Control Panel
// and the lock screen.
//
// Aero is Windows 7: glossy gradients, a bevelled frame round what is
// picked, small radii. Modern keeps the glass and the blue and drops the
// rest: flat fills, rounder corners, picks marked by a plain tint, no gloss.
//
// Light and dark: Aero's taskbar and window frames are dark glass either
// way, as on Windows 7; modern's frames follow it.
Singleton {
    readonly property bool dark: Settings.dark
    readonly property bool modern: Settings.modern

    // ----- shapes ---------------------------------------------------------------

    readonly property int popupRadius: modern ? 8 : 4
    readonly property int menuRadius: modern ? 10 : 5
    readonly property int itemRadius: modern ? 5 : 3
    readonly property int fieldRadius: modern ? 6 : 2
    readonly property int buttonRadius: modern ? 6 : 3
    /// The frame round a picked item: Aero draws one, modern does not.
    readonly property int selectionBorderWidth: modern ? 0 : 1

    // ----- the taskbar ------------------------------------------------------------

    /// Modern: one flat tint of smoked glass, a faint line along the top.
    readonly property color barFlat: dark ? "#d9121820" : "#cc1f2a35"
    readonly property color barLine: "#1fffffff"
    readonly property color taskRest: "#00000000"
    readonly property color taskHover: "#1affffff"
    readonly property color taskActive: "#2bffffff"
    /// The bar under a running program's button.
    readonly property color taskIndicator: "#7cc4ff"

    // ----- popups: jump lists, tray menus, the power menu, the calendar --------------

    readonly property color popupTop: modern
        ? (dark ? "#f2222a33" : "#f5f7f9fb")
        : (dark ? "#f02a3644" : Theme.popupTop)
    readonly property color popupBottom: modern
        ? popupTop
        : (dark ? "#f01d2731" : Theme.popupBottom)
    readonly property color popupOutline: modern
        ? (dark ? "#66000000" : "#29000000")
        : (dark ? "#cc6b8aa6" : Theme.popupOutline)

    // ----- the Start menu ----------------------------------------------------------

    readonly property color menuTop: modern
        ? (dark ? "#f21c232b" : "#f2f6f8fa")
        : (dark ? "#f2273340" : Theme.menuTop)
    readonly property color menuBottom: modern
        ? menuTop
        : (dark ? "#f21b242e" : Theme.menuBottom)
    readonly property color menuOutline: modern
        ? (dark ? "#80000000" : "#33000000")
        : (dark ? "#cc6b8aa6" : Theme.menuOutline)
    readonly property color menuDivider: modern
        ? (dark ? "#1fffffff" : "#1a000000")
        : (dark ? "#59a8c4dc" : Theme.menuDivider)
    readonly property color menuHeading: dark ? "#9cc8f0" : Theme.menuHeading

    /// The places pane: Aero's dark blue glass, or modern's faint tint.
    readonly property color placesTop: modern
        ? (dark ? "#0dffffff" : "#0a000000")
        : Theme.menuPlacesTop
    readonly property color placesBottom: modern ? placesTop : Theme.menuPlacesBottom
    readonly property color placesText: modern ? text : Theme.textOnPlaces

    // ----- text ------------------------------------------------------------------

    readonly property color text: dark ? "#e8f1f9" : Theme.textOnMenu
    readonly property color textMuted: dark ? "#9fb3c4" : "#4d6578"
    readonly property color placeholder: dark ? "#7f94a6" : "#7d95a9"

    // ----- picks -------------------------------------------------------------------

    readonly property color selection: modern
        ? (dark ? "#2effffff" : "#1f1f6fc5")
        : (dark ? "#664b8fd0" : Theme.selection)
    readonly property color selectionBorder: dark ? "#b37fb6e6" : Theme.selectionBorder

    // ----- fields -----------------------------------------------------------------

    readonly property color field: modern
        ? (dark ? "#14ffffff" : "#ffffff")
        : (dark ? "#1a232c" : "#ffffff")
    readonly property color fieldBorder: modern
        ? (dark ? "#26ffffff" : "#26000000")
        : (dark ? "#5d7489" : "#8fa9bf")
    readonly property color fieldFocus: "#3f82b4"

    // ----- buttons ----------------------------------------------------------------

    readonly property color buttonTop: modern
        ? (dark ? "#1affffff" : "#0d000000")
        : (dark ? "#3b4a58" : "#eef6ff")
    readonly property color buttonBottom: modern
        ? buttonTop
        : (dark ? "#2a3642" : "#c3d7e8")
    readonly property color buttonHotTop: modern
        ? (dark ? "#29ffffff" : "#1a000000")
        : (dark ? "#4c5f71" : "#ffffff")
    readonly property color buttonHotBottom: modern
        ? buttonHotTop
        : (dark ? "#364554" : "#d4e6f5")
    readonly property color buttonPressedTop: modern
        ? (dark ? "#14ffffff" : "#26000000")
        : (dark ? "#2a3642" : "#c6dcef")
    readonly property color buttonPressedBottom: modern
        ? buttonPressedTop
        : (dark ? "#22303c" : "#9fc0dc")
    readonly property color buttonOutline: modern
        ? (dark ? "#1fffffff" : "#1f000000")
        : menuOutline

    /// The accent: the blue the modern style fills its main buttons with.
    readonly property color accent: dark ? "#4b9be0" : "#2f7bc0"
    readonly property color accentHot: dark ? "#5eaaee" : "#3a8ad2"

    // ----- the calendar -------------------------------------------------------------

    readonly property color link: dark ? "#7fbfff" : Theme.calendarLink
    readonly property color today: Theme.calendarToday
    readonly property color hover: modern
        ? (dark ? "#1affffff" : "#14000000")
        : (dark ? "#334b8fd0" : Theme.calendarHover)
    readonly property color otherMonth: dark ? "#66808f" : Theme.calendarOtherMonth

    // ----- the Control Panel ----------------------------------------------------------

    readonly property color page: dark ? "#1e262e" : "#ffffff"
    readonly property color sideTop: modern
        ? (dark ? "#1a2128" : "#f3f5f8")
        : (dark ? "#24313d" : "#eaf3fb")
    readonly property color sideBottom: modern
        ? sideTop
        : (dark ? "#1a242d" : "#c9def0")
    readonly property color pageHeading: dark ? "#9cc8f0" : "#1e3287"
    readonly property color link2: dark ? "#7fbfff" : "#0066cc"
}
