/**
 * Whether the desktop animates. The shell passes on the system's preference
 * (org.gnome.desktop.interface enable-animations) over its socket; the
 * window manager and the window frames read it here.
 */

let animations = true;

export function setAnimations(on: boolean): void {
  animations = on;
}

/** The length of an animation: `ms`, or 0 when animations are off. */
export function motion(ms: number): number {
  return animations ? ms : 0;
}
