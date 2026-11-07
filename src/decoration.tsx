/**
 * The window frame of Windows 7 Aero, rebuilt from measured pixels.
 *
 * The lines, the glass and the rounded top corners are painted by
 * `assets/frame.frag` over the compositor's blur. The three caption buttons
 * are one joined group of 103 x 18 px, each button an SVG drawn at its exact
 * size by `tools/make-caption-svgs.py`. The title is crisp dark text over a
 * soft light shape, not an outline.
 *
 * Every number comes from `src/theme.ts`, and every number there from
 * `docs/aero-measurements.md`.
 */

import {
  AppIcon,
  Box,
  Button,
  ClientWindow,
  Image,
  Label,
  Popup,
  ShaderEffect,
  WindowBorder,
  computed,
  paintShader,
  useState,
  type CompiledEffectHandle,
  type ReadonlySignal,
  type ShaderUniformVec4,
  type SSDStyle,
  type WaylandWindow,
} from "shoji_wm";
import type { CompositionRenderable } from "shoji_wm/types";
import { CAPTION_BUTTON, COLOR, FONT, FRAME } from "./theme";
import { WIN_MAXIMIZED } from "./window-manager";

/** Which picture a caption button shows. */
type ButtonState = "" | "-hover" | "-press" | "-inactive";

function buttonSrc(kind: string, state: ButtonState): string {
  return `./assets/caption/${kind}${state}.svg`;
}

/** The pale label Windows shows under a caption button you rest on. */
const Tooltip = ({ text }: { text: string | ReadonlySignal<string> }) => (
  <Popup
    trigger="hover"
    openDelay={600}
    closeDelay={0}
    placement="bottom"
    offset={6}
  >
    <Box
      style={{
        background: "#f7fbffee",
        borderRadius: 3,
        paddingLeft: 7,
        paddingRight: 7,
        paddingY: 3,
        border: { px: 1, color: "#7a9ab8cc" },
        boxShadow: [{ y: 2, blur: 8, color: "#00000059" }],
      }}
    >
      <Label
        text={text}
        style={{ color: "#17374f", fontFamily: FONT, fontSize: 11 }}
      />
    </Box>
  </Popup>
);

interface CaptionButtonProps {
  /** `min`, `max`, `restore` or `close`, or a signal picking between them. */
  kind: string | ReadonlySignal<string>;
  width: number;
  label: string;
  focused: ReadonlySignal<boolean>;
  enabled?: ReadonlySignal<boolean>;
  onClick: () => void;
}

/**
 * One caption button: its picture, and a transparent button over it that
 * takes the clicks and reports hover and press.
 */
const CaptionButton = ({
  kind,
  width,
  label,
  focused,
  enabled,
  onClick,
}: CaptionButtonProps) => {
  const [hover, setHover] = useState(false);
  const [pressed, setPressed] = useState(false);

  const usable = computed(() => enabled?.() ?? true);
  const src = computed(() => {
    const name = typeof kind === "string" ? kind : kind();
    if (!focused()) {
      return buttonSrc(name, "-inactive");
    }
    if (!usable()) {
      return buttonSrc(name, "-inactive");
    }
    if (pressed()) {
      return buttonSrc(name, "-press");
    }
    return buttonSrc(name, hover() ? "-hover" : "");
  });

  return (
    <Box
      style={{
        position: "relative",
        width,
        height: FRAME.buttonHeight,
        flexShrink: 0,
      }}
    >
      <Image
        src={src}
        fit="fill"
        style={{
          position: "absolute",
          inset: 0,
          width,
          height: FRAME.buttonHeight,
          pointerEvents: "none",
        }}
      />
      <Button
        onHoverChange={setHover}
        onActiveChange={setPressed}
        onClick={() => {
          if (usable()) {
            onClick();
          }
        }}
        style={{
          position: "absolute",
          inset: 0,
          width,
          height: FRAME.buttonHeight,
          background: "#00000000",
        }}
      >
        <Tooltip text={label} />
      </Button>
    </Box>
  );
};

