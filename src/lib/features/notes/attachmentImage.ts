// The `image` node of a note (D-47): a picture stored as an attachment. The document only
// keeps the attachment id, its type and the width the user gave it; the bytes are read from
// the store when the image is drawn. Loaded together with the editor.
import { mergeAttributes, Node } from "@tiptap/core";

import { t } from "../../i18n";
import { nodeRemovers } from "../context/nodeRemovers";

/** What an image needs from the app to be drawn, previewed and created. */
export interface ImageHost {
  /** Address the image can be shown from; `null` when its file is gone. */
  url(id: string, mime: string): Promise<string | null>;
  /** Stores a dropped or pasted file; `null` when it is refused. */
  add(file: File): Promise<{ id: string; mime: string } | null>;
  /** Shows the image large. */
  preview(id: string, mime: string): void;
}

export interface ImageOptions {
  host: ImageHost | null;
}

/** The narrowest an image may be made, in pixels. */
export const MIN_IMAGE_WIDTH = 48;

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    image: {
      setImage: (attributes: {
        attachmentId: string;
        mime: string;
      }) => ReturnType;
    };
  }
}

export const AttachmentImage = Node.create<ImageOptions>({
  name: "image",
  group: "block",
  atom: true,
  draggable: true,

  addOptions() {
    return { host: null };
  },

  addAttributes() {
    return {
      attachmentId: {
        default: null,
        parseHTML: (dom) => dom.getAttribute("data-attachment-id"),
        renderHTML: (attributes) => ({
          "data-attachment-id": attributes.attachmentId as string | null,
        }),
      },
      mime: {
        default: "image/png",
        parseHTML: (dom) => dom.getAttribute("data-mime") ?? "image/png",
        renderHTML: (attributes) => ({
          "data-mime": attributes.mime as string,
        }),
      },
      width: {
        default: null,
        parseHTML: (dom) => {
          const value = Number(dom.getAttribute("data-width"));
          return Number.isFinite(value) && value > 0 ? value : null;
        },
        renderHTML: (attributes) =>
          attributes.width ? { "data-width": String(attributes.width) } : {},
      },
    };
  },

  parseHTML() {
    return [{ tag: "figure[data-attachment-id]" }];
  },

  renderHTML({ HTMLAttributes }) {
    return ["figure", mergeAttributes(HTMLAttributes)];
  },

  addCommands() {
    return {
      setImage:
        (attributes) =>
        ({ commands }) =>
          commands.insertContent({ type: this.name, attrs: attributes }),
    };
  },

  addNodeView() {
    const { host } = this.options;
    return ({ node, getPos, editor }) => {
      const id = String(node.attrs.attachmentId ?? "");
      const mime = String(node.attrs.mime ?? "image/png");
      const dom = document.createElement("figure");
      dom.className = "note-image";
      dom.contentEditable = "false";
      dom.setAttribute("data-attachment-id", id);

      const picture = document.createElement("img");
      picture.alt = "";
      picture.draggable = false;
      picture.className = "is-loading";
      picture.title = t.notesApp.imageOpen;
      dom.append(picture);

      const width = Number(node.attrs.width) || 0;
      if (width > 0) picture.style.width = `${width}px`;

      let gone = false;
      void host?.url(id, mime).then((url) => {
        if (gone) return;
        picture.classList.remove("is-loading");
        if (url) {
          picture.src = url;
        } else {
          dom.classList.add("is-gone");
          picture.removeAttribute("title");
          dom.append(
            Object.assign(document.createElement("span"), {
              className: "image-gone",
              textContent: t.notesApp.imageGone,
            }),
          );
        }
      });

      picture.addEventListener("click", () => host?.preview(id, mime));

      // Drag the corner to resize; the width is kept in the document.
      const grip = document.createElement("span");
      grip.className = "image-grip";
      grip.setAttribute("role", "separator");
      grip.setAttribute("aria-label", t.notesApp.imageResize);
      grip.tabIndex = 0;
      const apply = (value: number) => {
        const at = getPos();
        if (typeof at !== "number") return;
        editor
          .chain()
          .command(({ tr }) => {
            tr.setNodeMarkup(at, undefined, { ...node.attrs, width: value });
            return true;
          })
          .run();
      };
      grip.addEventListener("pointerdown", (event) => {
        event.preventDefault();
        event.stopPropagation();
        const startX = event.clientX;
        const startWidth = picture.getBoundingClientRect().width;
        const maxWidth = dom.parentElement?.clientWidth ?? startWidth;
        const clamp = (value: number) =>
          Math.round(Math.min(Math.max(value, MIN_IMAGE_WIDTH), maxWidth));
        grip.setPointerCapture?.(event.pointerId);
        const move = (next: PointerEvent) => {
          picture.style.width = `${clamp(startWidth + next.clientX - startX)}px`;
        };
        const up = (next: PointerEvent) => {
          grip.removeEventListener("pointermove", move);
          grip.removeEventListener("pointerup", up);
          grip.removeEventListener("pointercancel", up);
          apply(clamp(startWidth + next.clientX - startX));
        };
        grip.addEventListener("pointermove", move);
        grip.addEventListener("pointerup", up);
        grip.addEventListener("pointercancel", up);
      });
      // The keyboard does the same in steps of 40 px.
      grip.addEventListener("keydown", (event) => {
        if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
        event.preventDefault();
        const now = picture.getBoundingClientRect().width;
        const maxWidth = dom.parentElement?.clientWidth ?? now;
        const next = Math.round(
          Math.min(
            Math.max(
              now + (event.key === "ArrowRight" ? 40 : -40),
              MIN_IMAGE_WIDTH,
            ),
            maxWidth,
          ),
        );
        picture.style.width = `${next}px`;
        apply(next);
      });
      dom.append(grip);

      nodeRemovers.set(dom, () => {
        const at = getPos();
        if (typeof at !== "number") return;
        editor
          .chain()
          .focus()
          .deleteRange({ from: at, to: at + node.nodeSize })
          .run();
      });

      return {
        dom,
        stopEvent: (event) =>
          event.target === grip ||
          (event.target as Element)?.closest?.(".image-grip") !== null,
        ignoreMutation: () => true,
        destroy: () => {
          gone = true;
        },
      };
    };
  },
});
