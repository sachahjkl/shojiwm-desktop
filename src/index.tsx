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
  ManagedWindow,
  backdropSource,
  compileEffect,
  compileLayerEffect,
  computed,
  dualKawaseBlur,
  type WaylandWindow,
} from "shoji_wm";
import { aeroFrame } from "./decoration";
import { createAltTab } from "./alt-tab";
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

/** Where the Quickshell shell lives, as a shell word. */
const SHELL_PATH = "~/.config/shojiwm/shell";

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
COMPOSITOR.key.bind("show-desktop", "Super+D", () =>
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
  pipeline: [dualKawaseBlur({ radius: 5, passes: 3 })],
});

/** The blur behind the taskbar and the Start menu. */
const LAYER_GLASS = compileLayerEffect({
  input: backdropSource(),
  capturePadding: 24,
  invalidate: { kind: "on-source-damage-box", damagePadding: 8 },
  pipeline: [dualKawaseBlur({ radius: 5, passes: 3 })],
});

COMPOSITOR.effect.layer = (layer) => {
  // The wallpaper has nothing behind it to blur.
  if (layer.namespace() === "no_blur") {
    return {};
  }
  return { behind: LAYER_GLASS };
};

// Alt+Tab draws each window alone in its own texture, where a backdrop blur
// has nothing under it to show. Leave the glass out while it is open.
COMPOSITOR.rendering.composition = (output) =>
  ALT_TAB.compose(output) ?? <DefaultComposition />;

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
