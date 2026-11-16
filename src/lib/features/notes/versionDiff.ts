// Compares two versions of a note by its blocks of text (F13-03): one block per paragraph,
// heading, list item or table cell, which is as fine as a person reads a change.

interface DocNode {
  type?: string;
  text?: string;
  content?: DocNode[];
}

const LEAF_BLOCKS = new Set([
  "paragraph",
  "heading",
  "codeBlock",
  "tableCell",
  "tableHeader",
]);

function textOf(node: DocNode): string {
  if (node.type === "text") return node.text ?? "";
  if (node.type === "hardBreak") return " ";
  return (node.content ?? []).map(textOf).join("");
}

/** The non-empty blocks of text of a document, in order. */
export function blocksOf(contentJson: string): string[] {
  if (!contentJson.trim().startsWith("{")) return [];
  let root: DocNode;
  try {
    root = JSON.parse(contentJson) as DocNode;
  } catch {
    return [];
  }
  const blocks: string[] = [];
  const walk = (node: DocNode) => {
    if (node.type && LEAF_BLOCKS.has(node.type)) {
      const text = textOf(node).trim();
      if (text !== "") blocks.push(text);
      return;
    }
    for (const child of node.content ?? []) walk(child);
  };
  walk(root);
  return blocks;
}

export type DiffKind = "same" | "added" | "removed";

export interface DiffLine {
  kind: DiffKind;
  text: string;
}

/**
 * Longest-common-subsequence diff of `before` against `after`. Notes are small (a few
 * hundred blocks at most), so the quadratic table is fine.
 */
export function diffBlocks(before: string[], after: string[]): DiffLine[] {
  const rows = before.length;
  const cols = after.length;
  const table: number[][] = Array.from({ length: rows + 1 }, () =>
    new Array<number>(cols + 1).fill(0),
  );
  for (let i = rows - 1; i >= 0; i -= 1) {
    for (let j = cols - 1; j >= 0; j -= 1) {
      table[i]![j] =
        before[i] === after[j]
          ? table[i + 1]![j + 1]! + 1
          : Math.max(table[i + 1]![j]!, table[i]![j + 1]!);
    }
  }
  const lines: DiffLine[] = [];
  let i = 0;
  let j = 0;
  while (i < rows && j < cols) {
    if (before[i] === after[j]) {
      lines.push({ kind: "same", text: before[i]! });
      i += 1;
      j += 1;
    } else if (table[i + 1]![j]! >= table[i]![j + 1]!) {
      lines.push({ kind: "removed", text: before[i]! });
      i += 1;
    } else {
      lines.push({ kind: "added", text: after[j]! });
      j += 1;
    }
  }
  for (; i < rows; i += 1) lines.push({ kind: "removed", text: before[i]! });
  for (; j < cols; j += 1) lines.push({ kind: "added", text: after[j]! });
  return lines;
}
