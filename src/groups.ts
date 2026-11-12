/**
 * Window groups: several windows in one frame, as tabs.
 *
 * A group is an ordered list of windows and the one shown, its active
 * window. The others are "away": not drawn, not focusable, still listed in
 * the taskbar (activating one there shows its tab). The frame of the active
 * window draws a tab per window of its group.
 *
 * A window joins a group when another window is dropped on its caption, or a
 * tab on its tab strip. While such a drag is over a caption, `drop` says
 * where the window would go, and that caption shows a ghost tab there.
 *
 * This module keeps the model and the tab layout only. The window manager
 * moves windows, gives the keyboard and applies the states: it calls
 * `join`, `activate`, `remove` and reads the rest.
 */
import { signal } from "shoji_wm";

/** Where a dragged window would land if dropped now. */
export interface DropTarget {
  /** The window dragged. */
  dragged: string;
  /** The window whose caption is under the pointer. */
  target: string;
  /** Its place among the target's tabs (the dragged one left out). */
  index: number;
}

/** The room of a frame's tab strip, from the theme (themes/choice.ts). */
export interface TabMetrics {
  /** The window's left edge to the strip's. */
  tabsLeft: number;
  /** The strip's right end to the window's right edge (the caption buttons). */
  tabsRight: number;
  /** The widest a tab gets. */
  tabMaxWidth: number;
  /** The narrowest: past it the strip scrolls, between two arrows. */
  tabMinWidth: number;
  /** The width of each arrow of a strip that scrolls. */
  arrowWidth: number;
}

/** Where a strip's tabs are: the drawing and the hit tests read the same. */
export interface StripLayout {
  /** The x of the first tab shown, relative to the window. */
  left: number;
  /** Every tab's width. */
  width: number;
  /** The first tab shown, and how many are shown from it. */
  first: number;
  visible: number;
  /** The tabs do not all fit: the strip scrolls, between two arrows. */
  overflow: boolean;
}

/**
 * Where the `count` tabs of a window `windowWidth` wide are. They share the
 * room up to `tabMaxWidth` each; under `tabMinWidth` the strip shows as many
 * as fit from `first`, between two arrows. With `follow`, `shown` (an index)
 * is kept in view.
 */
export function stripLayout(
  windowWidth: number,
  count: number,
  metrics: TabMetrics,
  first = 0,
  shown = -1,
  follow = true,
): StripLayout {
  const room = Math.max(0, windowWidth - metrics.tabsLeft - metrics.tabsRight);
  const natural = Math.floor(
    Math.min(metrics.tabMaxWidth, room / Math.max(1, count)),
  );
  if (natural >= metrics.tabMinWidth || count <= 1) {
    return {
      left: metrics.tabsLeft,
      width: natural,
      first: 0,
      visible: count,
      overflow: false,
    };
  }
  const inner = Math.max(0, room - 2 * metrics.arrowWidth);
  const visible = Math.max(1, Math.floor(inner / metrics.tabMinWidth));
  const width = Math.floor(Math.min(metrics.tabMaxWidth, inner / visible));
  let from = Math.max(0, Math.min(first, count - visible));
  if (follow && shown >= 0) {
    if (shown < from) {
      from = shown;
    } else if (shown >= from + visible) {
      from = shown - visible + 1;
    }
  }
  return {
    left: metrics.tabsLeft + metrics.arrowWidth,
    width,
    first: from,
    visible,
    overflow: true,
  };
}

/** The tab under `x` (relative to the window) among `count`, by `layout`. */
export function tabAt(x: number, layout: StripLayout, count: number): number {
  const at =
    layout.first + Math.floor((x - layout.left) / Math.max(1, layout.width));
  return Math.max(0, Math.min(count - 1, at));
}

/**
 * The programs whose new windows open as tabs of the window used last, when
 * it is theirs (FloatingWindowManager.openAsTab).
 */
export const TABBED_PROGRAMS: ReadonlySet<string> = new Set([
  "com.mitchellh.ghostty",
]);

interface Group {
  members: string[];
  active: string;
  /** The first tab its strip shows when it scrolls, and whether it follows the shown tab. */
  first: number;
  follow: boolean;
}

export class WindowGroups {
  private readonly groups = new Map<string, Group>();
  /** Each grouped window's group, by window id. */
  private readonly groupOf = new Map<string, Group>();
  /** Bumped on every change, so whatever reads a group follows it. */
  private readonly version = signal(0);
  /** The drop a drag would make now, for the ghost tab. */
  public readonly drop = signal<DropTarget | null>(null);

  private changed(): void {
    this.version.set(this.version.peek() + 1);
  }

  private find(id: string, follow: boolean): Group | undefined {
    if (follow) {
      this.version();
    }
    return this.groupOf.get(id);
  }

  /** The windows of the window's group, in tab order; itself alone if none. */
  public members(id: string): string[] {
    return this.find(id, true)?.members.slice() ?? [id];
  }

