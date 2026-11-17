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

/**
 * Which side a submenu opens on and how far it must slide sideways so it stays inside the window.
 * It prefers the right of its parent, then the left; when neither fits it takes the side with more
 * room and slides back in, so it is never cut by an edge (the sidebar sits against the left one).
 */
export function subSidePlacement(
  parentLeft: number,
  parentRight: number,
  size: number,
  limit: number,
): { flip: boolean; shift: number } {
  const GAP = 2;
  const rightOverflow = Math.max(0, parentRight + GAP + size - (limit - EDGE));
  const leftOverflow = Math.max(0, EDGE - (parentLeft - GAP - size));
  if (rightOverflow === 0) return { flip: false, shift: 0 };
  if (leftOverflow === 0) return { flip: true, shift: 0 };
  return rightOverflow <= leftOverflow
    ? { flip: false, shift: -rightOverflow }
    : { flip: true, shift: leftOverflow };
}

/**
 * Where a dropdown of `width` × `height` hangs from the `anchor` button, as window coordinates:
 * aligned to the button's right edge and below it, pulled inside the window sideways, and above
 * the button when it would fall off the bottom and there is room above.
 */
export function dropdownPosition(
  anchor: { top: number; bottom: number; right: number },
  size: { width: number; height: number },
  limit: { width: number; height: number },
  gap = 6,
): { left: number; top: number } {
  const left = clampStart(anchor.right - size.width, size.width, limit.width);
  const below = anchor.bottom + gap;
  const above = anchor.top - gap - size.height;
  const fitsBelow = below + size.height <= limit.height - EDGE;
  const top = fitsBelow || above < EDGE ? below : above;
  return { left, top: clampStart(top, size.height, limit.height) };
}
