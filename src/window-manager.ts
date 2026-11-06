/**
 * A floating window manager, in the manner of Windows 7.
 *
 * Every window floats. The manager owns one rectangle per window, places new
 * windows, and answers the move, resize, maximise, minimise, fullscreen and
 * activate requests that come from the title bar, from a keybinding, or from
 * the taskbar through wlr-foreign-toplevel.
 *
 * It keeps no workspaces and no tiling. That is the whole feature set of the
 * desktop this config reproduces.
 */

import {
  COMPOSITOR,
  createWindowStack,
  createWindowState,
  cubicBezier,
  read,
  signal,
  type PointerMoveEvent,
  type ReadonlySignal,
  type Signal,
  type WaylandWindow,
  type WindowActivateRequestEvent,
  type WindowFullscreenRequestEvent,
  type WindowMaximizeRequestEvent,
  type WindowMinimizeRequestEvent,
  type WindowMoveEvent,
  type WindowResizeEvent,
} from "shoji_wm";
import type {
  ManagedWindowRect,
  WindowDecorationState,
  WindowSizeConstraints,
} from "shoji_wm/types";
import { FRAME_HEIGHT_EXTRA, FRAME_WIDTH_EXTRA, TIMING } from "./theme";

/** The frame rectangle of a window: the chrome, not the client surface. */
export const WIN_RECT = createWindowState<ManagedWindowRect>("rect", {
  default: (window) => window.rect,
});
/** Where an unmaximised window goes back to. */
export const WIN_RESTORE_RECT = createWindowState<ManagedWindowRect | null>(
  "restoreRect",
  { default: null },
);
/** Where an unfullscreened window goes back to. */
export const WIN_FULLSCREEN_RECT = createWindowState<ManagedWindowRect | null>(
  "fullscreenRestoreRect",
  { default: null },
);
export const WIN_MINIMIZED = createWindowState<boolean>("minimized", {
  default: false,
});
export const WIN_MAXIMIZED = createWindowState<boolean>("maximized", {
  default: false,
});
export const WIN_FULLSCREEN = createWindowState<boolean>("fullscreen", {
  default: false,
});
/** `true` once the manager has given the window its first rectangle. */
export const WIN_PLACED = createWindowState<boolean>("placed", {
  default: false,
});

const OPEN_EASING = cubicBezier(0.1, 0.9, 0.2, 1);
const CLOSE_EASING = cubicBezier(0.4, 0, 1, 1);
const MINIMIZE_CHANNEL = "win7.minimize";
const OPEN_CHANNEL = "win7.open";
const CLOSE_CHANNEL = "win7.close";

/** Two presses this close together, in milliseconds, are a double click. */
const DOUBLE_CLICK_MS = 400;
/** How far a press may move and still count as a click, in logical pixels. */
const CLICK_SLACK_PX = 4;
/** How close to an edge the pointer must be for Aero Snap, in logical px. */
const SNAP_EDGE_PX = 8;

/** Where a dragged window would land. */
export type SnapZoneKind = "maximize" | "left" | "right";

export interface SnapZone {
  kind: SnapZoneKind;
  outputName: string;
  rect: Rect;
}

/** How far each new window is placed below and right of the previous one. */
const CASCADE_STEP = 26;
/** How many cascade steps before the placement returns to the first slot. */
const CASCADE_SLOTS = 8;

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

/** A rectangle of plain numbers. */
export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

function snapshotRect(rect: ManagedWindowRect): Rect {
  return {
    x: read(rect.x),
    y: read(rect.y),
    width: read(rect.width),
    height: read(rect.height),
  };
}

/**
 * `true` while the client draws its own title bar. Such a window is shown
 * bare: adding our frame to it would give it two title bars.
 */
export function usesClientDecorationState(
  decoration: WindowDecorationState,
): boolean {
  return (
    decoration.mode === "client" &&
    !(
      decoration.clientPreference === "server" &&
      decoration.configuredMode === "server"
    )
  );
}

/** `usesClientDecorationState` for the current, non-reactive value. */
export function usesClientDecoration(window: WaylandWindow): boolean {
  return usesClientDecorationState(window.decoration.peek());
}

