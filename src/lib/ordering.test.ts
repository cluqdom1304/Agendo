import { describe, expect, it } from "vitest";

import { groupByDay, sortDay, sortTodo, splitCompleted } from "./ordering";
import { makeTask } from "./testing";

const ids = (tasks: { id: string }[]) => tasks.map((task) => task.id);

describe("sortTodo", () => {
  it("orders pending tasks by position and puts completed ones last by completion time", () => {
    const sorted = sortTodo([
      makeTask({
        id: "done-late",
        isCompleted: true,
        completedAt: "2026-03-02T09:00:00Z",
      }),
      makeTask({ id: "b", position: 20 }),
      makeTask({
        id: "done-early",
        isCompleted: true,
        completedAt: "2026-03-01T09:00:00Z",
      }),
      makeTask({ id: "a", position: 10 }),
    ]);
    expect(ids(sorted)).toEqual(["a", "b", "done-early", "done-late"]);
  });

  it("breaks ties by id so the input order does not matter", () => {
    const tasks = [
      makeTask({ id: "z", position: 1 }),
      makeTask({ id: "m", position: 1 }),
    ];
    expect(ids(sortTodo(tasks))).toEqual(ids(sortTodo([...tasks].reverse())));
    expect(ids(sortTodo(tasks))).toEqual(["m", "z"]);
  });
});

describe("sortDay", () => {
  it("puts timed tasks first by time, then untimed by position, then completed", () => {
    const day = "2026-03-10";
    const sorted = sortDay([
      makeTask({ id: "untimed-2", dueDate: day, position: 2 }),
      makeTask({
        id: "done",
        dueDate: day,
        isCompleted: true,
        completedAt: "2026-03-10T07:00:00Z",
      }),
      makeTask({ id: "late", dueDate: day, dueTime: "18:00" }),
      makeTask({ id: "untimed-1", dueDate: day, position: 1 }),
      makeTask({ id: "early", dueDate: day, dueTime: "08:30" }),
    ]);
    expect(ids(sorted)).toEqual([
      "early",
      "late",
      "untimed-1",
      "untimed-2",
      "done",
    ]);
  });

  it("orders tasks at the same time by position", () => {
    const day = "2026-03-10";
    const sorted = sortDay([
      makeTask({ id: "b", dueDate: day, dueTime: "09:00", position: 2 }),
      makeTask({ id: "a", dueDate: day, dueTime: "09:00", position: 1 }),
    ]);
    expect(ids(sorted)).toEqual(["a", "b"]);
  });
});

describe("groupByDay", () => {
  it("lists the latest day first and leaves undated tasks out", () => {
    const groups = groupByDay([
      makeTask({ id: "old", dueDate: "2026-03-01" }),
      makeTask({ id: "undated" }),
      makeTask({ id: "new", dueDate: "2026-03-12" }),
      makeTask({ id: "new-2", dueDate: "2026-03-12", dueTime: "07:00" }),
    ]);
    expect(groups.map((group) => group.date)).toEqual([
      "2026-03-12",
      "2026-03-01",
    ]);
    expect(ids(groups[0].tasks)).toEqual(["new-2", "new"]);
  });
});

describe("splitCompleted", () => {
  it("separates the Completadas subsection keeping the order", () => {
    const { open, completed } = splitCompleted([
      { id: "a", isCompleted: false },
      { id: "b", isCompleted: true },
      { id: "c", isCompleted: false },
    ]);
    expect(ids(open)).toEqual(["a", "c"]);
    expect(ids(completed)).toEqual(["b"]);
  });
});
