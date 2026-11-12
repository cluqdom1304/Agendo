// @vitest-environment jsdom
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

import { TasksStore } from "../../stores/tasks.svelte";
import { FakeApi, installLayoutStubs, makeTask } from "../../testing";
import type { NoteEditorHandle, TaskRefHost, TaskRefInfo } from "./editor";
import NoteTaskDialog from "./NoteTaskDialog.svelte";
import NoteTasksPanel from "./NoteTasksPanel.svelte";

installLayoutStubs();

afterEach(() => vi.useRealTimers());

const ref = (taskId: string) => ({ type: "taskRef", attrs: { taskId } });
const doc = (...content: object[]) => JSON.stringify({ type: "doc", content });

function makeHost(initial: Record<string, TaskRefInfo | null>) {
  const infos = { ...initial };
  const host = {
    infos,
    toggle: vi.fn<(id: string, completed: boolean) => void>(),
    open: vi.fn<(id: string) => void>(),
    restore: vi.fn<(id: string) => Promise<boolean>>(async () => true),
    lookup: (id: string) => infos[id] ?? null,
  };
  return host satisfies TaskRefHost & { infos: typeof infos };
}

async function open(contentJson: string, host: TaskRefHost) {
  const { createNoteEditor } = await import("./editor");
  const element = document.createElement("div");
  document.body.append(element);
  const changes: string[] = [];
  const handle: NoteEditorHandle = createNoteEditor({
    element,
    contentJson,
    label: "nota",
    placeholder: "",
    onChange: (json) => changes.push(json),
    onState: () => {},
    onLinkShortcut: () => {},
    onOpenLink: () => {},
    taskHost: host,
  });
  return { element, handle, changes };
}

const task = (overrides: Partial<TaskRefInfo> = {}): TaskRefInfo => ({
  title: "Entregar el informe",
  isCompleted: false,
  due: "12/03/2026 · 10:30",
  tags: [{ name: "Trabajo", color: "#4C8DFF" }],
  ...overrides,
});

describe("task references in a note", () => {
  it("draws the task live: title, date, tags, and a checkbox that completes it", async () => {
    const host = makeHost({ a: task() });
    const { element, handle } = await open(doc(ref("a")), host);
    expect(element.querySelector(".tr-title")?.textContent).toBe(
      "Entregar el informe",
    );
    expect(element.querySelector(".tr-due")?.textContent).toContain("10:30");
    expect(element.querySelector(".tr-tag")?.textContent).toBe("Trabajo");

    const check = element.querySelector<HTMLButtonElement>(".tr-check");
    expect(check?.getAttribute("aria-checked")).toBe("false");
    check?.click();
    expect(host.toggle).toHaveBeenCalledWith("a", true);

    element.querySelector<HTMLButtonElement>(".tr-title")?.click();
    expect(host.open).toHaveBeenCalledWith("a");
    handle.destroy();
  });

  it("follows the task when it changes, without touching the document", async () => {
    const host = makeHost({ a: task() });
    const { element, handle, changes } = await open(doc(ref("a")), host);
    host.infos.a = task({
      title: "Informe final",
      isCompleted: true,
      tags: [],
    });
    handle.refreshTaskRefs();
    expect(element.querySelector(".tr-title")?.textContent).toBe(
      "Informe final",
    );
    expect(element.querySelector(".task-ref")?.classList).toContain("is-done");
    expect(element.querySelector(".tr-tag")).toBeNull();
    expect(changes).toEqual([]);
    handle.destroy();
  });

  it("shows a deleted task struck through, with a way to restore it", async () => {
    const host = makeHost({ a: null });
    const { element, handle } = await open(doc(ref("a")), host);
    expect(element.querySelector(".tr-title.gone")?.textContent).toBe(
      "Tarea eliminada",
    );
    const restore = element.querySelector<HTMLButtonElement>(".tr-restore");
    restore?.click();
    expect(host.restore).toHaveBeenCalledWith("a");

    host.restore.mockResolvedValueOnce(false);
    restore?.click();
    await waitFor(() =>
      expect(restore?.textContent).toBe("Ya no se puede restaurar"),
    );
    handle.destroy();
  });

  it("inserts a reference that the saved document keeps", async () => {
    const host = makeHost({ a: task() });
    const { handle, changes } = await open(doc(), host);
    handle.insertTaskRef("a");
    const saved = JSON.parse(changes.at(-1) ?? "{}") as {
      content: { type: string; attrs?: { taskId: string } }[];
    };
    expect(saved.content[0]).toEqual({
      type: "taskRef",
      attrs: { taskId: "a" },
    });
    handle.destroy();
  });

  it("turns every checklist item into a reference, in order", async () => {
    const host = makeHost({ x: task(), y: task() });
    const list = {
      type: "taskList",
      content: [
        {
          type: "taskItem",
          attrs: { checked: false },
          content: [
            { type: "paragraph", content: [{ type: "text", text: "uno" }] },
          ],
        },
        {
          type: "taskItem",
          attrs: { checked: true },
          content: [
            { type: "paragraph", content: [{ type: "text", text: "dos" }] },
          ],
        },
      ],
    };
    const { element, handle, changes } = await open(doc(list), host);
    expect(handle.checklistItems()).toEqual([
      { text: "uno", checked: false },
      { text: "dos", checked: true },
    ]);
    const create = vi.fn(async () => ["x", "y"]);
    expect(await handle.convertChecklists(create)).toBe(2);
    expect(create).toHaveBeenCalledWith([
      { text: "uno", checked: false },
      { text: "dos", checked: true },
    ]);
    const saved = JSON.parse(changes.at(-1) ?? "{}") as {
      content: { type: string; attrs?: { taskId: string } }[];
    };
    expect(saved.content.map((node) => node.attrs?.taskId)).toEqual(["x", "y"]);
    expect(element.querySelector("ul[data-type='taskList']")).toBeNull();
    handle.destroy();
  });

  it("converts nothing when there are no checklists or the tasks were not made", async () => {
    const host = makeHost({});
    const none = await open(doc(), host);
    expect(await none.handle.convertChecklists(async () => ["z"])).toBe(0);
    none.handle.destroy();

    const list = {
      type: "taskList",
      content: [
        {
          type: "taskItem",
          attrs: { checked: false },
          content: [
            { type: "paragraph", content: [{ type: "text", text: "a" }] },
          ],
        },
      ],
    };
    const failing = await open(doc(list), host);
    expect(await failing.handle.convertChecklists(async () => null)).toBe(0);
    expect(failing.handle.checklistItems()).toHaveLength(1);
    failing.handle.destroy();
  });
});