/** What the frame adds to the client surface, per axis. */
export function frameExtra(window: WaylandWindow): {
  width: number;
  height: number;
} {
  if (usesClientDecoration(window)) {
    return { width: 0, height: 0 };
  }
  return { width: FRAME_WIDTH_EXTRA, height: FRAME_HEIGHT_EXTRA };
}

/** The frame rectangle a window would have at the size the client chose. */
export function naturalRect(window: WaylandWindow): Rect {
  const client = window.position;
  const extra = frameExtra(window);
  // The left, right and bottom insets are equal; the rest of the height is
  // the caption bar on top.
  const sideInset = extra.width / 2;
  return {
    x: client.x - sideInset,
    y: client.y - (extra.height - sideInset),
    width: client.width + extra.width,
    height: client.height + extra.height,
  };
}

function maxSize(
  constraints: WindowSizeConstraints,
  axis: "width" | "height",
  extra: number,
): number {
  const max = constraints.max?.[axis];
  return max && max > 0 ? max + extra : Number.POSITIVE_INFINITY;
}

export class FloatingWindowManager {
  private readonly stack = createWindowStack();
  private readonly windowsById = new Map<string, WaylandWindow>();
  /** Window ids, most recently focused first. Drives Alt+Tab. */
  private mru: string[] = [];
  private cascade = 0;
  private currentOutput: string | undefined;
  /** Windows that "show desktop" minimised, so the next call restores them. */
  private hiddenByShowDesktop: string[] = [];
  /** The window being dragged, and the rectangle its drag started from. */
  private moveDrag: { id: string; base: Rect } | null = null;
  /** The last caption press that neither moved nor dragged the window. */
  private lastCaptionClick: {
    id: string;
    at: number;
    x: number;
    y: number;
  } | null = null;
  /** The window whose current drag is ignored (a double click started it). */
  private suppressedDragId: string | null = null;
  /**
   * Where the window would land if the drag ended now, for the preview the
   * composition draws. `null` while no edge is touched.
   */
  public readonly snapPreview: Signal<SnapZone | null> =
    signal<SnapZone | null>(null);

  // ----- queries ----------------------------------------------------------

  /** Stacking order of one window; higher is in front. */
  public zIndex(window: WaylandWindow): ReadonlySignal<number> {
    return this.stack.zIndex(window);
  }

  /** Every managed window, most recently focused first. */
  public recentWindows(): WaylandWindow[] {
    const windows: WaylandWindow[] = [];
    for (const id of this.mru) {
      const window = this.windowsById.get(id);
      if (window) {
        windows.push(window);
      }
    }
    return windows;
  }

  /** The output the pointer was last seen on. */
  public outputName(): string | undefined {
    return this.currentOutput ?? COMPOSITOR.output.list[0];
  }

  // ----- placement --------------------------------------------------------

  /** The area a maximised window covers: the output minus the taskbar. */
  private workArea(outputName: string | undefined): Rect | null {
    if (!outputName) {
      return null;
    }
    const usable = COMPOSITOR.layer.usableArea(outputName);
    if (usable) {
      return {
        x: usable.x,
        y: usable.y,
        width: usable.width,
        height: usable.height,
      };
    }
    const output = COMPOSITOR.output.get(outputName);
    if (!output?.resolution) {
      return null;
    }
    return {
      x: output.position.x,
      y: output.position.y,
      width: output.resolution.width / output.scale,
      height: output.resolution.height / output.scale,
    };
  }

  /** The whole output, taskbar included. Used by fullscreen. */
  private outputRect(outputName: string | undefined): Rect | null {
    if (!outputName) {
      return null;
    }
    const output = COMPOSITOR.output.get(outputName);
    if (!output?.resolution) {
      return null;
    }
    return {
      x: output.position.x,
      y: output.position.y,
      width: output.resolution.width / output.scale,
      height: output.resolution.height / output.scale,
    };
  }

  private outputOf(window: WaylandWindow): string | undefined {
    return window.output.peek() ?? this.outputName();
  }

