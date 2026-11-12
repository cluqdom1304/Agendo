// The `noteLink` node of a note (D-46): a link to another note. The document only stores the
// note id (`attrs.noteId`); the title shown is read from the note each time, so renaming a note
// never breaks a link or leaves an old name behind. Loaded together with the editor.
import { mergeAttributes, Node } from "@tiptap/core";

import { t } from "../../i18n";

/** What a link needs from the app to draw itself and open its note. */
export interface NoteLinkHost {
  /**
   * The note's title now; `undefined` while it is still being looked up and `null` when the
   * note is in the trash or gone.
   */
  lookup(id: string): string | null | undefined;
  open(id: string): void;
}

export interface NoteLinkOptions {
  host: NoteLinkHost | null;
  /** Redraw functions of the links on screen, so `refreshNoteLinks` can reach them. */
  registry: Set<() => void>;
}

export const NoteLink = Node.create<NoteLinkOptions>({
  name: "noteLink",
  group: "inline",
  inline: true,
  atom: true,
  selectable: true,

  addOptions() {
    return { host: null, registry: new Set() };
  },

  addAttributes() {
    return {
      noteId: {
        default: null,
        parseHTML: (dom) => dom.getAttribute("data-note-id"),
        renderHTML: (attributes) => ({
          "data-note-id": attributes.noteId as string | null,
        }),
      },
    };
  },

  parseHTML() {
    return [{ tag: "span[data-note-link]" }];
  },

  renderHTML({ HTMLAttributes }) {
    return ["span", mergeAttributes(HTMLAttributes, { "data-note-link": "" })];
  },

  addNodeView() {
    const { host, registry } = this.options;
    return ({ node }) => {
      const id = String(node.attrs.noteId ?? "");
      const dom = document.createElement("span");
      dom.className = "note-link";
      dom.contentEditable = "false";
      dom.setAttribute("data-note-link", "");
      dom.setAttribute("role", "link");
      dom.tabIndex = 0;

      const draw = () => {
        const title = host?.lookup(id);
        dom.classList.toggle("is-gone", title === null);
        dom.classList.toggle("is-loading", title === undefined);
        if (title === null) {
          dom.textContent = t.notesApp.linkGone;
          dom.title = t.notesApp.linkGoneHint;
        } else if (title === undefined) {
          dom.textContent = "…";
          dom.removeAttribute("title");
        } else {
          dom.textContent = title || t.notesApp.untitled;
          dom.title = t.notesApp.linkOpen(title || t.notesApp.untitled);
        }
      };
      const open = () => {
        if (host?.lookup(id) !== null) host?.open(id);
      };
      dom.addEventListener("click", open);
      dom.addEventListener("keydown", (event) => {
        if (event.key === "Enter") {
          event.preventDefault();
          open();
        }
      });

      draw();
      registry.add(draw);
      return {
        dom,
        stopEvent: () => true,
        ignoreMutation: () => true,
        destroy: () => {
          registry.delete(draw);
        },
      };
    };
  },
});