/** The caption title: crisp text over a soft light shape. */
const CaptionTitle = ({ window }: { window: WaylandWindow }) => (
  <Box
    style={{
      position: "relative",
      height: FRAME.titleHeight,
      flexShrink: 1,
      minWidth: 0,
    }}
  >
    <Image
      src="./assets/title-glow.svg"
      fit="fill"
      style={{
        position: "absolute",
        top: -FRAME.glowMargin,
        right: -FRAME.glowMargin,
        bottom: -FRAME.glowMargin,
        left: -FRAME.glowMargin,
        zIndex: 0,
        pointerEvents: "none",
        opacity: window.isFocused((focused: boolean) => (focused ? 1 : 0.45)),
      }}
    />
    <Label
      text={window.title}
      style={{
        zIndex: 1,
        color: window.isFocused((focused: boolean) =>
          focused ? COLOR.titleActive : COLOR.titleInactive,
        ),
        fontFamily: FONT,
        fontSize: FRAME.fontSize,
        fontWeight: "normal",
        lineHeight: FRAME.titleHeight,
        textOverflow: "ellipsis",
        flexShrink: 1,
        minWidth: 0,
      }}
    />
  </Box>
);

export interface FrameOptions {
  /** The compiled blur that stands behind the glass. */
  glass: CompiledEffectHandle;
  /** `true` while the blur must be left out (the Alt+Tab panel is open). */
  glassSuspended: boolean;
  onMinimize: () => void;
  onToggleMaximize: () => void;
}

/** `#rrggbbaa` to the four numbers a shader uniform takes. */
function colour(hex: string): [number, number, number, number] {
  const value = hex.replace("#", "");
  const part = (at: number) => parseInt(value.slice(at, at + 2), 16) / 255;
  return [part(0), part(2), part(4), value.length >= 8 ? part(6) : 1];
}

/**
 * A colour uniform that follows the window's focus. A paint shader itself
 * cannot be swapped for another while the window lives — `paint` takes a
 * handle, not a signal — but each component of a uniform may be a signal.
 */
function focusColour(
  focused: ReadonlySignal<boolean>,
  activeHex: string,
  inactiveHex: string,
): ShaderUniformVec4 {
  const on = colour(activeHex);
  const off = colour(inactiveHex);
  return [
    computed(() => (focused() ? on[0] : off[0])),
    computed(() => (focused() ? on[1] : off[1])),
    computed(() => (focused() ? on[2] : off[2])),
    computed(() => (focused() ? on[3] : off[3])),
  ];
}

/**
 * The whole frame of one window: the painted glass and lines, the caption,
 * and the client surface in the middle.
 */
