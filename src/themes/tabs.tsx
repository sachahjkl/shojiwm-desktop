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
  paintShader,
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
  /** The shown tab's plate: in the focused window, in another. */
  active: string;
  activeAway: string;
  /** Every other tab's plate: in the focused window, in another; and under the pointer. */
  idle: string;
  idleAway: string;
  hover: string;
  /** The line on the right of a tab that is not shown, parting it from the next. */
  separator: string;
  /** The outline of a tab that is not shown, and the sides of the shown one. */
  edge: string;
  /** The bar along the shown tab's top (in the focused window, in another), this many px tall; 0 for none. */
  accent: string;
  accentAway: string;
  accentHeight: number;
  /** The line of light under that bar. */
  light: string;
  /**
   * Where the plates start, in px from the strip's top: the shown tab's and
   * the others'. The shown tab reaches the strip's bottom; the others stop
   * `idleBottom` px above it.
   */
  top: number;
  idleTop: number;
  idleBottom: number;
  /** The ghost tab: its plate, its outline, and that outline's dashes in px (0: a solid line). */
  ghost: string;
  ghostLine: string;
  ghostDash: number;
  /** The shown tab's title (focused window, other) and the others' titles (focused window, other). */
  text: string;
  textAway: string;
  textIdle: string;
  textIdleAway: string;
  /** The opacity of the icon of a tab that is not shown. */
  iconIdle: number;
  /** A soft glow under the text (Aero), or none. */
  textGlow: string | null;
  fontFamily: string | string[];
  fontSize: number;
  radius: number;
  /** The close button's icon, an SVG path from the configuration's root, and its plate under the pointer. */
  closeIcon: string;
  closeHover: string;
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

const CLEAR = "#00000000";

/** `#rrggbbaa` to the four numbers a shader uniform takes. */
function colour(hex: string): [number, number, number, number] {
  const value = hex.replace("#", "");
  const part = (at: number) => parseInt(value.slice(at, at + 2), 16) / 255;
  return [part(0), part(2), part(4), value.length >= 8 ? part(6) : 1];
}

/** The ghost tab's plate with a dashed outline. */
const DashedGhost = ({
  width,
  height,
  style,
}: {
  width: ReadonlySignal<number>;
  height: ReadonlySignal<number>;
  style: TabStyle;
}) => (
  <Box
    paint={paintShader("./assets/tab-ghost.frag", {
      uniforms: {
        fill: colour(style.ghost),
        line: colour(style.ghostLine),
        dash: style.ghostDash,
      },
    })}
    style={{
      position: "absolute",
      left: 0,
      top: style.idleTop,
      width,
      height: computed(() =>
        Math.max(0, height() - style.idleTop - style.idleBottom),
      ),
      pointerEvents: "none",
    }}
  />
);

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
  const top = shown ? style.top : style.idleTop;
  const bottom = shown ? 0 : style.idleBottom;
  const plateHeight = computed(() => Math.max(0, height() - top - bottom));
  const plate = computed(() =>
    shown
      ? focused()
        ? style.active
        : style.activeAway
      : hover()
        ? style.hover
        : focused()
          ? style.idle
          : style.idleAway,
  );
  const textColour = computed(() =>
    shown
      ? focused()
        ? style.text
        : style.textAway
      : focused()
        ? style.textIdle
        : style.textIdleAway,
  );
  const plates = ghost ? (
    style.ghostDash > 0 ? (
      <DashedGhost width={width} height={height} style={style} />
    ) : (
      <Box
        style={{
          position: "absolute",
          left: 0,
          top: style.idleTop,
          width,
          height: plateHeight,
          background: style.ghost,
          borderRadius: style.radius,
          border: { px: 1, color: style.ghostLine },
          pointerEvents: "none",
        }}
      />
    )
  ) : (
    <Box
      style={{
        position: "absolute",
        inset: 0,
        width,
        height,
        pointerEvents: "none",
      }}
    >
      <Box
        style={{
          position: "absolute",
          left: 0,
          top,
          width,
          height: plateHeight,
          background: plate,
          borderRadius: style.radius,
          pointerEvents: "none",
        }}
      />
      {/* the outline of the shown tab: its top and its sides, open at its foot,
          where it joins the window */}
      <Box
        style={{
          position: "absolute",
          left: 0,
          top,
          width,
          height: plateHeight,
          borderTop:
            shown && style.edge !== CLEAR
              ? { px: 1, color: style.edge }
              : undefined,
          borderLeft:
            shown && style.edge !== CLEAR
              ? { px: 1, color: style.edge }
              : undefined,
          borderRight:
            shown && style.edge !== CLEAR
              ? { px: 1, color: style.edge }
              : undefined,
          borderRadius: style.radius,
          background: CLEAR,
          pointerEvents: "none",
        }}
      />
      {shown && style.accentHeight > 0 ? (
        <Box
          style={{
            position: "absolute",
            left: 0,
            top,
            width,
            height: style.accentHeight,
            background: focused((on: boolean) =>
              on ? style.accent : style.accentAway,
            ),
            pointerEvents: "none",
          }}
        />
      ) : null}
      {shown && style.accentHeight > 0 ? (
        <Box
          style={{
            position: "absolute",
            left: 1,
            top: top + style.accentHeight,
            width: computed(() => Math.max(0, width() - 2)),
            height: 1,
            background: style.light,
            pointerEvents: "none",
          }}
        />
      ) : null}
    </Box>
  );
  // the content sits in the middle of the plate
  const contentTop = shown ? top + style.accentHeight : top;
  const contentHeight = computed(() =>
    Math.max(0, height() - contentTop - bottom),
  );
  return (
    <Box style={{ position: "relative", width, height, flexShrink: 0 }}>
      {plates}
      {/* the line on its right, parting it from the next */}
      <Box
        style={{
          position: "absolute",
          right: 0,
          top: 6,
          width: 1,
          height: computed(() => Math.max(0, height() - 12)),
          background: shown || ghost ? CLEAR : style.separator,
          pointerEvents: "none",
        }}
      />
      <Box
        direction="row"
        style={{
          position: "absolute",
          left: 0,
          top: contentTop,
          width,
          height: contentHeight,
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
          style={{
            width: 16,
            height: 16,
            flexShrink: 0,
            opacity: shown || ghost ? 1 : style.iconIdle,
            // Over the title's glow, which spreads past the label's box.
            zIndex: 1,
          }}
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
          // A middle click closes the tab, as in a browser.
          onMiddleClick={() => window.close()}
          style={{
            position: "absolute",
            inset: 0,
            width,
            height,
            background: CLEAR,
          }}
        />
      )}
      {ghost ? null : (
        <Box
          direction="row"
          style={{
            position: "absolute",
            right: 4,
            top: contentTop,
            width: 18,
            height: contentHeight,
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
                borderRadius: Math.min(3, style.radius),
                background: computed(() =>
                  closeHover() ? style.closeHover : CLEAR,
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
                background: CLEAR,
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
