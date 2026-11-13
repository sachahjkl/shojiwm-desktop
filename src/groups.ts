/**
 * Window groups: several windows in one frame, as tabs.
 *
 * A group is an ordered list of windows and the one shown, its active
 * window. The others are "away": not drawn, not focusable, still listed in
 * the taskbar (activating one there shows its tab). The frame of the active
 * window draws a tab per window of its group.
 *
 * A window joins a group when another window is dropped on its caption, or a
 * tab on its tab strip, when `canGroup` allows it (the Control Panel's
 * choices). While such a drag is over a caption, `dropOn` says where the
 * window would go, and that caption shows a ghost tab there.
 *
 * This module keeps the model and the tab layout only. The window manager
 * moves windows, gives the keyboard and applies the states: it calls
 * `join`, `activate`, `remove` and reads the rest.
 */
import { signal, type Signal } from "shoji_wm";

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
 * it is theirs (FloatingWindowManager.openAsTab), while the Control Panel's
 * "Open new terminal windows as tabs" is on (`opensAsTab`).
 */
export const TABBED_PROGRAMS: ReadonlySet<string> = new Set([
  "com.mitchellh.ghostty",
  // foot and footclient: their spawn-terminal key opens a tab, in the folder
  // of the shell shown.
  "foot",
]);

// The Control Panel's two choices, sent by the shell ("tabs", peek.tsx).
// Until it has sent them, its defaults: the compositor runs before the shell.
let tabsEverywhere = true;
let terminalTabs = true;

/** The shell's `{ everywhere, terminal }`; a field missing keeps its value. */
export function setTabSettings(params: unknown): void {
  if (typeof params !== "object" || params === null) {
    return;
  }
  const choice = params as { everywhere?: unknown; terminal?: unknown };
  if (typeof choice.everywhere === "boolean") {
    tabsEverywhere = choice.everywhere;
  }
  if (typeof choice.terminal === "boolean") {
    terminalTabs = choice.terminal;
  }
}

/** Whether a new window of `appId` opens as a tab of its program's last window. */
export function opensAsTab(appId: string | null | undefined): boolean {
  return terminalTabs && !!appId && TABBED_PROGRAMS.has(appId);
}

/**
 * Whether the windows of the programs `dragged` can join a group of the
 * programs `target` (each list one app id per window, null when unknown).
 * Every path that makes or grows a group asks this; detaching, switching and
 * closing tabs do not, so a group made before a setting went off keeps
 * working. With terminal tabs alone, a group holds one tabbed program only.
 */
export function canGroup(
  dragged: readonly (string | null | undefined)[],
  target: readonly (string | null | undefined)[],
): boolean {
  if (tabsEverywhere) {
    return true;
  }
  if (!terminalTabs) {
    return false;
  }
  const first = dragged[0];
  return (
    !!first &&
    TABBED_PROGRAMS.has(first) &&
    [...dragged, ...target].every((appId) => appId === first)
  );
}

