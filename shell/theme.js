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
var taskbarPadding = 3;
var orbSize = 34;
var taskButtonHeight = 30;
var taskButtonMaxWidth = 168;
var taskButtonMinWidth = 44;
var taskButtonGap = 3;
var trayIconSize = 16;
var trayGap = 8;
var clockWidth = 74;
var showDesktopWidth = 10;

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
// ---------------------------------------------------------------------------

// Taskbar glass: dark blue-grey over the blur, lit along the top edge.
var barTop = "#4c6881a8";
var barMiddle = "#1b2c3cc4";
var barBottom = "#16222ecc";
var barHighlight = "#bfd8ef73";
var barEdge = "#0a131bcc";

// Task buttons.
var buttonRest = "#ffffff1c";
var buttonRestBorder = "#ffffff30";
var buttonHover = "#aad4f55e";
var buttonHoverBorder = "#cfe9ff8c";
var buttonActive = "#d6ecff52";
var buttonActiveBorder = "#e8f5ff99";
var buttonGloss = "#ffffff2b";

// Start menu glass.
var menuTop = "#e8f2fbf2";
var menuBottom = "#cfe0eff2";
var menuPlacesTop = "#2f536fe6";
var menuPlacesBottom = "#1d394fe6";
var menuBorder = "#ffffff8c";
var menuOutline = "#12314bcc";
var menuDivider = "#8fb0c959";

// Text.
var textOnGlass = "#f2f8ff";
var textMuted = "#c3d6e6";
var textOnMenu = "#10283c";
var textOnPlaces = "#eaf4ff";
var selection = "#9ad0f766";
var selectionBorder = "#cfe9ffb3";

var fontFamily = "Inter";
var fontFallback = "Noto Sans";
var fontSize = 12;
var fontSizeSmall = 11;

// ---------------------------------------------------------------------------
// Timing, in milliseconds. Short: the desktop this copies was quick.
// ---------------------------------------------------------------------------

var fade = 90;
var menuFade = 120;
