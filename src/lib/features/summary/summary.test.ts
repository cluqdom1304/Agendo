import { describe, expect, it } from "vitest";

import type { WeekSummary } from "../../types";
import {
  focusLabel,
  habitRows,
  isQuiet,
  minutesLabel,
  percent,
  shiftWeek,
  tagRows,
  weekLabel,
} from "./summary";

const empty: WeekSummary = {
  start: "2026-03-09",
  end: "2026-03-15",
  completed: 0,
  pending: 0,
  overdue: 0,
  byTag: [],
  events: 0,
  eventMinutes: 0,
  habits: [],
  focusSeconds: 0,
  focusSessions: 0,
  focusByDay: [0, 0, 0, 0, 0, 0, 0],
};

describe("summary helpers (F26-02)", () => {
  it("labels a week and moves by whole weeks across months and years", () => {
    expect(weekLabel("2026-03-09", "2026-03-15")).toMatch(
      /^9 mar.* – 15 mar.* 2026$/,
    );
    expect(shiftWeek("2026-03-10", 1)).toBe("2026-03-17");
    expect(shiftWeek("2026-03-02", -1)).toBe("2026-02-23");
    expect(shiftWeek("2026-12-30", 1)).toBe("2027-01-06");
  });

  it("computes percentages without dividing by zero", () => {
    expect(percent(1, 3)).toBe(33);
    expect(percent(5, 5)).toBe(100);
    expect(percent(0, 0)).toBe(0);
  });

  it("writes minutes and focus seconds as hours and minutes", () => {
    expect(minutesLabel(45)).toBe("45 min");
    expect(minutesLabel(60)).toBe("1 h");
    expect(minutesLabel(85)).toBe("1 h 25 min");
    expect(focusLabel(1800)).toBe("30 min");
    expect(focusLabel(5100)).toBe("1 h 25 min");
  });

  it("knows when a week has nothing in it", () => {
    expect(isQuiet(empty)).toBe(true);
    expect(isQuiet({ ...empty, overdue: 1 })).toBe(false);
    expect(isQuiet({ ...empty, focusSeconds: 60 })).toBe(false);
    expect(
      isQuiet({
        ...empty,
        habits: [{ habitId: "h", title: "Leer", scheduled: 1, done: 0 }],
      }),
    ).toBe(false);
  });

  it("orders tags by name with the untagged last and names the missing ones", () => {
    const tally = (tagId: string | null) => ({
      tagId,
      completed: 1,
      pending: 0,
      overdue: 0,
    });
    const known: Record<string, { name: string; color: string | null }> = {
      a: { name: "Casa", color: "#111" },
      b: { name: "Trabajo", color: null },
    };
    const rows = tagRows(
      [tally(null), tally("b"), tally("gone"), tally("a")],
      (id) => known[id],
      { untagged: "Sin etiqueta", gone: "Etiqueta eliminada" },
    );
    expect(rows.map((row) => row.name)).toEqual([
      "Casa",
      "Etiqueta eliminada",
      "Trabajo",
      "Sin etiqueta",
    ]);
    expect(rows[0].color).toBe("#111");
  });

  it("puts the least kept habits first", () => {
    const rows = habitRows([
      { habitId: "1", title: "Leer", scheduled: 5, done: 5 },
      { habitId: "2", title: "Correr", scheduled: 4, done: 1 },
    ]);
    expect(rows.map((row) => [row.title, row.percent])).toEqual([
      ["Correr", 25],
      ["Leer", 100],
    ]);
  });
});
