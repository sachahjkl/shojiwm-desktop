/**
 * The window frame: a Windows 7 Aero caption drawn with ShojiWM components.
 *
 * The glass is the compositor's own blur of whatever is behind the window,
 * tinted blue-grey and covered with a drawn sheen. The caption buttons sit at
 * the right, the icon and the title at the left.
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
  useState,
  type CompiledEffectHandle,
  type ReadonlySignal,
  type SSDStyle,
  type WaylandWindow,
} from "shoji_wm";
import type { CompositionRenderable } from "shoji_wm/types";
import { CAPTION_BUTTON, COLOR, FONT, FRAME } from "./theme";
import { WIN_MAXIMIZED } from "./window-manager";

type ButtonTone = "normal" | "close";

/** The pale label Windows shows under a caption button you rest on. */
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
  width: number;
  glyph: string | ReadonlySignal<string>;
  tone: ButtonTone;
  label: string;
  enabled?: ReadonlySignal<boolean>;
  onClick: () => void;
}

/**
 * One caption button: a glass pill, a drawn gloss, and a glyph. Under the
 * pointer it lights up and glows; the close button glows red.
 */
const CaptionButton = ({
  width,
  glyph,
  tone,
  label,
  enabled,
  onClick,
}: CaptionButtonProps) => {
  const [hover, setHover] = useState(false);
  const [pressed, setPressed] = useState(false);

  const usable = computed(() => enabled?.() ?? true);
  const background = computed(() => {
    if (!usable()) {
      return "#ffffff0f";
    }
    if (pressed()) {
      return tone === "close" ? COLOR.closePress : COLOR.buttonPress;
    }
    if (hover()) {
      return tone === "close" ? COLOR.closeHover : COLOR.buttonHover;
    }
    return tone === "close" ? COLOR.closeRest : COLOR.buttonRest;
  });
  const glow = computed(() =>
    hover() && usable()
      ? [
          {
            blur: 9,
            spread: 1,
            color: tone === "close" ? COLOR.closeGlow : "#bfe4ff8c",
          },
        ]
      : [],
  );

  return (
    <Box style={{ position: "relative", flexShrink: 0 }}>
      <Button
        onHoverChange={setHover}
        onActiveChange={setPressed}
        onClick={() => {
          if (usable()) {
            onClick();
          }
        }}
        style={{
          width,
          height: CAPTION_BUTTON.height,
          borderRadius: 3,
          background,
          border: { px: 1, color: COLOR.buttonBorder },
          boxShadow: glow,
        }}
      >
        <Tooltip text={label} />
      </Button>
      <Image
        src="./assets/caption-button-gloss.svg"
        fit="fill"
        style={{
          width,
          height: CAPTION_BUTTON.height,
          position: "absolute",
          inset: 0,
          zIndex: 1,
          borderRadius: 3,
          pointerEvents: "none",
        }}
      />
      <Image
        src={glyph}
        fit="fill"
        style={{
          width,
          height: CAPTION_BUTTON.height,
          position: "absolute",
          inset: 0,
          zIndex: 2,
          opacity: computed(() => (usable() ? 1 : 0.45)),
          pointerEvents: "none",
        }}
      />
    </Box>
  );
};

/** Minimise, maximise or restore, and close. */
const CaptionButtons = ({
  window,
  onMinimize,
  onToggleMaximize,
}: {
  window: WaylandWindow;
  onMinimize: () => void;
  onToggleMaximize: () => void;
}) => (
  <Box
    direction="row"
    style={{ gap: 2, alignItems: "center", flexShrink: 0, paddingLeft: 6 }}
  >
    <CaptionButton
      width={CAPTION_BUTTON.width}
      glyph="./assets/caption-minimize.svg"
      tone="normal"
      label="Minimize"
      onClick={onMinimize}
    />
    <CaptionButton
      width={CAPTION_BUTTON.width}
      glyph={window.state[WIN_MAXIMIZED]((maximized: boolean) =>
        maximized
          ? "./assets/caption-restore.svg"
          : "./assets/caption-maximize.svg",
      )}
      tone="normal"
      label="Maximize"
      enabled={window.isResizable}
      onClick={onToggleMaximize}
    />
    <CaptionButton
      width={CAPTION_BUTTON.closeWidth}
      glyph="./assets/caption-close.svg"
      tone="close"
      label="Close"
      onClick={() => window.close()}
    />
  </Box>
);

