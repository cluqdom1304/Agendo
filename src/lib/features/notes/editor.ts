// The TipTap editor, kept apart from the Svelte components so it is loaded with `import()` only
// when the first note opens (D-10). Nothing outside `NoteEditor.svelte` may import this file
// statically, or TipTap would end up in the initial bundle.
import { Editor } from "@tiptap/core";
import { TextSelection } from "@tiptap/pm/state";
import { Bold } from "@tiptap/extension-bold";
import { Code } from "@tiptap/extension-code";
import { Document } from "@tiptap/extension-document";
import { HardBreak } from "@tiptap/extension-hard-break";
import { Italic } from "@tiptap/extension-italic";
import { Strike } from "@tiptap/extension-strike";
import { Underline } from "@tiptap/extension-underline";
import { Blockquote } from "@tiptap/extension-blockquote";
import { CodeBlock } from "@tiptap/extension-code-block";
import { Heading } from "@tiptap/extension-heading";
import { Highlight } from "@tiptap/extension-highlight";
import { HorizontalRule } from "@tiptap/extension-horizontal-rule";
import { Link } from "@tiptap/extension-link";
import {
  BulletList,
  ListItem,
  OrderedList,
  TaskItem,
  TaskList,
} from "@tiptap/extension-list";
import { Paragraph } from "@tiptap/extension-paragraph";
import { Text } from "@tiptap/extension-text";
import {
  Table,
  TableCell,
  TableHeader,
  TableRow,
} from "@tiptap/extension-table";
import { Placeholder, UndoRedo } from "@tiptap/extensions";

import { sanitizeLink } from "../../links";
import { editorActions } from "../context/editorActions";
import { AttachmentImage, type ImageHost } from "./attachmentImage";
import { CodeHighlight } from "./codeHighlight";
import { imageFiles } from "./imageCache";
import { findLinkQuery } from "./linkQuery";
import { NoteSearch, searchStateOf, searchTransaction } from "./noteSearch";
import { NoteLink, type NoteLinkHost } from "./noteLink";
import { findTagQuery } from "./tagQuery";
import { TaskRef, type TaskRefHost } from "./taskRef";

export type { ImageHost } from "./attachmentImage";
export type { NoteLinkHost } from "./noteLink";
export type { TaskRefHost, TaskRefInfo } from "./taskRef";

export const EMPTY_NOTE = '{"type":"doc","content":[]}';

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

function parseContent(contentJson: string): object | undefined {
  if (!contentJson.trim().startsWith("{")) return undefined;
  try {
    return JSON.parse(contentJson) as object;
  } catch {
    return undefined;
  }
}

