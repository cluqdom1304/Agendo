import { describe, expect, it } from "vitest";

import { makeTask } from "../../testing";
import { pendingByDate, stripDates, summarizeDay } from "./summary";

describe("today summary (F20-01)", () => {
  it("spans fourteen days from today across a month change", () => {
    const dates = stripDates("2026-03-25");
    expect(dates).toHaveLength(14);
    expect(dates[0]).toBe("2026-03-25");
    expect(dates[13]).toBe("2026-04-07");
  });

  it("counts open dated tasks per day", () => {
    const counts = pendingByDate([
      makeTask({ id: "a", dueDate: "2026-03-10" }),
      makeTask({ id: "b", dueDate: "2026-03-10" }),
      makeTask({ id: "c", dueDate: "2026-03-10", isCompleted: true }),
      makeTask({ id: "d", dueDate: null }),
    ]);
    expect([...counts]).toEqual([["2026-03-10", 2]]);
  });

  it("summarizes a day and counts overdue tasks only for today", () => {
    const tasks = [
      makeTask({ id: "a", dueDate: "2026-03-10", isCompleted: true }),
      makeTask({ id: "b", dueDate: "2026-03-10" }),
      makeTask({ id: "c", dueDate: "2026-03-09" }),
      makeTask({ id: "d", dueDate: "2026-03-01", isCompleted: true }),
      makeTask({ id: "e", dueDate: "2026-03-12" }),
    ];
    expect(summarizeDay(tasks, "2026-03-10", "2026-03-10")).toEqual({
      total: 2,
      done: 1,
      pending: 1,
      overdue: 1,
    });
    expect(summarizeDay(tasks, "2026-03-12", "2026-03-10")).toEqual({
      total: 1,
      done: 0,
      pending: 1,
      overdue: 0,
    });
  });
});
