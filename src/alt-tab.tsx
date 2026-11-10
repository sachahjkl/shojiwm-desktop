/**
 * Alt+Tab: a panel of live window thumbnails, as on Windows 7.
 *
 * The panel is built with ShojiWM's output composition: one render texture
 * per window, drawn in a grid over a translucent card. While it is open an
 * input grab takes the keyboard and the pointer, so Tab, the arrow keys, the
 * wheel and a click all pick a window without reaching the windows below.
 *
 * A composition draws no text: each window's title, under its thumbnail, is
 * the shell's (AltTabTitles.qml), on the overlay layer, which the panel's
 * composition draws over itself. The panel tells the shell where its cells
 * are and their titles (`announce`).
 */

import {
  COMPOSITOR,
  LayerPopups,
  Layers,
  Solid,
  TextureView,
  Windows,
  computed,
  outputLogicalSize,
  renderTexture,
  signal,
  type CompositionRenderable,
  type InputGrab,
  type OutputInfo,
  type ReadonlySignal,
  type WaylandWindow,
} from "shoji_wm";
import type { FloatingWindowManager } from "./window-manager";

/** Sizes of the panel, in logical pixels. */
const PANEL = {
  cellWidth: 168,
  /** The thumbnail's part of a cell. */
  cellHeight: 126,
  /** The title's part, under the thumbnail. */
  titleHeight: 22,
  gap: 8,
  padding: 16,
  /** Space kept between a thumbnail and the edge of its cell. */
  cellInset: 7,
} as const;

/** Margin left around a window inside its texture, for the drop shadow. */
const TEXTURE_MARGIN = 40;
/** A composition may declare at most 64 textures. Stay well under it. */
const MAX_WINDOWS = 24;

interface Session {
  outputName: string;
  windows: WaylandWindow[];
  index: number;
}

interface CellLayout {
  x: number;
  y: number;
  width: number;
  height: number;
}

interface PanelLayout {
  x: number;
  y: number;
  width: number;
  height: number;
  cells: CellLayout[];
}

function panelLayout(output: OutputInfo, count: number): PanelLayout {
  const { width: outputWidth, height: outputHeight } =
    outputLogicalSize(output);
  const available = Math.max(
    PANEL.cellWidth,
    outputWidth * 0.92 - PANEL.padding * 2,
  );
  const perRow = Math.max(
    1,
    Math.floor((available + PANEL.gap) / (PANEL.cellWidth + PANEL.gap)),
  );
  const columns = Math.min(count, perRow);
  const rows = Math.ceil(count / columns);

  const width =
    PANEL.padding * 2 + columns * PANEL.cellWidth + (columns - 1) * PANEL.gap;
  const cellHeight = PANEL.cellHeight + PANEL.titleHeight;
  const height = PANEL.padding * 2 + rows * cellHeight + (rows - 1) * PANEL.gap;
  const x = Math.round((outputWidth - width) / 2);
  const y = Math.round((outputHeight - height) / 2);

  const cells: CellLayout[] = [];
  for (let index = 0; index < count; index++) {
    const column = index % columns;
    const row = Math.floor(index / columns);
    cells.push({
      x: x + PANEL.padding + column * (PANEL.cellWidth + PANEL.gap),
      y: y + PANEL.padding + row * (cellHeight + PANEL.gap),
      width: PANEL.cellWidth,
      height: cellHeight,
    });
  }

  return { x, y, width, height, cells };
}

/** What the shell draws over the panel: each cell's title, in the output's logical pixels. */
export interface AltTabTitles {
  output: string;
  selected: number;
  titleHeight: number;
  cells: { x: number; y: number; width: number; title: string }[];
}

export interface AltTab {
  /** Open the panel, or step through it when it is already open. */
  open(step: number): void;
  /** The composition of one output, or `null` when the panel is closed. */
  compose(output: OutputInfo): CompositionRenderable | null;
  /** `true` while window blur must be left out (see the module comment). */
  blurSuspended: ReadonlySignal<boolean>;
  /** Id of the window the panel has selected, or `null`. */
  selectedWindowId: ReadonlySignal<string | null>;
}

