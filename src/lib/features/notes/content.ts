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

interface DocNode {
  type?: string;
  attrs?: Record<string, unknown>;
  content?: DocNode[];
}

/** Ids of the `type` nodes of a document, read from `attribute`, in order and without repeats. */
function referenceIds(
  contentJson: string,
  type: string,
  attribute: string,
): string[] {
  if (!contentJson.trim().startsWith("{")) return [];
  let root: DocNode;
  try {
    root = JSON.parse(contentJson) as DocNode;
  } catch {
    return [];
  }
  const ids: string[] = [];
  const walk = (node: DocNode) => {
    if (node.type === type) {
      const id = node.attrs?.[attribute];
      if (typeof id === "string" && id !== "" && !ids.includes(id)) {
        ids.push(id);
      }
    }
    node.content?.forEach(walk);
  };
  walk(root);
  return ids;
}

/** Ids of the tasks a document points at with `taskRef` nodes, in order and without repeats. */
export function taskRefIds(contentJson: string): string[] {
  return referenceIds(contentJson, "taskRef", "taskId");
}

/** Ids of the notes a document links to with `noteLink` nodes, in order and without repeats. */
export function noteLinkIds(contentJson: string): string[] {
  return referenceIds(contentJson, "noteLink", "noteId");
}

/** A note document made of a single reference to `taskId`. */
export function taskRefDoc(taskId: string): string {
  return JSON.stringify({
    type: "doc",
    content: [{ type: "taskRef", attrs: { taskId } }],
  });
}
