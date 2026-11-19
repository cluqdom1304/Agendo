// Types of the note editor, apart from `editor.ts` so the parts that only need them do not
// pull TipTap in.

import type { ImageHost } from "./attachmentImage";
import type { NoteLinkHost } from "./noteLink";
import type { TaskRefHost } from "./taskRef";

export type ToggleName =
  | "bold"
  | "italic"
  | "underline"
  | "strike"
  | "code"
  | "bulletList"
  | "orderedList"
  | "taskList"
  | "heading"
  | "blockquote"
  | "codeBlock"
  | "highlight";

/** Changes to the table the cursor is in. */
export type TableAction =
  | "addRowBefore"
  | "addRowAfter"
  | "addColumnBefore"
  | "addColumnAfter"
  | "deleteRow"
  | "deleteColumn"
  | "deleteTable"
  | "toggleHeaderRow";

export interface FindState {
  /** How many places match. */
  count: number;
  /** The current one, from 0; -1 when there is none. */
  current: number;
}

/** What the toolbar draws; recomputed on every transaction. */
export interface ToolbarState {
  bold: boolean;
  italic: boolean;
  underline: boolean;
  strike: boolean;
  code: boolean;
  bulletList: boolean;
  orderedList: boolean;
  taskList: boolean;
  /** Level of the heading under the cursor, 0 if it is not in one. */
  heading: number;
  blockquote: boolean;
  codeBlock: boolean;
  highlight: boolean;
  link: boolean;
  /** Address of the link under the cursor, for the link field. */
  linkHref: string;
  canUndo: boolean;
  canRedo: boolean;
}

export interface NoteEditorHandle {
  state(): ToolbarState;
  toggle(name: ToggleName): void;
  /** Deletes the `#text` being typed, once its tag has been chosen. */
  removeTagQuery(): void;
  /** Puts a reference to the task after the cursor. */
  insertTaskRef(taskId: string): void;
  /** Replaces the `[[text` being typed with a link to the note. */
  insertNoteLink(noteId: string): void;
  /** Deletes the `[[text` being typed, to make room for something else. */
  removeLinkQuery(): void;
  /** Puts a table of 3 × 3 cells after the cursor. */
  insertTable(): void;
  insertRule(): void;
  tableAction(action: TableAction): void;
  /** Marks `query` in the text and moves to its first match; empty clears the marks. */
  find(query: string): FindState;
  /** Moves to the next or previous match, wrapping. */
  findStep(direction: 1 | -1): FindState;
  clearFind(): void;
  /** Stores the images and puts them after the cursor (or at `at`, a document position). */
  insertImages(files: File[], at?: number): Promise<void>;
  /** Redraws every note link: the titles they show may have changed. */
  refreshNoteLinks(): void;
  /** Redraws every task reference: the tasks they show have changed. */
  refreshTaskRefs(): void;
  /** The checklist items (D-37) of the document, in order. */
  checklistItems(): ChecklistItem[];
  /**
   * Replaces every checklist with references to the tasks `create` makes for its items, in the
   * same order. Resolves with how many were converted (0 when there were none, or the note was
   * edited meanwhile).
   */
  convertChecklists(
    create: (items: ChecklistItem[]) => Promise<string[] | null>,
  ): Promise<number>;
  /** Text selected in the document; empty when nothing is. */
  selectedText(): string;
  /** Sets the link on the selection, or removes it with `null`. False if the address is refused. */
  setLink(href: string | null): boolean;
  undo(): void;
  redo(): void;
  /** Takes the cursor, at the end of the text unless told otherwise. */
  focus(position?: "start" | "end"): void;
  isEmpty(): boolean;
  destroy(): void;
}

export interface NoteEditorOptions {
  element: HTMLElement;
  contentJson: string;
  label: string;
  /** Shown while the note has no text. */
  placeholder: string;
  /** `false` shows the note without letting it be edited (a note in the trash). */
  editable?: boolean;
  /** Called with the serialised document after every edit. */
  onChange: (contentJson: string) => void;
  onState: (state: ToolbarState) => void;
  onLinkShortcut: () => void;
  onOpenLink: (href: string) => void;
  /**
   * The `#text` being typed at the cursor (without the `#`) and where to draw its suggestions,
   * or `null` when there is none. Only called when it changes.
   */
  onTagQuery?: (query: TagQuery | null) => void;
  /** First look at a key while a `#text` is open; returns true when it was used. */
  onTagKey?: (event: KeyboardEvent) => boolean;
  /** Turns on links between notes: how to read and open the notes they point at. */
  linkHost?: NoteLinkHost;
  /** The `[[text` being typed (without the brackets) and where to draw its suggestions. */
  onLinkQuery?: (query: TagQuery | null) => void;
  /** First look at a key while a `[[text` is open; returns true when it was used. */
  onLinkKey?: (event: KeyboardEvent) => boolean;
  /** Headings, quotes, code blocks, tables, highlight and find: for the body of a note. */
  advanced?: boolean;
  /** Turns on images (D-47): where they are stored and shown. Off for text that is not a note. */
  images?: ImageHost;
  /** Turns on task references: how to read and drive the tasks they point at. */
  taskHost?: TaskRefHost;
  /** Ctrl+Mayús+T or `/tarea` + space: the user wants a task made from here. */
  onTaskRequest?: (prefill: string) => void;
}

export interface ChecklistItem {
  text: string;
  checked: boolean;
}

export interface TagQuery {
  query: string;
  left: number;
  top: number;
  bottom: number;
}
