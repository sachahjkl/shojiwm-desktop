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
  computed,
  type CompositionRenderable,
  type OutputInfo,
  type WaylandWindow,
} from "shoji_wm";
import { windowFrame } from "./themes/frames";
import { createAltTab } from "./alt-tab";
import { SHELL_SOCKET, createPeek } from "./peek";
import { createZoom } from "./zoom";
import { program } from "./programs";
// The screens: SHOJIWM_WIN7_OUTPUTS and the Control Panel's choices.
import "./display";
import { layerGlassEffect, popupGlassEffect, windowGlassEffect } from "./glass";
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

/**
 * The browser the keys start. The terminal, the file manager, the task
 * manager, the clipboard history and the screenshot are the Control Panel's
 * choices (src/programs.ts).
 */
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

// ---------------------------------------------------------------------------
// Decoration policy
// ---------------------------------------------------------------------------

// Draw the theme's frame for every client that lets us, and keep the
// client's own title bar when it insists on one (Firefox, GTK header bars).
// A client that speaks no decoration protocol (GTK 4, as File Roller) draws
// its own, as xdg-shell has it: a frame would add a second title bar.
// Ghostty asks for ours (window-decoration = server); its tabs are windows of
// one group in our frame (src/groups.ts, TABBED_PROGRAMS).
COMPOSITOR.window.decoration.configure((_window, context) => ({
  mode: context.canNegotiate
    ? (context.clientPreference ?? "server")
    : "client",
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
// (PEEK is read when the panel changes, after both are made)
const ALT_TAB = createAltTab(WINDOWS, (titles) => PEEK.tell("alt-tab", titles));
const PEEK = createPeek(WINDOWS);
const ZOOM = createZoom();

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
  COMPOSITOR.process.spawn({ command: program("terminal") });
});
COMPOSITOR.key.bind("files", "Super+E", () => {
  COMPOSITOR.process.spawn({ command: program("fileManager") });
});
COMPOSITOR.key.bind("browser", "Super+B", () => {
  COMPOSITOR.process.spawn({ command: BROWSER });
});
/** Ask the shell for "Shut Down Windows" on the current monitor. */
function openShutdownDialog(): void {
  const monitor = WINDOWS.outputName() ?? "";
  COMPOSITOR.process.spawn({
    command: `quickshell -p ${SHELL_PATH} ipc call shutdown openOn "${monitor}"`,
  });
}

// Alt+F4 closes the focused window; with none, as on the desktop, it asks
// what the computer should do, as Windows 7 does.
COMPOSITOR.key.bind("close-window", "Alt+F4", () => {
  const window = WINDOWS.focusedWindow();
  if (window) {
    window.close();
  } else {
    openShutdownDialog();
  }
});
// ShojiWM quits on Super+Shift+Q unless the configuration binds it: here it
// closes the focused window, and the session ends from the power menu.
COMPOSITOR.key.bind(
  "close-window-super",
  "Super+Shift+Q",
  withFocused((window) => window.close()),
);
COMPOSITOR.key.bind("shutdown-dialog", "Super+Shift+E", openShutdownDialog);
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
  withFocused((window) => WINDOWS.restoreOrMinimize(window)),
);
COMPOSITOR.key.bind(
  "window-to-left-output",
  "Super+Shift+Left",
  withFocused((window) => WINDOWS.moveToOutput(window, -1)),
);
COMPOSITOR.key.bind(
  "window-to-right-output",
  "Super+Shift+Right",
  withFocused((window) => WINDOWS.moveToOutput(window, 1)),
);
// The tabs of a group (src/groups.ts): the next, the one before, and the
// shown one out of its group.
COMPOSITOR.key.bind(
  "next-tab",
  "Super+Ctrl+Right",
  withFocused((window) => WINDOWS.nextTab(window, 1)),
);
COMPOSITOR.key.bind(
  "previous-tab",
  "Super+Ctrl+Left",
  withFocused((window) => WINDOWS.nextTab(window, -1)),
);
COMPOSITOR.key.bind(
  "detach-tab",
  "Super+Ctrl+Down",
  withFocused((window) => WINDOWS.detachTab(window)),
);
COMPOSITOR.key.bind(
  "minimize-others",
  "Super+Home",
  withFocused((window) => WINDOWS.toggleOthers(window)),
);
COMPOSITOR.key.bind(
  "snap-left",
  "Super+Left",
  withFocused((window) => WINDOWS.snapKey(window, "left")),
);
COMPOSITOR.key.bind(
  "snap-right",
  "Super+Right",
  withFocused((window) => WINDOWS.snapKey(window, "right")),
);
COMPOSITOR.key.bind("lock", "Super+L", () => {
  COMPOSITOR.process.spawn({
    command: `quickshell -p ${SHELL_PATH} ipc call lock lock`,
  });
});
// Print Screen and Ctrl+Print Screen: the Control Panel's screenshot program,
// by default the whole screen to the clipboard, as Windows 7 does.
for (const [id, key] of [
  ["screenshot", "Print"],
  ["screenshot-ctrl", "Ctrl+Print"],
] as const) {
  COMPOSITOR.key.bind(id, key, () => {
    COMPOSITOR.process.spawn({ command: program("screenshot") });
  });
}
// Super+X: the Start orb's right-click menu on the focused screen, the
// power user's menu Windows opens with Win+X.
COMPOSITOR.key.bind("admin-menu", "Super+X", () => {
  const monitor = WINDOWS.outputName() ?? "";
  COMPOSITOR.process.spawn({
    command: `quickshell -p ${SHELL_PATH} ipc call adminmenu toggleOn "${monitor}"`,
  });
});
// Super+Shift+D: the shell's launcher, the Start menu's search in the middle
// of the focused screen.
COMPOSITOR.key.bind("launcher", "Super+Shift+D", () => {
  const monitor = WINDOWS.outputName() ?? "";
  COMPOSITOR.process.spawn({
    command: `quickshell -p ${SHELL_PATH} ipc call launcher toggleOn "${monitor}"`,
  });
});
// More keys from SHOJIWM_WIN7_KEYS: a JSON object of shortcut to shell
// command, for programs that would register a global shortcut through a
// portal, for example {"Ctrl+Shift+ugrave": "papier toggle"}.
const EXTRA_KEYS = COMPOSITOR.env.get("SHOJIWM_WIN7_KEYS");
if (EXTRA_KEYS) {
  const keys = JSON.parse(EXTRA_KEYS) as Record<string, string>;
  for (const [shortcut, command] of Object.entries(keys)) {
    COMPOSITOR.key.bind(`extra:${shortcut}`, shortcut, () => {
      COMPOSITOR.process.spawn({ command });
    });
  }
}

