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
   * The same, maximised. Windows pushes the frame off the screen then: only
   * the caption bar is left, and the client meets the screen's edges.
   */
  captionMaximized: 22,
  /** Left, right and bottom. */
  border: 8,
  /** None maximised: the client fills the width and reaches the taskbar. */
  borderMaximized: 0,
  /** The two top corners. */
  radiusTop: 8,
  /** The two bottom corners: as round as the top ones, as on Windows 7. */
  radiusBottom: 8,
  /**
   * The caption button group hangs from the top edge of the window: it
   * starts right under the dark outline, covering the light line there, and
   * has no top border of its own. Maximised, it starts at the very top.
   */
  buttonHeight: 18,
  buttonTop: 1,
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
  /** The height of the title line. */
  titleHeight: 16,
} as const;

/** The room the frame takes around the client (themes/choice.ts). */
export const AERO_METRICS = {
  caption: FRAME.caption,
  captionMaximized: FRAME.captionMaximized,
  border: FRAME.border,
  borderMaximized: FRAME.borderMaximized,
};

/** Widths of the three caption buttons, dark separators included. */
export const CAPTION_BUTTON = {
  minimize: 28,
  maximize: 27,
  close: 48,
} as const;

/**
 * Colours. Every value is `#rrggbbaa`, the order ShojiWM reads.
 *
 * The four lines and the glass follow the ShojiWM author's own Windows 7
 * setup (a screenshot the owner sent). The lines are translucent black and
 * white: over his blue wallpaper they read as navy and pale cyan, and the
 * alphas below are what gives those colours over that blue. Outside, black
 * then white; round the client, white then black.
 *
 * The glass is a tint over the compositor's blur, so its alpha matters: the
 * numbers below reproduce both a window over a black desktop and one over a
 * pale sky, which is how they were solved.
 */
export const COLOR = {
  /** The glass of an active window. */
  glassActive: "#d2e4f429",
  /**
   * The glass of every other window. Windows drains the colour out of it and
   * makes it heavier, so the caption stays legible over any wallpaper.
   */
  glassInactive: "#e2e9ef57",
  /** The 1 px line around the outside. */
  outlineActive: "#000000a3",
  outlineInactive: "#00000073",
  /** The 1 px light line just inside it. */
  highlightActive: "#ffffff9c",
  highlightInactive: "#ffffff73",
  /** The two lines around the client area. */
  clientLightActive: "#ffffff8c",
  clientLightInactive: "#ffffff80",
  clientDarkActive: "#00000073",
  clientDarkInactive: "#0000004d",
  /** Caption text: black on a soft white glow, as Windows 7 draws it, so it
   * reads over any glass. */
  titleActive: "#000000",
  titleInactive: "#3a464f",
  titleGlowActive: "#ffffffe6",
  titleGlowInactive: "#ffffffa6",
} as const;

/**
 * The faces to ask for, in order. Selawik is Microsoft's own metric match
 * for Segoe UI under the MIT licence; Open Sans is the closest thing in
 * nixpkgs.
 */
export const FONT = ["Selawik", "Open Sans", "Noto Sans", "DejaVu Sans"];
