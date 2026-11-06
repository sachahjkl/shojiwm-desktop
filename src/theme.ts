/**
 * Every size and colour the window frame is drawn from.
 *
 * The frame is built like the Aero frame of Windows 7: a thin dark outline, a
 * translucent glass body, a bright inner line, then the client surface.
 */

/** Frame metrics, in logical pixels. */
export const FRAME = {
  /** The dark outline around the whole window. */
  outline: 1,
  /** The glass body on the left, right and bottom edges. */
  glass: 5,
  /** The bright line between the glass and the client surface. */
  clientEdge: 1,
  /** Height of the caption bar, outline excluded. */
  caption: 28,
  /** Corner radius of the frame. */
  radius: 6,
} as const;

/** Width the frame adds to the client surface. */
export const FRAME_WIDTH_EXTRA =
  2 * (FRAME.outline + FRAME.glass + FRAME.clientEdge);

/** Height the frame adds to the client surface. */
export const FRAME_HEIGHT_EXTRA =
  FRAME.outline +
  FRAME.caption +
  2 * FRAME.clientEdge +
  FRAME.glass +
  FRAME.outline;

/** Caption button sizes, in logical pixels. */
export const CAPTION_BUTTON = {
  width: 29,
  closeWidth: 45,
  height: 20,
  /** Distance from the button group to the right edge of the caption. */
  inset: 2,
} as const;

/**
 * Colours. Every value is `#rrggbbaa`. The glass colours are tints: the
 * compositor blurs the desktop behind them, and these are painted on top.
 */
export const COLOR = {
  /** Caption and frame tint of the focused window. */
  glassActive: "#5b86ad6b",
  /** Caption and frame tint of every other window. */
  glassInactive: "#8ea3b566",
  /** The dark outline of the focused window. */
  outlineActive: "#15314dcc",
  /** The dark outline of every other window. */
  outlineInactive: "#5c6c7a99",
  /** The bright line around the client surface. */
  clientEdgeActive: "#ffffff73",
  clientEdgeInactive: "#ffffff40",
  /** Caption text. */
  titleActive: "#ffffff",
  titleInactive: "#e8f0f7cc",
  /** The pale copies drawn under the title of the focused window. */
  titleGlow: "#eaf6ff",
  /** Caption button, at rest and under the pointer. */
  buttonRest: "#ffffff1a",
  buttonHover: "#bfe4ff73",
  buttonPress: "#8fc8ef8c",
  buttonBorder: "#ffffff4d",
  /** The close button: red at rest, bright red under the pointer. */
  closeRest: "#c3493599",
  closeHover: "#e85a42e6",
  closePress: "#b63c29f2",
  closeGlow: "#ff5a3aa6",
} as const;

/** Text. Inter is a free face; the fallbacks cover a system without it. */
export const FONT = ["Inter", "Noto Sans", "DejaVu Sans"];

/** Animation lengths, in milliseconds. Short, as on Windows 7. */
export const TIMING = {
  open: 140,
  close: 110,
  minimize: 130,
  management: 160,
} as const;