  /**
   * The first rectangle of a window: its own size, cascaded from the middle
   * of the work area and kept inside it.
   */
  private initialRect(window: WaylandWindow): Rect {
    const natural = naturalRect(window);
    const area = this.workArea(this.outputOf(window));
    if (!area) {
      return natural;
    }

    const width = Math.min(natural.width, area.width);
    const height = Math.min(natural.height, area.height);
    const offset = (this.cascade % CASCADE_SLOTS) * CASCADE_STEP;
    this.cascade += 1;

    const x =
      area.x +
      (area.width - width) / 2 -
      (CASCADE_SLOTS * CASCADE_STEP) / 2 +
      offset;
    const y =
      area.y +
      (area.height - height) / 3 -
      (CASCADE_SLOTS * CASCADE_STEP) / 2 +
      offset;
    return {
      x: Math.round(clamp(x, area.x, area.x + area.width - width)),
      y: Math.round(clamp(y, area.y, area.y + area.height - height)),
      width: Math.round(width),
      height: Math.round(height),
    };
  }

  // ----- lifecycle --------------------------------------------------------

  public onOpen(window: WaylandWindow): void {
    this.windowsById.set(window.id, window);
    this.stack.add(window);
    this.mru = [window.id, ...this.mru.filter((id) => id !== window.id)];
  }

  public onFirstCommit(window: WaylandWindow): void {
    if (window.state[WIN_PLACED]()) {
      return;
    }
    window.state[WIN_PLACED].set(true);

    if (window.isFullscreen()) {
      this.setFullscreen(window, true);
    } else if (window.isMaximized()) {
      window.state[WIN_RESTORE_RECT].set(this.initialRect(window));
      this.setMaximized(window, true);
    } else {
      window.state[WIN_RECT].set(this.initialRect(window));
    }

    window.scheduleAnimation({
      channel: OPEN_CHANNEL,
      rect: {
        from: { x: 0, y: 24, width: 0, height: 0 },
        to: { x: 0, y: 0, width: 0, height: 0 },
        duration: TIMING.open,
        easing: OPEN_EASING,
        mode: "add",
      },
      opacity: {
        from: 0,
        to: 1,
        duration: TIMING.open,
        easing: OPEN_EASING,
        mode: "multiply",
      },
    });
  }

  public onStartClose(window: WaylandWindow): void {
    window.setCloseAnimationDuration(TIMING.close);
    window.scheduleAnimation({
      channel: CLOSE_CHANNEL,
      rect: {
        from: { x: 0, y: 0, width: 0, height: 0 },
        to: { x: 0, y: 16, width: 0, height: 0 },
        duration: TIMING.close,
        easing: CLOSE_EASING,
        mode: "add",
      },
      opacity: {
        from: 1,
        to: 0,
        duration: TIMING.close,
        easing: CLOSE_EASING,
        mode: "multiply",
      },
    });
  }

  public onClose(window: WaylandWindow): void {
    this.stack.remove(window);
    this.windowsById.delete(window.id);
    this.mru = this.mru.filter((id) => id !== window.id);
    this.focusTopWindow();
  }

  public onFocus(window: WaylandWindow, focused: boolean): void {
    if (!focused) {
      return;
    }
    this.stack.raise(window);
    this.mru = [window.id, ...this.mru.filter((id) => id !== window.id)];
    this.currentOutput = this.outputOf(window);
  }

  public onPointerMove(event: PointerMoveEvent): void {
    if (event.outputName) {
      this.currentOutput = event.outputName;
    }
  }

  // ----- move and resize --------------------------------------------------

