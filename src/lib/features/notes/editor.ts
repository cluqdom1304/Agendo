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
import { BulletList, ListItem, OrderedList } from "@tiptap/extension-list";
import { Paragraph } from "@tiptap/extension-paragraph";
import { Text } from "@tiptap/extension-text";
import { UndoRedo } from "@tiptap/extensions";

import { sanitizeLink } from "../../links";

export const EMPTY_NOTE = '{"type":"doc","content":[]}';

export type ToggleName =
  "bold" | "italic" | "code" | "bulletList" | "orderedList";

/** What the toolbar draws; recomputed on every transaction. */
export interface ToolbarState {
  bold: boolean;
  italic: boolean;
  code: boolean;
  bulletList: boolean;
  orderedList: boolean;
  link: boolean;
  /** Address of the link under the cursor, for the link field. */
  linkHref: string;
  canUndo: boolean;
  canRedo: boolean;
}

export interface NoteEditorHandle {
  state(): ToolbarState;
  toggle(name: ToggleName): void;
  /** Sets the link on the selection, or removes it with `null`. False if the address is refused. */
  setLink(href: string | null): boolean;
  undo(): void;
  redo(): void;
  focus(): void;
  destroy(): void;
}

export interface NoteEditorOptions {
  element: HTMLElement;
  contentJson: string;
  label: string;
  /** Called with the serialised document after every edit. */
  onChange: (contentJson: string) => void;
  onState: (state: ToolbarState) => void;
  onLinkShortcut: () => void;
  onOpenLink: (href: string) => void;
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
    link: editor.isActive("link"),
    linkHref: (editor.getAttributes("link").href as string | undefined) ?? "",
    canUndo: editor.can().undo(),
    canRedo: editor.can().redo(),
  };
}

export function createNoteEditor(options: NoteEditorOptions): NoteEditorHandle {
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
      UndoRedo,
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
    onTransaction: ({ editor: current }) =>
      options.onState(toolbarState(current)),
  });

  return {
    state: () => toolbarState(editor),
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
    destroy: () => editor.destroy(),
  };
}
