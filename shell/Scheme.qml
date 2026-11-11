pragma Singleton

import QtQuick
import Quickshell
import "themes/aero.js" as Aero
import "themes/kernel.js" as Kernel

// The colours and shapes of the shell, for the theme chosen in the Control
// Panel (Settings.theme) and, in Aero, for light or dark: the taskbar, the
// menus, the Start menu, the calendar, the Control Panel and the lock screen.
//
// Aero's light values are in themes/aero.js, its dark ones below. Kernel is
// always dark; its values are in themes/kernel.js and the `kernel ?`
// branches below. In Aero, light or dark, the taskbar and the window frames
// are dark glass, as on Windows 7.
Singleton {
    /// The theme's sizes and colours (themes/aero.js or themes/kernel.js):
    /// `Scheme.theme.taskbarHeight`. A binding that reads it follows a change
    /// of theme.
    readonly property var theme: kernel ? Kernel : Aero
    /// The Kernel theme (Settings.theme). It is always dark: `dark` speaks
    /// for Aero.
    readonly property bool kernel: Settings.theme === "kernel"
    readonly property bool dark: Settings.dark

    // ----- shapes ---------------------------------------------------------------

    /// The square variant of the theme (Settings.squareCorners).
    readonly property bool square: Settings.squareCorners

    /// A corner's radius: `radius` in the rounded theme, 0 in the square
    /// one. Every rounded corner goes through it; a circle does not.
    function corner(radius) {
        if (kernel) {
            return Math.min(radius, 2);
        }
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

    readonly property color popupTop: kernel ? Kernel.popupTop : (dark ? "#f02a3644" : Aero.popupTop)
    readonly property color popupBottom: kernel ? Kernel.popupBottom : (dark ? "#f01d2731" : Aero.popupBottom)
    readonly property color popupOutline: kernel ? Kernel.popupOutline : (dark ? "#cc6b8aa6" : Aero.popupOutline)
    /// The light line just inside a popup's outline.
    readonly property color popupInnerLine: kernel ? "#14ffffff" : dark ? "#33ffffff" : "#e6ffffff"

    // ----- the Start menu ----------------------------------------------------------

    /// The programs pane: white, as on Windows 7.
    readonly property color menuTop: kernel ? Kernel.menuTop : dark ? "#f2273340" : "#ffffff"
    readonly property color menuBottom: kernel ? Kernel.menuBottom : dark ? "#f21b242e" : "#f7f9fb"
    readonly property color menuOutline: kernel ? Kernel.menuOutline : (dark ? "#cc6b8aa6" : Aero.menuOutline)
    readonly property color menuDivider: kernel ? Kernel.menuDivider : (dark ? "#59a8c4dc" : Aero.menuDivider)
    readonly property color menuHeading: kernel ? Kernel.menuHeading : dark ? "#9cc8f0" : Aero.menuHeading

    /// The glass of the Start menu, round the white programs pane and under
    /// the places: steel blue, lighter towards the bottom (measured on the
    /// ShojiWM author's Windows 7 Start menu).
    readonly property color startGlassTop: kernel ? "#fa1c2631" : dark ? "#e61c2f3c" : "#d93f6b85"
    readonly property color startGlassMiddle: kernel ? "#fa18212b" : dark ? "#e6182836" : "#d93a6079"
    readonly property color startGlassBottom: kernel ? "#fa151d26" : dark ? "#e6223a4b" : "#d95a97bb"
    readonly property color startGlassOutline: kernel ? "#ff03060a" : "#a6000000"
    readonly property color startGlassHighlight: kernel ? "#1fffffff" : "#66ffffff"

    /// The places pane: dark blue glass.
    readonly property color placesTop: kernel ? Kernel.menuPlacesTop : Aero.menuPlacesTop
    readonly property color placesBottom: kernel ? Kernel.menuPlacesBottom : Aero.menuPlacesBottom
    readonly property color placesText: kernel ? Kernel.textOnPlaces : Aero.textOnPlaces

    // ----- text ------------------------------------------------------------------

    readonly property color text: kernel ? Kernel.textOnMenu : dark ? "#e8f1f9" : Aero.textOnMenu
    readonly property color textMuted: kernel ? Kernel.textMuted : dark ? "#9fb3c4" : "#4d6578"
    readonly property color placeholder: kernel ? "#5d6b79" : dark ? "#7f94a6" : "#7d95a9"

    // ----- picks -------------------------------------------------------------------

    readonly property color selection: kernel ? Kernel.selection : (dark ? "#664b8fd0" : Aero.selection)
    readonly property color selectionBorder: kernel ? Kernel.selectionBorder : dark ? "#b37fb6e6" : Aero.selectionBorder

    // ----- fields -----------------------------------------------------------------

    readonly property color field: kernel ? "#0d131a" : (dark ? "#1a232c" : "#ffffff")
    readonly property color fieldBorder: kernel ? "#2b3744" : (dark ? "#5d7489" : "#8fa9bf")
    readonly property color fieldFocus: kernel ? "#3f7fcd" : "#3f82b4"

    // ----- buttons ----------------------------------------------------------------

    readonly property color buttonTop: kernel ? "#2a3644" : (dark ? "#3b4a58" : "#eef6ff")
    readonly property color buttonBottom: kernel ? "#1e2833" : (dark ? "#2a3642" : "#c3d7e8")
    readonly property color buttonHotTop: kernel ? "#344354" : (dark ? "#4c5f71" : "#ffffff")
    readonly property color buttonHotBottom: kernel ? "#26323f" : (dark ? "#364554" : "#d4e6f5")
    readonly property color buttonPressedTop: kernel ? "#1d3858" : (dark ? "#2a3642" : "#c6dcef")
    readonly property color buttonPressedBottom: kernel ? "#182f4a" : (dark ? "#22303c" : "#9fc0dc")

    // ----- the calendar -------------------------------------------------------------

    readonly property color link: kernel ? Kernel.calendarLink : dark ? "#7fbfff" : Aero.calendarLink
    readonly property color today: kernel ? Kernel.calendarToday : Aero.calendarToday
    readonly property color hover: kernel ? Kernel.calendarHover : (dark ? "#334b8fd0" : Aero.calendarHover)
    readonly property color otherMonth: kernel ? Kernel.calendarOtherMonth : dark ? "#66808f" : Aero.calendarOtherMonth

    // ----- the Control Panel ----------------------------------------------------------

    readonly property color page: kernel ? "#121921" : dark ? "#1e262e" : "#ffffff"
    readonly property color sideTop: kernel ? "#18212b" : (dark ? "#24313d" : "#eaf3fb")
    readonly property color sideBottom: kernel ? "#131a22" : (dark ? "#1a242d" : "#c9def0")
    readonly property color pageHeading: kernel ? "#7fb2ea" : dark ? "#9cc8f0" : "#1e3287"
    readonly property color link2: kernel ? "#7fb2ea" : dark ? "#7fbfff" : "#0066cc"
}