  public onWindowMove(event: WindowMoveEvent): void {
    const window = event.window;
    if (window.state[WIN_FULLSCREEN]()) {
      return;
    }

    // The second click of a double click on the caption bar also starts a
    // move. Ignore that move until the button is released.
    if (this.suppressedDragId === window.id) {
      if (event.phase === "end" || event.phase === "cancel") {
        this.suppressedDragId = null;
      }
      return;
    }

    if (event.phase === "start") {
      if (this.isDoubleClick(event)) {
        this.suppressedDragId = window.id;
        this.lastCaptionClick = null;
        this.toggleMaximized(window);
        return;
      }
      this.lastCaptionClick = null;
    }

    if (event.phase === "start" || this.moveDrag?.id !== window.id) {
      window.focus();
      this.moveDrag = {
        id: window.id,
        base: this.moveBaseRect(event),
      };
    }

    const drag = this.moveDrag;
    if (!drag) {
      return;
    }

    if (event.phase === "cancel") {
      window.state[WIN_RECT].set(drag.base);
      this.moveDrag = null;
      this.setSnapPreview(null);
      return;
    }

    window.state[WIN_RECT].set({
      x: drag.base.x + (event.currentRect.x - event.startRect.x),
      y: drag.base.y + (event.currentRect.y - event.startRect.y),
      width: drag.base.width,
      height: drag.base.height,
    });

    this.setSnapPreview(this.snapZoneAt(event));

    if (event.phase === "end") {
      const zone = this.snapPreview.peek();
      this.setSnapPreview(null);
      this.moveDrag = null;

      // A click that did not move the window can be the first half of a
      // double click.
      if (
        event.source === "ssd" &&
        Math.abs(event.delta.x) <= CLICK_SLACK_PX &&
        Math.abs(event.delta.y) <= CLICK_SLACK_PX
      ) {
        this.lastCaptionClick = {
          id: window.id,
          at: event.timestamp,
          x: event.startPointer.x,
          y: event.startPointer.y,
        };
      }

      if (zone) {
        this.applySnap(window, zone.kind);
      }
    }
  }

  /** `true` when this press follows a press that neither moved nor dragged. */
  private isDoubleClick(event: WindowMoveEvent): boolean {
    const previous = this.lastCaptionClick;
    return (
      event.source === "ssd" &&
      previous !== null &&
      previous.id === event.window.id &&
      event.timestamp - previous.at <= DOUBLE_CLICK_MS &&
      Math.abs(event.startPointer.x - previous.x) <= CLICK_SLACK_PX &&
      Math.abs(event.startPointer.y - previous.y) <= CLICK_SLACK_PX
    );
  }

  /**
   * Aero Snap: the zone the pointer is in while dragging, or `null`. The top
   * edge maximises, the left and right edges take half the work area.
   */
  private snapZoneAt(event: WindowMoveEvent): SnapZone | null {
    const outputName = event.outputName ?? this.outputName();
    const screen = this.outputRect(outputName);
    const area = this.workArea(outputName);
    if (!outputName || !screen || !area) {
      return null;
    }

    const x = event.currentPointer.x;
    const y = event.currentPointer.y;
    const halfWidth = Math.round(area.width / 2);

    if (y <= screen.y + SNAP_EDGE_PX) {
      return { kind: "maximize", outputName, rect: area };
    }
    if (x <= screen.x + SNAP_EDGE_PX) {
      return {
        kind: "left",
        outputName,
        rect: { x: area.x, y: area.y, width: halfWidth, height: area.height },
      };
    }
    if (x >= screen.x + screen.width - SNAP_EDGE_PX) {
      return {
        kind: "right",
        outputName,
        rect: {
          x: area.x + area.width - halfWidth,
          y: area.y,
          width: halfWidth,
          height: area.height,
        },
      };
    }
    return null;
  }

  private setSnapPreview(zone: SnapZone | null): void {
    const current = this.snapPreview.peek();
    if (current === zone) {
      return;
    }
    if (current && zone && current.kind === zone.kind) {
      return;
    }
    this.snapPreview.set(zone);
  }

  private applySnap(window: WaylandWindow, kind: SnapZoneKind): void {
    if (kind === "maximize") {
      this.setMaximized(window, true);
      return;
    }
    this.snap(window, kind);
  }

