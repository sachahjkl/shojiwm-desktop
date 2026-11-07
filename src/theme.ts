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
 * The four lines were measured, then softened by eye against Windows 7
 * screenshots the owner supplied: at full strength the black outline and
 * the white bevel read as harsh, where Windows' read as soft.
 *
 * The glass is a tint over the compositor's blur, so its alpha matters: the
 * numbers below reproduce both a window over a black desktop and one over a
 * pale sky, which is how they were solved.
 */
export const COLOR = {
  /** The glass of an active window. */
  glassActive: "#6f9fcc3d",
  /**
   * The glass of every other window. Windows drains the colour out of it and
   * makes it heavier, so the caption stays legible over any wallpaper.
   */
  glassInactive: "#c2ced766",
  /** The 1 px line around the outside. */
  outlineActive: "#15212ead",
  outlineInactive: "#2a36408c",
  /** The 1 px light line just inside it. */
  highlightActive: "#ffffff99",
  highlightInactive: "#ffffff73",
  /** The two lines around the client area. */
  clientLightActive: "#ffffff8c",
  clientLightInactive: "#ffffff80",
  clientDarkActive: "#1b2b3a99",
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
