.pragma library

// Every size and colour the shell is drawn from. Change one and the rest of
// the layout follows, because nothing below repeats a number from here.
//
// The compositor blurs whatever is behind each surface, so the fills are
// translucent: they tint that blur instead of hiding it.

// ---------------------------------------------------------------------------
// Taskbar
// ---------------------------------------------------------------------------

var taskbarHeight = 40;
var taskbarPadding = 2;
var orbSize = 36;
var orbLeft = 8;
// Buttons fill the bar but for a pixel or two, as on Windows 7.
var taskButtonHeight = 36;
var taskButtonMaxWidth = 168;
var taskButtonMinWidth = 44;
var taskButtonGap = 2;
// Room at the right of a button for the edges of the windows stacked behind
// it.
var stackRoom = 4;
// A pinned program with no window open: the icon alone, as Windows draws it.
var pinnedButtonWidth = 62;
var pinnedIconSize = 24;
// A button in the compact taskbar: the icon alone, for a running program too.
var compactButtonWidth = 62;
var trayIconSize = 16;
var trayGap = 8;
var clockWidth = 74;
var showDesktopWidth = 15;
// How long the pointer rests on the show-desktop strip before Aero Peek.
var peekDelay = 500;

// ---------------------------------------------------------------------------
// Thumbnails, over a taskbar button
// ---------------------------------------------------------------------------

var thumbWidth = 184;
var thumbHeight = 104;
var thumbTitleHeight = 20;
var thumbGap = 6;
var thumbPadding = 8;
// Show the window itself in a thumbnail, rather than the program's icon.
//
// Off, because it does not work here: asking Quickshell 0.3.0 or 0.3.2 to
// capture a ShojiWM window ends the shell's Wayland connection ("fatal
// error: Invalid argument") and the whole bar disappears. The code is in
// ThumbnailPopup.qml; turn this on to try it on your own pair.
var liveThumbnails = false;

// How long the pointer rests on a button before the thumbnails come up.
var thumbDelay = 350;

// ---------------------------------------------------------------------------
// Popup menus: the jump list and the tray menus
// ---------------------------------------------------------------------------

var popupGap = 2;
var popupItemHeight = 26;
var popupMinWidth = 170;
var popupMaxWidth = 340;
var popupPadX = 10;
var popupPadY = 5;
// How long the pointer rests on a line before its submenu opens.
var submenuDelay = 300;

// ---------------------------------------------------------------------------
// Start menu
// ---------------------------------------------------------------------------

var menuWidth = 490;
var menuHeight = 560;
var menuMargin = 2;
var menuProgramsWidth = 300;
var menuRowHeight = 32;
var menuPadding = 8;
var menuSearchHeight = 26;
var menuFooterHeight = 42;
// The account picture stands above the menu's top edge by this much, and
// is this big, frame included.
var menuTileOverhang = 30;
var menuTileSize = 68;
// The glass round the white programs pane.
var menuGlassBorder = 8;

// ---------------------------------------------------------------------------
// Colours
//
// Qt reads a hex colour as #AARRGGBB: the alpha comes first, unlike CSS.
// ---------------------------------------------------------------------------

// Taskbar glass: a pale haze over the blur, so the wallpaper's own colour
// comes through, lit along the top and a little darker towards the bottom,
// as the Windows 7 bar is over the default wallpaper.
var barTop = "#4dffffff";
var barUpper = "#29d9ebf7";
var barMiddle = "#1f3a5a75";
var barLower = "#33223a50";
var barBottom = "#4d0e1c28";
var barHighlight = "#80ffffff";
var barEdge = "#99000000";
var barBottomEdge = "#80000000";

// Task buttons: glass plates with a dark outline and a light line inside it,
// the upper half lighter than the lower. A running program has one at rest;
// the window you are looking at has a brighter one.
var buttonOutline = "#73000000";
var buttonRestTop = "#40ffffff";
var buttonRestUpper = "#24ffffff";
var buttonRestLower = "#0affffff";
var buttonRestBottom = "#21ffffff";
var buttonRestBorder = "#73ffffff";
var buttonHoverTop = "#66ffffff";
var buttonHoverUpper = "#40ffffff";
var buttonHoverLower = "#1fffffff";
var buttonHoverBottom = "#4dffffff";
var buttonHoverBorder = "#a6ffffff";
var buttonActiveTop = "#80ffffff";
var buttonActiveUpper = "#59ffffff";
var buttonActiveLower = "#2effffff";
var buttonActiveBottom = "#59ffffff";
var buttonActiveBorder = "#bfffffff";
var buttonPressedTop = "#33ffffff";
var buttonPressedUpper = "#1affffff";
var buttonPressedLower = "#26000000";
var buttonPressedBottom = "#1affffff";

// The windows stacked behind a button that stands for several.
var buttonStack = "#29ffffff";
var buttonStackBorder = "#8cffffff";

// Popup menus and thumbnails.
var popupTop = "#eef4f9ff";
var popupBottom = "#eedbe8f4";
var popupBorder = "#ccffffff";
// A neutral grey line, as Windows 7 menus have.
var popupOutline = "#6e000000";
var thumbFrame = "#1b3c58";

// Start menu glass.
var menuTop = "#f2e8f2fb";
var menuBottom = "#f2cfe0ef";
var menuPlacesTop = "#e62f536f";
var menuPlacesBottom = "#e61d394f";
var menuBorder = "#8cffffff";
var menuOutline = "#cc12314b";
var menuDivider = "#598fb0c9";

// Text.
var textOnGlass = "#f2f8ff";
var textMuted = "#c3d6e6";
var textOnMenu = "#10283c";
var menuHeading = "#1e3287";

// The calendar over the clock.
var calendarLink = "#0066cc";
var calendarToday = "#3f82b4";
var calendarHover = "#33cfe4f7";
var calendarOtherMonth = "#93a6b6";
var textOnPlaces = "#eaf4ff";
var selection = "#669ad0f7";
var selectionBorder = "#b3cfe9ff";

var fontFamily = "Inter";
var fontFallback = "Noto Sans";
var fontSize = 12;
var fontSizeSmall = 11;

// ---------------------------------------------------------------------------
// Timing, in milliseconds. Short: the desktop this copies was quick.
// ---------------------------------------------------------------------------

var fade = 90;
var menuFade = 120;