export function aeroFrame(
  window: WaylandWindow,
  options: FrameOptions,
): CompositionRenderable {
  const focused = window.isFocused;
  const maximized = window.state[WIN_MAXIMIZED];

  // Maximised, Windows pushes the frame off the screen: square corners, no
  // outer lines, a shorter caption and only the two lines around the client.
  const captionHeight = computed(() =>
    maximized() ? FRAME.captionMaximized : FRAME.caption,
  );
  const borderWidth = computed(() =>
    maximized() ? FRAME.borderMaximized : FRAME.border,
  );
  const buttonTop = computed(() =>
    maximized() ? FRAME.buttonTopMaximized : FRAME.buttonTop,
  );
  const buttonRight = computed(() =>
    maximized() ? FRAME.buttonRightMaximized : FRAME.buttonRight,
  );

  const paint = paintShader("./assets/frame.frag", {
    uniforms: {
      radius_top: computed(() => (maximized() ? 0 : FRAME.radiusTop)),
      radius_bottom: computed(() => (maximized() ? 0 : FRAME.radiusBottom)),
      outer_lines: computed(() => (maximized() ? 0 : 1)),
      caption_h: captionHeight,
      border_w: borderWidth,
      tint: focusColour(focused, COLOR.glassActive, COLOR.glassInactive),
      line_outer: focusColour(
        focused,
        COLOR.outlineActive,
        COLOR.outlineInactive,
      ),
      line_inner: focusColour(
        focused,
        COLOR.highlightActive,
        COLOR.highlightInactive,
      ),
      edge_light: focusColour(
        focused,
        COLOR.clientLightActive,
        COLOR.clientLightInactive,
      ),
      edge_dark: focusColour(
        focused,
        COLOR.clientDarkActive,
        COLOR.clientDarkInactive,
      ),
    },
  });

  const shadow = focused((on: boolean) =>
    on
      ? [{ y: 6, blur: 20, spread: -3, color: "#0000008c" }]
      : [{ y: 3, blur: 12, spread: -3, color: "#00000059" }],
  );

  const captionStyle: SSDStyle = {
    position: "relative",
    height: captionHeight,
    paddingTop: buttonTop,
    paddingLeft: FRAME.iconLeft,
    paddingRight: buttonRight,
    gap: FRAME.iconGap,
    alignItems: "center",
    flexShrink: 0,
  };

  const caption = [
    <AppIcon
      icon={window.icon}
      style={{
        width: FRAME.iconSize,
        height: FRAME.iconSize,
        flexShrink: 0,
      }}
    />,
    <CaptionTitle window={window} />,
    <Box style={{ flexGrow: 1, minWidth: 0 }} />,
    // The group hangs from the top of the caption. The wrapper fills the
    // caption's content box so that `alignItems: center` above cannot push
    // the buttons down; inside it they sit at the top, as Windows draws
    // them, with the frame's own lines as their top edge.
    <Box
      direction="column"
      style={{
        height: computed(() => captionHeight() - buttonTop()),
        flexShrink: 0,
        alignItems: "start",
        justifyContent: "start",
      }}
    >
      <Box
        direction="row"
        style={{ height: FRAME.buttonHeight, flexShrink: 0 }}
      >
        <CaptionButton
          kind="min"
          width={CAPTION_BUTTON.minimize}
          label="Minimize"
          focused={focused}
          onClick={options.onMinimize}
        />
        <CaptionButton
          kind={maximized((on: boolean) => (on ? "restore" : "max"))}
          width={CAPTION_BUTTON.maximize}
          label={"Maximize"}
          focused={focused}
          enabled={window.isResizable}
          onClick={options.onToggleMaximize}
        />
        <CaptionButton
          kind="close"
          width={CAPTION_BUTTON.close}
          label="Close"
          focused={focused}
          onClick={() => window.close()}
        />
      </Box>
    </Box>,
  ];

  const body = [
    <Box direction="row" style={captionStyle}>
      {caption}
    </Box>,
    <Box
      direction="column"
      style={{
        flexGrow: 1,
        paddingLeft: borderWidth,
        paddingRight: borderWidth,
        paddingBottom: borderWidth,
      }}
    >
      <Box direction="column" style={{ flexGrow: 1 }}>
        <ClientWindow />
      </Box>
    </Box>,
  ];

  return (
    <WindowBorder
      style={{
        borderRadius: computed(() => (maximized() ? 0 : FRAME.radiusTop)),
        boxShadow: shadow,
        background: "#00000000",
      }}
      interaction={{
        resizeHitArea: {
          edgePx: 6,
          cornerPx: 16,
          insetPx: 6,
        },
      }}
    >
      {options.glassSuspended ? (
        <Box
          direction="column"
          paint={paint}
          style={{ flexGrow: 1, background: "#00000000" }}
        >
          {body}
        </Box>
      ) : (
        <ShaderEffect
          shader={options.glass}
          paint={paint}
          direction="column"
          style={{ flexGrow: 1 }}
        >
          {body}
        </ShaderEffect>
      )}
    </WindowBorder>
  );
}
