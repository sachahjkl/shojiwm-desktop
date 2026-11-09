/**
 * Aero Peek, and the socket the shell talks to the compositor through.
 *
 * Resting the pointer on the show-desktop strip at the end of the taskbar
 * makes every window a sheet of glass: the desktop shows through, and each
 * window is left as an outline where it stands. Moving away brings them back.
 * A click on the strip puts every window away, and a second click brings back
 * the ones it put away.
 *
 * The shell is a separate process, so it asks for both over a socket that
 * speaks ShojiWM's newline-delimited JSON (`shoji_wm/ipc`):
 *
 *   { "method": "peek", "params": { "on": true } }
 *   { "method": "show-desktop" }
 */

import {
  LayerPopups,
  Layers,
  Solid,
  createIpcServer,
  defaultSocketPath,
  isSignal,
  signal,
  type CompositionRenderable,
  type MaybeSignal,
  type OutputInfo,
  type WaylandWindow,
} from "shoji_wm";
import {
  type FloatingWindowManager,
  WIN_FULLSCREEN,
  WIN_MINIMIZED,
  WIN_RECT,
} from "./window-manager";

/** The colours of a window while it is peeked through. */
const PEEK = {
  /** The outer line, dark so the sheet reads on a light wallpaper. */
  outline: "#0f1e2c66",
  /** The inner line, the lit edge of the glass. */
  edge: "#ffffffb3",
  /** The sheet itself: barely there. */
  fill: "#ffffff14",
} as const;

/**
 * The socket the shell connects to. Kept apart from ShojiWM's default one,
 * which its own sample configuration claims for workspaces.
 */
export const SHELL_SOCKET = defaultSocketPath().replace(
  /\.sock$/,
  "-win7.sock",
);

function numberOf(value: MaybeSignal<number>): number {
  return isSignal<number>(value) ? value() : value;
}

export interface Peek {
  /** The output composition while peeking, or null. */
  compose(output: OutputInfo): CompositionRenderable | null;
}

export function createPeek(manager: FloatingWindowManager): Peek {
  const peeking = signal(false);

  const server = createIpcServer(SHELL_SOCKET);
  server.handle("peek", (params) => {
    const on =
      typeof params === "object" &&
      params !== null &&
      (params as { on?: unknown }).on === true;
    peeking.set(on);
    return on;
  });
  server.handle("animations", (params) => {
    const on =
      typeof params !== "object" ||
      params === null ||
      (params as { on?: unknown }).on !== false;
    manager.setAnimations(on);
    return on;
  });
  server.handle("show-desktop", () => {
    // The desktop is what the pointer was peeking at; show it for real.
    peeking.set(false);
    manager.toggleShowDesktop();
    return true;
  });

  /** The outline of one window, in output coordinates. */
  function sheet(
    output: OutputInfo,
    window: WaylandWindow,
  ): CompositionRenderable | null {
    const rect = window.state[WIN_RECT]();
    const x = Math.round(numberOf(rect.x) - output.position.x);
    const y = Math.round(numberOf(rect.y) - output.position.y);
    const width = Math.round(numberOf(rect.width));
    const height = Math.round(numberOf(rect.height));
    if (width < 4 || height < 4) {
      return null;
    }

    const line = (color: string, inset: number): CompositionRenderable[] => [
      <Solid
        color={color}
        x={x + inset}
        y={y + inset}
        width={width - inset * 2}
        height={1}
      />,
      <Solid
        color={color}
        x={x + inset}
        y={y + height - inset - 1}
        width={width - inset * 2}
        height={1}
      />,
      <Solid
        color={color}
        x={x + inset}
        y={y + inset + 1}
        width={1}
        height={height - inset * 2 - 2}
      />,
      <Solid
        color={color}
        x={x + width - inset - 1}
        y={y + inset + 1}
        width={1}
        height={height - inset * 2 - 2}
      />,
    ];

    return (
      <>
        <Solid
          color={PEEK.fill}
          x={x + 2}
          y={y + 2}
          width={width - 4}
          height={height - 4}
        />
        {line(PEEK.outline, 0)}
        {line(PEEK.edge, 1)}
      </>
    );
  }

  function compose(output: OutputInfo): CompositionRenderable | null {
    if (!peeking()) {
      return null;
    }

    const windows = manager
      .recentWindows()
      .filter(
        (window) =>
          !window.state[WIN_MINIMIZED]() &&
          !window.state[WIN_FULLSCREEN]() &&
          (window.output() ?? output.name) === output.name,
      )
      // Back to front, so a front sheet's edge lies over the one behind it.
      .reverse();

    return (
      <>
        <Layers layers={["background", "bottom"]} />
        {windows.map((window) => sheet(output, window))}
        <Layers layers={["top", "overlay"]} />
        <LayerPopups />
      </>
    );
  }

  return { compose };
}
