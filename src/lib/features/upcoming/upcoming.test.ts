import { describe, expect, it } from "vitest";

import { makeTask } from "../../testing";
import { upcomingGroups } from "./upcoming";

describe("upcomingGroups (F20-05)", () => {
  it("splits open dated tasks into overdue, days within the horizon and later", () => {
    const groups = upcomingGroups(
      [
        makeTask({ id: "late2", dueDate: "2026-03-09" }),
        makeTask({ id: "late1", dueDate: "2026-03-01" }),
        makeTask({ id: "today", dueDate: "2026-03-10" }),
        makeTask({ id: "soon-b", dueDate: "2026-03-12", position: 2 }),
        makeTask({
          id: "soon-a",
          dueDate: "2026-03-12",
          priority: 3,
          position: 5,
        }),
        makeTask({ id: "edge", dueDate: "2026-04-08" }),
        makeTask({ id: "far", dueDate: "2026-04-09" }),
        makeTask({ id: "done", dueDate: "2026-03-11", isCompleted: true }),
        makeTask({ id: "undated" }),
      ],
      "2026-03-10",
    );
    expect(groups.overdue.map((task) => task.id)).toEqual(["late1", "late2"]);
    expect(
      groups.days.map((day) => [day.date, day.tasks.map((task) => task.id)]),
    ).toEqual([
      ["2026-03-10", ["today"]],
      ["2026-03-12", ["soon-a", "soon-b"]],
      ["2026-04-08", ["edge"]],
    ]);
    expect(groups.later.map((task) => task.id)).toEqual(["far"]);
  });
});
