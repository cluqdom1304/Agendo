import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { FakeApi, makeTask } from "../testing";
import { TasksStore } from "./tasks.svelte";

const NOW = new Date("2026-03-10T10:00:00Z");

function setup(tasks = [makeTask({ id: "a", position: 1 })]) {
  const api = new FakeApi(tasks);
  const store = new TasksStore({
    api,
    now: () => NOW,
    settleMs: () => 0,
    undoMs: 5000,
    noticeMs: 6000,
  });
  return { api, store };
}

const titles = (list: { title: string }[]) => list.map((task) => task.title);

afterEach(() => vi.useRealTimers());

describe("load", () => {
  it("merges both sections and exposes the blocks in display order", async () => {
    const { store } = setup([
      makeTask({ id: "t2", position: 2 }),
      makeTask({ id: "t1", position: 1 }),
      makeTask({ id: "old", dueDate: "2026-03-01" }),
      makeTask({ id: "new", dueDate: "2026-03-12" }),
    ]);
    await store.load();
    expect(store.status).toBe("ready");
    expect(titles(store.todoBlock.open)).toEqual(["t1", "t2"]);
    expect(store.agendaBlocks.map((block) => block.date)).toEqual([
      "2026-03-12",
      "2026-03-01",
    ]);
  });

  it("reports a failed load without crashing", async () => {
    const { api, store } = setup();
    api.failing.add("tasksList");
    await store.load();
    expect(store.status).toBe("error");
    expect(store.notice?.message).toContain("base de datos");
    expect(store.notice?.message).not.toContain("DB:");
  });
});

describe("complete", () => {
  it("moves the task into Completadas of its own day", async () => {
    const { store } = setup([
      makeTask({ id: "x", dueDate: "2026-03-10", position: 1 }),
      makeTask({ id: "y", dueDate: "2026-03-10", position: 2 }),
    ]);
    await store.load();
    await store.setCompleted("x", true);
    const day = store.agendaBlocks[0];
    expect(titles(day.open)).toEqual(["y"]);
    expect(titles(day.completed)).toEqual(["x"]);
  });

  it("reopens a completed task into the pending list", async () => {
    const { store } = setup([
      makeTask({
        id: "x",
        isCompleted: true,
        completedAt: "2026-03-09T10:00:00Z",
      }),
    ]);
    await store.load();
    await store.setCompleted("x", false);
    expect(titles(store.todoBlock.open)).toEqual(["x"]);
    expect(store.todoBlock.completed).toEqual([]);
  });

  it("applies the change before the backend answers", async () => {
    const { api, store } = setup();
    await store.load();
    let release = () => {};
    api.gate = new Promise<void>((resolve) => (release = resolve));
    const pending = store.setCompleted("a", true);
    await vi.waitFor(() => expect(store.find("a")?.isCompleted).toBe(true));
    expect(api.tasks[0].isCompleted).toBe(false);
    release();
    await pending;
    expect(store.find("a")?.completedAt).toBe("2026-03-10T10:00:00Z");
  });

  it("puts the task back and reports the error when the backend refuses", async () => {
    const { api, store } = setup();
    await store.load();
    const before = store.find("a");
    api.failing.add("taskSetCompleted");
    await store.setCompleted("a", true);
    expect(store.find("a")).toEqual(before);
    expect(store.settling.has("a")).toBe(false);
    expect(store.notice).not.toBeNull();
  });

  it("waits for the exit animation before moving the row", async () => {
    vi.useFakeTimers();
    const api = new FakeApi([makeTask({ id: "a" })]);
    const store = new TasksStore({ api, settleMs: () => 220 });
    await store.load();
    const pending = store.setCompleted("a", true);
    await vi.advanceTimersByTimeAsync(100);
    expect(store.settling.has("a")).toBe(true);
    expect(store.find("a")?.isCompleted).toBe(false);
    await vi.advanceTimersByTimeAsync(120);
    await pending;
    expect(store.settling.has("a")).toBe(false);
    expect(store.find("a")?.isCompleted).toBe(true);
  });
});

