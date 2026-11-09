pragma Singleton

import QtQuick
import Quickshell
import "theme.js" as Theme

// The colours and shapes that change with light or dark:
// the taskbar, the menus, the Start menu, the calendar, the Control Panel
// and the lock screen.
//
// Light and dark: the taskbar and the window frames are dark glass either
// way, as on Windows 7.
Singleton {
    readonly property bool dark: Settings.dark

    // ----- shapes ---------------------------------------------------------------

    /// The square variant of the theme (Settings.squareCorners).
    readonly property bool square: Settings.squareCorners

    /// A corner's radius: `radius` in the rounded theme, 0 in the square
    /// one. Every rounded corner goes through it; a circle does not.
    function corner(radius) {
        return square ? 0 : radius;
    }

    readonly property int popupRadius: corner(4)
    readonly property int menuRadius: corner(5)
    readonly property int itemRadius: corner(3)
    readonly property int fieldRadius: corner(2)
    readonly property int buttonRadius: corner(3)
    /// The frame round a picked item.
    readonly property int selectionBorderWidth: 1

    // ----- popups: jump lists, tray menus, the power menu, the calendar --------------

    readonly property color popupTop: (dark ? "#f02a3644" : Theme.popupTop)
    readonly property color popupBottom: (dark ? "#f01d2731" : Theme.popupBottom)
    readonly property color popupOutline: (dark ? "#cc6b8aa6" : Theme.popupOutline)
    /// The light line just inside a popup's outline.
    readonly property color popupInnerLine: dark ? "#33ffffff" : "#e6ffffff"

    // ----- the Start menu ----------------------------------------------------------

    /// The programs pane: white, as on Windows 7.
    readonly property color menuTop: dark ? "#f2273340" : "#ffffff"
    readonly property color menuBottom: dark ? "#f21b242e" : "#f7f9fb"
    readonly property color menuOutline: (dark ? "#cc6b8aa6" : Theme.menuOutline)
    readonly property color menuDivider: (dark ? "#59a8c4dc" : Theme.menuDivider)
    readonly property color menuHeading: dark ? "#9cc8f0" : Theme.menuHeading

    /// The glass of the Start menu, round the white programs pane and under
    /// the places: steel blue, lighter towards the bottom (measured on the
    /// ShojiWM author's Windows 7 Start menu).
    readonly property color startGlassTop: dark ? "#e61c2f3c" : "#d93f6b85"
    readonly property color startGlassMiddle: dark ? "#e6182836" : "#d93a6079"
    readonly property color startGlassBottom: dark ? "#e6223a4b" : "#d95a97bb"
    readonly property color startGlassOutline: "#a6000000"
    readonly property color startGlassHighlight: "#66ffffff"

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
