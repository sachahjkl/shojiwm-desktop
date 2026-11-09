/**
 * A Windows 7 Aero desktop for ShojiWM.
 *
 * This file wires everything together: the environment, the programs the
 * keys start, the shell (taskbar and Start menu, drawn by Quickshell), the
 * blur behind the glass, the window frame, and the window manager.
 *
 * Edit it and press Super+Shift+R to apply the change to the running session.
 */

import {
  COMPOSITOR,
  ClientWindow,
  DefaultComposition,
  LayerPopups,
  Layers,
  ManagedWindow,
  Solid,
  Windows,
  backdropSource,
  compileEffect,
  compileLayerEffect,
  computed,
  dualKawaseBlur,
  layerSource,
  loadShader,
  shaderStage,
  xrayBackdropSource,
  type CompositionRenderable,
  type DisplayConfigDraft,
  type OutputInfo,
  type WaylandWindow,
} from "shoji_wm";
import { aeroFrame } from "./decoration";
import { createAltTab } from "./alt-tab";
import { SHELL_SOCKET, createPeek } from "./peek";
import {
  FloatingWindowManager,
  WIN_FULLSCREEN,
  WIN_MINIMIZED,
  WIN_RECT,
  usesClientDecorationState,
} from "./window-manager";

// ---------------------------------------------------------------------------
// Programs
// ---------------------------------------------------------------------------

/** The terminal, the file manager and the browser the keys start. */
const TERMINAL = ["kitty"];
const FILE_MANAGER = ["thunar"];
const BROWSER = ["chromium", "--ozone-platform-hint=auto"];

/**
 * The launcher Super+D opens: Vicinae, a fast search for programs, files and
 * commands. The Nix package of this configuration writes the path of its own
 * Vicinae here, so it is a dependency rather than something to install.
 */
const VICINAE = "vicinae";

/** Where the Quickshell shell lives, as a shell word. */
const SHELL_PATH = "~/.config/shojiwm/shell";

/**
 * The outputs, from SHOJIWM_WIN7_OUTPUTS: Hyprland monitor rules
 * `name,mode,position,scale` separated by `;`, for example
 * `DP-2,2560x1440@180,0x0,1.25;HDMI-A-1,preferred,auto,1.875`.
 * The value has no quotes, which session environment files do not escape.
 * Without it, ShojiWM picks the best mode of each output at scale 1.
 */
function parseOutputs(rules: string): DisplayConfigDraft {
  const layout: DisplayConfigDraft = {};
  for (const rule of rules.split(";")) {
    const [name, mode, position, scale] = rule.split(",");
    if (!name || !mode || !position || !scale) {
      throw new Error(`SHOJIWM_WIN7_OUTPUTS: invalid rule "${rule}"`);
    }
    const [size, refresh] = mode.split("@");
    const [width, height] = (size ?? "").split("x").map(Number);
    const [x, y] = position.split("x").map(Number);
    layout[name] = {
      resolution:
        mode === "preferred"
          ? "best"
          : {
              width: width ?? 0,
              height: height ?? 0,
              ...(refresh ? { refreshRate: Number(refresh) } : {}),
            },
      position: position === "auto" ? "auto" : { x: x ?? 0, y: y ?? 0 },
      scale: Number(scale),
    };
  }
  return layout;
}

const OUTPUTS = COMPOSITOR.env.get("SHOJIWM_WIN7_OUTPUTS");
if (OUTPUTS) {
  const layout = parseOutputs(OUTPUTS);
  COMPOSITOR.output.configure(() => layout);
}

COMPOSITOR.env.apply({
  QT_QPA_PLATFORM: "wayland;xcb",
  ELECTRON_OZONE_PLATFORM_HINT: "wayland",
  MOZ_ENABLE_WAYLAND: 1,
  XCURSOR_SIZE: 24,
});
COMPOSITOR.env.publish();