  /**
   * The rectangle a drag starts from. Dragging a maximised window restores
   * it, as on Windows 7: it takes back its old size under the pointer.
   */
  private moveBaseRect(event: WindowMoveEvent): Rect {
    const window = event.window;
    if (!window.state[WIN_MAXIMIZED]()) {
      return snapshotRect(event.startRect);
    }

    const restore = window.state[WIN_RESTORE_RECT]() ?? naturalRect(window);
    const width = read(restore.width);
    const height = read(restore.height);
    window.state[WIN_MAXIMIZED].set(false);
    window.unmaximize();

    // Keep the pointer at the same place along the caption bar.
    const ratio =
      event.startRect.width > 0
        ? (event.startPointer.x - event.startRect.x) / event.startRect.width
        : 0.5;
    return {
      x: Math.round(event.startPointer.x - width * ratio),
      y: Math.round(event.startPointer.y - FRAME_HEIGHT_EXTRA / 2),
      width,
      height,
    };
  }

  public onWindowResize(event: WindowResizeEvent): void {
    const window = event.window;
    if (!window.isResizable() || window.state[WIN_FULLSCREEN]()) {
      return;
    }
    if (event.phase === "start" && window.state[WIN_MAXIMIZED]()) {
      window.state[WIN_MAXIMIZED].set(false);
      window.unmaximize();
    }

    const constraints = window.sizeConstraints();
    const extra = frameExtra(window);
    const minWidth = Math.max(1, constraints.min?.width ?? 1) + extra.width;
    const minHeight = Math.max(1, constraints.min?.height ?? 1) + extra.height;
    const width = clamp(
      event.currentRect.width,
      minWidth,
      Math.max(minWidth, maxSize(constraints, "width", extra.width)),
    );
    const height = clamp(
      event.currentRect.height,
      minHeight,
      Math.max(minHeight, maxSize(constraints, "height", extra.height)),
    );

    window.state[WIN_RECT].set({
      x: event.edges.left
        ? event.startRect.x + event.startRect.width - width
        : event.currentRect.x,
      y: event.edges.top
        ? event.startRect.y + event.startRect.height - height
        : event.currentRect.y,
      width,
      height,
    });
  }

  // ----- window states ----------------------------------------------------

  public setMaximized(window: WaylandWindow, maximized: boolean): void {
    if (maximized) {
      if (!window.state[WIN_MAXIMIZED]()) {
        window.state[WIN_RESTORE_RECT].set(
          snapshotRect(window.state[WIN_RECT]()),
        );
      }
      const area = this.workArea(this.outputOf(window));
      if (area) {
        window.state[WIN_RECT].set(area);
      }
      window.state[WIN_MAXIMIZED].set(true);
      window.maximize();
      return;
    }

    const restore = window.state[WIN_RESTORE_RECT]();
    if (restore) {
      window.state[WIN_RECT].set(snapshotRect(restore));
    }
    window.state[WIN_MAXIMIZED].set(false);
    window.unmaximize();
  }

  public toggleMaximized(window: WaylandWindow): void {
    this.setMaximized(window, !window.state[WIN_MAXIMIZED]());
  }

  public setFullscreen(
    window: WaylandWindow,
    fullscreen: boolean,
    outputName?: string,
  ): void {
    if (fullscreen) {
      if (!window.state[WIN_FULLSCREEN]()) {
        window.state[WIN_FULLSCREEN_RECT].set(
          snapshotRect(window.state[WIN_RECT]()),
        );
      }
      const rect = this.outputRect(outputName ?? this.outputOf(window));
      if (rect) {
        window.state[WIN_RECT].set(rect);
      }
      window.state[WIN_FULLSCREEN].set(true);
      window.fullscreen();
      return;
    }

    const restore = window.state[WIN_FULLSCREEN_RECT]();
    if (restore) {
      window.state[WIN_RECT].set(snapshotRect(restore));
    }
    window.state[WIN_FULLSCREEN].set(false);
    window.unfullscreen();
  }

  public toggleFullscreen(window: WaylandWindow): void {
    this.setFullscreen(window, !window.state[WIN_FULLSCREEN]());
  }

