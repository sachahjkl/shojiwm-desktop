/**
 * The square variant of the theme: the window frames, and the tooltips of
 * their caption buttons, get square corners. The shell sends the Control
 * Panel's choice over its socket; the frames that read it draw again on a
 * change.
 */
import { signal } from "shoji_wm";

const square = signal(false);

/** Whether corners are square. Reading it makes the reader draw again on a change. */
export function squareCorners(): boolean {
  return square();
}

export function setSquareCorners(on: unknown): void {
  square.set(on === true);
}

/** A corner's radius: `radius` in the rounded theme, 0 in the square one. */
export function corner(radius: number): number {
  return square() ? 0 : radius;
}