/** The icon, the title and the buttons, laid out across the caption. */
function captionContent(
  window: WaylandWindow,
  onMinimize: () => void,
  onToggleMaximize: () => void,
): CompositionRenderable[] {
  return [
    <Image
      src="./assets/caption-gloss.svg"
      fit="fill"
      style={{
        position: "absolute",
        inset: 0,
        zIndex: 0,
        pointerEvents: "none",
        opacity: window.isFocused((focused: boolean) => (focused ? 1 : 0.55)),
      }}
    />,
    <AppIcon
      icon={window.icon}
      style={{ width: 16, height: 16, flexShrink: 0 }}
    />,
    <Label
      text={window.title}
      style={{
        color: window.isFocused((focused: boolean) =>
          focused ? COLOR.titleActive : COLOR.titleInactive,
        ),
        fontFamily: FONT,
        fontSize: 12,
        fontWeight: "medium",
        textOverflow: "ellipsis",
        flexGrow: 1,
        flexShrink: 1,
        minWidth: 0,
      }}
    />,
    <CaptionButtons
      window={window}
      onMinimize={onMinimize}
      onToggleMaximize={onToggleMaximize}
    />,
  ];
}

export interface FrameOptions {
  /** The compiled blur used behind the caption glass. */
  glass: CompiledEffectHandle;
  /** `true` while the blur must be left out (the Alt+Tab panel is open). */
  glassSuspended: boolean;
  onMinimize: () => void;
  onToggleMaximize: () => void;
}

/**
 * The whole frame of one window: outline, glass caption, glass edges and the
 * client surface inside a bright inner line.
 */
export function aeroFrame(
  window: WaylandWindow,
  options: FrameOptions,
): CompositionRenderable {
  const tint = window.isFocused((focused: boolean) =>
    focused ? COLOR.glassActive : COLOR.glassInactive,
  );
  const outline = window.isFocused((focused: boolean) =>
    focused ? COLOR.outlineActive : COLOR.outlineInactive,
  );
  const clientEdge = window.isFocused((focused: boolean) =>
    focused ? COLOR.clientEdgeActive : COLOR.clientEdgeInactive,
  );
  const shadow = window.isFocused((focused: boolean) =>
    focused
      ? [{ y: 8, blur: 26, spread: -4, color: "#000000a6" }]
      : [{ y: 3, blur: 14, spread: -4, color: "#00000073" }],
  );

  const captionStyle: SSDStyle = {
    position: "relative",
    height: FRAME.caption,
    paddingLeft: 7,
    paddingRight: CAPTION_BUTTON.inset,
    gap: 6,
    alignItems: "center",
    background: tint,
  };

  const content = captionContent(
    window,
    options.onMinimize,
    options.onToggleMaximize,
  );
  const caption = options.glassSuspended ? (
    <Box direction="row" style={{ ...captionStyle, background: "#41678ce6" }}>
      {content}
    </Box>
  ) : (
    <ShaderEffect shader={options.glass} direction="row" style={captionStyle}>
      {content}
    </ShaderEffect>
  );

  return (
    <WindowBorder
      style={{
        border: { px: FRAME.outline, color: outline },
        borderRadius: FRAME.radius,
        background: tint,
        boxShadow: shadow,
      }}
      interaction={{
        resizeHitArea: {
          edgePx: 6,
          cornerPx: 14,
          insetPx: FRAME.glass,
        },
      }}
    >
      <Box direction="column">
        {caption}
        <Box
          direction="column"
          style={{
            flexGrow: 1,
            paddingLeft: FRAME.glass,
            paddingRight: FRAME.glass,
            paddingBottom: FRAME.glass,
          }}
        >
          <Box
            direction="column"
            style={{
              flexGrow: 1,
              border: { px: FRAME.clientEdge, color: clientEdge },
            }}
          >
            <ClientWindow />
          </Box>
        </Box>
      </Box>
    </WindowBorder>
  );
}