  /** `members`, not followed. */
  public membersPeek(id: string): string[] {
    return this.find(id, false)?.members.slice() ?? [id];
  }

  /** Whether the window is a tab of a group that shows another. */
  public isAway(id: string): boolean {
    const group = this.find(id, true);
    return group !== undefined && group.active !== id;
  }

  /** `isAway`, not followed. */
  public isAwayPeek(id: string): boolean {
    const group = this.find(id, false);
    return group !== undefined && group.active !== id;
  }

  /** The window its group shows (itself when in none). */
  public activeOf(id: string): string {
    return this.find(id, false)?.active ?? id;
  }

  public isGrouped(id: string): boolean {
    return this.groupOf.has(id);
  }

  /**
   * Put `dragged` among `target`'s tabs at `index` (counted without it) and
   * make it the shown one. Returns the window shown before, whose place it
   * takes.
   */
  public join(dragged: string, target: string, index: number): string {
    const shown = this.activeOf(target);
    if (dragged === target) {
      return shown;
    }
    this.leave(dragged);
    let group = this.groupOf.get(target);
    if (!group) {
      group = { members: [target], active: target, first: 0, follow: true };
      this.groupOf.set(target, group);
      this.groups.set(target, group);
    }
    const at = Math.max(0, Math.min(group.members.length, index));
    group.members.splice(at, 0, dragged);
    group.active = dragged;
    this.groupOf.set(dragged, group);
    this.changed();
    return shown;
  }

  /** Move a tab to another place in its own group. */
  public reorder(id: string, index: number): void {
    const group = this.groupOf.get(id);
    if (!group) {
      return;
    }
    group.members = group.members.filter((member) => member !== id);
    group.members.splice(
      Math.max(0, Math.min(group.members.length, index)),
      0,
      id,
    );
    this.changed();
  }

  /** Show `id`'s tab. Returns the window shown before (itself if no change). */
  public activate(id: string): string {
    const group = this.groupOf.get(id);
    if (!group || group.active === id) {
      return id;
    }
    const before = group.active;
    group.active = id;
    group.follow = true;
    this.changed();
    return before;
  }

  /** Where the window's strip starts and whether it follows the shown tab. */
  public scroll(id: string): { first: number; follow: boolean } {
    const group = this.find(id, true);
    return { first: group?.first ?? 0, follow: group?.follow ?? true };
  }

  /** `scroll`, not followed. */
  public scrollPeek(id: string): { first: number; follow: boolean } {
    const group = this.find(id, false);
    return { first: group?.first ?? 0, follow: group?.follow ?? true };
  }

  /** The window's strip starts at tab `first` (an arrow was pressed). */
  public scrollTo(id: string, first: number): void {
    const group = this.groupOf.get(id);
    if (!group) {
      return;
    }
    group.first = Math.max(0, first);
    group.follow = false;
    this.changed();
  }

  /** The tab `step` places from the shown one, round the group. */
  public neighbour(id: string, step: number): string {
    const group = this.groupOf.get(id);
    if (!group) {
      return id;
    }
    const at = group.members.indexOf(group.active);
    const count = group.members.length;
    return group.members[(((at + step) % count) + count) % count];
  }

  /**
   * Take the window out of its group (it closes, or leaves as a window of
   * its own). Returns the window the group shows now if it was the shown
   * one, else null. A group left with one window ends.
   */
  public remove(id: string): string | null {
    const result = this.leave(id);
    if (result.changed) {
      this.changed();
    }
    return result.next;
  }

  private leave(id: string): { changed: boolean; next: string | null } {
    const group = this.groupOf.get(id);
    if (!group) {
      return { changed: false, next: null };
    }
    const at = group.members.indexOf(id);
    group.members.splice(at, 1);
    this.groupOf.delete(id);
    let next: string | null = null;
    if (group.active === id) {
      // The tab on its right, else the one on its left, as browsers do.
      group.active = group.members[Math.min(at, group.members.length - 1)];
      next = group.active;
    }
    if (group.members.length === 1) {
      this.groupOf.delete(group.members[0]);
    }
    for (const [key, value] of this.groups) {
      if (value === group) {
        this.groups.delete(key);
      }
    }
    if (group.members.length > 1) {
      this.groups.set(group.members[0], group);
    }
    return { changed: true, next };
  }

  /** Set or clear the drop shown; unchanged values are not set again. */
  public setDrop(drop: DropTarget | null): void {
    const current = this.drop.peek();
    if (
      current === drop ||
      (current &&
        drop &&
        current.dragged === drop.dragged &&
        current.target === drop.target &&
        current.index === drop.index)
    ) {
      return;
    }
    this.drop.set(drop);
  }
}

/**
 * The groups of the session: one model, read by the window manager and by the
 * frame sizes (`frameExtra`), which a client-decorated window in a group
 * needs (its tab strip).
 */
export const GROUPS = new WindowGroups();
