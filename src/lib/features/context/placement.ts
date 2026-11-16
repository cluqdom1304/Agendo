/** Space kept between a menu and the edge of the window. */
export const EDGE = 8;

/**
 * Where a menu of `size` pixels starting at `start` has to start to stay inside `limit` (the
 * window), keeping `EDGE` pixels free on both sides. A menu larger than the window starts at the
 * margin: the panel itself scrolls (see `max-height` in `ContextMenu.svelte`).
 */
export function clampStart(start: number, size: number, limit: number): number {
  const latest = limit - size - EDGE;
  return Math.max(EDGE, Math.min(start, latest));
}

/**
 * How far a submenu whose top edge is at `top` (and whose height is `size`) must move
 * vertically to fit inside `limit`: up if it falls off the bottom, never past the top margin.
 */
export function slideToFit(top: number, size: number, limit: number): number {
  const shift = Math.min(0, limit - EDGE - (top + size));
  return Math.max(shift, EDGE - top);
}
