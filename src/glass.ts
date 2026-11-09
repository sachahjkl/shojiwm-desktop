/**
 * The blur behind the glass: the caption bars, the taskbar and the shell's
 * menus.
 *
 * The strength is a level from 0 (no blur) to 4. The shell sends the level
 * the Control Panel sets over its socket. Each level is compiled once at
 * start; a change picks other compiled effects, and the window frames and
 * layers that read the level draw again.
 */
import {
  backdropSource,
  compileEffect,
  compileLayerEffect,
  compilePopupEffect,
  dualKawaseBlur,
  layerSource,
  loadShader,
  popupSource,
  shaderStage,
  signal,
  xrayBackdropSource,
} from "shoji_wm";

/** The blur of each level: 0 is none. */
const LEVELS: readonly { radius: number; passes: number }[] = [
  { radius: 0, passes: 0 },
  { radius: 2, passes: 2 },
  { radius: 4, passes: 2 },
  { radius: 4, passes: 3 },
  { radius: 5, passes: 4 },
];

/**
 * The colour of the glass the Control Panel sets, as Windows 7's Window
 * Color: "" for the plain glass, else a colour mixed into the caption bars'
 * tint at a strength from 0 to 1.
 */
const colourChoice = signal({ colour: "", strength: 0 });

export function glassColour(): { colour: string; strength: number } {
  return colourChoice();
}

export function setGlassColour(colour: unknown, strength: unknown): void {
  const hex =
    typeof colour === "string" && /^#[0-9a-fA-F]{6}$/.test(colour)
      ? colour
      : "";
  const amount = Number(strength);
  colourChoice.set({
    colour: hex,
    strength: Number.isFinite(amount) ? Math.max(0, Math.min(1, amount)) : 0,
  });
}

export const DEFAULT_GLASS_LEVEL = 2;
const level = signal(DEFAULT_GLASS_LEVEL);

/** The current level, 0 to 4. Reading it makes the reader draw again on a change. */
export function glassLevel(): number {
  return level();
}

export function setGlassLevel(value: number): void {
  const next = Math.round(value);
  level.set(
    Number.isFinite(next)
      ? Math.max(0, Math.min(LEVELS.length - 1, next))
      : DEFAULT_GLASS_LEVEL,
  );
}

/** The blur behind the caption bar of a window. */
function windowGlass(blur: { radius: number; passes: number }) {
  return compileEffect({
    input: backdropSource(),
    capturePadding: 24,
    invalidate: { kind: "on-source-damage-box", damagePadding: 8 },
    pipeline: [dualKawaseBlur(blur)],
  });
}

/**
 * The blur behind a shell surface.
 *
 * The second stage clips the blur to the pixels the surface itself draws, so
 * the rounded corners of the Start menu have no blurred square behind them.
 */
function layerGlass(
  input: ReturnType<typeof backdropSource | typeof xrayBackdropSource>,
  blur: { radius: number; passes: number },
) {
  return compileLayerEffect({
    input,
    capturePadding: 24,
    invalidate: { kind: "on-source-damage-box", damagePadding: 8 },
    alpha: "preserve",
    pipeline: [
      dualKawaseBlur(blur),
      shaderStage(loadShader("./assets/layer-mask.frag"), {
        textures: { layer_mask: layerSource() },
        // The taskbar's glass is only 12 to 30 % opaque, so anything the
        // surface draws at all counts; the transparent corners do not.
        uniforms: {
          opacity_threshold: 0.03,
          mask_feather: 0.02,
        },
      }),
    ],
  });
}

/**
 * The blur behind the shell's popups (the panels of the notification area,
 * the calendar, the jump lists): the taskbar's glass, masked by the popup's
 * own pixels in the same way.
 */
function popupGlass(blur: { radius: number; passes: number }) {
  return compilePopupEffect({
    input: backdropSource(),
    capturePadding: 24,
    invalidate: { kind: "on-source-damage-box", damagePadding: 8 },
    alpha: "preserve",
    pipeline: [
      dualKawaseBlur(blur),
      shaderStage(loadShader("./assets/layer-mask.frag"), {
        textures: { layer_mask: popupSource() },
        uniforms: {
          opacity_threshold: 0.03,
          mask_feather: 0.02,
        },
      }),
    ],
  });
}

// Level 0 has no blur, so nothing is compiled for it.
const WINDOW = LEVELS.map((blur, index) =>
  index === 0 ? null : windowGlass(blur),
);
const LAYER = LEVELS.map((blur, index) =>
  index === 0 ? null : layerGlass(backdropSource(), blur),
);
const POPUP = LEVELS.map((blur, index) =>
  index === 0 ? null : popupGlass(blur),
);

/** The blur behind caption bars at the current level, or null for none. */
export function windowGlassEffect() {
  return WINDOW[level()] ?? null;
}

/**
 * The blur behind the shell's surfaces (taskbar, Start menu, popups) at the
 * current level, or null for none. It blurs the whole scene, windows
 * included, so a window under the taskbar shows through it.
 */
export function layerGlassEffect() {
  return LAYER[level()] ?? null;
}

/** The blur behind the shell's popups at the current level, or null. */
export function popupGlassEffect() {
  return POPUP[level()] ?? null;
}
