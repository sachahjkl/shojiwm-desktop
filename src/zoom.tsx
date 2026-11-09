/**
 * Screen zoom: Super+Shift and the wheel magnify the screen around the
 * pointer, as Hyprland's cursor zoom does.
 *
 * The output is drawn into a texture, then drawn larger, so that the point
 * under the pointer stays where it is. The pointer therefore still clicks
 * what it shows. The zoom follows the pointer, and Super+Shift+wheel down to
 * 1x turns it off.
 */
import {
  COMPOSITOR,
  DefaultComposition,
  TextureView,
  outputLogicalSize,
  renderTexture,
  signal,
  type CompositionRenderable,
  type OutputInfo,
} from "shoji_wm";

/** Each wheel notch multiplies or divides the zoom by this. */
const STEP = 1.25;
const MAX_FACTOR = 10;

export interface Zoom {
  /** The output composition while zoomed, or null. */
  compose(output: OutputInfo): CompositionRenderable | null;
}

export function createZoom(): Zoom {
  const factor = signal(1);
  const centre = signal({ x: 0, y: 0 });

  // The pointer, always: the zoom starts where the pointer is.
  let pointer = { x: 0, y: 0 };
  COMPOSITOR.event.onPointerMove((event) => {
    pointer = { x: event.position.x, y: event.position.y };
    if (factor.peek() > 1) {
      centre.set(pointer);
    }
  });

  function change(by: number): void {
    centre.set(pointer);
    const next = Math.min(MAX_FACTOR, factor.peek() * by);
    // Close to 1 is 1: the zoom is off and the output draws as usual.
    factor.set(next < 1.05 ? 1 : next);
  }

  COMPOSITOR.key.bind("zoom-in", "Super+Shift+WheelScrollUp", () =>
    change(STEP),
  );
  COMPOSITOR.key.bind("zoom-out", "Super+Shift+WheelScrollDown", () =>
    change(1 / STEP),
  );
  COMPOSITOR.key.bind("zoom-reset", "Super+Shift+0", () => factor.set(1));

  return {
    compose(output) {
      const scale = factor();
      if (scale <= 1) {
        return null;
      }
      const { width, height } = outputLogicalSize(output);
      const point = centre();
      // The pointer in output coordinates, kept on the output.
      const cx = Math.min(width, Math.max(0, point.x - output.position.x));
      const cy = Math.min(height, Math.max(0, point.y - output.position.y));
      const screen = renderTexture({
        key: `zoom:${output.name}`,
        content: <DefaultComposition />,
      });
      return (
        <TextureView
          texture={screen}
          x={cx - cx * scale}
          y={cy - cy * scale}
          width={width * scale}
          height={height * scale}
        />
      );
    },
  };
}
