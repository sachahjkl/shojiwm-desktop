/**
 * The window frame of the Kernel theme (metrics.ts says what it is).
 *
 * `assets/kernel/frame.frag` paints the frame: the outer line, the tinted line,
 * the caption plate. The caption buttons are plates of `assets/kernel/plate.frag`
 * with vector icons over them: the close button always red, the others clear
 * until the pointer is on them. An unfocused window takes the same drawing in
 * slate, its title and its icons dimmed.
 */

import {
  AppIcon,
  Box,
  Button,
  ClientWindow,
  Image,
  Label,
  Popup,
  WindowBorder,
  computed,
  paintShader,
  useState,
  type ReadonlySignal,
  type ShaderUniformVec4,
  type WaylandWindow,
} from "shoji_wm";
import type { CompositionRenderable } from "shoji_wm/types";
import type { FrameOptions } from "../frames";
import { WIN_MAXIMIZED } from "../../window-manager";
import { KCOLOR, KFONT_TEXT, KFONT_TITLE, KFRAME } from "./metrics";
import { captionTabs, type TabStyle } from "../tabs";

/** The tabs of a group, on the caption plate. */
const KTAB: TabStyle = {
  active: "#ffffff24",
  hover: "#ffffff14",
  separator: "#ffffff26",
  ghost: "#3f7fcd40",
  ghostLine: "#7fb2eaff",
  text: KCOLOR.title,
  textAway: KCOLOR.titleInactive,
  textGlow: null,
  fontFamily: KFONT_TITLE,
  fontSize: 12,
  radius: 2,
  closeIcon: "./assets/kernel/close.svg",
};

/** `#rrggbbaa` to the four numbers a shader uniform takes. */
function colour(hex: string): [number, number, number, number] {
  const value = hex.replace("#", "");
  const part = (at: number) => parseInt(value.slice(at, at + 2), 16) / 255;
  return [part(0), part(2), part(4), value.length >= 8 ? part(6) : 1];
}

/** A colour uniform that is `on` or `off` as `when` says. */
function pick(
  when: ReadonlySignal<boolean>,
  on: string,
  off: string,
): ShaderUniformVec4 {
  const a = colour(on);
  const b = colour(off);
  return [0, 1, 2, 3].map((i) =>
    computed(() => (when() ? a[i] : b[i])),
  ) as unknown as ShaderUniformVec4;
}

const Tooltip = ({ text }: { text: string }) => (
  <Popup
    trigger="hover"
    openDelay={600}
    closeDelay={0}
    placement="bottom"
    offset={6}
  >
    <Box
      style={{
        background: KCOLOR.tooltip,
        paddingLeft: 8,
        paddingRight: 8,
        paddingY: 4,
        border: { px: 1, color: KCOLOR.tooltipLine },
        boxShadow: [{ y: 3, blur: 10, color: "#00000080" }],
      }}
    >
      <Label
        text={text}
        style={{
          color: KCOLOR.tooltipText,
          fontFamily: KFONT_TEXT,
          fontSize: 11.5,
        }}
      />
    </Box>
  </Popup>
);

interface ButtonProps {
  kind: string | ReadonlySignal<string>;
  width: number;
  height: ReadonlySignal<number>;
  label: string;
  focused: ReadonlySignal<boolean>;
  enabled?: ReadonlySignal<boolean>;
  close?: boolean;
  onClick: () => void;
}

/** A caption button: its plate, its icon, and a clear button over them that takes the clicks. */
const CaptionButton = ({
  kind,
  width,
  height,
  label,
  focused,
  enabled,
  close,
  onClick,
}: ButtonProps) => {
  const [hover, setHover] = useState(false);
  const [pressed, setPressed] = useState(false);
  const usable = computed(() => enabled?.() ?? true);
  const live = computed(() => usable() && focused());
  const src = computed(
    () => `./assets/kernel/${typeof kind === "string" ? kind : kind()}.svg`,
  );
  // close: red, lighter under the pointer, its gradient upside down when held; inactive: a muted red.
  // the others: clear, a pale plate under the pointer, a dark one when held.
  const top = computed(() => {
    if (close) {
      if (!focused()) {
        return colour(KCOLOR.closeTopInactive);
      }
      if (pressed()) {
        return colour(KCOLOR.closeBottom);
      }
      return colour(hover() ? KCOLOR.closeTopHover : KCOLOR.closeTop);
    }
    if (!usable()) {
      return colour("#00000000");
    }
    if (pressed()) {
      return colour(KCOLOR.buttonPress);
    }
    return colour(hover() ? KCOLOR.buttonHover : "#00000000");
  });
  const bottom = computed(() => {
    if (close) {
      if (!focused()) {
        return colour(KCOLOR.closeBottomInactive);
      }
      if (pressed()) {
        return colour(KCOLOR.closeTop);
      }
      return colour(hover() ? KCOLOR.closeBottomHover : KCOLOR.closeBottom);
    }
    if (!usable()) {
      return colour("#00000000");
    }
    if (pressed()) {
      return colour(KCOLOR.buttonPress);
    }
    return colour(hover() ? "#ffffff0a" : "#00000000");
  });
  const plate = paintShader("./assets/kernel/plate.frag", {
    uniforms: {
      top: [0, 1, 2, 3].map((i) =>
        computed(() => top()[i]),
      ) as unknown as ShaderUniformVec4,
      bottom: [0, 1, 2, 3].map((i) =>
        computed(() => bottom()[i]),
      ) as unknown as ShaderUniformVec4,
      light: computed(() =>
        close ? (focused() ? 0.32 : 0.12) : hover() && !pressed() ? 0.14 : 0,
      ),
      seam: computed(() => (close ? 0.55 : 0)),
    },
  });
  return (
    <Box style={{ position: "relative", width, height, flexShrink: 0 }}>
      <Box
        paint={plate}
        style={{
          position: "absolute",
          inset: 0,
          width,
          height,
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
          alignItems: "center",
          justifyContent: "center",
          pointerEvents: "none",
        }}
      >
        <Image
          src={src}
          fit="fill"
          style={{
            width: KFRAME.glyph,
            height: KFRAME.glyph,
            opacity: computed(() =>
              live() || (close && focused()) ? 1 : 0.45,
            ),
          }}
        />
      </Box>
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
          height,
          background: "#00000000",
        }}
      >
        <Tooltip text={label} />
      </Button>
    </Box>
  );
};

