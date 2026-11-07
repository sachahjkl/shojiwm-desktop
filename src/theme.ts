/**
 * Every size and colour of the window frame, measured off real Windows 7
 * screenshots. The readings and their sources are in
 * `docs/aero-measurements.md`; change a number here only with a measurement
 * to back it.
 */

/** Frame metrics, in logical pixels at 100%. */
export const FRAME = {
  /** Window top to client top. */
  caption: 30,
  /**
   * The same, maximised. Windows pushes the frame off the screen then, so
   * only the two lines around the client are left of the border.
   */
  captionMaximized: 22,
  /** Left, right and bottom. */
  border: 8,
  borderMaximized: 2,
  /** The two top corners. */
  radiusTop: 6,
  /** The two bottom corners. */
  radiusBottom: 5,
  /**
   * The caption button group hangs from the top edge of the window: it
   * starts on the first glass row, right under the two lines of the frame,
   * and has no top border of its own. Maximised, it starts at the very top.
   */
  buttonHeight: 18,
  buttonTop: 2,
  buttonTopMaximized: 0,
  /** Distance from the group's right edge to the window's right edge. */
  buttonRight: 7,
  buttonRightMaximized: 1,
  /** The icon, and the space around it. */
  iconSize: 16,
  iconLeft: 11,
  iconGap: 3,
  /** Caption text: Segoe UI 9 pt is 12 px at 96 DPI. */
  fontSize: 12,
  /** The box the title and its glow live in. */
  titleHeight: 16,
  /** How far the glow reaches past the text. */
  glowMargin: 9,
} as const;

/** Width the frame adds to the client surface. */
export const FRAME_WIDTH_EXTRA = 2 * FRAME.border;

/** Height the frame adds to the client surface. */
export const FRAME_HEIGHT_EXTRA = FRAME.caption + FRAME.border;

/** Widths of the three caption buttons, dark separators included. */
export const CAPTION_BUTTON = {
  minimize: 28,
  maximize: 27,
  close: 48,
} as const;

/**
 * Colours. Every value is `#rrggbbaa`, the order ShojiWM reads.
 *
 * The glass is a tint over the compositor's blur, so its alpha matters: the
 * numbers below reproduce both a window over a black desktop and one over a
 * pale sky, which is how they were solved.
 */
export const COLOR = {
  /** The glass of an active window. */
  glassActive: "#538ebe7a",
  /**
   * The glass of every other window. Windows drains the colour out of it and
   * makes it heavier, so the caption stays legible over any wallpaper.
   */
  glassInactive: "#c2ced79e",
  /** The 1 px line around the outside. */
  outlineActive: "#000000cc",
  outlineInactive: "#243039b3",
  /** The 1 px light line just inside it. */
  highlightActive: "#ffffffeb",
  highlightInactive: "#ffffffb3",
  /** The two lines around the client area. */
  clientLightActive: "#ffffffb3",
  clientLightInactive: "#ffffff80",
  clientDarkActive: "#0b1b2abf",
  clientDarkInactive: "#55656fb3",
  /** Caption text. */
  titleActive: "#000000",
  titleInactive: "#46545e",
} as const;

/**
 * The faces to ask for, in order. Selawik is Microsoft's own metric match
 * for Segoe UI under the MIT licence; Open Sans is the closest thing in
 * nixpkgs.
 */
export const FONT = ["Selawik", "Open Sans", "Noto Sans", "DejaVu Sans"];

/** Animation lengths, in milliseconds. Short, as on Windows 7. */
export const TIMING = {
  open: 140,
  close: 110,
  minimize: 130,
  management: 160,
} as const;

// ---------------------------------------------------------------------------
// The modern style: the same glass, without what dates it. Not measured off
// anything: a flatter, quieter take on the frame above. Rounder corners,
// thinner sides, no outer black line or white bevel, no glow behind the
// title, flat caption buttons, and a tint that follows light or dark.
// ---------------------------------------------------------------------------

export const MODERN_FRAME = {
  caption: 34,
  captionMaximized: 32,
  border: 4,
  borderMaximized: 0,
  radius: 9,
  /** The caption buttons: separate, rounded, and lit only under the pointer. */
  buttonWidth: 38,
  buttonHeight: 26,
  buttonGap: 2,
  buttonRadius: 6,
  buttonRight: 5,
  buttonTop: 4,
  buttonTopMaximized: 3,
  glyphSize: 10,
  iconLeft: 12,
  iconGap: 8,
  fontSize: 12,
} as const;

export const MODERN_FRAME_WIDTH_EXTRA = 2 * MODERN_FRAME.border;
export const MODERN_FRAME_HEIGHT_EXTRA =
  MODERN_FRAME.caption + MODERN_FRAME.border;

/** Colours of the modern frame, light and dark. `#rrggbbaa`. */
export const MODERN_COLOR = {
  light: {
    glassActive: "#e8f0f7a3",
    glassInactive: "#eef1f4d9",
    outlineActive: "#0000003d",
    outlineInactive: "#00000026",
    highlightActive: "#ffffff73",
    highlightInactive: "#ffffff4d",
    clientLight: "#00000000",
    clientDark: "#0000001a",
    titleActive: "#1b2733",
    titleInactive: "#7a858f",
    buttonHover: "#0000000f",
    buttonPress: "#0000001f",
  },
  dark: {
    glassActive: "#1b232cb3",
    glassInactive: "#242a31e0",
    outlineActive: "#000000b3",
    outlineInactive: "#00000080",
    highlightActive: "#ffffff26",
    highlightInactive: "#ffffff14",
    clientLight: "#00000000",
    clientDark: "#00000059",
    titleActive: "#eef3f8",
    titleInactive: "#8d98a3",
    buttonHover: "#ffffff17",
    buttonPress: "#ffffff0d",
  },
  /** The close button under the pointer, and pressed: the same in both. */
  closeHover: "#c42b1c",
  closePress: "#a3261a",
} as const;