describe("rename", () => {
  it("shows the new title at once and keeps what the backend returns", async () => {
    const { api, store } = setup();
    await store.load();
    await store.rename("a", "  Nuevo título  ");
    expect(store.find("a")?.title).toBe("Nuevo título");
    expect(api.calls).toContain("taskUpdate:a=Nuevo título");
  });

  it("restores the old title when the backend refuses", async () => {
    const { api, store } = setup([makeTask({ id: "a", title: "Original" })]);
    await store.load();
    api.failing.add("taskUpdate");
    await store.rename("a", "Otro");
    expect(store.find("a")?.title).toBe("Original");
    expect(store.notice).not.toBeNull();
  });

  it("ignores empty or unchanged titles without calling the backend", async () => {
    const { api, store } = setup([makeTask({ id: "a", title: "Igual" })]);
    await store.load();
    const calls = api.calls.length;
    await store.rename("a", "   ");
    await store.rename("a", "Igual");
    expect(api.calls.length).toBe(calls);
  });
});

describe("drafts", () => {
  it("creates in POR HACER and opens the next draft on Enter", async () => {
    const { api, store } = setup([]);
    await store.load();
    store.startDraft({ section: "todo" });
    const first = store.draft;
    await store.commitDraft(first?.key ?? -1, "  Comprar pan ", true);
    expect(titles(store.todoBlock.open)).toEqual(["Comprar pan"]);
    expect(api.calls).toContain("taskCreate:Comprar pan");
    expect(store.draft).not.toBeNull();
    expect(store.draft?.key).not.toBe(first?.key);
  });

  it("creates on a given day and closes the draft when focus is lost", async () => {
    const { store } = setup([]);
    await store.load();
    store.startDraft(store.todayTarget());
    await store.commitDraft(store.draft?.key ?? -1, "Estudiar", false);
    expect(store.agendaBlocks[0].date).toBe("2026-03-10");
    expect(titles(store.agendaBlocks[0].open)).toEqual(["Estudiar"]);
    expect(store.draft).toBeNull();
  });

  it("shows the draft's day even when it has no tasks yet", async () => {
    const { store } = setup([makeTask({ id: "o", dueDate: "2026-03-01" })]);
    await store.load();
    store.startDraft({ section: "agenda", date: "2026-03-10" });
    expect(store.agendaBlocks.map((block) => block.date)).toEqual([
      "2026-03-10",
      "2026-03-01",
    ]);
  });

  it("closes an empty draft without calling the backend", async () => {
    const { api, store } = setup([]);
    await store.load();
    store.startDraft({ section: "todo" });
    await store.commitDraft(store.draft?.key ?? -1, "   ", true);
    expect(store.draft).toBeNull();
    expect(api.calls.some((call) => call.startsWith("taskCreate"))).toBe(false);
  });

  it("keeps the draft open when saving fails, and never saves one draft twice", async () => {
    const { api, store } = setup([]);
    await store.load();
    store.startDraft({ section: "todo" });
    const key = store.draft?.key ?? -1;
    api.failing.add("taskCreate");
    await store.commitDraft(key, "Texto", true);
    expect(store.draft?.key).toBe(key);
    expect(store.isEmpty).toBe(true);

    await store.commitDraft(key, "Texto", false);
    await store.commitDraft(key, "Texto", false);
    expect(api.calls.filter((c) => c.startsWith("taskCreate")).length).toBe(2);
  });

  it("chooses the target from the selected task", async () => {
    const { store } = setup([makeTask({ id: "d", dueDate: "2026-03-05" })]);
    await store.load();
    expect(store.defaultTarget()).toEqual({ section: "todo" });
    store.selectedId = "d";
    expect(store.defaultTarget()).toEqual({
      section: "agenda",
      date: "2026-03-05",
    });
  });
});