describe("NoteTasksPanel", () => {
  const setup = async () => {
    const store = new TasksStore({
      api: new FakeApi([
        makeTask({ id: "a", title: "Uno", isCompleted: true }),
        makeTask({ id: "b", title: "Dos" }),
        makeTask({ id: "c", title: "Tres", sourceNoteId: "n1" }),
        makeTask({ id: "d", title: "Ajena" }),
      ]),
      settleMs: () => 0,
    });
    await store.load();
    return store;
  };

  it("lists the referenced tasks and the ones made from the note, with progress", async () => {
    const tasks = await setup();
    render(NoteTasksPanel, {
      tasks,
      noteId: "n1",
      contentJson: doc(ref("a"), ref("b"), ref("gone")),
      onOpen: () => {},
    });
    expect(screen.getByText("Uno")).toBeTruthy();
    expect(screen.getByText("Dos")).toBeTruthy();
    expect(screen.getByText("Tres")).toBeTruthy();
    expect(screen.queryByText("Ajena")).toBeNull();
    expect(screen.getByText("1 de 3")).toBeTruthy();
    expect(screen.getByRole("progressbar").getAttribute("aria-valuenow")).toBe(
      "1",
    );
  });

  it("completes a task from the panel and opens it on click", async () => {
    const tasks = await setup();
    const onOpen = vi.fn();
    render(NoteTasksPanel, {
      tasks,
      noteId: "n1",
      contentJson: doc(ref("b")),
      onOpen,
    });
    await fireEvent.click(
      screen.getByRole("checkbox", { name: "Marcar «Dos» como hecha" }),
    );
    await waitFor(() => expect(tasks.find("b")?.isCompleted).toBe(true));
    await fireEvent.click(screen.getByText("Dos"));
    expect(onOpen).toHaveBeenCalledWith(expect.objectContaining({ id: "b" }));
  });

  it("stays out of the way when the note has no tasks", async () => {
    const tasks = await setup();
    render(NoteTasksPanel, {
      tasks,
      noteId: "none",
      contentJson: doc(),
      onOpen: () => {},
    });
    expect(screen.queryByRole("region")).toBeNull();
    expect(screen.queryByRole("progressbar")).toBeNull();
  });
});

describe("NoteTaskDialog", () => {
  it("creates a task that remembers its note and reports it", async () => {
    const api = new FakeApi([]);
    const tasks = new TasksStore({ api, settleMs: () => 0 });
    const created = vi.fn();
    const onClose = vi.fn();
    render(NoteTaskDialog, {
      tasks,
      noteId: "n1",
      prefill: "Llamar al cliente",
      tagIds: [],
      onCreated: created,
      onClose,
    });
    const title = screen.getByRole("textbox") as HTMLInputElement;
    expect(title.value).toBe("Llamar al cliente");
    await fireEvent.click(screen.getByRole("button", { name: "Crear tarea" }));
    await waitFor(() => expect(created).toHaveBeenCalled());
    expect(created.mock.calls[0]?.[0]).toMatchObject({
      title: "Llamar al cliente",
      sourceNoteId: "n1",
    });
    expect(onClose).toHaveBeenCalled();
  });

  it("asks for a title first", async () => {
    const tasks = new TasksStore({ api: new FakeApi([]), settleMs: () => 0 });
    const created = vi.fn();
    render(NoteTaskDialog, {
      tasks,
      noteId: "n1",
      onCreated: created,
      onClose: () => {},
    });
    await fireEvent.click(screen.getByRole("button", { name: "Crear tarea" }));
    expect(screen.getByRole("alert")).toBeTruthy();
    expect(created).not.toHaveBeenCalled();
  });
});
