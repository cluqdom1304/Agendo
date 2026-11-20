import { describe, expect, it } from "vitest";

import { makeTask } from "../testing";
import { isFiltering, matchesFilter, NO_FILTER } from "./listFilter";

const TODAY = "2026-03-10";
const ids = (filter: Parameters<typeof matchesFilter>[1]) =>
  [
    makeTask({ id: "undated" }),
    makeTask({ id: "late", dueDate: "2026-03-09" }),
    makeTask({ id: "late-done", dueDate: "2026-03-09", isCompleted: true }),
    makeTask({ id: "today", dueDate: TODAY, priority: 2 }),
    makeTask({ id: "week", dueDate: "2026-03-16", priority: 1 }),
    makeTask({ id: "far", dueDate: "2026-03-17", priority: 3 }),
  ]
    .filter((task) => matchesFilter(task, filter, TODAY))
    .map((task) => task.id);

describe("list filters (F20-05)", () => {
  it("lets everything through without filters", () => {
    expect(isFiltering(NO_FILTER)).toBe(false);
    expect(ids(NO_FILTER)).toHaveLength(6);
  });

  it("filters by date", () => {
    expect(ids({ ...NO_FILTER, date: "undated" })).toEqual(["undated"]);
    expect(ids({ ...NO_FILTER, date: "overdue" })).toEqual(["late"]);
    expect(ids({ ...NO_FILTER, date: "today" })).toEqual(["today"]);
    expect(ids({ ...NO_FILTER, date: "week" })).toEqual(["today", "week"]);
  });

  it("filters by lowest priority and hides completed tasks", () => {
    expect(ids({ ...NO_FILTER, minPriority: 2 })).toEqual(["today", "far"]);
    expect(ids({ ...NO_FILTER, showCompleted: false })).not.toContain(
      "late-done",
    );
    expect(isFiltering({ ...NO_FILTER, showCompleted: false })).toBe(true);
  });
});
