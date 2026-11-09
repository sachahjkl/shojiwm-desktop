/**
 * Whether the desktop animates, and how fast. The shell passes on the
 * system's preference (org.gnome.desktop.interface enable-animations) and
 * the speed the Control Panel sets over its socket; the window manager and
 * the window frames read them here.
 */

let animations = true;
let speed = 1;

export function setAnimations(on: boolean): void {
  animations = on;
}

/** 1 is the normal speed; 2 plays every animation in half the time. */
export function setAnimationSpeed(value: number): void {
  speed = Number.isFinite(value) && value > 0 ? value : 1;
}

/** The length of an animation: `ms` at the chosen speed, or 0 when off. */
export function motion(ms: number): number {
  return animations ? Math.round(ms / speed) : 0;
}
