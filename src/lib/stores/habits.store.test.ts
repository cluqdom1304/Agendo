import { describe, expect, it } from "vitest";

import { FakeHabitsApi } from "../testing";
import { HabitsStore } from "./habits.svelte";

function setup() {
  const api = new FakeHabitsApi();
  const store = new HabitsStore({ api, noticeMs: 60_000 });
  return { api, store };
}

describe("HabitsStore (D-95)", () => {
  it("loads the habits and the marks of the window", async () => {
    const { api, store } = setup();
    const habit = api.makeHabit({ title: "Leer" });
    api.makeHabit({ title: "Viejo", isArchived: true });
    api.logs.push(
      { habitId: habit.id, date: "2026-03-10" },
      { habitId: habit.id, date: "2026-02-01" },
    );
    await store.show("2026-03-04", "2026-03-10");
    expect(store.active.map((item) => item.title)).toEqual(["Leer"]);
    expect(store.archived.map((item) => item.title)).toEqual(["Viejo"]);
    expect(store.isDone(habit.id, "2026-03-10")).toBe(true);
    expect(store.isDone(habit.id, "2026-02-01")).toBe(false);
  });

  it("hides what the tag filter hides", async () => {
    const { api, store } = setup();
    api.makeHabit({ tagId: "t1" });
    api.makeHabit({ tagId: null });
    store.filter = (habit) => habit.tagId === null;
    await store.show("2026-03-04", "2026-03-10");
    expect(store.active).toHaveLength(1);
  });

  it("ticks the day at once and keeps the streak the backend answers with", async () => {
    const { api, store } = setup();
    const habit = api.makeHabit();
    await store.show("2026-03-04", "2026-03-10");
    const pending = store.setDone(habit, "2026-03-10", true, true);
    expect(store.isDone(habit.id, "2026-03-10")).toBe(true);
    await pending;
    expect(store.habits[0].currentStreak).toBe(1);
    expect(store.marked?.date).toBe("2026-03-10");
  });

  it("undoes the tick it offered", async () => {
    const { api, store } = setup();
    const habit = api.makeHabit();
    await store.show("2026-03-04", "2026-03-10");
    await store.setDone(habit, "2026-03-10", true, true);
    await store.undoMark();
    expect(store.isDone(habit.id, "2026-03-10")).toBe(false);
    expect(api.logs).toEqual([]);
    expect(store.marked).toBeNull();
  });

  it("untick does not offer to undo", async () => {
    const { api, store } = setup();
    const habit = api.makeHabit();
    api.logs.push({ habitId: habit.id, date: "2026-03-09" });
    await store.show("2026-03-04", "2026-03-10");
    await store.setDone(habit, "2026-03-09", false, true);
    expect(store.marked).toBeNull();
    expect(store.isDone(habit.id, "2026-03-09")).toBe(false);
  });

  it("puts the tick back and reports when the backend refuses", async () => {
    const { api, store } = setup();
    const habit = api.makeHabit();
    await store.show("2026-03-04", "2026-03-10");
    api.failWith = "VALIDATION: día futuro";
    expect(await store.setDone(habit, "2026-03-11", true)).toBe(false);
    expect(store.isDone(habit.id, "2026-03-11")).toBe(false);
    expect(store.notice?.message).toContain("día futuro");
  });

  it("creates, edits, archives and deletes with the chance to undo", async () => {
    const { api, store } = setup();
    await store.show("2026-03-04", "2026-03-10");
    const created = await store.create({ title: "Estirar", days: 0b11111 });
    expect(created?.title).toBe("Estirar");
    expect(store.active).toHaveLength(1);
    await store.update(created?.id ?? "", { title: "Estirar 10 min" });
    expect(store.active[0].title).toBe("Estirar 10 min");
    await store.update(created?.id ?? "", { archived: true });
    expect(store.active).toHaveLength(0);
    expect(store.archived).toHaveLength(1);
    expect(await store.remove(store.archived[0])).toBe(true);
    expect(api.habits).toHaveLength(0);
    await store.undoDelete();
    expect(store.archived).toHaveLength(1);
    expect(store.deleted).toBeNull();
  });
});
