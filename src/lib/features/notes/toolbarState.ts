import type { ToolbarState } from "./editor";

/** What the format bar shows before the editor has loaded: nothing on, nothing to undo. */
export const IDLE_TOOLBAR: ToolbarState = {
  bold: false,
  italic: false,
  underline: false,
  strike: false,
  code: false,
  bulletList: false,
  orderedList: false,
  taskList: false,
  heading: 0,
  blockquote: false,
  codeBlock: false,
  highlight: false,
  link: false,
  linkHref: "",
  canUndo: false,
  canRedo: false,
};
