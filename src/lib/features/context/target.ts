// Which element the user pointed at (D-50). Components only carry `data-*` attributes; this
// module reads them, so no component needs to know about the menu.

export type MenuTarget =
  /** A text field: the browser keeps its own menu (copy, paste, spelling). */
  | { kind: "field" }
  | { kind: "note-link"; noteId: string; node: Element }
  | { kind: "task-ref"; taskId: string; node: Element }
  | { kind: "editor"; node: Element; inTable: boolean }
  | { kind: "task"; taskId: string }
  | { kind: "note"; noteId: string }
  | { kind: "folder"; folderId: string }
  | { kind: "tag"; tagId: string }
  | { kind: "note-tag"; tagId: string }
  | { kind: "agenda"; date: string | null }
  | { kind: "notes" }
  | { kind: "none" };

const NON_TEXT_INPUTS = new Set([
  "checkbox",
  "radio",
  "button",
  "submit",
  "reset",
  "range",
  "color",
  "file",
]);

function isField(el: Element): boolean {
  const field = el.closest("input, textarea, select");
  if (!field) return false;
  return !(
    field instanceof HTMLInputElement && NON_TEXT_INPUTS.has(field.type)
  );
}

const valueOf = (el: Element, selector: string, attribute: string) =>
  el.closest(selector)?.getAttribute(attribute) ?? null;

export function resolveTarget(el: Element | null): MenuTarget {
  if (!el) return { kind: "none" };
  if (el.closest("[data-context-menu]")) return { kind: "none" };
  if (isField(el)) return { kind: "field" };

  const link = el.closest("[data-note-link]");
  const linked = link?.getAttribute("data-note-id");
  if (link && linked) return { kind: "note-link", noteId: linked, node: link };

  const ref = el.closest("[data-task-ref]");
  const referred = ref?.getAttribute("data-task-id");
  if (ref && referred) return { kind: "task-ref", taskId: referred, node: ref };

  const editor = el.closest(".tiptap, [contenteditable='true']");
  if (editor) {
    return { kind: "editor", node: editor, inTable: !!el.closest("td, th") };
  }

  const task = valueOf(el, "[data-task-id]", "data-task-id");
  if (task) return { kind: "task", taskId: task };
  const note = valueOf(el, "[data-note-id]", "data-note-id");
  if (note) return { kind: "note", noteId: note };
  const folder = valueOf(el, "[data-folder-id]", "data-folder-id");
  if (folder) return { kind: "folder", folderId: folder };
  const tag = valueOf(el, "[data-tag-id]", "data-tag-id");
  if (tag) return { kind: "tag", tagId: tag };
  const noteTag = valueOf(el, "[data-note-tag-id]", "data-note-tag-id");
  if (noteTag) return { kind: "note-tag", tagId: noteTag };

  // Dialogs have their own controls: nothing to add on their background.
  if (el.closest("[role='dialog']")) return { kind: "none" };

  const area = valueOf(el, "[data-menu-area]", "data-menu-area");
  if (area === "agenda") {
    return { kind: "agenda", date: valueOf(el, "[data-date]", "data-date") };
  }
  if (area === "notes") return { kind: "notes" };
  return { kind: "none" };
}
