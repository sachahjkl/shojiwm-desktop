/**
 * Every window's frame, drawn by the theme now (choice.ts). Reading the theme
 * here makes the window's composition run again when it changes: the old
 * frame goes, the new one is built (a paint shader cannot change while a
 * frame lives).
 */
import type { CompiledEffectHandle, WaylandWindow } from "shoji_wm";
import type { CompositionRenderable } from "shoji_wm/types";
import { aeroFrame } from "./aero/frame";
import { kernelFrame } from "./kernel/frame";
import { themeName, type ThemeName } from "./choice";

/** What a frame is given, whatever the theme. */
export interface FrameOptions {
  /** The compiled blur that stands behind the frame; null for none. */
  glass: CompiledEffectHandle | null;
  /** `true` while the blur must be left out (the Alt+Tab panel is open). */
  glassSuspended: boolean;
  onMinimize: () => void;
  onToggleMaximize: () => void;
}

const FRAMES: Record<
  ThemeName,
  (window: WaylandWindow, options: FrameOptions) => CompositionRenderable
> = {
  aero: aeroFrame,
  kernel: kernelFrame,
};

export function windowFrame(
  window: WaylandWindow,
  options: FrameOptions,
): CompositionRenderable {
  return FRAMES[themeName()](window, options);
}
