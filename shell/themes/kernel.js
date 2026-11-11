.pragma library
.import "aero.js" as Aero

// The Kernel theme: a deep blue-black desk, panels of slate, the steel blue
// of the window captions as its accent, a red close, crisp text. Every name
// of aero.js is defined here too (the flake's check "theme-names" holds the
// two lists equal); Scheme.qml picks one file or the other.
//
// Qt reads a hex colour as #AARRGGBB: the alpha comes first.

// ---------------------------------------------------------------------------
// Sizes: Aero's, until a Kernel surface needs its own.
// ---------------------------------------------------------------------------

var barSide = "top";
var taskbarHeight = Aero.taskbarHeight;
var taskbarPadding = Aero.taskbarPadding;
var orbSize = Aero.orbSize;
var orbLeft = Aero.orbLeft;
var taskButtonHeight = Aero.taskButtonHeight;
var taskButtonMaxWidth = Aero.taskButtonMaxWidth;
var taskButtonMinWidth = Aero.taskButtonMinWidth;
var taskButtonGap = Aero.taskButtonGap;
var stackRoom = Aero.stackRoom;
var pinnedButtonWidth = Aero.pinnedButtonWidth;
var taskIconSize = Aero.taskIconSize;
var compactButtonWidth = Aero.compactButtonWidth;
var trayIconSize = Aero.trayIconSize;
var trayGap = Aero.trayGap;
var clockWidth = Aero.clockWidth;
var showDesktopWidth = Aero.showDesktopWidth;
var peekDelay = Aero.peekDelay;
var thumbWidth = Aero.thumbWidth;
var thumbHeight = Aero.thumbHeight;
var thumbTitleHeight = Aero.thumbTitleHeight;
var thumbGap = Aero.thumbGap;
var thumbPadding = Aero.thumbPadding;
var liveThumbnails = Aero.liveThumbnails;
var thumbDelay = Aero.thumbDelay;
var popupGap = Aero.popupGap;
var popupItemHeight = Aero.popupItemHeight;
var popupMinWidth = Aero.popupMinWidth;
var popupMaxWidth = Aero.popupMaxWidth;
var popupPadX = Aero.popupPadX;
var popupPadY = Aero.popupPadY;
var submenuDelay = Aero.submenuDelay;
var menuWidth = Aero.menuWidth;
var menuHeight = Aero.menuHeight;
var menuMargin = Aero.menuMargin;
var menuProgramsWidth = Aero.menuProgramsWidth;
var menuRowHeight = Aero.menuRowHeight;
var menuIconSize = Aero.menuIconSize;
var menuRowHeightSmall = Aero.menuRowHeightSmall;
var menuIconSizeSmall = Aero.menuIconSizeSmall;
var menuHeightSmall = Aero.menuHeightSmall;
var menuPadding = Aero.menuPadding;
var menuSearchHeight = Aero.menuSearchHeight;
var menuFooterHeight = Aero.menuFooterHeight;
// Under a bar at the top, the account picture stays inside the menu.
var menuTileOverhang = 0;
var menuTileSize = Aero.menuTileSize;
var menuGlassBorder = Aero.menuGlassBorder;

// ---------------------------------------------------------------------------
// Colours
// ---------------------------------------------------------------------------

// The bar: a solid slate, a little lighter at its top, a fine line of light
// along its top edge and a dark one under it, as a window's caption plate.
var barTop = "#f2222d3a";
var barUpper = "#f21c2631";
var barMiddle = "#f218212b";
var barLower = "#f2151d26";
var barBottom = "#f2121921";
var barHighlight = "#33ffffff";
var barEdge = "#ff03060a";
var barBottomEdge = "#ff03060a";

// Task buttons: flat plates. At rest a faint slate, under the pointer a
// lighter one, the window you are looking at in the caption's steel blue.
var buttonOutline = "#cc03060a";
var buttonRestTop = "#14ffffff";
var buttonRestUpper = "#0dffffff";
var buttonRestLower = "#08ffffff";
var buttonRestBottom = "#08ffffff";
var buttonRestBorder = "#1affffff";
var buttonHoverTop = "#29ffffff";
var buttonHoverUpper = "#1fffffff";
var buttonHoverLower = "#14ffffff";
var buttonHoverBottom = "#14ffffff";
var buttonHoverBorder = "#33ffffff";
var buttonActiveTop = "#ff3f7fcd";
var buttonActiveUpper = "#ff3571b8";
var buttonActiveLower = "#ff2b5d98";
var buttonActiveBottom = "#ff254f82";
var buttonActiveBorder = "#4dffffff";
var buttonPressedTop = "#ff1d3858";
var buttonPressedUpper = "#ff1d3858";
var buttonPressedLower = "#ff182f4a";
var buttonPressedBottom = "#ff182f4a";

var buttonStack = "#1affffff";
var buttonStackBorder = "#40ffffff";

// Popup menus and thumbnails: slate panels with a dark outline.
var popupTop = "#fa18212b";
var popupBottom = "#fa131a22";
var popupBorder = "#14ffffff";
var popupOutline = "#ff03060a";
var thumbFrame = "#3f7fcd";

// The Start menu.
var menuTop = "#fa18212b";
var menuBottom = "#fa131a22";
var menuPlacesTop = "#fa1d3858";
var menuPlacesBottom = "#fa142840";
var menuBorder = "#14ffffff";
var menuOutline = "#ff03060a";
var menuDivider = "#262f4b66";

// Text.
var textOnGlass = "#eef5fd";
var textMuted = "#7d8a98";
var textOnMenu = "#dbe6f2";
var menuHeading = "#7fb2ea";

// The calendar over the clock.
var calendarLink = "#7fb2ea";
var calendarToday = "#3f7fcd";
var calendarHover = "#263f7fcd";
var calendarOtherMonth = "#55616e";
var textOnPlaces = "#eef5fd";
var selection = "#593f7fcd";
var selectionBorder = "#cc3f7fcd";

var fontFamily = "Inter";
var fontFallback = "Noto Sans";
var fontSize = 12;
var fontSizeSmall = 11;

// ---------------------------------------------------------------------------
// Timing, in milliseconds.
// ---------------------------------------------------------------------------

var fade = Aero.fade;
var menuFade = Aero.menuFade;
var hoverIn = Aero.hoverIn;
var hoverOut = Aero.hoverOut;
