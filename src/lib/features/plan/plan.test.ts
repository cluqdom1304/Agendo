import { describe, expect, it } from "vitest";

import { makeTask } from "../../testing";
import { CAPACITY_MINUTES, planCandidates, planTotals } from "./plan";

describe("planCandidates (F24-02)", () => {
  it("splits the open tasks into overdue, undated and today's", () => {
    const tasks = [
      makeTask({ id: "late2", dueDate: "2026-03-09" }),
      makeTask({ id: "late1", dueDate: "2026-03-05" }),
      makeTask({ id: "none" }),
      makeTask({ id: "now", dueDate: "2026-03-10" }),
      makeTask({ id: "later", dueDate: "2026-03-11" }),
      makeTask({ id: "done", dueDate: "2026-03-01", isCompleted: true }),
    ];
    const result = planCandidates(tasks, "2026-03-10");
    expect(result.overdue.map((task) => task.id)).toEqual(["late1", "late2"]);
    expect(result.undated.map((task) => task.id)).toEqual(["none"]);
    expect(result.today.map((task) => task.id)).toEqual(["now"]);
  });
});

describe("planTotals", () => {
  it("adds the known durations and counts the rest apart", () => {
    const totals = planTotals([
      makeTask({ id: "a", durationMinutes: 90 }),
      makeTask({ id: "b", durationMinutes: 30 }),
      makeTask({ id: "c", durationMinutes: null }),
    ]);
    expect(totals).toEqual({
      tasks: 3,
      minutes: 120,
      unestimated: 1,
      overloaded: false,
    });
  });

  it("flags only what is above the capacity", () => {
    const task = (minutes: number) =>
      makeTask({ id: "x", durationMinutes: minutes });
    expect(planTotals([task(CAPACITY_MINUTES)]).overloaded).toBe(false);
    expect(planTotals([task(CAPACITY_MINUTES), task(1)]).overloaded).toBe(true);
  });
});
