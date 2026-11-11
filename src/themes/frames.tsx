/**
 * Every window's frame, drawn by the theme now (choice.ts). Reading the theme
 * here makes the window's composition run again when it changes: the old
 * frame goes, the new one is built (a paint shader cannot change while a
 * frame lives).
 */
import {
  Box,
  ClientWindow,
  WindowBorder,
  computed,
  type CompiledEffectHandle,
  type WaylandWindow,
} from "shoji_wm";
import type { CompositionRenderable } from "shoji_wm/types";
import { ATAB, aeroFrame } from "./aero/frame";
import { KTAB, kernelFrame } from "./kernel/frame";
import { KCOLOR } from "./kernel/metrics";
import { frameMetrics, themeName, type ThemeName } from "./choice";
import { captionTabs, type TabOptions } from "./tabs";
import { highlightShadow } from "../highlight";

/** What a frame is given, whatever the theme. */
export interface FrameOptions {
  /** The compiled blur that stands behind the frame; null for none. */
  glass: CompiledEffectHandle | null;
  /** `true` while the blur must be left out (the Alt+Tab panel is open). */
  glassSuspended: boolean;
  onMinimize: () => void;
  onToggleMaximize: () => void;
  /** The window's group, drawn as tabs in its caption. */
  tabs: TabOptions;
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

/**
 * A client-decorated window alone: no frame of ours, only the border box
 * that carries its highlight (highlight.ts).
 */
export function clientFrame(window: WaylandWindow): CompositionRenderable {
  return (
    <WindowBorder
      style={{ background: "#00000000", boxShadow: highlightShadow(window) }}
    >
      <ClientWindow />
    </WindowBorder>
  );
}

/**
 * The tab strip of a client-decorated window in a group: the theme's tabs on
 * a band of the caption's colour, over the window's own title bar. The window
 * keeps its title bar; the strip is where its group's tabs are.
 */
export function tabStrip(
  window: WaylandWindow,
  options: TabOptions,
): CompositionRenderable {
  const kernel = themeName() === "kernel";
  const style = kernel ? KTAB : ATAB;
  const band = window.isFocused((on: boolean) =>
    kernel
      ? on
        ? KCOLOR.captionRightActive
        : KCOLOR.captionRightInactive
      : on
        ? STRIP_AERO_ACTIVE
        : STRIP_AERO_INACTIVE,
  );
  const height = frameMetrics().caption;
  return (
    <WindowBorder
      style={{ background: "#00000000", boxShadow: highlightShadow(window) }}
    >
      <Box direction="column" style={{ flexGrow: 1 }}>
        <Box
          direction="row"
          style={{
            position: "relative",
            height,
            flexShrink: 0,
            paddingLeft: 4,
            paddingRight: 4,
            alignItems: "end",
            background: band,
          }}
        >
          {/* Kernel: the dark line under a caption, which the shown tab covers */}
          {kernel ? (
            <Box
              style={{
                position: "absolute",
                left: 0,
                bottom: 0,
                width: "100%",
                height: 1,
                background: KCOLOR.outline,
                pointerEvents: "none",
              }}
            />
          ) : null}
          {captionTabs(
            window,
            computed(() => (kernel ? height : height - 3)),
            style,
            options,
            4,
            [],
          )}
        </Box>
        <Box direction="column" style={{ flexGrow: 1, overflow: "hidden" }}>
          <ClientWindow />
        </Box>
      </Box>
    </WindowBorder>
  );
}

/** The band of a strip in Aero: the caption's pale glass, opaque (no blur behind it here). */
const STRIP_AERO_ACTIVE = "#c4d6e6ff";
const STRIP_AERO_INACTIVE = "#d9e2eaff";
