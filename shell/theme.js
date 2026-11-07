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
var taskButtonHeight = 30;
var taskButtonMaxWidth = 168;
var taskButtonMinWidth = 44;
var taskButtonGap = 3;
// A pinned program with no window open: the icon alone, as Windows draws it.
var pinnedButtonWidth = 46;
var pinnedIconSize = 24;
var trayIconSize = 16;
var trayGap = 8;
var clockWidth = 74;
var showDesktopWidth = 10;
// How long the pointer rests on the show-desktop strip before Aero Peek.
var peekDelay = 500;

// Combine every window of one program into one button, as Windows 7 does by
// default. Set this to false for one button per window ("never combine").
var combineWindows = true;

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

// ---------------------------------------------------------------------------
// Colours
//
// Qt reads a hex colour as #AARRGGBB: the alpha comes first, unlike CSS.
// ---------------------------------------------------------------------------

// Taskbar glass: dark blue-grey over the blur, lit along the top edge.
var barTop = "#b84a6b8a";
var barUpper = "#c7203247";
var barMiddle = "#d10d1621";
var barLower = "#cc1a2a38";
var barBottom = "#d6090e13";
var barHighlight = "#73bfd8ef";
var barEdge = "#cc0a131b";
var barBottomEdge = "#d9070d12";

// Task buttons.
var buttonRest = "#1cffffff";
var buttonRestBorder = "#30ffffff";
var buttonHover = "#5eaad4f5";
var buttonHoverBorder = "#8ccfe9ff";
var buttonActive = "#52d6ecff";
var buttonActiveBorder = "#99e8f5ff";
var buttonGloss = "#2bffffff";

// The stack drawn behind a button that stands for several windows.
var buttonStack = "#14ffffff";
var buttonStackBorder = "#26ffffff";

// Popup menus and thumbnails.
var popupTop = "#eef4f9ff";
var popupBottom = "#eedbe8f4";
var popupBorder = "#ccffffff";
var popupOutline = "#cc2c5779";
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
