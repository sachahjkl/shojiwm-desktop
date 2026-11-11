/**
 * The desktop's theme: how window frames are drawn (and, in the shell, every
 * surface of its own). Each theme is a folder of `src/themes`: its metrics, its
 * colours and its frame. The shell sends the Control Panel's choice over its
 * socket (`theme`); every window's frame reads it, so a change draws them all
 * again, and the space a frame takes (`frameMetrics`) follows.
 *
 * This file holds the choice and the metrics only: the window manager reads
 * them, and the frames (`frames.tsx`) import the window manager.
 */
import { signal } from "shoji_wm";
import type { TabMetrics } from "../groups";
import { AERO_METRICS } from "./aero/metrics";
import { KERNEL_METRICS } from "./kernel/metrics";

export type ThemeName = "aero" | "kernel";

export const THEME_NAMES: readonly ThemeName[] = ["aero", "kernel"];

/**
 * The room a frame takes around the client surface, and its tab strip
 * (groups.ts), in logical pixels.
 */
export interface FrameMetrics extends TabMetrics {
  /** Window top to client top. */
  caption: number;
  /** The same, maximised. */
  captionMaximized: number;
  /** Left, right and bottom. */
  border: number;
  /** The same, maximised. */
  borderMaximized: number;
}

const METRICS: Record<ThemeName, FrameMetrics> = {
  aero: AERO_METRICS,
  kernel: KERNEL_METRICS,
};

const chosen = signal<ThemeName>("kernel");

/** The theme now. Reading it makes the reader draw again on a change. */
export function themeName(): ThemeName {
  return chosen();
}

/** The theme now, without following it. */
export function themeNamePeek(): ThemeName {
  return chosen.peek();
}

/** The shell's choice: `{ name }`, one of THEME_NAMES; anything else is Kernel. */
export function setTheme(params: unknown): ThemeName {
  const name =
    typeof params === "object" && params !== null
      ? (params as { name?: unknown }).name
      : undefined;
  const theme = THEME_NAMES.includes(name as ThemeName)
    ? (name as ThemeName)
    : "kernel";
  chosen.set(theme);
  return theme;
}

/** The frame's room in the theme now (not followed: a window's rect is read when it moves). */
export function frameMetrics(): FrameMetrics {
  return METRICS[chosen.peek()];
}