describe("delete with undo", () => {
  beforeEach(() => vi.useFakeTimers());

  function fakeTimerStore() {
    const api = new FakeApi([
      makeTask({ id: "a", position: 1 }),
      makeTask({ id: "b", position: 2 }),
    ]);
    return { api, store: new TasksStore({ api, undoMs: 5000 }) };
  }

  it("removes the task at once and offers undo for 5 seconds", async () => {
    const { store } = fakeTimerStore();
    await store.load();
    await store.remove("a");
    expect(titles(store.todoBlock.open)).toEqual(["b"]);
    expect(store.undoStack).toHaveLength(1);
    await vi.advanceTimersByTimeAsync(4999);
    expect(store.undoStack).toHaveLength(1);
    await vi.advanceTimersByTimeAsync(2);
    expect(store.undoStack).toHaveLength(0);
  });

  it("restores the task in its place when undone", async () => {
    const { api, store } = fakeTimerStore();
    await store.load();
    await store.remove("a");
    await store.undoDelete();
    expect(titles(store.todoBlock.open)).toEqual(["a", "b"]);
    expect(store.undoStack).toHaveLength(0);
    expect(api.calls).toContain("taskRestore:a");
  });

  it("undoes the latest deletion first", async () => {
    const { store } = fakeTimerStore();
    await store.load();
    await store.remove("a");
    await store.remove("b");
    await store.undoDelete();
    expect(titles(store.todoBlock.open)).toEqual(["b"]);
    expect(store.undoStack).toHaveLength(1);
  });

  it("brings the task back and reports when the backend refuses to delete", async () => {
    const { api, store } = fakeTimerStore();
    await store.load();
    api.failing.add("taskDelete");
    await store.remove("a");
    expect(titles(store.todoBlock.open)).toEqual(["a", "b"]);
    expect(store.undoStack).toHaveLength(0);
    expect(store.notice).not.toBeNull();
  });

  it("hides the error notice after a while", async () => {
    const { api, store } = fakeTimerStore();
    await store.load();
    api.failing.add("taskDelete");
    await store.remove("a");
    await vi.advanceTimersByTimeAsync(6001);
    expect(store.notice).toBeNull();
  });
});

describe("notes", () => {
  const doc = '{"type":"doc","content":[]}';

  it("saveNotes replaces the task with the backend's answer", async () => {
    const { api, store } = setup();
    await store.load();
    await store.saveNotes("a", doc);
    expect(api.calls).toContain(`taskUpdate:a#notes=${doc}`);
    expect(store.find("a")?.contentJson).toBe(doc);
  });

  it("saveNotes reports a refusal and rejects so the editor keeps its text", async () => {
    const { api, store } = setup();
    await store.load();
    api.failing.add("taskUpdate");
    await expect(store.saveNotes("a", doc)).rejects.toBeDefined();
    expect(store.notice?.message).toContain("base de datos");
    expect(store.find("a")?.contentJson).toBe("");
  });

  it("toggleNotes keeps one task open at a time", async () => {
    const { store } = setup([makeTask({ id: "a" }), makeTask({ id: "b" })]);
    await store.load();
    store.toggleNotes("a");
    store.toggleNotes("b");
    expect(store.notesOpenId).toBe("b");
    store.toggleNotes("b");
    expect(store.notesOpenId).toBeNull();
  });

  it("completing a task closes its notes", async () => {
    const { store } = setup();
    await store.load();
    store.toggleNotes("a");
    await store.setCompleted("a", true);
    expect(store.notesOpenId).toBeNull();
  });

  it("reveal selects the task and opens its notes only when it has text", async () => {
    const withText =
      '{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"hola"}]}]}';
    const { store } = setup([
      makeTask({ id: "a", contentJson: withText }),
      makeTask({ id: "b" }),
    ]);
    await store.load();
    expect(store.reveal("a")).toBe(true);
    expect([store.selectedId, store.notesOpenId]).toEqual(["a", "a"]);
    expect(store.reveal("b")).toBe(true);
    expect([store.selectedId, store.notesOpenId]).toEqual(["b", null]);
    expect(store.reveal("zzz")).toBe(false);
  });
});