export function createAltTab(
  manager: FloatingWindowManager,
  announce: (titles: AltTabTitles | null) => void,
): AltTab {
  const [session, setPanel] = signal<Session | null>(null);

  /** Sets the panel's state and tells the shell its titles (null: closed). */
  function setSession(next: Session | null): void {
    setPanel(next);
    const output = next ? COMPOSITOR.output.get(next.outputName) : undefined;
    if (!next || !output) {
      announce(null);
      return;
    }
    const cells = panelLayout(output, next.windows.length).cells;
    announce({
      output: next.outputName,
      selected: next.index,
      titleHeight: PANEL.titleHeight,
      cells: next.windows.map((window, index) => ({
        x: cells[index].x,
        y: cells[index].y + PANEL.cellHeight,
        width: cells[index].width,
        title: window.title.peek(),
      })),
    });
  }
  let grab: InputGrab | null = null;
  let layout: PanelLayout | null = null;

  function close(activate: boolean): void {
    const current = session.peek();
    setSession(null);
    layout = null;
    grab?.release();
    grab = null;
    if (!current) {
      return;
    }
    const window = current.windows[current.index];
    if (activate && window) {
      manager.activate(window);
    }
  }

  function step(delta: number): void {
    const current = session.peek();
    if (!current) {
      return;
    }
    const count = current.windows.length;
    const index = (((current.index + delta) % count) + count) % count;
    setSession({ ...current, index });
  }

  function selectAt(x: number, y: number): number | null {
    const current = session.peek();
    if (!current || !layout) {
      return null;
    }
    const output = COMPOSITOR.output.get(current.outputName);
    if (!output) {
      return null;
    }
    const localX = x - output.position.x;
    const localY = y - output.position.y;
    for (const [index, cell] of layout.cells.entries()) {
      if (
        localX >= cell.x &&
        localX < cell.x + cell.width &&
        localY >= cell.y &&
        localY < cell.y + cell.height
      ) {
        return index;
      }
    }
    return null;
  }

  function open(delta: number): void {
    const current = session.peek();
    if (current) {
      step(delta);
      return;
    }

    const windows = manager.recentWindows().slice(0, MAX_WINDOWS);
    if (windows.length < 2) {
      return;
    }
    const outputName = manager.outputName();
    if (!outputName) {
      return;
    }

    const count = windows.length;
    setSession({
      outputName,
      windows,
      index: ((delta % count) + count) % count,
    });

    grab = COMPOSITOR.input.grab({
      onKey(event) {
        if (event.state === "released") {
          if (event.key === "Alt_L" || event.key === "Alt_R") {
            close(true);
          }
          return;
        }
        switch (event.key) {
          case "Tab":
            step(event.modifiers.shift ? -1 : 1);
            break;
          case "Right":
          case "Down":
            step(1);
            break;
          case "Left":
          case "Up":
            step(-1);
            break;
          case "Return":
          case "space":
            close(true);
            break;
          case "Escape":
            close(false);
            break;
          default:
            break;
        }
      },
      onPointerMotion(event) {
        const index = selectAt(event.position.x, event.position.y);
        const current = session.peek();
        if (index !== null && current && index !== current.index) {
          setSession({ ...current, index });
        }
      },
      onPointerButton(event) {
        if (event.state !== "pressed") {
          return;
        }
        const index = selectAt(event.position.x, event.position.y);
        if (index === null) {
          close(false);
          return;
        }
        const current = session.peek();
        if (current) {
          setSession({ ...current, index });
        }
        close(true);
      },
      onCancel() {
        grab = null;
        setSession(null);
        layout = null;
      },
    });
  }

  function thumbnail(
    output: OutputInfo,
    window: WaylandWindow,
    cell: CellLayout,
  ): CompositionRenderable {
    const rect = window.rect;
    const textureWidth = rect.width + TEXTURE_MARGIN * 2;
    const textureHeight = rect.height + TEXTURE_MARGIN * 2;
    const texture = renderTexture({
      key: `alt-tab-${window.id}`,
      width: textureWidth,
      height: textureHeight,
      content: (
        <Windows
          windows={[window]}
          offsetX={output.position.x - rect.x + TEXTURE_MARGIN}
          offsetY={output.position.y - rect.y + TEXTURE_MARGIN}
        />
      ),
    });

    const boxWidth = cell.width - PANEL.cellInset * 2;
    // (the thumbnail's part of the cell: the title is under it)
    const boxHeight = PANEL.cellHeight - PANEL.cellInset * 2;
    const scale = Math.min(
      boxWidth / Math.max(1, textureWidth),
      boxHeight / Math.max(1, textureHeight),
      1,
    );
    const width = Math.round(textureWidth * scale);
    const height = Math.round(textureHeight * scale);

    return (
      <TextureView
        texture={texture}
        x={Math.round(cell.x + (cell.width - width) / 2)}
        y={Math.round(cell.y + (PANEL.cellHeight - height) / 2)}
        width={width}
        height={height}
      />
    );
  }

  function compose(output: OutputInfo): CompositionRenderable | null {
    const current = session();
    if (!current || current.outputName !== output.name) {
      return null;
    }

    const panel = panelLayout(output, current.windows.length);
    layout = panel;
    const selected = panel.cells[current.index];

    return (
      <>
        <Layers layers={["background", "bottom"]} />
        <Windows />
        <Solid color="#0a1c2b4d" />
        {/* The panel goes over the taskbar, as the one on Windows does. */}
        <Layers layers={["top"]} />
        <Solid
          color="#ffffff59"
          x={panel.x - 1}
          y={panel.y - 1}
          width={panel.width + 2}
          height={panel.height + 2}
        />
        <Solid
          color="#102c44e6"
          x={panel.x}
          y={panel.y}
          width={panel.width}
          height={panel.height}
        />
        {selected ? (
          <Solid
            color="#ffffff38"
            x={selected.x}
            y={selected.y}
            width={selected.width}
            height={selected.height}
          />
        ) : null}
        {current.windows.map((window, index) =>
          thumbnail(output, window, panel.cells[index]),
        )}
        {/* The titles (the shell's, on the overlay layer) over the panel. */}
        <Layers layers={["overlay"]} />
        <LayerPopups />
      </>
    );
  }

  return {
    open,
    compose,
    blurSuspended: computed(() => session() !== null),
    selectedWindowId: computed(() => {
      const current = session();
      return current ? (current.windows[current.index]?.id ?? null) : null;
    }),
  };
}
