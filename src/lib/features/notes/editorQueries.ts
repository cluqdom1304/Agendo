// What the editor reads from the document under the cursor: toolbar state, the `#` and `[[`
// being typed, and checklists.

import type { Editor } from "@tiptap/core";

import type { ChecklistItem, ToolbarState } from "./editorTypes";
import { findLinkQuery } from "./linkQuery";
import { findTagQuery } from "./tagQuery";

export function toolbarState(editor: Editor): ToolbarState {
  return {
    bold: editor.isActive("bold"),
    italic: editor.isActive("italic"),
    underline: editor.isActive("underline"),
    strike: editor.isActive("strike"),
    code: editor.isActive("code"),
    bulletList: editor.isActive("bulletList"),
    orderedList: editor.isActive("orderedList"),
    taskList: editor.isActive("taskList"),
    heading: editor.isActive("heading")
      ? ((editor.getAttributes("heading").level as number | undefined) ?? 0)
      : 0,
    blockquote: editor.isActive("blockquote"),
    codeBlock: editor.isActive("codeBlock"),
    highlight: editor.isActive("highlight"),
    link: editor.isActive("link"),
    linkHref: (editor.getAttributes("link").href as string | undefined) ?? "",
    canUndo: editor.can().undo(),
    canRedo: editor.can().redo(),
  };
}

export interface QueryRange {
  query: string;
  from: number;
  to: number;
}

export function queryAtCursor(editor: Editor): QueryRange | null {
  const { selection } = editor.state;
  if (!selection.empty) return null;
  const { $from } = selection;
  if (!$from.parent.isTextblock) return null;
  const before = $from.parent.textBetween(
    0,
    $from.parentOffset,
    undefined,
    "\ufffc",
  );
  const query = findTagQuery(before);
  if (query === null) return null;
  return { query, from: selection.from - query.length - 1, to: selection.from };
}

export function linkQueryAtCursor(editor: Editor): QueryRange | null {
  const { selection } = editor.state;
  if (!selection.empty) return null;
  const { $from } = selection;
  if (!$from.parent.isTextblock) return null;
  const before = $from.parent.textBetween(
    0,
    $from.parentOffset,
    undefined,
    "￼",
  );
  const query = findLinkQuery(before);
  if (query === null) return null;
  return { query, from: selection.from - query.length - 2, to: selection.from };
}

export function checklists(
  editor: Editor,
): { pos: number; size: number; items: ChecklistItem[] }[] {
  const found: { pos: number; size: number; items: ChecklistItem[] }[] = [];
  editor.state.doc.descendants((node, pos) => {
    if (node.type.name !== "taskList") return true;
    const items: ChecklistItem[] = [];
    node.forEach((item) =>
      items.push({
        text: item.textContent.trim(),
        checked: item.attrs.checked === true,
      }),
    );
    found.push({ pos, size: node.nodeSize, items });
    return false;
  });
  return found;
}