  /**
   * Minimise or restore. The window keeps its surface and stays listed in the
   * taskbar: it fades out and is marked idle, which is what ShojiWM reports
   * as "minimised" to a taskbar over wlr-foreign-toplevel.
   */
  public setMinimized(window: WaylandWindow, minimized: boolean): void {
    if (window.state[WIN_MINIMIZED]() === minimized) {
      return;
    }
    window.state[WIN_MINIMIZED].set(minimized);
    window.scheduleAnimation({
      channel: MINIMIZE_CHANNEL,
      rect: {
        from: { x: 0, y: minimized ? 0 : 40, width: 0, height: 0 },
        to: { x: 0, y: minimized ? 40 : 0, width: 0, height: 0 },
        duration: TIMING.minimize,
        easing: minimized ? CLOSE_EASING : OPEN_EASING,
        mode: "add",
      },
      opacity: {
        from: minimized ? 1 : 0,
        to: minimized ? 0 : 1,
        duration: TIMING.minimize,
        easing: minimized ? CLOSE_EASING : OPEN_EASING,
        mode: "override",
      },
    });

    if (minimized) {
      this.mru = [...this.mru.filter((id) => id !== window.id), window.id];
      this.focusTopWindow();
    } else {
      window.focus();
    }
  }

  /**
   * Aero Snap: put the window against one side of the work area, or send it
   * back to the size it had before.
   */
  public snap(window: WaylandWindow, side: "left" | "right"): void {
    const area = this.workArea(this.outputOf(window));
    if (!area) {
      return;
    }
    if (window.state[WIN_MAXIMIZED]()) {
      this.setMaximized(window, false);
    } else {
      window.state[WIN_RESTORE_RECT].set(
        snapshotRect(window.state[WIN_RECT]()),
      );
    }
    const width = Math.round(area.width / 2);
    window.state[WIN_RECT].set({
      x: side === "left" ? area.x : area.x + area.width - width,
      y: area.y,
      width,
      height: area.height,
    });
  }

  /** Put the window back where it was before a snap or a maximise. */
  public restore(window: WaylandWindow): void {
    if (window.state[WIN_MAXIMIZED]()) {
      this.setMaximized(window, false);
      return;
    }
    const rect = window.state[WIN_RESTORE_RECT]();
    if (rect) {
      window.state[WIN_RECT].set(snapshotRect(rect));
    }
  }

  /**
   * Show the desktop: minimise every window, or restore the ones the last
   * call minimised.
   */
  public toggleShowDesktop(): void {
    const windows = this.recentWindows();
    if (this.hiddenByShowDesktop.length > 0) {
      for (const id of this.hiddenByShowDesktop) {
        const window = this.windowsById.get(id);
        if (window) {
          this.setMinimized(window, false);
        }
      }
      this.hiddenByShowDesktop = [];
      return;
    }
    this.hiddenByShowDesktop = [];
    for (const window of windows) {
      if (!window.state[WIN_MINIMIZED]()) {
        this.hiddenByShowDesktop.push(window.id);
        this.setMinimized(window, true);
      }
    }
  }

  /** Show a window and give it the keyboard. Used by Alt+Tab and the taskbar. */
  public activate(window: WaylandWindow): void {
    if (window.state[WIN_MINIMIZED]()) {
      this.setMinimized(window, false);
      return;
    }
    window.focus();
  }

  /** Give the keyboard to the front window that is not minimised. */
  private focusTopWindow(): void {
    for (const window of this.recentWindows()) {
      if (!window.state[WIN_MINIMIZED]()) {
        window.focus();
        return;
      }
    }
  }

  /** The window that has the keyboard, if any. */
  public focusedWindow(): WaylandWindow | undefined {
    for (const window of this.recentWindows()) {
      if (window.isFocused()) {
        return window;
      }
    }
    return undefined;
  }

  // ----- client requests --------------------------------------------------

  public onMaximizeRequest(event: WindowMaximizeRequestEvent): void {
    this.setMaximized(event.window, event.maximized);
  }

  public onMinimizeRequest(event: WindowMinimizeRequestEvent): void {
    this.setMinimized(event.window, event.minimized);
  }

  public onFullscreenRequest(event: WindowFullscreenRequestEvent): void {
    this.setFullscreen(event.window, event.fullscreen, event.outputName);
  }

  public onActivateRequest(event: WindowActivateRequestEvent): void {
    this.activate(event.window);
  }
}
