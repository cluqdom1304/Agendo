// The TipTap editor, kept apart from the Svelte components so it is loaded with `import()` only
// when the first note opens (D-10). Nothing outside `NoteEditor.svelte` may import this file
// statically, or TipTap would end up in the initial bundle.
import { Editor } from "@tiptap/core";
import { Bold } from "@tiptap/extension-bold";
import { Code } from "@tiptap/extension-code";
import { Document } from "@tiptap/extension-document";
import { HardBreak } from "@tiptap/extension-hard-break";
import { Italic } from "@tiptap/extension-italic";
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
import { Placeholder, UndoRedo } from "@tiptap/extensions";

import { sanitizeLink } from "../../links";
import { findTagQuery } from "./tagQuery";

export const EMPTY_NOTE = '{"type":"doc","content":[]}';

export type ToggleName =
  "bold" | "italic" | "code" | "bulletList" | "orderedList" | "taskList";

/** What the toolbar draws; recomputed on every transaction. */
export interface ToolbarState {
  bold: boolean;
  italic: boolean;
  code: boolean;
  bulletList: boolean;
  orderedList: boolean;
  taskList: boolean;
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
  /** Sets the link on the selection, or removes it with `null`. False if the address is refused. */
  setLink(href: string | null): boolean;
  undo(): void;
  redo(): void;
  focus(): void;
  isEmpty(): boolean;
  destroy(): void;
}

export interface NoteEditorOptions {
  element: HTMLElement;
  contentJson: string;
  label: string;
  /** Shown while the note has no text. */
  placeholder: string;
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
    code: editor.isActive("code"),
    bulletList: editor.isActive("bulletList"),
    orderedList: editor.isActive("orderedList"),
    taskList: editor.isActive("taskList"),
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

export function createNoteEditor(options: NoteEditorOptions): NoteEditorHandle {
  let open: QueryRange | null = null;
  const editor = new Editor({
    element: options.element,
    content: parseContent(options.contentJson),
    extensions: [
      Document,
      Paragraph,
      Text,
      HardBreak,
      Bold,
      Italic,
      Code,
      BulletList,
      OrderedList,
      ListItem,
      TaskList,
      TaskItem.configure({ nested: false }),
      UndoRedo,
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
      handleKeyDown: (_view, event) => {
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
      if (!options.onTagQuery) return;
      const next = queryAtCursor(current);
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

  return {
    state: () => toolbarState(editor),
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
    undo: () => void editor.chain().focus().undo().run(),
    redo: () => void editor.chain().focus().redo().run(),
    focus: () => void editor.commands.focus("end"),
    isEmpty: () => editor.isEmpty,
    destroy: () => editor.destroy(),
  };
}
