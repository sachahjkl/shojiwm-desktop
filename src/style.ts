/**
 * Which look the desktop wears, as the shell's Control Panel sets it, and
 * whether the system prefers dark.
 *
 * The shell owns both settings: it keeps the style in
 * ~/.config/shojiwm/settings.json and reads light or dark from dconf. It
 * sends them over the socket of `peek.tsx` when it connects and whenever
 * they change; until then the frame is Aero, light.
 */

import { signal, type Signal } from "shoji_wm";

/** `aero`: Windows 7 as it was. `modern`: the same glass, flatter. */
export type Style = "aero" | "modern";

export const STYLE: Signal<Style> = signal<Style>("aero");
export const DARK: Signal<boolean> = signal(false);

/** Apply what the shell sent; anything else is ignored. */
export function applyAppearance(params: unknown): void {
  if (typeof params !== "object" || params === null) {
    return;
  }
  const { style, dark } = params as { style?: unknown; dark?: unknown };
  if (style === "aero" || style === "modern") {
    STYLE.set(style);
  }
  if (typeof dark === "boolean") {
    DARK.set(dark);
  }
}
