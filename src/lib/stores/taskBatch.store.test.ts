import { afterEach, describe, expect, it, vi } from "vitest";

import { FakeApi, makeTask } from "../testing";
import { TasksStore } from "./tasks.svelte";

const NOW = new Date("2026-03-10T10:00:00Z");

async function setup() {
  const api = new FakeApi([
    makeTask({ id: "a", dueDate: "2026-03-10", position: 1 }),
    makeTask({ id: "b", dueDate: "2026-03-10", position: 2 }),
    makeTask({ id: "c", dueDate: "2026-03-10", position: 3 }),
    makeTask({ id: "d", position: 1 }),
  ]);
  const store = new TasksStore({
    api,
    now: () => NOW,
    settleMs: () => 0,
    undoMs: 5000,
    noticeMs: 6000,
  });
  await store.load();
  return { api, store };
}

afterEach(() => vi.useRealTimers());

describe("selection (F17-03)", () => {
  it("toggles single tasks and extends a range in list order", async () => {
    const { store } = await setup();
    // POR HACER first, then the days: d, a, b, c.
    expect(store.visibleOrder).toEqual(["d", "a", "b", "c"]);
    store.batch.toggle("a");
    store.batch.extendTo("c");
    expect([...store.batch.ids].sort()).toEqual(["a", "b", "c"]);
    store.batch.toggle("b");
    expect([...store.batch.ids].sort()).toEqual(["a", "c"]);
    store.batch.clear();
    expect(store.batch.count).toBe(0);
  });

  it("completes the selection and one undo reopens all of it", async () => {
    const { store } = await setup();
    store.batch.toggle("a");
    store.batch.toggle("b");
    await store.batch.complete();
    expect(store.find("a")?.isCompleted).toBe(true);
    expect(store.find("b")?.isCompleted).toBe(true);
    expect(store.find("c")?.isCompleted).toBe(false);
    expect(store.batch.count).toBe(0);
    expect(store.batch.entry?.message).toBe("2 tareas completadas");

    await store.batch.undo();
    expect(store.find("a")?.isCompleted).toBe(false);
    expect(store.find("b")?.isCompleted).toBe(false);
    expect(store.batch.entry).toBeNull();
  });

  it("adds a tag, moves to another day and puts everything back with one undo", async () => {
    const { store } = await setup();
    store.batch.toggle("a");
    store.batch.toggle("d");
    await store.batch.addTag("t1");
    expect(store.find("a")?.tagIds).toEqual(["t1"]);
    expect(store.find("d")?.tagIds).toEqual(["t1"]);
    await store.batch.undo();
    expect(store.find("a")?.tagIds).toEqual([]);
    expect(store.find("d")?.tagIds).toEqual([]);

    store.batch.toggle("a");
    store.batch.toggle("d");
    await store.batch.moveTo("2026-03-20");
    expect(store.find("a")?.dueDate).toBe("2026-03-20");
    expect(store.find("d")?.dueDate).toBe("2026-03-20");
    await store.batch.undo();
    expect(store.find("a")?.dueDate).toBe("2026-03-10");
    expect(store.find("d")?.dueDate).toBeNull();
  });

  it("deletes the selection and one undo restores it; the entry expires", async () => {
    vi.useFakeTimers();
    const { store } = await setup();
    store.batch.toggle("a");
    store.batch.toggle("b");
    await store.batch.remove();
    expect(store.find("a")).toBeUndefined();
    expect(store.find("c")).toBeDefined();
    await store.batch.undo();
    expect(store.find("a")).toBeDefined();
    expect(store.find("b")).toBeDefined();

    store.batch.toggle("a");
    await store.batch.remove();
    await vi.advanceTimersByTimeAsync(5001);
    expect(store.batch.entry).toBeNull();
  });

  it("reports a failure and still applies the rest", async () => {
    const { api, store } = await setup();
    api.failing.add("taskSetCompleted");
    store.batch.toggle("a");
    store.batch.toggle("b");
    await store.batch.complete();
    expect(store.notice?.kind).toBe("error");
    expect(store.batch.entry?.message).toBe("1 tarea completada");
  });
});
