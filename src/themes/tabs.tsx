/**
 * The tab strip of a group's frame (groups.ts), drawn in the theme's style.
 *
 * A window alone shows what its theme puts there (its icon and title). In a
 * group, or while a drag would drop a window on its caption, the frame shows
 * one tab per window instead: each the same width (`tabLayout`), the shown
 * one lit, a close button on the one under the pointer, and a ghost tab where
 * the dragged window would land.
 */
import {
  AppIcon,
  Box,
  Button,
  Image,
  Label,
  computed,
  useState,
  type ReadonlySignal,
  type WaylandWindow,
} from "shoji_wm";
import type { CompositionRenderable } from "shoji_wm/types";
import { tabLayout, type WindowGroups } from "../groups";
import { WIN_RECT } from "../window-manager";
import { frameMetrics } from "./choice";

/** How a theme draws its tabs. Colours are `#rrggbbaa`. */
export interface TabStyle {
  /** The shown tab's plate, and a tab's under the pointer. */
  active: string;
  hover: string;
  /** The line between two tabs. */
  separator: string;
  /** The ghost tab: its plate and its outline. */
  ghost: string;
  ghostLine: string;
  text: string;
  textAway: string;
  /** A soft glow under the text (Aero), or none. */
  textGlow: string | null;
  fontFamily: string | string[];
  fontSize: number;
  radius: number;
  /** The close button's icon, an SVG path from the configuration's root. */
  closeIcon: string;
}

/** What a frame gives its tab strip. */
export interface TabOptions {
  groups: WindowGroups;
  windowOf: (id: string) => WaylandWindow | undefined;
  /** A press on a tab: the move it starts is the tab's (a click or a drag). */
  onTabPress: (window: WaylandWindow) => void;
}

interface TabProps {
  window: WaylandWindow;
  width: ReadonlySignal<number>;
  height: ReadonlySignal<number>;
  shown: boolean;
  ghost: boolean;
  focused: ReadonlySignal<boolean>;
  style: TabStyle;
  options: TabOptions;
}

