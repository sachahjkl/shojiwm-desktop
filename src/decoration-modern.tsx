/**
 * The modern frame: the glass of `decoration.tsx`, without what dates it.
 *
 * The same paint shader draws it (`assets/frame.frag`), with rounder
 * corners, thin sides and quiet lines. The tint follows light or dark. The
 * title has no glow; the caption buttons are separate rounded shapes, lit
 * only under the pointer, the close button red.
 */

import {
  AppIcon,
  Box,
  Button,
  ClientWindow,
  Image,
  Label,
  ShaderEffect,
  WindowBorder,
  computed,
  paintShader,
  useState,
  type ReadonlySignal,
  type ShaderUniformVec4,
  type WaylandWindow,
} from "shoji_wm";
import type { CompositionRenderable } from "shoji_wm/types";
import type { FrameOptions } from "./decoration";
import { DARK } from "./style";
import { FONT, MODERN_COLOR, MODERN_FRAME } from "./theme";
import { WIN_MAXIMIZED } from "./window-manager";

type Palette = Record<keyof (typeof MODERN_COLOR)["light"], string>;

function palette(): Palette {
  return DARK() ? MODERN_COLOR.dark : MODERN_COLOR.light;
}

/** `#rrggbbaa` to the four numbers a shader uniform takes. */
function rgba(hex: string): [number, number, number, number] {
  const value = hex.replace("#", "");
  const part = (at: number) => parseInt(value.slice(at, at + 2), 16) / 255;
  return [part(0), part(2), part(4), value.length >= 8 ? part(6) : 1];
}

/** A colour uniform that follows focus and light or dark. */
function uniform(
  focused: ReadonlySignal<boolean>,
  pick: (palette: Palette, focused: boolean) => string,
): ShaderUniformVec4 {
  const channel = (index: number) =>
    computed(() => rgba(pick(palette(), focused()))[index]);
  return [channel(0), channel(1), channel(2), channel(3)];
}

interface ButtonProps {
  kind: string | ReadonlySignal<string>;
  /** What the button does, for whoever reads this; it shows no label. */
  label: string;
  close?: boolean;
  focused: ReadonlySignal<boolean>;
  enabled?: ReadonlySignal<boolean>;
  onClick: () => void;
}

const ModernButton = ({
  kind,
  close,
  focused,
  enabled,
  onClick,
}: ButtonProps) => {
  const [hover, setHover] = useState(false);
  const [pressed, setPressed] = useState(false);
  const usable = computed(() => enabled?.() ?? true);

  const background = computed(() => {
    if (!usable() || !hover()) {
      return "#00000000";
    }
    if (close) {
      return pressed() ? MODERN_COLOR.closePress : MODERN_COLOR.closeHover;
    }
    return pressed() ? palette().buttonPress : palette().buttonHover;
  });

  // The glyph's ink: white on the red close button, else the title's ink.
  const src = computed(() => {
    const name = typeof kind === "string" ? kind : kind();
    const ink = close && hover() ? "white" : DARK() ? "light" : "ink";
    return `./assets/caption-modern/${name}-${ink}.svg`;
  });

  const opacity = computed(() => (!usable() ? 0.35 : focused() ? 1 : 0.55));

  return (
    <Button
      onHoverChange={setHover}
      onActiveChange={setPressed}
      onClick={() => {
        if (usable()) {
          onClick();
        }
      }}
      style={{
        width: MODERN_FRAME.buttonWidth,
        height: MODERN_FRAME.buttonHeight,
        flexShrink: 0,
        borderRadius: MODERN_FRAME.buttonRadius,
        background,
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <Image
        src={src}
        fit="fill"
        style={{
          width: MODERN_FRAME.glyphSize,
          height: MODERN_FRAME.glyphSize,
          opacity,
          pointerEvents: "none",
        }}
      />
    </Button>
  );
};

export function modernFrame(
  window: WaylandWindow,
  options: FrameOptions,
): CompositionRenderable {
  const focused = window.isFocused;
  const maximized = window.state[WIN_MAXIMIZED];

  const captionHeight = computed(() =>
    maximized() ? MODERN_FRAME.captionMaximized : MODERN_FRAME.caption,
  );
  const borderWidth = computed(() =>
    maximized() ? MODERN_FRAME.borderMaximized : MODERN_FRAME.border,
  );
  const radius = computed(() => (maximized() ? 0 : MODERN_FRAME.radius));

  const paint = paintShader("./assets/frame.frag", {
    uniforms: {
      radius_top: radius,
      radius_bottom: radius,
      outer_lines: computed(() => (maximized() ? 0 : 1)),
      caption_h: captionHeight,
      border_w: borderWidth,
      tint: uniform(focused, (p, on) => (on ? p.glassActive : p.glassInactive)),
      line_outer: uniform(focused, (p, on) =>
        on ? p.outlineActive : p.outlineInactive,
      ),
      line_inner: uniform(focused, (p, on) =>
        on ? p.highlightActive : p.highlightInactive,
      ),
      edge_light: uniform(focused, (p) => p.clientLight),
      edge_dark: uniform(focused, (p) => p.clientDark),
    },
  });

  // A wide, soft shadow, as glass on a desk casts.
  const shadow = focused((on: boolean) =>
    on
      ? [{ y: 10, blur: 32, spread: -6, color: "#00000070" }]
      : [{ y: 4, blur: 18, spread: -6, color: "#00000040" }],
  );

  const titleColour = computed(() =>
    focused() ? palette().titleActive : palette().titleInactive,
  );

  const caption = (
    <Box
      direction="row"
      style={{
        position: "relative",
        height: captionHeight,
        paddingLeft: MODERN_FRAME.iconLeft,
        paddingRight: MODERN_FRAME.buttonRight,
        gap: MODERN_FRAME.iconGap,
        alignItems: "center",
        flexShrink: 0,
      }}
    >
      <AppIcon
        icon={window.icon}
        style={{ width: 16, height: 16, flexShrink: 0 }}
      />
      <Label
        text={window.title}
        style={{
          color: titleColour,
          fontFamily: FONT,
          fontSize: MODERN_FRAME.fontSize,
          fontWeight: "normal",
          textOverflow: "ellipsis",
          flexShrink: 1,
          minWidth: 0,
        }}
      />
      <Box style={{ flexGrow: 1, minWidth: 0 }} />
      <Box
        direction="row"
        style={{
          gap: MODERN_FRAME.buttonGap,
          flexShrink: 0,
          alignItems: "center",
        }}
      >
        <ModernButton
          kind="min"
          label="Minimize"
          focused={focused}
          onClick={options.onMinimize}
        />
        <ModernButton
          kind={maximized((on: boolean) => (on ? "restore" : "max"))}
          label="Maximize"
          focused={focused}
          enabled={window.isResizable}
          onClick={options.onToggleMaximize}
        />
        <ModernButton
          kind="close"
          label="Close"
          close={true}
          focused={focused}
          onClick={() => window.close()}
        />
      </Box>
    </Box>
  );

  const body = [
    caption,
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
        borderRadius: radius,
        boxShadow: shadow,
        background: "#00000000",
      }}
      interaction={{
        resizeHitArea: { edgePx: 6, cornerPx: 16, insetPx: 6 },
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