// The taskbar, the Start menu and the wallpaper.
COMPOSITOR.process.once("shell", {
  command: `exec quickshell --path ${SHELL_PATH}`,
  runPolicy: "once-per-session",
  env: {
    // Qt asks for a text-input protocol as soon as the search box of the
    // Start menu takes the keyboard. Closing the menu then took the whole
    // shell down with "fatal error: Invalid argument". The search box reads
    // plain keys, so the protocol is left out.
    QT_WAYLAND_TEXT_INPUT_PROTOCOL: "none",
    // Where the shell asks for Aero Peek and the show-desktop toggle.
    SHOJIWM_WIN7_SOCKET: SHELL_SOCKET,
  },
});

// The launcher, kept running so Super+D shows it at once.
//
// It reads ~/.config/shojiwm/vicinae.json, written once here if it is
// missing. That file imports assets/vicinae/aero.json, so what Vicinae's own
// settings write lands in it and the theme stays underneath. The Aero theme
// is found through XDG_DATA_DIRS, in assets/vicinae/vicinae/themes.
//
// Vicinae's welcome window sets a theme, a hotkey and a start at login, all
// three of which this configuration already does; it could also replace the
// Aero theme. It is marked as seen, the first time only, in the state file
// Vicinae itself writes when the window is finished.
COMPOSITOR.process.once("launcher", {
  command:
    `config="$HOME/.config/shojiwm/vicinae.json"; ` +
    `[ -e "$config" ] || printf '{\\n  "imports": ["assets/vicinae/aero.json"]\\n}\\n' >"$config"; ` +
    `state="\${XDG_STATE_HOME:-$HOME/.local/state}/vicinae"; mkdir -p "$state"; ` +
    `[ -e "$state/onboarding.json" ] || printf '{"version":1,"completedAt":""}\\n' >"$state/onboarding.json"; ` +
    `XDG_DATA_DIRS="$HOME/.config/shojiwm/assets/vicinae:\${XDG_DATA_DIRS:-/usr/local/share:/usr/share}" ` +
    `exec ${VICINAE} server --config "$config"`,
  runPolicy: "once-per-session",
});

// ---------------------------------------------------------------------------
// Decoration policy
// ---------------------------------------------------------------------------

// Draw the Aero frame for every client that lets us, and keep the client's
// own title bar when it insists on one (Firefox, GTK header bars).
COMPOSITOR.window.decoration.configure((_window, context) => ({
  mode: context.clientPreference ?? "server",
}));

// GTK 3 applications such as Thunar pick their title bar once, from this
// default, and ignore later per-window answers.
COMPOSITOR.window.decoration.kdeDefaultMode(() => "server");

// Triple buffering.
COMPOSITOR.rendering.framePacing = "throughput";

// ---------------------------------------------------------------------------
// Window management
// ---------------------------------------------------------------------------

const WINDOWS = new FloatingWindowManager();
const ALT_TAB = createAltTab(WINDOWS);
const PEEK = createPeek(WINDOWS);

const FULLSCREEN_Z_INDEX = 2_000_000_000;

COMPOSITOR.event.onOpen((window) => WINDOWS.onOpen(window));
COMPOSITOR.event.onFirstCommit((window) => WINDOWS.onFirstCommit(window));
COMPOSITOR.event.onStartClose((window) => WINDOWS.onStartClose(window));
COMPOSITOR.event.onClose((window) => WINDOWS.onClose(window));
COMPOSITOR.event.onFocus((window, focused) => WINDOWS.onFocus(window, focused));
COMPOSITOR.event.onPointerMove((event) => WINDOWS.onPointerMove(event));
COMPOSITOR.event.onWindowMove((event) => WINDOWS.onWindowMove(event));
COMPOSITOR.event.onWindowResize((event) => WINDOWS.onWindowResize(event));
COMPOSITOR.event.onWindowMaximizeRequest((event) =>
  WINDOWS.onMaximizeRequest(event),
);
COMPOSITOR.event.onWindowMinimizeRequest((event) =>
  WINDOWS.onMinimizeRequest(event),
);
COMPOSITOR.event.onWindowFullscreenRequest((event) =>
  WINDOWS.onFullscreenRequest(event),
);
COMPOSITOR.event.onWindowActivateRequest((event) =>
  WINDOWS.onActivateRequest(event),
);

// Hold Super and drag anywhere on a window to move it, or to resize it.
COMPOSITOR.pointer.bindWindowMoveModifier("Super");
COMPOSITOR.pointer.bindWindowResizeModifier("Super");

