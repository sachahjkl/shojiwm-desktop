/**
 * The Kernel theme: a dark desk of deep blue-black, frames with a caption
 * plate in a gradient from steel blue to night blue, a dark outer line and a
 * line of the caption's tint inside it, crisp vector icons, a monospaced title.
 * Every size and colour of its frame.
 */

/** Frame metrics, in logical pixels at 100%. */
export const KFRAME = {
  caption: 28,
  /** Maximised: the frame's lines are off the screen, the caption a little shorter. */
  captionMaximized: 26,
  /** Left, right and bottom: the dark outer line and the tinted line inside it. */
  border: 2,
  borderMaximized: 0,
  /** Square corners, top and bottom: Kernel draws no rounded frame. */
  radius: 0,
  /** A caption button: as tall as the plate under the frame's line, this wide. */
  buttonWidth: 40,
  closeWidth: 46,
  iconSize: 16,
  iconLeft: 9,
  iconGap: 7,
  /** The icon of a caption button. */
  glyph: 16,
  fontSize: 12.5,
} as const;

export const KERNEL_METRICS = {
  caption: KFRAME.caption,
  captionMaximized: KFRAME.captionMaximized,
  border: KFRAME.border,
  borderMaximized: KFRAME.borderMaximized,
  // The tab strip of a group (groups.ts): where the icon starts, to the
  // caption buttons (two of 40 px, the close one of 46).
  tabsLeft: KFRAME.iconLeft,
  tabsRight: 1 + 2 * KFRAME.buttonWidth + KFRAME.closeWidth + KFRAME.iconGap,
  tabMaxWidth: 220,
};

/** Colours, `#rrggbbaa`. */
export const KCOLOR = {
  /** The caption plate, left to right. */
  captionLeftActive: "#3f7fcdff",
  captionRightActive: "#1d3858ff",
  captionLeftInactive: "#2d3742ff",
  captionRightInactive: "#1a2028ff",
  /** The outer line, round the whole window. */
  outline: "#03060aff",
  outlineInactive: "#05080cff",
  /** The line inside it, of the caption's tint. */
  tintActive: "#2c5d93ff",
  tintInactive: "#29313aff",
  title: "#eef5fdff",
  titleInactive: "#7d8a98ff",
  /** The close button's plate, top to bottom. */
  closeTop: "#e4545eff",
  closeBottom: "#a11f2bff",
  closeTopHover: "#f26a73ff",
  closeBottomHover: "#b92a37ff",
  closeTopInactive: "#5c3d44ff",
  closeBottomInactive: "#3b262bff",
  /** A minimise or maximise button: clear until the pointer is on it. */
  buttonHover: "#ffffff1f",
  buttonPress: "#00000040",
  /** A group's tabs: the shown one is a plate of the frame's tint, joined to the client, a bar of the accent on its top. */
  tabActive: "#2c5d93ff",
  tabActiveAway: "#29313aff",
  tabAccent: "#6aa8f0ff",
  tabAccentAway: "#5b6876ff",
  /** The other tabs: recessed plates, darker than the caption. */
  tabIdle: "#14263aff",
  tabIdleAway: "#13181eff",
  tabHover: "#1f3a58ff",
  tabText: "#b9c6d4ff",
  tabTextAway: "#aab5c1ff",
  tabTextIdleAway: "#8d99a6ff",
  /** The ghost tab while a window is dragged on a caption: a dashed box of the accent. */
  tabGhost: "#3f7fcd33",
  tabGhostLine: "#8cbcf2ff",
  tooltip: "#0f1620f2",
  tooltipLine: "#2c5d93ff",
  tooltipText: "#dbe6f2ff",
} as const;

/** The title's face: monospaced, crisp at every size (no bitmap). */
export const KFONT_TITLE = [
  "JetBrainsMono NF",
  "JetBrains Mono",
  "DejaVu Sans Mono",
];
/** Text in the frame's tooltips. */
export const KFONT_TEXT = ["Inter", "Noto Sans", "DejaVu Sans"];
