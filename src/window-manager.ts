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
import { TIMING, motion } from "./motion";
import {
  TABBED_PROGRAMS,
  WindowGroups,
  tabLayout,
  type DropTarget,
} from "./groups";
import { frameMetrics } from "./themes/choice";

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
/** The side the window is snapped to with Aero Snap, or null. */
export const WIN_SNAPPED = createWindowState<"left" | "right" | null>(
  "snapped",
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
/** How far a pressed tab moves before it is dragged rather than clicked. */
const TAB_DRAG_PX = 6;
/** Where a tab taken out with the keyboard goes, from its group's place. */
const DETACH_OFFSET_PX = 32;

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
  // The theme's frame: its border on both sides, its caption and its bottom
  // border.
  const m = frameMetrics();
  return { width: 2 * m.border, height: m.caption + m.border };
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

/** A tab dragged by the move of its frame (FloatingWindowManager.moveTab). */
interface TabDrag {
  tab: WaylandWindow;
  /** Out of its group, following the pointer. */
  detached: boolean;
  /** Where the pointer holds it, from its top left corner. */
  grabX: number;
  grabY: number;
}

export class FloatingWindowManager {
  private readonly stack = createWindowStack();
  private readonly windowsById = new Map<string, WaylandWindow>();
  /** Window ids, most recently focused first. Drives Alt+Tab. */
  private mru: string[] = [];
  /**
   * The window chosen to take the keyboard when a closing window goes. It is
   * chosen when the close starts; the compositor can hand the keyboard to
   * the window under the pointer before the close animation ends.
   */
  private afterClose: string | null = null;
  private cascade = 0;
  /**
   * Where each program's last window was when it closed, by app id, with
   * the screen it was on. The next window of that program opens at that
   * place when none of its windows is open, as programs reopen where they
   * were on Windows, but on the screen the user is on: a program called up
   * by a key (Papier) comes where the user looks. Programs that make a new
   * window each time they show stay in place instead of walking along the
   * cascade.
   */
  private lastPlace = new Map<string, { output: string; rect: Rect }>();
  /**
   * Each window's app id, read while it lives: a window whose client has
   * gone has none any more when it closes.
   */
  private appIds = new Map<string, string>();
  /**
   * Windows whose close has started: they stay in windowsById until their
   * close animation ends, but no longer hold their program's place.
   */
  private closing = new Set<string>();
  private currentOutput: string | undefined;
  /** Windows that "show desktop" minimised, so the next call restores them. */
  private hiddenByShowDesktop: string[] = [];
  /**
   * The window being dragged, and the rectangle its drag started from.
   * `base` stays null for a maximised window until the pointer moves past
   * a click: a click on its caption bar leaves it maximised.
   */
  private moveDrag: { id: string; base: Rect | null } | null = null;
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
  /** Windows shown as tabs of one frame (groups.ts). */
  public readonly groups = new WindowGroups();
  /** The tab last pressed, until the move its press starts takes it. */
  private tabPress: string | null = null;
  /** The move of a frame that drags one of its tabs. */
  private tabDrag: TabDrag | null = null;

  // ----- queries ----------------------------------------------------------

  /** Stacking order of one window; higher is in front. */
  public zIndex(window: WaylandWindow): ReadonlySignal<number> {
    return this.stack.zIndex(window);
  }

  /**
   * Every managed window, most recently focused first. A tab its group does
   * not show is left out: it is reached through its group.
   */
  public recentWindows(): WaylandWindow[] {
    const windows: WaylandWindow[] = [];
    for (const id of this.mru) {
      const window = this.windowsById.get(id);
      if (window && !this.groups.isAwayPeek(id)) {
        windows.push(window);
      }
    }
    return windows;
  }

  /** A managed window by id. */
  public windowById(id: string): WaylandWindow | undefined {
    return this.windowsById.get(id);
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
    const remembered = this.rememberedRect(window, natural);
    if (remembered) {
      return remembered;
    }
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

  /**
   * The place the program's last window had, at this window's own size, on
   * the screen the user is on: the same place in its work area as it had in
   * its own screen's, kept inside. null when there is none, or when another
   * window of the program is open (that one keeps the place).
   */
  private rememberedRect(window: WaylandWindow, natural: Rect): Rect | null {
    const appId = window.appId();
    const place = appId ? this.lastPlace.get(appId) : undefined;
    if (!appId || !place) {
      return null;
    }
    for (const other of this.windowsById.values()) {
      if (
        other.id !== window.id &&
        !this.closing.has(other.id) &&
        other.appId() === appId
      ) {
        return null;
      }
    }
    const area = this.workArea(this.outputName());
    if (!area) {
      return null;
    }
    // From the screen it was on to this one: the same offset in the work
    // area. On the same screen, the same place.
    const from = this.workArea(place.output) ?? area;
    const x = area.x + (place.rect.x - from.x);
    const y = area.y + (place.rect.y - from.y);
    const width = Math.min(natural.width, area.width);
    const height = Math.min(natural.height, area.height);
    return {
      x: Math.round(clamp(x, area.x, area.x + area.width - width)),
      y: Math.round(clamp(y, area.y, area.y + area.height - height)),
      width: Math.round(width),
      height: Math.round(height),
    };
  }

  /** Remembers where a closing window is, for the program's next window. */
  private rememberPlace(window: WaylandWindow): void {
    const appId = window.appId() ?? this.appIds.get(window.id);
    const output = this.outputOf(window);
    if (!appId || !output || window.state[WIN_FULLSCREEN]()) {
      return;
    }
    const rect = window.state[WIN_MAXIMIZED]()
      ? window.state[WIN_RESTORE_RECT]()
      : window.state[WIN_RECT]();
    if (rect) {
      this.lastPlace.set(appId, { output, rect: snapshotRect(rect) });
    }
  }

  // ----- lifecycle --------------------------------------------------------

  public onOpen(window: WaylandWindow): void {
    this.windowsById.set(window.id, window);
    this.stack.add(window);
    this.mru = [window.id, ...this.mru.filter((id) => id !== window.id)];
  }

  public onFirstCommit(window: WaylandWindow): void {
    const appId = window.appId();
    if (appId) {
      this.appIds.set(window.id, appId);
    }
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
    this.openAsTab(window);

    window.scheduleAnimation({
      channel: OPEN_CHANNEL,
      rect: {
        from: { x: 0, y: 24, width: 0, height: 0 },
        to: { x: 0, y: 0, width: 0, height: 0 },
        duration: motion(TIMING.open),
        easing: OPEN_EASING,
        mode: "add",
      },
      opacity: {
        from: 0,
        to: 1,
        duration: motion(TIMING.open),
        easing: OPEN_EASING,
        mode: "multiply",
      },
    });
  }

  public onStartClose(window: WaylandWindow): void {
    this.rememberPlace(window);
    this.closing.add(window.id);
    // A tab of a group: the next tab takes its place (and the keyboard, if
    // it was the shown one). An away tab goes without a trace.
    const wasAway = this.groups.isAwayPeek(window.id);
    const shownNext = this.leaveGroup(window);
    this.mru = this.mru.filter((id) => id !== window.id);
    if (wasAway) {
      return;
    }
    // The next window takes the keyboard now, as on Windows, from the most
    // recently used ones, before anything else can.
    const next =
      shownNext ??
      this.recentWindows().find((other) => !other.state[WIN_MINIMIZED]());
    this.afterClose = next?.id ?? null;
    next?.focus();

    window.setCloseAnimationDuration(motion(TIMING.close));
    window.scheduleAnimation({
      channel: CLOSE_CHANNEL,
      rect: {
        from: { x: 0, y: 0, width: 0, height: 0 },
        to: { x: 0, y: 16, width: 0, height: 0 },
        duration: motion(TIMING.close),
        easing: CLOSE_EASING,
        mode: "add",
      },
      opacity: {
        from: 1,
        to: 0,
        duration: motion(TIMING.close),
        easing: CLOSE_EASING,
        mode: "multiply",
      },
    });
  }

  public onClose(window: WaylandWindow): void {
    // A window can also go without the close animation (onStartClose).
    this.rememberPlace(window);
    this.leaveGroup(window);
    this.appIds.delete(window.id);
    this.closing.delete(window.id);
    this.stack.remove(window);
    this.windowsById.delete(window.id);
    this.mru = this.mru.filter((id) => id !== window.id);
    // The window chosen when the close started, even if the keyboard went
    // elsewhere during the animation.
    const chosen = this.afterClose
      ? this.windowsById.get(this.afterClose)
      : undefined;
    this.afterClose = null;
    if (chosen && !chosen.state[WIN_MINIMIZED]()) {
      this.mru = [chosen.id, ...this.mru.filter((id) => id !== chosen.id)];
      chosen.focus();
      return;
    }
    this.focusTopWindow();
  }

  public onFocus(window: WaylandWindow, focused: boolean): void {
    if (!focused) {
      return;
    }
    if (this.groups.isAwayPeek(window.id)) {
      this.showTab(window);
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

    // A press on a tab: the move is the tab's (moveTab).
    if (event.phase === "start" && this.tabPress !== null) {
      const tab = this.windowsById.get(this.tabPress);
      this.tabPress = null;
      this.tabDrag =
        tab && this.groups.membersPeek(window.id).includes(tab.id)
          ? { tab, detached: false, grabX: 0, grabY: 0 }
          : null;
    }
    if (this.tabDrag) {
      this.moveTab(event, this.tabDrag);
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
        base: window.state[WIN_MAXIMIZED]() ? null : this.moveBaseRect(event),
      };
    }

    const drag = this.moveDrag;
    if (!drag) {
      return;
    }

    // A maximised window is restored only once the press becomes a drag.
    if (drag.base === null) {
      if (this.isClick(event)) {
        if (event.phase === "end" || event.phase === "cancel") {
          this.moveDrag = null;
          if (event.phase === "end") {
            this.noteClick(event);
          }
        }
        return;
      }
      drag.base = this.moveBaseRect(event);
    }
    const base = drag.base;

    if (event.phase === "cancel") {
      window.state[WIN_RECT].set(base);
      this.moveDrag = null;
      this.setSnapPreview(null);
      this.groups.setDrop(null);
      return;
    }

    window.state[WIN_RECT].set({
      x: base.x + (event.currentRect.x - event.startRect.x),
      y: base.y + (event.currentRect.y - event.startRect.y),
      width: base.width,
      height: base.height,
    });

    // Over another window's caption the window would join its group (a
    // ghost tab shows where); elsewhere, an edge of the screen snaps it.
    const drop = this.dropAt(event.currentPointer, window);
    this.groups.setDrop(drop);
    this.setSnapPreview(drop ? null : this.snapZoneAt(event));

    if (event.phase === "end") {
      const zone = this.snapPreview.peek();
      this.setSnapPreview(null);
      this.groups.setDrop(null);
      this.moveDrag = null;

      this.noteClick(event);

      if (drop) {
        // The whole group goes over, in its order, the dragged window shown.
        this.dropInto(this.groups.membersPeek(window.id), drop, window);
      } else if (zone) {
        this.applySnap(window, zone.kind);
      }
    }
  }

  /** `true` while the pointer has not moved past a click. */
  private isClick(event: WindowMoveEvent): boolean {
    return (
      Math.abs(event.delta.x) <= CLICK_SLACK_PX &&
      Math.abs(event.delta.y) <= CLICK_SLACK_PX
    );
  }

  /**
   * A click that did not move the window can be the first half of a double
   * click: on the caption bar, or anywhere with Super held.
   */
  private noteClick(event: WindowMoveEvent): void {
    if (
      (event.source === "ssd" || event.source === "modifier") &&
      this.isClick(event)
    ) {
      this.lastCaptionClick = {
        id: event.window.id,
        at: event.timestamp,
        x: event.startPointer.x,
        y: event.startPointer.y,
      };
    }
  }

  /** `true` when this press follows a press that neither moved nor dragged. */
  private isDoubleClick(event: WindowMoveEvent): boolean {
    const previous = this.lastCaptionClick;
    return (
      (event.source === "ssd" || event.source === "modifier") &&
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
    window.state[WIN_SNAPPED].set(null);
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
      y: Math.round(event.startPointer.y - frameExtra(window).height / 2),
      width,
      height,
    };
  }

  public onWindowResize(event: WindowResizeEvent): void {
    const window = event.window;
    if (!window.isResizable() || window.state[WIN_FULLSCREEN]()) {
      return;
    }
    if (event.phase === "start") {
      window.state[WIN_SNAPPED].set(null);
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

  // `window.maximize()` and the other client actions come back as a request
  // event with the source "api". The request handlers below only apply the
  // state and ignore a request for the state the window already has: calling
  // the action again from the handler loops until the compositor aborts.

  public setMaximized(window: WaylandWindow, maximized: boolean): void {
    this.applyMaximized(window, maximized);
    if (maximized) {
      window.maximize();
    } else {
      window.unmaximize();
    }
  }

  private applyMaximized(window: WaylandWindow, maximized: boolean): void {
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
      window.state[WIN_SNAPPED].set(null);
      return;
    }

    const restore = window.state[WIN_RESTORE_RECT]();
    if (restore) {
      window.state[WIN_RECT].set(snapshotRect(restore));
    }
    window.state[WIN_MAXIMIZED].set(false);
  }

  public toggleMaximized(window: WaylandWindow): void {
    this.setMaximized(window, !window.state[WIN_MAXIMIZED]());
  }

  public setFullscreen(
    window: WaylandWindow,
    fullscreen: boolean,
    outputName?: string,
  ): void {
    this.applyFullscreen(window, fullscreen, outputName);
    if (fullscreen) {
      window.fullscreen();
    } else {
      window.unfullscreen();
    }
  }

  private applyFullscreen(
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
      return;
    }

    const restore = window.state[WIN_FULLSCREEN_RECT]();
    if (restore) {
      window.state[WIN_RECT].set(snapshotRect(restore));
    }
    window.state[WIN_FULLSCREEN].set(false);
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
        duration: motion(TIMING.minimize),
        easing: minimized ? CLOSE_EASING : OPEN_EASING,
        mode: "add",
      },
      opacity: {
        from: minimized ? 1 : 0,
        to: minimized ? 0 : 1,
        duration: motion(TIMING.minimize),
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
  public snap(
    window: WaylandWindow,
    side: "left" | "right",
    outputName: string | undefined = this.outputOf(window),
  ): void {
    const area = this.workArea(outputName);
    if (!area) {
      return;
    }
    if (window.state[WIN_MAXIMIZED]()) {
      this.setMaximized(window, false);
    } else if (window.state[WIN_SNAPPED]() === null) {
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
    window.state[WIN_SNAPPED].set(side);
    if (outputName) {
      this.currentOutput = outputName;
    }
  }

  /**
   * Super+Left or Right, as Windows 7: snap the window to that side; from
   * the other side, put it back; from that side already, snap it to the far
   * side of the next screen that way.
   */
  public snapKey(window: WaylandWindow, side: "left" | "right"): void {
    const snapped = window.state[WIN_SNAPPED]();
    if (snapped === side) {
      const next = this.outputBeside(window, side === "left" ? -1 : 1);
      if (next) {
        this.snap(window, side === "left" ? "right" : "left", next);
      }
      return;
    }
    if (snapped !== null) {
      this.restore(window);
      return;
    }
    this.snap(window, side);
  }

  /** The name of the next output left (-1) or right (1) of the window's. */
  private outputBeside(
    window: WaylandWindow,
    step: 1 | -1,
  ): string | undefined {
    const outputs = [...COMPOSITOR.output.outputs].sort(
      (a, b) => a.position.x - b.position.x || a.position.y - b.position.y,
    );
    const from = outputs.findIndex(
      (output) => output.name === this.outputOf(window),
    );
    return from < 0 ? undefined : outputs[from + step]?.name;
  }

  /**
   * Super+Down, as Windows 7: a maximised window is restored, any other is
   * minimised.
   */
  public restoreOrMinimize(window: WaylandWindow): void {
    if (window.state[WIN_MAXIMIZED]()) {
      this.setMaximized(window, false);
      return;
    }
    this.setMinimized(window, true);
  }

  /**
   * Super+Shift+Left or Right: the window goes to the next output that way,
   * at the same place in its work area. A maximised window stays maximised.
   */
  public moveToOutput(window: WaylandWindow, step: 1 | -1): void {
    const outputs = [...COMPOSITOR.output.outputs].sort(
      (a, b) => a.position.x - b.position.x || a.position.y - b.position.y,
    );
    const from = outputs.findIndex(
      (output) => output.name === this.outputOf(window),
    );
    const to = outputs[from + step];
    if (from < 0 || !to) {
      return;
    }
    const source = this.workArea(outputs[from].name);
    const target = this.workArea(to.name);
    if (!source || !target) {
      return;
    }
    const place = (rect: Rect): Rect => {
      const width = Math.min(rect.width, target.width);
      const height = Math.min(rect.height, target.height);
      const fx =
        source.width > rect.width
          ? (rect.x - source.x) / (source.width - rect.width)
          : 0;
      const fy =
        source.height > rect.height
          ? (rect.y - source.y) / (source.height - rect.height)
          : 0;
      return {
        x: Math.round(
          target.x + Math.max(0, Math.min(1, fx)) * (target.width - width),
        ),
        y: Math.round(
          target.y + Math.max(0, Math.min(1, fy)) * (target.height - height),
        ),
        width,
        height,
      };
    };
    const restore = window.state[WIN_RESTORE_RECT]();
    if (window.state[WIN_MAXIMIZED]()) {
      if (restore) {
        window.state[WIN_RESTORE_RECT].set(place(snapshotRect(restore)));
      }
      window.state[WIN_RECT].set(target);
    } else {
      window.state[WIN_RECT].set(place(snapshotRect(window.state[WIN_RECT]())));
    }
    this.currentOutput = to.name;
  }

  /** The windows Super+Home minimised, to bring back on the next press. */
  private hiddenByFocus: string[] = [];

  /**
   * Super+Home, as Windows 7: minimise every window but the focused one, or
   * bring them back.
   */
  public toggleOthers(window: WaylandWindow): void {
    if (this.hiddenByFocus.length > 0) {
      for (const id of this.hiddenByFocus) {
        const other = this.windowsById.get(id);
        if (other) {
          this.setMinimized(other, false);
        }
      }
      this.hiddenByFocus = [];
      window.focus();
      return;
    }
    for (const other of this.recentWindows()) {
      if (other.id !== window.id && !other.state[WIN_MINIMIZED]()) {
        this.hiddenByFocus.push(other.id);
        this.setMinimized(other, true);
      }
    }
    window.focus();
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
    window.state[WIN_SNAPPED].set(null);
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
    if (this.groups.isAwayPeek(window.id)) {
      this.showTab(window);
      return;
    }
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

  // ----- groups -----------------------------------------------------------

  /**
   * `window` takes `from`'s place: its rectangle, its maximised or snapped
   * state, shown and not minimised. Used when a tab replaces another.
   */
  private takePlace(window: WaylandWindow, from: WaylandWindow): void {
    if (window === from) {
      return;
    }
    const maximized = from.state[WIN_MAXIMIZED]();
    window.state[WIN_RESTORE_RECT].set(from.state[WIN_RESTORE_RECT]());
    window.state[WIN_SNAPPED].set(from.state[WIN_SNAPPED]());
    window.state[WIN_RECT].set(snapshotRect(from.state[WIN_RECT]()));
    if (window.state[WIN_MAXIMIZED]() !== maximized) {
      window.state[WIN_MAXIMIZED].set(maximized);
      if (maximized) {
        window.maximize();
      } else {
        window.unmaximize();
      }
    }
    // The tab it replaces is hidden by its group, not minimised: it shows
    // again as it was when its tab is chosen.
    window.state[WIN_MINIMIZED].set(from.state[WIN_MINIMIZED]());
    from.state[WIN_MINIMIZED].set(false);
  }

  /** Show `window`'s tab in its group, give it the keyboard. */
  public showTab(window: WaylandWindow): void {
    const before = this.groups.activate(window.id);
    const shown = this.windowsById.get(before);
    if (shown && shown !== window) {
      this.takePlace(window, shown);
    }
    if (window.state[WIN_MINIMIZED]()) {
      this.setMinimized(window, false);
      return;
    }
    this.stack.raise(window);
    window.focus();
  }

  /** The tab `step` places from the one shown (Super+Ctrl+Left or Right). */
  public nextTab(window: WaylandWindow, step: number): void {
    const next = this.windowsById.get(this.groups.neighbour(window.id, step));
    if (next && next !== window) {
      this.showTab(next);
    }
  }

  /**
   * Take the window out of its group. If it was the shown tab, the next one
   * takes its place and is returned.
   */
  private leaveGroup(window: WaylandWindow): WaylandWindow | null {
    if (!this.groups.isGrouped(window.id)) {
      return null;
    }
    const wasAway = this.groups.isAwayPeek(window.id);
    const nextId = this.groups.remove(window.id);
    const next = nextId ? this.windowsById.get(nextId) : undefined;
    if (!next) {
      return null;
    }
    if (!wasAway) {
      this.takePlace(next, window);
      this.stack.raise(next);
    }
    return next;
  }

  /** Super+Ctrl+Down: the shown tab leaves its group, a little aside. */
  public detachTab(window: WaylandWindow): void {
    if (!this.groups.isGrouped(window.id)) {
      return;
    }
    const next = this.leaveGroup(window);
    if (window.state[WIN_MAXIMIZED]()) {
      this.setMaximized(window, false);
    }
    const rect = snapshotRect(window.state[WIN_RECT]());
    window.state[WIN_RECT].set({
      ...rect,
      x: rect.x + DETACH_OFFSET_PX,
      y: rect.y + DETACH_OFFSET_PX,
    });
    window.state[WIN_SNAPPED].set(null);
    if (next) {
      this.stack.raise(window);
    }
    window.focus();
  }

  /**
   * The drop a drag would make with the pointer at `point`: the caption of
   * the front window there, and the tab place under the pointer. null when
   * the pointer is over no caption, or over another part of a window (the
   * windows of `dragged`'s group are looked through).
   */
  private dropAt(
    point: { x: number; y: number },
    dragged: WaylandWindow,
  ): DropTarget | null {
    if (usesClientDecoration(dragged)) {
      return null;
    }
    const own = new Set(this.groups.membersPeek(dragged.id));
    const metrics = frameMetrics();
    const candidates = [...this.windowsById.values()]
      .filter(
        (window) =>
          !own.has(window.id) &&
          !this.closing.has(window.id) &&
          !this.groups.isAwayPeek(window.id) &&
          !window.state[WIN_MINIMIZED]() &&
          !window.state[WIN_FULLSCREEN](),
      )
      .sort(
        (a, b) => this.stack.zIndex(b).peek() - this.stack.zIndex(a).peek(),
      );
    for (const window of candidates) {
      const rect = window.state[WIN_RECT]();
      const x = read(rect.x);
      const y = read(rect.y);
      const width = read(rect.width);
      const height = read(rect.height);
      if (
        point.x < x ||
        point.x >= x + width ||
        point.y < y ||
        point.y >= y + height
      ) {
        continue;
      }
      const caption = window.state[WIN_MAXIMIZED]()
        ? metrics.captionMaximized
        : metrics.caption;
      if (usesClientDecoration(window) || point.y >= y + caption) {
        // The pointer is on this window, not on a caption with tabs.
        return null;
      }
      const tabs = this.groups.membersPeek(window.id).length;
      const layout = tabLayout(width, tabs + 1, metrics);
      const index = clamp(
        Math.floor((point.x - x - layout.left) / Math.max(1, layout.width)),
        0,
        tabs,
      );
      return { dragged: dragged.id, target: window.id, index };
    }
    return null;
  }

  /**
   * Put `ids` (in order) among the tabs of the drop's target, at its place,
   * and show `shown` in the place of the tab shown before.
   */
  private dropInto(
    ids: string[],
    drop: DropTarget,
    shown: WaylandWindow,
  ): void {
    const before = this.windowsById.get(this.groups.activeOf(drop.target));
    ids.forEach((id, at) => {
      this.groups.join(id, drop.target, drop.index + at);
    });
    this.groups.activate(shown.id);
    if (before) {
      this.takePlace(shown, before);
    }
    this.stack.raise(shown);
    shown.focus();
  }

  /** A new window of a program in TABBED_PROGRAMS opens as a tab of its last window. */
  private openAsTab(window: WaylandWindow): void {
    const appId = window.appId();
    if (
      !appId ||
      !TABBED_PROGRAMS.has(appId) ||
      window.isTransient() ||
      usesClientDecoration(window)
    ) {
      return;
    }
    const host = this.recentWindows().find(
      (other) =>
        other !== window &&
        other.appId() === appId &&
        !this.closing.has(other.id) &&
        !other.isTransient() &&
        !other.state[WIN_MINIMIZED]() &&
        !usesClientDecoration(other),
    );
    if (!host) {
      return;
    }
    const tabs = this.groups.membersPeek(host.id).length;
    this.dropInto(
      [window.id],
      { dragged: window.id, target: host.id, index: tabs },
      window,
    );
  }

  /**
   * A press on a tab (themes/tabs.tsx). ShojiWM goes on with a move of the
   * frame's window (a press on a caption is one); `onWindowMove` turns it
   * into the tab's: a click shows it, a drag along the strip moves it among
   * its tabs, out of the strip the window leaves its group and follows the
   * pointer, and over another caption it would join that group.
   */
  public pressTab(tab: WaylandWindow): void {
    this.tabPress = tab.id;
  }

  /** The move of a frame started by a press on one of its tabs. */
  private moveTab(event: WindowMoveEvent, drag: TabDrag): void {
    const tab = drag.tab;
    const host = event.window;
    const point = event.currentPointer;
    const end = (): void => {
      this.groups.setDrop(null);
      this.tabDrag = null;
    };
    if (event.phase === "cancel") {
      end();
      return;
    }
    if (!drag.detached && this.isClick(event)) {
      if (event.phase === "end") {
        end();
        this.showTab(tab);
      }
      return;
    }
    if (!drag.detached) {
      const own = this.ownStripAt(point, tab, host);
      if (own) {
        this.groups.setDrop(own);
        if (event.phase === "end") {
          end();
          this.groups.reorder(tab.id, own.index);
        }
        return;
      }
      // Out of the strip: the tab leaves its group, at the frame's size,
      // the pointer where it held the caption.
      const rect = snapshotRect(host.state[WIN_RECT]());
      drag.grabX = clamp(event.startPointer.x - rect.x, 0, rect.width);
      drag.grabY = clamp(
        event.startPointer.y - rect.y,
        0,
        frameMetrics().caption,
      );
      const wasAway = this.groups.isAwayPeek(tab.id);
      this.leaveGroup(tab);
      if (wasAway) {
        this.takePlace(tab, host);
        host.state[WIN_MINIMIZED].set(false);
      }
      if (tab.state[WIN_MAXIMIZED]()) {
        const restore = tab.state[WIN_RESTORE_RECT]() ?? rect;
        tab.state[WIN_MAXIMIZED].set(false);
        tab.unmaximize();
        tab.state[WIN_RECT].set(snapshotRect(restore));
        drag.grabX = Math.min(drag.grabX, read(restore.width) / 2);
      }
      tab.state[WIN_SNAPPED].set(null);
      drag.detached = true;
      this.stack.raise(tab);
      tab.focus();
    }
    const rect = snapshotRect(tab.state[WIN_RECT]());
    tab.state[WIN_RECT].set({
      ...rect,
      x: Math.round(point.x - drag.grabX),
      y: Math.round(point.y - drag.grabY),
    });
    const drop = this.dropAt(point, tab);
    this.groups.setDrop(drop);
    if (event.phase === "end") {
      end();
      if (drop) {
        this.dropInto([tab.id], drop, tab);
      }
    }
  }

  /**
   * While a tab is dragged along its own strip: its new place there, as a
   * drop on its own group. null once the pointer is off the strip.
   */
  private ownStripAt(
    point: { x: number; y: number },
    window: WaylandWindow,
    shown: WaylandWindow,
  ): DropTarget | null {
    const metrics = frameMetrics();
    const rect = shown.state[WIN_RECT]();
    const x = read(rect.x);
    const y = read(rect.y);
    const width = read(rect.width);
    const caption = shown.state[WIN_MAXIMIZED]()
      ? metrics.captionMaximized
      : metrics.caption;
    // A little slack above and below, so a drag along the strip stays on it.
    if (
      point.x < x ||
      point.x >= x + width ||
      point.y < y - 12 ||
      point.y >= y + caption + 12
    ) {
      return null;
    }
    const others = this.groups.membersPeek(window.id).length - 1;
    const layout = tabLayout(width, others + 1, metrics);
    const index = clamp(
      Math.floor((point.x - x - layout.left) / Math.max(1, layout.width)),
      0,
      others,
    );
    return { dragged: window.id, target: shown.id, index };
  }

  // ----- client requests --------------------------------------------------

  public onMaximizeRequest(event: WindowMaximizeRequestEvent): void {
    if (event.window.state[WIN_MAXIMIZED]() === event.maximized) {
      return;
    }
    this.applyMaximized(event.window, event.maximized);
  }

  public onMinimizeRequest(event: WindowMinimizeRequestEvent): void {
    this.setMinimized(event.window, event.minimized);
  }

  public onFullscreenRequest(event: WindowFullscreenRequestEvent): void {
    if (event.window.state[WIN_FULLSCREEN]() === event.fullscreen) {
      return;
    }
    this.applyFullscreen(event.window, event.fullscreen, event.outputName);
  }

  public onActivateRequest(event: WindowActivateRequestEvent): void {
    this.activate(event.window);
  }
}