/** A group as kept across a configuration reload. */
export interface GroupSnapshot {
  members: string[];
  active: string;
  first: number;
  follow: boolean;
}

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
  /**
   * Per window, bumped when its group changes, so whatever reads a window's
   * group follows that group only. Made when first read.
   */
  private readonly versions = new Map<string, Signal<number>>();
  /** The drop a drag would make now. */
  private drop: DropTarget | null = null;
  /** Per window, the drop whose target is in its group (`dropOn`). */
  private readonly drops = new Map<string, Signal<DropTarget | null>>();
  /** Per window, whether it is dragged over another group (`dragsOut`). */
  private readonly dragging = new Map<string, Signal<boolean>>();

  /** The windows of `id`'s group, itself alone if none, not followed. */
  private around(id: string): readonly string[] {
    return this.groupOf.get(id)?.members ?? [id];
  }

  /** Whether the two windows are one, or tabs of one group. Not followed. */
  public sameGroup(a: string, b: string): boolean {
    if (a === b) {
      return true;
    }
    const group = this.groupOf.get(a);
    return group !== undefined && group === this.groupOf.get(b);
  }

  /** Bump the windows `ids` and update their drop signals. */
  private changed(ids: Iterable<string>): void {
    for (const id of ids) {
      const version = this.versions.get(id);
      if (version) {
        version.set(version.peek() + 1);
      }
    }
    this.refreshDrop(ids);
  }

  private refreshDrop(ids: Iterable<string>): void {
    const drop = this.drop;
    for (const id of ids) {
      const on = drop !== null && this.sameGroup(id, drop.target) ? drop : null;
      this.drops.get(id)?.set(on);
      this.dragging
        .get(id)
        ?.set(
          drop !== null &&
            drop.dragged === id &&
            !this.sameGroup(id, drop.target),
        );
    }
  }

  private find(id: string, follow: boolean): Group | undefined {
    if (follow) {
      let version = this.versions.get(id);
      if (!version) {
        version = signal(0);
        this.versions.set(id, version);
      }
      version();
    }
    return this.groupOf.get(id);
  }

  /**
   * The drop shown on the window's strip: the one whose target is in its
   * group, else null. Followed for this window only.
   */
  public dropOn(id: string): DropTarget | null {
    let value = this.drops.get(id);
    if (!value) {
      const drop = this.drop;
      value = signal<DropTarget | null>(
        drop !== null && this.sameGroup(id, drop.target) ? drop : null,
      );
      this.drops.set(id, value);
    }
    return value();
  }

  /**
   * Whether the window is dragged over the caption of a group it would
   * join (not along its own strip). Followed for this window only.
   */
  public dragsOut(id: string): boolean {
    let value = this.dragging.get(id);
    if (!value) {
      const drop = this.drop;
      value = signal(
        drop !== null &&
          drop.dragged === id &&
          !this.sameGroup(id, drop.target),
      );
      this.dragging.set(id, value);
    }
    return value();
  }

  /** The window has closed: drop what was kept for it. */
  public forget(id: string): void {
    this.versions.delete(id);
    this.drops.delete(id);
    this.dragging.delete(id);
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
    const before = [...this.around(dragged), ...this.around(target)];
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
    this.changed([...before, ...group.members]);
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
    this.changed(group.members);
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
    this.changed(group.members);
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
    this.changed(group.members);
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
    const before = [...this.around(id)];
    const result = this.leave(id);
    if (result.changed) {
      this.changed(before);
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

  /** The groups, to keep across a configuration reload. */
  public snapshot(): GroupSnapshot[] {
    return [...this.groups.values()].map((group) => ({
      members: [...group.members],
      active: group.active,
      first: group.first,
      follow: group.follow,
    }));
  }

  /**
   * Makes a kept group again. Its windows must be open; one already in a
   * group (a new one, made since) leaves the kept group as it is.
   */
  public restore(kept: GroupSnapshot): void {
    if (
      kept.members.length < 2 ||
      kept.members.some((id) => this.groupOf.has(id))
    ) {
      return;
    }
    const group: Group = {
      members: [...kept.members],
      active: kept.members.includes(kept.active)
        ? kept.active
        : kept.members[0],
      first: kept.first,
      follow: kept.follow,
    };
    for (const id of group.members) {
      this.groupOf.set(id, group);
    }
    this.groups.set(group.members[0], group);
    this.changed(group.members);
  }

  /** Set or clear the drop shown; unchanged values are not set again. */
  public setDrop(drop: DropTarget | null): void {
    const current = this.drop;
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
    this.drop = drop;
    // Only the windows of the groups the drop leaves and reaches, and the
    // windows dragged.
    const ids = new Set<string>();
    for (const value of [current, drop]) {
      if (value) {
        ids.add(value.dragged);
        for (const id of this.around(value.target)) {
          ids.add(id);
        }
      }
    }
    this.refreshDrop(ids);
  }
}

/**
 * The groups of the session: one model, read by the window manager and by the
 * frame sizes (`frameExtra`), which a client-decorated window in a group
 * needs (its tab strip).
 */
export const GROUPS = new WindowGroups();
