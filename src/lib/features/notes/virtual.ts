// Windowing for the note list: only the rows near the viewport are in the DOM.

export interface VirtualRange {
  /** First row to render. */
  start: number;
  /** One past the last row to render. */
  end: number;
  /** Space above the first rendered row. */
  before: number;
  /** Space below the last rendered row. */
  after: number;
}

/** Rows are all `itemHeight` tall; `overscan` extra rows are kept on each side. */
export function virtualRange(
  scrollTop: number,
  viewportHeight: number,
  itemHeight: number,
  count: number,
  overscan = 6,
): VirtualRange {
  if (count <= 0 || itemHeight <= 0) {
    return { start: 0, end: 0, before: 0, after: 0 };
  }
  const first = Math.floor(Math.max(0, scrollTop) / itemHeight);
  const visible = Math.ceil(Math.max(0, viewportHeight) / itemHeight) + 1;
  const start = Math.min(count - 1, Math.max(0, first - overscan));
  const end = Math.min(count, first + visible + overscan);
  return {
    start,
    end,
    before: start * itemHeight,
    after: (count - end) * itemHeight,
  };
}
