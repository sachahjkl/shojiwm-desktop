/**
 * The screens: their modes and scales.
 *
 * The session sets the starting layout in SHOJIWM_WIN7_OUTPUTS, Hyprland
 * monitor rules `name,mode,position,scale` separated by `;`, for example
 * `DP-2,2560x1440@180,0x0,1.25;HDMI-A-1,preferred,auto,1.875`. Without it,
 * ShojiWM picks the best mode of each output at scale 1.
 *
 * The Control Panel's Screen Resolution page can then choose another mode
 * and scale for a screen. Its choices replace the rule's mode and scale, and
 * the screens are then placed side by side from left to right, in the order
 * of their current positions, so a new scale leaves no gap or overlap.
 */
import { COMPOSITOR, type DisplayConfigDraft } from "shoji_wm";

/** The mode and scale the Control Panel chose for one screen. */
export interface DisplayChoice {
  width: number;
  height: number;
  refreshRate?: number;
  scale: number;
}

/** What the Control Panel shows of a screen. */
export interface ScreenInfo {
  name: string;
  description: string;
  width: number;
  height: number;
  refreshRate: number;
  scale: number;
  x: number;
  y: number;
  modes: { width: number; height: number; refreshRate: number }[];
}

function parseRules(rules: string): DisplayConfigDraft {
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

const RULES = COMPOSITOR.env.get("SHOJIWM_WIN7_OUTPUTS");
const base: DisplayConfigDraft = RULES ? parseRules(RULES) : {};
let choices: Record<string, DisplayChoice> = {};

function layout(): DisplayConfigDraft {
  const names = Object.keys(choices);
  if (names.length === 0) {
    // The rules, and for a screen they do not name, ShojiWM's own choice
    // said out loud: a draft that leaves a screen out keeps what it shows,
    // so a reverted scale would stay.
    const draft: DisplayConfigDraft = { ...base };
    for (const output of COMPOSITOR.output.outputs) {
      if (!(output.name in draft)) {
        draft[output.name] = { resolution: "best", position: "auto", scale: 1 };
      }
    }
    return draft;
  }
  // Every connected screen, left to right as they are now, each with its
  // mode and scale: the choice, else what it shows now.
  const screens = [...COMPOSITOR.output.outputs]
    .filter((output) => output.enabled)
    .sort((a, b) => a.position.x - b.position.x);
  const draft: DisplayConfigDraft = { ...base };
  let x = 0;
  for (const output of screens) {
    const choice = choices[output.name];
    const mode = choice ?? {
      width: output.resolution?.width ?? 0,
      height: output.resolution?.height ?? 0,
      refreshRate: output.resolution?.refreshRate,
      scale: output.scale || 1,
    };
    const entry = base[output.name];
    draft[output.name] = {
      ...(entry && !("mode" in entry) ? entry : {}),
      resolution: {
        width: mode.width,
        height: mode.height,
        ...(mode.refreshRate ? { refreshRate: mode.refreshRate } : {}),
      },
      position: { x, y: 0 },
      scale: mode.scale,
    };
    x += Math.round(mode.width / mode.scale);
  }
  return draft;
}

COMPOSITOR.output.configure(() => layout());

/** Apply the Control Panel's choices; an empty object goes back to the rules. */
export function setDisplayChoices(next: Record<string, DisplayChoice>): void {
  const clean: Record<string, DisplayChoice> = {};
  for (const [name, choice] of Object.entries(next ?? {})) {
    const width = Number(choice?.width);
    const height = Number(choice?.height);
    const scale = Number(choice?.scale);
    if (width > 0 && height > 0 && scale >= 0.5 && scale <= 4) {
      const refreshRate = Number(choice?.refreshRate);
      clean[name] = {
        width,
        height,
        scale,
        ...(refreshRate > 0 ? { refreshRate } : {}),
      };
    }
  }
  choices = clean;
  COMPOSITOR.output.reconfigure();
}

/** The connected screens, with their modes, for the Control Panel. */
export function screens(): ScreenInfo[] {
  return COMPOSITOR.output.outputs
    .filter((output) => output.enabled)
    .map((output) => {
      const seen = new Set<string>();
      const modes = output.availableModes
        .map((mode) => ({
          width: mode.width,
          height: mode.height,
          refreshRate: Math.round(mode.refreshRate),
        }))
        .filter((mode) => {
          const key = `${mode.width}x${mode.height}@${mode.refreshRate}`;
          if (seen.has(key)) {
            return false;
          }
          seen.add(key);
          return true;
        })
        .sort(
          (a, b) =>
            b.width * b.height - a.width * a.height ||
            b.refreshRate - a.refreshRate,
        );
      return {
        name: output.name,
        description:
          [output.make, output.model].filter(Boolean).join(" ") ||
          output.description ||
          output.name,
        width: output.resolution?.width ?? 0,
        height: output.resolution?.height ?? 0,
        refreshRate: Math.round(output.resolution?.refreshRate ?? 0),
        scale: output.scale,
        x: output.position.x,
        y: output.position.y,
        modes,
      };
    });
}
