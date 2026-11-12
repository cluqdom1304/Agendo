// The `taskRef` node of a note (D-45): a line that stands for a real task. The document only
// stores the task id (`attrs.taskId`); title, date and tags are read from the task itself each
// time, so editing, completing or deleting the task is seen by every note that points at it.
// Loaded together with the editor (see `editor.ts`).
import { mergeAttributes, Node } from "@tiptap/core";

import { t } from "../../i18n";

/** What a reference shows about its task. */
export interface TaskRefInfo {
  title: string;
  isCompleted: boolean;
  /** "12/03/2026 · 10:30", already formatted; `null` for a task without a date. */
  due: string | null;
  tags: { name: string; color: string }[];
}

/** What the editor needs from the app to draw and drive its references. */
export interface TaskRefHost {
  /** The task as it is now; `null` when it is in the trash or gone. */
  lookup(id: string): TaskRefInfo | null;
  toggle(id: string, completed: boolean): void;
  /** Opens the task's details. */
  open(id: string): void;
  /** Takes the task out of the trash; resolves `false` when it can no longer be restored. */
  restore(id: string): Promise<boolean>;
}

export interface TaskRefOptions {
  host: TaskRefHost | null;
  /** Redraw functions of the references on screen, so `refreshTaskRefs` can reach them. */
  registry: Set<() => void>;
}

function element<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  className: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const created = document.createElement(tag);
  created.className = className;
  if (text !== undefined) created.textContent = text;
  return created;
}

function button(className: string, text?: string): HTMLButtonElement {
  const created = element("button", className, text);
  created.type = "button";
  return created;
}

export const TaskRef = Node.create<TaskRefOptions>({
  name: "taskRef",
  group: "block",
  atom: true,
  selectable: true,
  draggable: true,

  addOptions() {
    return { host: null, registry: new Set() };
  },

  addAttributes() {
    return {
      taskId: {
        default: null,
        parseHTML: (dom) => dom.getAttribute("data-task-id"),
        renderHTML: (attributes) => ({
          "data-task-id": attributes.taskId as string | null,
        }),
      },
    };
  },

  parseHTML() {
    return [{ tag: "div[data-task-ref]" }];
  },

  renderHTML({ HTMLAttributes }) {
    return ["div", mergeAttributes(HTMLAttributes, { "data-task-ref": "" })];
  },

  addNodeView() {
    const { host, registry } = this.options;
    return ({ node }) => {
      const id = String(node.attrs.taskId ?? "");
      const dom = element("div", "task-ref");
      dom.contentEditable = "false";
      dom.setAttribute("data-task-ref", "");

      const draw = () => {
        dom.replaceChildren();
        const info = host?.lookup(id) ?? null;
        dom.classList.toggle("is-gone", info === null);
        dom.classList.toggle("is-done", info?.isCompleted === true);
        if (!info) {
          dom.append(element("span", "tr-mark gone"));
          dom.append(element("span", "tr-title gone", t.notesApp.taskGone));
          const restore = button("tr-restore", t.notesApp.taskRestore);
          restore.addEventListener("click", () => {
            void host?.restore(id).then((restored) => {
              if (!restored) restore.textContent = t.notesApp.taskGoneForGood;
            });
          });
          dom.append(restore);
          return;
        }
        const check = button("tr-check");
        check.setAttribute("role", "checkbox");
        check.setAttribute("aria-checked", String(info.isCompleted));
        check.setAttribute(
          "aria-label",
          info.isCompleted
            ? t.notesApp.taskReopen(info.title)
            : t.notesApp.taskComplete(info.title),
        );
        check.addEventListener("click", () =>
          host?.toggle(id, !info.isCompleted),
        );
        dom.append(check);

        const open = button("tr-title", info.title || t.notesApp.untitled);
        open.title = t.notesApp.taskOpen;
        open.addEventListener("click", () => host?.open(id));
        dom.append(open);

        if (info.due) dom.append(element("span", "tr-due", info.due));
        for (const tag of info.tags) {
          const chip = element("span", "tr-tag", tag.name);
          chip.style.setProperty("--tag", tag.color);
          dom.append(chip);
        }
      };

      draw();
      registry.add(draw);
      return {
        dom,
        // The editor must not treat clicks and DOM changes in the reference as typing.
        stopEvent: () => true,
        ignoreMutation: () => true,
        destroy: () => {
          registry.delete(draw);
        },
      };
    };
  },
});
