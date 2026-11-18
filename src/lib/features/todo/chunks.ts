// Long lists of tasks are mounted in chunks: only those near the viewport are in the DOM.

/** Below this many rows a list is rendered whole: windowing would only add work. */
export const WINDOW_THRESHOLD = 200;
/** Rows per chunk. */
export const CHUNK_SIZE = 50;
/** The single-line row height, used for chunks that have not been on screen yet. */
export const ESTIMATED_ROW_HEIGHT = 34;
/** Chunks mounted from the start, before the observer reports what is on screen. */
export const EAGER_CHUNKS = 3;

export function chunkRows<T>(items: readonly T[], size = CHUNK_SIZE): T[][] {
  const chunks: T[][] = [];
  for (let start = 0; start < items.length; start += size) {
    chunks.push(items.slice(start, start + size));
  }
  return chunks;
}
