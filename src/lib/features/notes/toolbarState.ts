import type { ToolbarState } from "./editor";

/** What the format bar shows before the editor has loaded: nothing on, nothing to undo. */
export const IDLE_TOOLBAR: ToolbarState = {
  bold: false,
  italic: false,
  code: false,
  bulletList: false,
  orderedList: false,
  taskList: false,
  link: false,
  linkHref: "",
  canUndo: false,
  canRedo: false,
};