// ---------------------------------------------------------------------------
// Keys
// ---------------------------------------------------------------------------

function withFocused(action: (window: WaylandWindow) => void): () => void {
  return () => {
    const window = WINDOWS.focusedWindow();
    if (window) {
      action(window);
    }
  };
}

/** Ask the shell to show or hide the Start menu on the current monitor. */
function toggleStartMenu(): void {
  const monitor = WINDOWS.outputName() ?? "";
  COMPOSITOR.process.spawn({
    command: `quickshell -p ${SHELL_PATH} ipc call startmenu toggleOn "${monitor}"`,
  });
}

COMPOSITOR.key.bind("start-menu", "Super", toggleStartMenu, {
  on: "release",
  within: 300,
});
COMPOSITOR.key.bind("terminal", "Super+Return", () => {
  COMPOSITOR.process.spawn({ command: TERMINAL });
});
COMPOSITOR.key.bind("files", "Super+E", () => {
  COMPOSITOR.process.spawn({ command: FILE_MANAGER });
});
COMPOSITOR.key.bind("browser", "Super+B", () => {
  COMPOSITOR.process.spawn({ command: BROWSER });
});
COMPOSITOR.key.bind(
  "close-window",
  "Alt+F4",
  withFocused((window) => window.close()),
);
COMPOSITOR.key.bind("switch-window", "Alt+Tab", () => ALT_TAB.open(1));
COMPOSITOR.key.bind("switch-window-back", "Alt+Shift+Tab", () =>
  ALT_TAB.open(-1),
);
COMPOSITOR.key.bind(
  "maximize-window",
  "Super+Up",
  withFocused((window) => WINDOWS.setMaximized(window, true)),
);
COMPOSITOR.key.bind(
  "restore-window",
  "Super+Down",
  withFocused((window) => WINDOWS.restore(window)),
);
COMPOSITOR.key.bind(
  "snap-left",
  "Super+Left",
  withFocused((window) => WINDOWS.snap(window, "left")),
);
COMPOSITOR.key.bind(
  "snap-right",
  "Super+Right",
  withFocused((window) => WINDOWS.snap(window, "right")),
);
COMPOSITOR.key.bind("lock", "Super+L", () => {
  COMPOSITOR.process.spawn({
    command: `quickshell -p ${SHELL_PATH} ipc call lock lock`,
  });
});
COMPOSITOR.key.bind("launcher", "Super+D", () => {
  COMPOSITOR.process.spawn({ command: `${VICINAE} toggle` });
});
// Super+D is the launcher, so showing the desktop moves to Super+M, the key
// Windows 7 minimises everything with.
COMPOSITOR.key.bind("show-desktop", "Super+M", () =>
  WINDOWS.toggleShowDesktop(),
);

// ---------------------------------------------------------------------------
// Glass
// ---------------------------------------------------------------------------

/** The blur behind the caption bar of a window. */
const WINDOW_GLASS = compileEffect({
  input: backdropSource(),
  capturePadding: 24,
  invalidate: { kind: "on-source-damage-box", damagePadding: 8 },
  pipeline: [dualKawaseBlur({ radius: 4, passes: 2 })],
});

/**
 * The blur behind a shell surface.
 *
 * The second stage clips the blur to the pixels the surface itself draws, so
 * the rounded corners of the Start menu have no blurred square behind them.
 */
function layerGlass(
  input: ReturnType<typeof backdropSource | typeof xrayBackdropSource>,
) {
  return compileLayerEffect({
    input,
    capturePadding: 24,
    invalidate: { kind: "on-source-damage-box", damagePadding: 8 },
    alpha: "preserve",
    pipeline: [
      dualKawaseBlur({ radius: 4, passes: 2 }),
      shaderStage(loadShader("./assets/layer-mask.frag"), {
        textures: { layer_mask: layerSource() },
        uniforms: {
          opacity_threshold: 0.25,
          mask_feather: 0.04,
        },
      }),
    ],
  });
}

/** The Start menu shows the windows behind it, so it blurs the whole scene. */
const MENU_GLASS = layerGlass(backdropSource());