// Super+1 to Super+9: the taskbar button at that place, as Windows 7 does;
// with Shift, another instance of it. ShojiWM matches the key's unshifted
// symbol, so the digit row is bound both as digits (QWERTY) and as the
// symbols AZERTY prints on it.
const NUMBER_ROW: readonly (readonly string[])[] = [
  ["1", "ampersand"],
  ["2", "eacute"],
  ["3", "quotedbl"],
  ["4", "apostrophe"],
  ["5", "parenleft"],
  ["6", "minus"],
  ["7", "egrave"],
  ["8", "underscore"],
  ["9", "ccedilla"],
];
NUMBER_ROW.forEach((keys, index) => {
  const number = index + 1;
  for (const key of keys) {
    COMPOSITOR.key.bind(`taskbar-${key}`, `Super+${key}`, () => {
      COMPOSITOR.process.spawn({
        command: `quickshell -p ${SHELL_PATH} ipc call taskbar activate ${number}`,
      });
    });
    COMPOSITOR.key.bind(`taskbar-new-${key}`, `Super+Shift+${key}`, () => {
      COMPOSITOR.process.spawn({
        command: `quickshell -p ${SHELL_PATH} ipc call taskbar launchNew ${number}`,
      });
    });
  }
});

// Super+V: the clipboard history the Control Panel chooses (for example
// "papier toggle"). Without one, the key does nothing.
COMPOSITOR.key.bind("clipboard-history", "Super+V", () => {
  const command = program("clipboard");
  if (command) {
    COMPOSITOR.process.spawn({ command });
  }
});

// Super+R: the Run dialog, above the focused screen's Start button, with
// its history.
COMPOSITOR.key.bind("run", "Super+R", () => {
  const monitor = WINDOWS.outputName() ?? "";
  COMPOSITOR.process.spawn({
    command: `quickshell -p ${SHELL_PATH} ipc call run toggleOn "${monitor}"`,
  });
});