export function kernelFrame(
  window: WaylandWindow,
  options: FrameOptions,
): CompositionRenderable {
  const focused = window.isFocused;
  const maximized = window.state[WIN_MAXIMIZED];
  const caption = computed(() =>
    maximized() ? KFRAME.captionMaximized : KFRAME.caption,
  );
  const border = computed(() =>
    maximized() ? KFRAME.borderMaximized : KFRAME.border,
  );
  // the buttons sit on the plate: under the outer line, above the line under the caption
  const outer = computed(() => (maximized() ? 0 : 1));
  const buttonHeight = computed(() => caption() - outer() - 1);

  const paint = paintShader("./assets/kernel/frame.frag", {
    uniforms: {
      radius: computed(() => (maximized() ? 0 : KFRAME.radius)),
      caption_h: caption,
      border_w: border,
      outer,
      cap_left: pick(
        focused,
        KCOLOR.captionLeftActive,
        KCOLOR.captionLeftInactive,
      ),
      cap_right: pick(
        focused,
        KCOLOR.captionRightActive,
        KCOLOR.captionRightInactive,
      ),
      line_outer: pick(focused, KCOLOR.outline, KCOLOR.outlineInactive),
      line_tint: pick(focused, KCOLOR.tintActive, KCOLOR.tintInactive),
      light: computed(() => (focused() ? 0.22 : 0.08)),
    },
  });

  const shadow = focused((on: boolean) =>
    on
      ? [{ y: 6, blur: 22, spread: 0, color: "#000000a6" }]
      : [{ y: 3, blur: 12, spread: 0, color: "#00000066" }],
  );

  const captionRow = (
    <Box
      direction="row"
      style={{
        height: caption,
        paddingTop: outer,
        paddingLeft: KFRAME.iconLeft,
        paddingRight: outer,
        gap: KFRAME.iconGap,
        alignItems: "center",
        flexShrink: 0,
      }}
    >
      {captionTabs(
        window,
        computed(() => buttonHeight() - 3),
        KTAB,
        options.tabs,
        KFRAME.iconGap,
        [
          <AppIcon
            icon={window.icon}
            style={{
              width: KFRAME.iconSize,
              height: KFRAME.iconSize,
              flexShrink: 0,
              opacity: focused((on: boolean) => (on ? 1 : 0.6)),
            }}
          />,
          <Label
            text={window.title}
            style={{
              color: focused((on: boolean) =>
                on ? KCOLOR.title : KCOLOR.titleInactive,
              ),
              fontFamily: KFONT_TITLE,
              fontSize: KFRAME.fontSize,
              textOverflow: "ellipsis",
              flexShrink: 1,
              minWidth: 0,
            }}
          />,
          <Box style={{ flexGrow: 1, minWidth: 0 }} />,
        ],
      )}
      {/* (the group fills the caption's height and sits at its top: the row's centring cannot push it down,
          and it stops short of the dark line under the caption) */}
      <Box
        direction="column"
        style={{
          height: computed(() => caption() - outer()),
          flexShrink: 0,
          alignItems: "start",
          justifyContent: "start",
        }}
      >
        <Box direction="row" style={{ height: buttonHeight, flexShrink: 0 }}>
          <CaptionButton
            kind="min"
            width={KFRAME.buttonWidth}
            height={buttonHeight}
            label="Minimize"
            focused={focused}
            onClick={options.onMinimize}
          />
          <CaptionButton
            kind={maximized((on: boolean) => (on ? "restore" : "max"))}
            width={KFRAME.buttonWidth}
            height={buttonHeight}
            label="Maximize"
            focused={focused}
            enabled={window.isResizable}
            onClick={options.onToggleMaximize}
          />
          <CaptionButton
            kind="close"
            width={KFRAME.closeWidth}
            height={buttonHeight}
            label="Close"
            focused={focused}
            close
            onClick={() => window.close()}
          />
        </Box>
      </Box>
    </Box>
  );

  return (
    <WindowBorder
      style={{
        borderRadius: computed(() => (maximized() ? 0 : KFRAME.radius)),
        boxShadow: shadow,
        background: "#00000000",
      }}
      interaction={{
        resizeHitArea: {
          edgePx: computed(() => (maximized() ? 0 : 6)),
          cornerPx: computed(() => (maximized() ? 0 : 16)),
          insetPx: computed(() => (maximized() ? 0 : 6)),
        },
      }}
    >
      <Box
        direction="column"
        paint={paint}
        style={{ flexGrow: 1, background: "#00000000" }}
      >
        {captionRow}
        <Box
          direction="column"
          style={{
            flexGrow: 1,
            paddingLeft: border,
            paddingRight: border,
            paddingBottom: border,
          }}
        >
          {/* Cropped to its place: a client's last picture, still larger while it shrinks, would spill over the frame. */}
          <Box direction="column" style={{ flexGrow: 1, overflow: "hidden" }}>
            <ClientWindow />
          </Box>
        </Box>
      </Box>
    </WindowBorder>
  );
}
