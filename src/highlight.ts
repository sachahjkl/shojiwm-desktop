/**
 * The window the taskbar's thumbnails point at. While the pointer rests on a
 * window's thumbnail, the shell names that window over its socket
 * (peek.tsx, method `highlight`), and the window gets a soft gold glow and a
 * gold outline, and stands over the other windows, so that the user can spot
 * it. The keyboard focus does not change.
 *
 * The shell names a window by its `ext_foreign_toplevel_list` identifier: the
 * one name both sides can know for the window's whole life, which a title is
 * not.
 */

import {
  animationVariable,
  computed,
  isSignal,
  signal,
  type BoxShadowValue,
  type MaybeSignal,
  type ReadonlySignal,
  type WaylandWindow,
} from "shoji_wm";
import { motion } from "./motion";
import { WIN_MINIMIZED } from "./window-manager";

/** The look of a highlighted window. */
const HIGHLIGHT = {
  /** The glow and the outline, without alpha: #e8b84a. */
  red: 0xe8,
  green: 0xb8,
  blue: 0x4a,
  /** The glow's alpha at full strength. */
  glowAlpha: 0.55,
  glowBlur: 20,
  glowSpread: 2,
  /** The outline: a hard shadow, so it follows the frame's rounded corners. */
  outlineAlpha: 0.9,
  outlineWidth: 2,
  /** The fade in and out, in milliseconds. */
  fade: 120,
  /** Over every window but a fullscreen one (index.tsx, 2_000_000_000). */
  zIndex: 1_900_000_000,
} as const;

/** The strength of a window's highlight, 0 to 1, per window. */
const STRENGTH = animationVariable("highlight");

/** The highlighted window's id (`WaylandWindow.id`), or null. */
const highlighted = signal<string | null>(null);
/** The highlighted window, to fade it out when another one takes its place. */
let current: WaylandWindow | null = null;

function fade(window: WaylandWindow, to: number): void {
  const duration = motion(HIGHLIGHT.fade);
  if (duration === 0) {
    window.animation.set(STRENGTH, to);
  } else {
    void window.animation.start(STRENGTH, { duration, to });
  }
}

/**
 * Highlights the window the shell names, or none. `identifier` is its
 * `ext_foreign_toplevel_list` identifier. A minimized window is not
 * highlighted: there is nothing of it on the screen.
 */
export function setHighlight(
  identifier: string | null,
  windows: readonly WaylandWindow[],
): boolean {
  const window =
    identifier === null
      ? null
      : (windows.find((w) => w.foreignToplevelId() === identifier) ?? null);
  const shown = window && !window.state[WIN_MINIMIZED]() ? window : null;
  if (shown === current) {
    return shown !== null;
  }
  if (current) {
    fade(current, 0);
  }
  current = shown;
  highlighted.set(shown?.id ?? null);
  if (shown) {
    fade(shown, 1);
  }
  return shown !== null;
}

/** Forgets a window that closes. */
export function forgetHighlight(window: WaylandWindow): void {
  if (current?.id === window.id) {
    current = null;
    highlighted.set(null);
  }
}

function gold(alpha: number): string {
  const byte = (value: number) => value.toString(16).padStart(2, "0");
  const a = Math.round(Math.min(1, Math.max(0, alpha)) * 255);
  return `#${byte(HIGHLIGHT.red)}${byte(HIGHLIGHT.green)}${byte(HIGHLIGHT.blue)}${byte(a)}`;
}

function valueOf<T>(value: MaybeSignal<T>): T {
  return isSignal<T>(value) ? value() : value;
}

/**
 * A frame's shadow with the window's highlight added over it: the outline,
 * then the glow, then the frame's own shadow.
 */
export function highlightShadow(
  window: WaylandWindow,
  base: MaybeSignal<BoxShadowValue | BoxShadowValue[]> = [],
): ReadonlySignal<BoxShadowValue[]> {
  const strength = window.animation.variable(STRENGTH);
  return computed(() => {
    const own = valueOf(base);
    const layers = Array.isArray(own) ? own : [own];
    const t = strength();
    if (t <= 0) {
      return layers;
    }
    return [
      {
        blur: 0,
        spread: HIGHLIGHT.outlineWidth,
        color: gold(HIGHLIGHT.outlineAlpha * t),
      },
      {
        blur: HIGHLIGHT.glowBlur,
        spread: HIGHLIGHT.glowSpread,
        color: gold(HIGHLIGHT.glowAlpha * t),
      },
      ...layers,
    ];
  });
}

/** The window's z index, over the others while it is highlighted. */
export function highlightZIndex(
  window: WaylandWindow,
  zIndex: ReadonlySignal<number>,
): ReadonlySignal<number> {
  return computed(() =>
    highlighted() === window.id ? HIGHLIGHT.zIndex : zIndex(),
  );
}