const Tab = ({
  window,
  width,
  height,
  shown,
  ghost,
  focused,
  style,
  options,
}: TabProps) => {
  const [hover, setHover] = useState(false);
  const [closeHover, setCloseHover] = useState(false);
  const plate = computed(() =>
    ghost
      ? style.ghost
      : shown
        ? style.active
        : hover()
          ? style.hover
          : "#00000000",
  );
  const textColour = computed(() =>
    shown && focused() ? style.text : style.textAway,
  );
  return (
    <Box style={{ position: "relative", width, height, flexShrink: 0 }}>
      <Box
        style={{
          position: "absolute",
          inset: 0,
          width,
          height,
          background: plate,
          borderRadius: style.radius,
          border: ghost ? { px: 1, color: style.ghostLine } : undefined,
          pointerEvents: "none",
        }}
      />
      {/* the line on its right, parting it from the next */}
      <Box
        style={{
          position: "absolute",
          right: 0,
          top: 6,
          width: 1,
          height: computed(() => Math.max(0, height() - 12)),
          background: shown || ghost ? "#00000000" : style.separator,
          pointerEvents: "none",
        }}
      />
      <Box
        direction="row"
        style={{
          position: "absolute",
          inset: 0,
          width,
          height,
          paddingLeft: 8,
          paddingRight: 24,
          gap: 6,
          alignItems: "center",
          pointerEvents: "none",
          opacity: ghost ? 0.75 : 1,
        }}
      >
        <AppIcon
          icon={window.icon}
          style={{ width: 16, height: 16, flexShrink: 0 }}
        />
        <Label
          text={window.title}
          style={{
            color: textColour,
            textShadow: style.textGlow
              ? { blur: 8, spread: 2, color: style.textGlow }
              : undefined,
            fontFamily: style.fontFamily,
            fontSize: style.fontSize,
            textOverflow: "ellipsis",
            flexShrink: 1,
            minWidth: 0,
          }}
        />
      </Box>
      {/* Not a button: a press here is a press on the caption, which moves
          the frame. The window manager is told which tab it was (pressTab)
          and makes the move the tab's: a click or a drag. */}
      {ghost ? null : (
        <Box
          onHoverChange={setHover}
          onActiveChange={(on: boolean) => {
            if (on) {
              options.onTabPress(window);
            }
          }}
          style={{
            position: "absolute",
            inset: 0,
            width,
            height,
            background: "#00000000",
          }}
        />
      )}
      {ghost ? null : (
        <Box
          direction="row"
          style={{
            position: "absolute",
            right: 4,
            top: 0,
            width: 18,
            height,
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          <Box style={{ position: "relative", width: 18, height: 18 }}>
            <Box
              style={{
                position: "absolute",
                inset: 0,
                width: 18,
                height: 18,
                borderRadius: 3,
                background: computed(() =>
                  closeHover() ? style.hover : "#00000000",
                ),
                pointerEvents: "none",
              }}
            />
            <Image
              src={style.closeIcon}
              fit="fill"
              style={{
                position: "absolute",
                left: 1,
                top: 1,
                width: 16,
                height: 16,
                pointerEvents: "none",
                opacity: computed(() =>
                  hover() || closeHover() || shown ? 0.9 : 0,
                ),
              }}
            />
            <Button
              onHoverChange={setCloseHover}
              onClick={() => window.close()}
              style={{
                position: "absolute",
                inset: 0,
                width: 18,
                height: 18,
                background: "#00000000",
              }}
            />
          </Box>
        </Box>
      )}
    </Box>
  );
};

/**
 * The start of a caption, up to its buttons: the strip of `window`'s tabs, or
 * `alone` (laid out in a row `gap` apart) when it is in no group and nothing
 * would be dropped on it. Reading the group and the drop here makes the
 * frame draw again when they change.
 */
export function captionTabs(
  window: WaylandWindow,
  height: ReadonlySignal<number>,
  style: TabStyle,
  options: TabOptions,
  gap: number,
  alone: CompositionRenderable[],
): CompositionRenderable {
  const groups = options.groups;
  const drop = groups.drop();
  const members = groups.members(window.id);
  const dropHere = drop !== null && members.includes(drop.target) ? drop : null;
  const tabs = members.filter((id) => id !== dropHere?.dragged);
  if (tabs.length < 2 && dropHere === null) {
    return (
      <Box
        direction="row"
        style={{ flexGrow: 1, minWidth: 0, gap, alignItems: "center" }}
      >
        {alone}
      </Box>
    );
  }
  const entries: { id: string; ghost: boolean }[] = tabs.map((id) => ({
    id,
    ghost: false,
  }));
  if (dropHere) {
    entries.splice(Math.min(dropHere.index, entries.length), 0, {
      id: dropHere.dragged,
      ghost: true,
    });
  }
  const shown = groups.activeOf(window.id);
  // Every tab as wide as the window's width allows (the same layout the
  // window manager hit-tests a drop with), following a resize.
  const metrics = frameMetrics();
  const rect = window.state[WIN_RECT];
  const tabWidth = computed(
    () => tabLayout(rect().width, entries.length, metrics).width,
  );
  return (
    <Box
      direction="row"
      style={{
        flexGrow: 1,
        minWidth: 0,
        height,
        alignItems: "end",
        overflow: "hidden",
      }}
    >
      {entries.map((entry) => {
        const tab = options.windowOf(entry.id);
        return tab ? (
          <Tab
            window={tab}
            width={tabWidth}
            height={height}
            shown={entry.id === shown && !entry.ghost}
            ghost={entry.ghost}
            focused={window.isFocused}
            style={style}
            options={options}
          />
        ) : null;
      })}
    </Box>
  );
}