function toolbarState(editor: Editor): ToolbarState {
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

interface QueryRange {
  query: string;
  from: number;
  to: number;
}

function queryAtCursor(editor: Editor): QueryRange | null {
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

function linkQueryAtCursor(editor: Editor): QueryRange | null {
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

function checklists(
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

export function createNoteEditor(options: NoteEditorOptions): NoteEditorHandle {
  let open: QueryRange | null = null;
  let openLink: QueryRange | null = null;
  const taskRefs = new Set<() => void>();
  const noteLinks = new Set<() => void>();
  let lastQuery = "";
  const findState = (): FindState => {
    const found = searchStateOf(editor.state);
    return { count: found.count, current: found.current };
  };

  // Defined before the editor exists, used only once it does.
  async function insertImages(files: File[], at?: number): Promise<void> {
    const host = options.images;
    if (!host) return;
    const stored: { id: string; mime: string }[] = [];
    for (const file of files) {
      const added = await host.add(file);
      if (added) stored.push(added);
    }
    if (stored.length === 0) return;
    const content = stored.map(({ id, mime }) => ({
      type: "image",
      attrs: { attachmentId: id, mime },
    }));
    const chain = editor.chain().focus();
    if (at === undefined) chain.insertContent(content).run();
    else chain.insertContentAt(at, content).run();
  }

  const editor = new Editor({
    element: options.element,
    content: parseContent(options.contentJson),
    editable: options.editable ?? true,
    extensions: [
      Document,
      Paragraph,
      Text,
      HardBreak,
      Bold,
      Italic,
      Underline,
      Strike,
      Code,
      BulletList,
      OrderedList,
      ListItem,
      TaskList,
      TaskItem.configure({ nested: false }),
      UndoRedo,
      ...(options.linkHost
        ? [NoteLink.configure({ host: options.linkHost, registry: noteLinks })]
        : []),
      ...(options.advanced
        ? [
            Heading.configure({ levels: [1, 2, 3] }),
            Blockquote,
            HorizontalRule,
            Highlight,
            CodeBlock,
            CodeHighlight,
            Table.configure({ resizable: false }),
            TableRow,
            TableHeader,
            TableCell,
            NoteSearch,
          ]
        : []),
      ...(options.images
        ? [AttachmentImage.configure({ host: options.images })]
        : []),
      ...(options.taskHost
        ? [TaskRef.configure({ host: options.taskHost, registry: taskRefs })]
        : []),
      Placeholder.configure({ placeholder: options.placeholder }),
      Link.configure({
        openOnClick: false,
        autolink: false,
        linkOnPaste: true,
        defaultProtocol: "https",
        isAllowedUri: (url) => sanitizeLink(url) !== null,
      }),
    ],
    editorProps: {
      attributes: {
        role: "textbox",
        "aria-multiline": "true",
        "aria-label": options.label,
      },
      handlePaste: (_view, event) => {
        const files = imageFiles(event.clipboardData?.files);
        if (!options.images || files.length === 0) return false;
        event.preventDefault();
        void insertImages(files);
        return true;
      },
      handleDrop: (view, event, _slice, moved) => {
        const files = imageFiles(event.dataTransfer?.files);
        if (moved || !options.images || files.length === 0) return false;
        event.preventDefault();
        const at = view.posAtCoords({
          left: event.clientX,
          top: event.clientY,
        });
        void insertImages(files, at?.pos);
        return true;
      },
      handleTextInput: (view, from, _to, text) => {
        if (!options.onTaskRequest || text !== " ") return false;
        const before = view.state.doc
          .resolve(from)
          .parent.textBetween(
            0,
            view.state.doc.resolve(from).parentOffset,
            undefined,
            "\ufffc",
          );
        const typed = /(?:^|\s)(\/tarea)$/i.exec(before);
        if (!typed?.[1]) return false;
        view.dispatch(view.state.tr.delete(from - typed[1].length, from));
        options.onTaskRequest("");
        return true;
      },
      handleKeyDown: (_view, event) => {
        if (
          options.onTaskRequest &&
          (event.ctrlKey || event.metaKey) &&
          event.shiftKey &&
          event.key.toLowerCase() === "t"
        ) {
          event.preventDefault();
          const { from, to } = editor.state.selection;
          options.onTaskRequest(
            editor.state.doc.textBetween(from, to, " ").trim(),
          );
          return true;
        }
        if (openLink && options.onLinkKey?.(event)) {
          event.preventDefault();
          return true;
        }
        if (open && options.onTagKey?.(event)) {
          event.preventDefault();
          return true;
        }
        if (
          (event.ctrlKey || event.metaKey) &&
          event.key.toLowerCase() === "l"
        ) {
          event.preventDefault();
          options.onLinkShortcut();
          return true;
        }
        return false;
      },
      handleDOMEvents: {
        click: (_view, event) => {
          const anchor =
            event.target instanceof Element ? event.target.closest("a") : null;
          if (!anchor) return false;
          // The webview must never navigate by itself; Ctrl+click opens the system browser.
          event.preventDefault();
          const href = sanitizeLink(anchor.getAttribute("href") ?? "");
          if (href && (event.ctrlKey || event.metaKey)) {
            options.onOpenLink(href);
          }
          return true;
        },
      },
    },
    onUpdate: ({ editor: current }) =>
      options.onChange(
        current.isEmpty ? EMPTY_NOTE : JSON.stringify(current.getJSON()),
      ),
    onTransaction: ({ editor: current }) => {
      options.onState(toolbarState(current));
      // A `[[` takes precedence over a `#` typed inside it.
      const link = options.onLinkQuery ? linkQueryAtCursor(current) : null;
      if (options.onLinkQuery) {
        if (link?.query !== openLink?.query || link?.from !== openLink?.from) {
          openLink = link;
          if (!link) {
            options.onLinkQuery(null);
          } else {
            const at = current.view.coordsAtPos(link.to);
            options.onLinkQuery({
              query: link.query,
              left: at.left,
              top: at.top,
              bottom: at.bottom,
            });
          }
        }
      }
      if (!options.onTagQuery) return;
      const next = link ? null : queryAtCursor(current);
      if (next?.query === open?.query && next?.from === open?.from) return;
      open = next;
      if (!next) {
        options.onTagQuery(null);
        return;
      }
      const at = current.view.coordsAtPos(next.to);
      options.onTagQuery({
        query: next.query,
        left: at.left,
        top: at.top,
        bottom: at.bottom,
      });
    },
  });

  const handle: NoteEditorHandle = {
    state: () => toolbarState(editor),
    insertTaskRef(taskId) {
      editor
        .chain()
        .focus()
        .insertContent([
          { type: "taskRef", attrs: { taskId } },
          { type: "paragraph" },
        ])
        .run();
    },
    insertImages,
    refreshTaskRefs() {
      for (const redraw of taskRefs) redraw();
    },
    refreshNoteLinks() {
      for (const redraw of noteLinks) redraw();
    },
    insertNoteLink(noteId) {
      const range = linkQueryAtCursor(editor);
      const chain = editor.chain().focus();
      if (range) chain.deleteRange({ from: range.from, to: range.to });
      chain
        .insertContent([
          { type: "noteLink", attrs: { noteId } },
          { type: "text", text: " " },
        ])
        .run();
    },
    removeLinkQuery() {
      const range = linkQueryAtCursor(editor);
      if (!range) return;
      editor
        .chain()
        .focus()
        .deleteRange({ from: range.from, to: range.to })
        .run();
    },
    selectedText() {
      const { from, to } = editor.state.selection;
      return editor.state.doc.textBetween(from, to, " ").trim();
    },
    checklistItems: () => checklists(editor).flatMap((list) => list.items),
    async convertChecklists(create) {
      const before = checklists(editor);
      const items = before.flatMap((list) => list.items);
      if (items.length === 0) return 0;
      const ids = await create(items);
      if (!ids || ids.length !== items.length) return 0;
      // The note may have been edited while the tasks were being made: only replace what is
      // still exactly the lists that were read.
      const now = checklists(editor);
      const same =
        now.length === before.length &&
        now.every(
          (list, index) =>
            list.items.length === before[index]?.items.length &&
            list.items.every(
              (item, at) =>
                item.text === before[index]?.items[at]?.text &&
                item.checked === before[index]?.items[at]?.checked,
            ),
        );
      if (!same) return 0;
      const schema = editor.state.schema;
      const refType = schema.nodes.taskRef;
      if (!refType) return 0;
      const tr = editor.state.tr;
      let used = ids.length;
      for (const list of [...now].reverse()) {
        const slice = ids.slice(used - list.items.length, used);
        used -= list.items.length;
        tr.replaceWith(
          list.pos,
          list.pos + list.size,
          slice.map((taskId) => refType.create({ taskId })),
        );
      }
      editor.view.dispatch(tr);
      return items.length;
    },
    removeTagQuery() {
      const range = queryAtCursor(editor);
      if (!range) return;
      editor
        .chain()
        .focus()
        .deleteRange({ from: range.from, to: range.to })
        .run();
    },
    toggle(name) {
      const chain = editor.chain().focus();
      switch (name) {
        case "bold":
          chain.toggleBold().run();
          break;
        case "italic":
          chain.toggleItalic().run();
          break;
        case "underline":
          chain.toggleUnderline().run();
          break;
        case "strike":
          chain.toggleStrike().run();
          break;
        case "code":
          chain.toggleCode().run();
          break;
        case "bulletList":
          chain.toggleBulletList().run();
          break;
        case "orderedList":
          chain.toggleOrderedList().run();
          break;
        case "taskList":
          chain.toggleTaskList().run();
          break;
        case "heading": {
          // One button cycles the levels: none, 1, 2, 3, none.
          const level = editor.isActive("heading")
            ? (editor.getAttributes("heading").level as number)
            : 0;
          if (level >= 3) chain.setParagraph().run();
          else chain.setHeading({ level: (level + 1) as 1 | 2 | 3 }).run();
          break;
        }
        case "blockquote":
          chain.toggleBlockquote().run();
          break;
        case "codeBlock":
          chain.toggleCodeBlock().run();
          break;
        case "highlight":
          chain.toggleHighlight().run();
          break;
      }
    },
    setLink(href) {
      if (href === null || href.trim() === "") {
        editor.chain().focus().extendMarkRange("link").unsetLink().run();
        return true;
      }
      const safe = sanitizeLink(href);
      if (!safe) return false;
      editor
        .chain()
        .focus()
        .extendMarkRange("link")
        .setLink({ href: safe })
        .run();
      return true;
    },
    insertTable: () =>
      void editor
        .chain()
        .focus()
        .insertTable({ rows: 3, cols: 3, withHeaderRow: true })
        .run(),
    insertRule: () => void editor.chain().focus().setHorizontalRule().run(),
    tableAction(action) {
      const chain = editor.chain().focus();
      switch (action) {
        case "addRowBefore":
          chain.addRowBefore().run();
          break;
        case "addRowAfter":
          chain.addRowAfter().run();
          break;
        case "addColumnBefore":
          chain.addColumnBefore().run();
          break;
        case "addColumnAfter":
          chain.addColumnAfter().run();
          break;
        case "deleteRow":
          chain.deleteRow().run();
          break;
        case "deleteColumn":
          chain.deleteColumn().run();
          break;
        case "deleteTable":
          chain.deleteTable().run();
          break;
        case "toggleHeaderRow":
          chain.toggleHeaderRow().run();
          break;
      }
    },
    find(query) {
      lastQuery = query;
      editor.view.dispatch(searchTransaction(editor.state, query, 0));
      return findState();
    },
    findStep(direction) {
      const now = searchStateOf(editor.state);
      if (now.count === 0) return { count: 0, current: -1 };
      const next = (now.current + direction + now.count) % now.count;
      editor.view.dispatch(searchTransaction(editor.state, lastQuery, next));
      const match = searchStateOf(editor.state).match;
      if (match) {
        // The field keeps the keyboard: only the selection and the scroll move.
        editor.view.dispatch(
          editor.state.tr
            .setSelection(
              TextSelection.create(editor.state.doc, match.from, match.to),
            )
            .scrollIntoView(),
        );
      }
      return findState();
    },
    clearFind() {
      lastQuery = "";
      editor.view.dispatch(searchTransaction(editor.state, "", 0));
    },
    undo: () => void editor.chain().focus().undo().run(),
    redo: () => void editor.chain().focus().redo().run(),
    focus: (position = "end") => void editor.commands.focus(position),
    isEmpty: () => editor.isEmpty,
    destroy: () => editor.destroy(),
  };
  editorActions.set(editor.view.dom, handle);
  return handle;
}
