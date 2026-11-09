/** Whether a notes document holds any text; an empty or missing document does not. */
export function hasNotes(contentJson: string): boolean {
  return /"text"\s*:\s*"[^"]/.test(contentJson);
}

/** A plain-text description as a notes document: one paragraph per non-empty line. */
export function textToDoc(text: string): string | null {
  const lines = text
    .split(/\r?\n/)
    .map((line) => line.trimEnd())
    .filter((line) => line.trim() !== "");
  if (lines.length === 0) return null;
  return JSON.stringify({
    type: "doc",
    content: lines.map((line) => ({
      type: "paragraph",
      content: [{ type: "text", text: line }],
    })),
  });
}