/**
 * The taskbar blurs only the wallpaper and the layers below the windows.
 *
 * Windows stay out of the taskbar's exclusive zone, so the result is the
 * same. `backdropSource()` reads the frame being composited, and ShojiWM
 * (9e778dc) then composites the taskbar onto itself on some frames: the
 * whole bar flashes lighter while the Start menu is open. The X-ray source
 * renders its own capture and does not read that frame.
 */
const TASKBAR_GLASS = layerGlass(xrayBackdropSource());

COMPOSITOR.effect.layer = (layer) => {
  switch (layer.namespace()) {
    // The wallpaper has nothing behind it to blur.
    case "no_blur":
      return {};
    case "shojiwm-win7-taskbar":
      return { behind: TASKBAR_GLASS };
    default:
      return { behind: MENU_GLASS };
  }
};

/**
 * The preview Windows draws while a window is dragged against an edge: the
 * rectangle it would take, over the desktop and under the taskbar.
 */
function snapPreviewComposition(
  output: OutputInfo,
): CompositionRenderable | null {
  const zone = WINDOWS.snapPreview();
  if (!zone || zone.outputName !== output.name) {
    return null;
  }
  const x = zone.rect.x - output.position.x;
  const y = zone.rect.y - output.position.y;
  const { width, height } = zone.rect;

  return (
    <>
      <Layers layers={["background", "bottom"]} />
      <Windows />
      <Solid color="#bfe4ff4d" x={x} y={y} width={width} height={height} />
      <Solid color="#eaf6ffcc" x={x} y={y} width={width} height={2} />
      <Solid
        color="#eaf6ffcc"
        x={x}
        y={y + height - 2}
        width={width}
        height={2}
      />
      <Solid color="#eaf6ffcc" x={x} y={y} width={2} height={height} />
      <Solid
        color="#eaf6ffcc"
        x={x + width - 2}
        y={y}
        width={2}
        height={height}
      />
      <Layers layers={["top", "overlay"]} />
      <LayerPopups />
    </>
  );
}

// Alt+Tab draws each window alone in its own texture, where a backdrop blur
// has nothing under it to show. Leave the glass out while it is open.
COMPOSITOR.rendering.composition = (output) =>
  ALT_TAB.compose(output) ??
  snapPreviewComposition(output) ??
  PEEK.compose(output) ?? <DefaultComposition />;

// ---------------------------------------------------------------------------
// The frame of every window
// ---------------------------------------------------------------------------

COMPOSITOR.window.composition = (window: WaylandWindow) => {
  const rect = window.state[WIN_RECT];
  const minimized = window.state[WIN_MINIMIZED];
  const forceRectSize = computed(
    () => window.isResizable() && !window.isTransient(),
  );
  const idle = computed<boolean>(() => minimized());
  const interactive = computed(() => !minimized());

  // Fullscreen: nothing but the client surface, edge to edge. Drawing only
  // this is what lets the compositor hand the buffer straight to the display.
  if (window.state[WIN_FULLSCREEN]()) {
    return (
      <ManagedWindow
        rect={rect}
        zIndex={FULLSCREEN_Z_INDEX}
        forceRectSize={forceRectSize}
        allowTearing={true}
      >
        <ClientWindow />
      </ManagedWindow>
    );
  }

  // A client that draws its own title bar gets no frame from us.
  if (usesClientDecorationState(window.decoration())) {
    return (
      <ManagedWindow
        rect={rect}
        zIndex={WINDOWS.zIndex(window)}
        forceRectSize={forceRectSize}
        idle={idle}
        interactive={interactive}
      >
        <ClientWindow />
      </ManagedWindow>
    );
  }

  return (
    <ManagedWindow
      rect={rect}
      zIndex={WINDOWS.zIndex(window)}
      forceRectSize={forceRectSize}
      idle={idle}
      interactive={interactive}
      // Tell the client it is tiled, so it squares off its own corners.
      tiled={true}
    >
      {aeroFrame(window, {
        glass: WINDOW_GLASS,
        glassSuspended: ALT_TAB.blurSuspended(),
        onMinimize: () => WINDOWS.setMinimized(window, true),
        onToggleMaximize: () => WINDOWS.toggleMaximized(window),
      })}
    </ManagedWindow>
  );
};

export default COMPOSITOR;
