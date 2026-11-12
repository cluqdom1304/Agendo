// The editors that are open, by their DOM node: the context menu only sees the DOM, and asks
// the editor under the pointer to change its table.
import type { NoteEditorHandle } from "../notes/editor";

export const editorActions = new WeakMap<Element, NoteEditorHandle>();
