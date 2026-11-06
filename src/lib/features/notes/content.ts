/** Whether a notes document holds any text; an empty or missing document does not. */
export function hasNotes(contentJson: string): boolean {
  return /"text"\s*:\s*"[^"]/.test(contentJson);
}