// Ctrl+Shift+Escape: the task manager, btop in the terminal.
COMPOSITOR.key.bind("task-manager", "Ctrl+Shift+Escape", () => {
  COMPOSITOR.process.spawn({ command: program("taskManager") });
});

// The media keys: the volume of the default output (the speaker in the
// notification area follows it), the microphone, and the player through
// MPRIS. A held volume key repeats.
const MEDIA_KEYS: Record<string, string> = {
  XF86AudioRaiseVolume: "wpctl set-volume -l 1 @DEFAULT_AUDIO_SINK@ 2%+",
  XF86AudioLowerVolume: "wpctl set-volume @DEFAULT_AUDIO_SINK@ 2%-",
  XF86AudioMute: "wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle",
  XF86AudioMicMute: "wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle",
  XF86AudioPlay: "playerctl play-pause",
  XF86AudioPause: "playerctl pause",
  XF86AudioStop: "playerctl stop",
  XF86AudioNext: "playerctl next",
  XF86AudioPrev: "playerctl previous",
};
for (const [key, command] of Object.entries(MEDIA_KEYS)) {
  const action = () => {
    COMPOSITOR.process.spawn({ command });
  };
  COMPOSITOR.key.bind(
    `media:${key}`,
    key,
    key.endsWith("Volume") ? action : action,
  );
}

// Super+D shows the desktop and then puts the windows back, as on Windows 7;
// Super+M, Windows 7's key to minimise everything, does the same.
for (const key of ["D", "M"]) {
  COMPOSITOR.key.bind(`show-desktop-${key}`, `Super+${key}`, () =>
    WINDOWS.toggleShowDesktop(),
  );
}

// ---------------------------------------------------------------------------
// Glass
// ---------------------------------------------------------------------------

COMPOSITOR.effect.layer = (layer) => {
  const namespace = layer.namespace() ?? "";
  // The shell's own glass surfaces. Other programs' layers and the shell's
  // full-screen catchers named "no_blur" get none: a blur costs a capture
  // and a pass every frame they change.
  if (namespace.startsWith("shojiwm-win7-")) {
    const glass = layerGlassEffect();
    return glass ? { behind: glass } : {};
  }
  return {};
};

// The popups of layer surfaces are the shell's own panels and menus (the
// notification area's, the calendar, the jump lists): the same glass. A
// window's popups (a program's menus) draw their own background.
COMPOSITOR.effect.popup = (popup) => {
  if (popup.parentKind !== "layer") {
    return {};
  }
  const glass = popupGlassEffect();
  return glass ? { behind: glass } : {};
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
  PEEK.compose(output) ??
  ZOOM.compose(output) ?? <DefaultComposition />;

// ---------------------------------------------------------------------------
// The frame of every window
// ---------------------------------------------------------------------------

COMPOSITOR.window.composition = (window: WaylandWindow) => {
  const rect = window.state[WIN_RECT];
  const minimized = window.state[WIN_MINIMIZED];
  const forceRectSize = computed(
    () => window.isResizable() && !window.isTransient(),
  );
  // A tab its group does not show is away: not drawn, not reached (idle
  // takes it out of the pointer's way, over the shown tab at the same place).
  const away = computed(() => WINDOWS.groups.isAway(window.id));
  const idle = computed<boolean>(() => minimized() || away());
  const interactive = computed(() => !minimized() && !away());
  // See-through while dragged over another group's caption it would join,
  // so the ghost tab under it shows (not while it moves along its own).
  const opacity = computed(() => {
    const drop = WINDOWS.groups.drop();
    return drop?.dragged === window.id &&
      !WINDOWS.groups.members(window.id).includes(drop.target)
      ? 0.55
      : 1;
  });

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
      visible={computed(() => !away())}
      opacity={opacity}
      // Tell the client it is tiled, so it squares off its own corners.
      tiled={true}
    >
      {windowFrame(window, {
        glass: windowGlassEffect(),
        glassSuspended: ALT_TAB.blurSuspended(),
        onMinimize: () => WINDOWS.setMinimized(window, true),
        onToggleMaximize: () => WINDOWS.toggleMaximized(window),
        tabs: {
          groups: WINDOWS.groups,
          windowOf: (id) => WINDOWS.windowById(id),
          onTabPress: (tab) => WINDOWS.pressTab(tab),
        },
      })}
    </ManagedWindow>
  );
};

export default COMPOSITOR;
