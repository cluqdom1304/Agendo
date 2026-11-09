import { describe, expect, it } from "vitest";

import { makeTask } from "../../testing";
import {
  addDays,
  addMonths,
  layoutDay,
  minutesToTime,
  monthGrid,
  quarterOf,
  snapSpan,
  startOfWeek,
  systemFirstDay,
  timeToMinutes,
  weekDates,
  weekdayOf,
} from "./grid";

// Vitest runs in Node; the project does not install its type definitions just for this.
const env = (
  globalThis as unknown as {
    process: { env: Record<string, string | undefined> };
  }
).process.env;

const MONDAY = 1;
const SUNDAY = 0;

describe("days and months", () => {
  it("adds days across month and year ends and leap days", () => {
    expect(addDays("2026-01-31", 1)).toBe("2026-02-01");
    expect(addDays("2026-12-31", 1)).toBe("2027-01-01");
    expect(addDays("2028-02-28", 1)).toBe("2028-02-29");
    expect(addDays("2026-03-01", -1)).toBe("2026-02-28");
  });

  it("clamps the day when the target month is shorter", () => {
    expect(addMonths("2026-01-31", 1)).toBe("2026-02-28");
    expect(addMonths("2028-01-31", 1)).toBe("2028-02-29");
    expect(addMonths("2026-12-15", 1)).toBe("2027-01-15");
    expect(addMonths("2026-01-15", -1)).toBe("2025-12-15");
    expect(addMonths("2026-03-31", -13)).toBe("2025-02-28");
  });

  it("knows the weekday", () => {
    expect(weekdayOf("2026-03-09")).toBe(1);
    expect(weekdayOf("2026-03-15")).toBe(0);
  });
});

describe("weeks", () => {
  it("starts on the configured day", () => {
    // 2026-03-11 is a Wednesday.
    expect(startOfWeek("2026-03-11", MONDAY)).toBe("2026-03-09");
    expect(startOfWeek("2026-03-11", SUNDAY)).toBe("2026-03-08");
    expect(startOfWeek("2026-03-09", MONDAY)).toBe("2026-03-09");
    expect(startOfWeek("2026-03-15", MONDAY)).toBe("2026-03-09");
    expect(startOfWeek("2026-03-14", 6)).toBe("2026-03-14");
  });

  it("lists seven consecutive dates", () => {
    expect(weekDates("2026-03-11", MONDAY)).toEqual([
      "2026-03-09",
      "2026-03-10",
      "2026-03-11",
      "2026-03-12",
      "2026-03-13",
      "2026-03-14",
      "2026-03-15",
    ]);
  });
});

describe("month grid", () => {
  it("covers whole weeks with the neighbouring days", () => {
    const weeks = monthGrid("2026-03-15", MONDAY);
    expect(weeks[0][0]).toBe("2026-02-23");
    expect(weeks.at(-1)?.[6]).toBe("2026-04-05");
    expect(weeks).toHaveLength(6);
    expect(weeks.every((week) => week.length === 7)).toBe(true);
  });

  it("uses four rows for a February that fits exactly (2026 starts on a Sunday)", () => {
    expect(monthGrid("2026-02-10", SUNDAY)).toHaveLength(4);
  });

  it("changes with the first day of the week", () => {
    expect(monthGrid("2026-03-15", SUNDAY)[0][0]).toBe("2026-03-01");
  });

  it("has every date exactly once and in order, for every month of several years", () => {
    for (const first of [MONDAY, SUNDAY, 6] as const) {
      for (let year = 2024; year <= 2028; year++) {
        for (let month = 1; month <= 12; month++) {
          const iso = `${year}-${String(month).padStart(2, "0")}-10`;
          const flat = monthGrid(iso, first).flat();
          for (let i = 1; i < flat.length; i++) {
            expect(flat[i]).toBe(addDays(flat[i - 1], 1));
          }
          expect(flat).toContain(`${iso.slice(0, 7)}-01`);
          expect(weekdayOf(flat[0])).toBe(first);
        }
      }
    }
  });
});

describe("daylight saving time", () => {
  it("never skips or repeats a date across the change, in any zone", () => {
    const originalZone = env.TZ;
    try {
      // Europe/Madrid changes on 2026-03-29 and 2026-10-25; the other zones change at
      // other dates and in the other direction.
      for (const zone of [
        "Europe/Madrid",
        "America/New_York",
        "Pacific/Auckland",
      ]) {
        env.TZ = zone;
        expect(weekDates("2026-03-29", MONDAY)).toEqual([
          "2026-03-23",
          "2026-03-24",
          "2026-03-25",
          "2026-03-26",
          "2026-03-27",
          "2026-03-28",
          "2026-03-29",
        ]);
        expect(addDays("2026-03-28", 1)).toBe("2026-03-29");
        expect(addDays("2026-03-29", 1)).toBe("2026-03-30");
        expect(addDays("2026-10-24", 1)).toBe("2026-10-25");
        expect(addDays("2026-10-25", 1)).toBe("2026-10-26");
        expect(addMonths("2026-03-29", 1)).toBe("2026-04-29");
        const october = monthGrid("2026-10-10", MONDAY).flat();
        expect(new Set(october).size).toBe(october.length);
        expect(october).toContain("2026-10-25");
      }
    } finally {
      if (originalZone === undefined) delete env.TZ;
      else env.TZ = originalZone;
    }
  });
});

describe("system first day", () => {
  it("maps Intl week info (1 = Monday … 7 = Sunday) to 0 = Sunday", () => {
    expect(systemFirstDay("es-ES")).toBe(1);
    expect(systemFirstDay("en-US")).toBe(0);
  });

  it("falls back to Monday for an invalid locale", () => {
    expect(systemFirstDay("not a locale!")).toBe(1);
  });
});

describe("time of day", () => {
  it("converts between HH:MM and minutes", () => {
    expect(timeToMinutes("09:30")).toBe(570);
    expect(minutesToTime(570)).toBe("09:30");
    expect(minutesToTime(-5)).toBe("00:00");
    expect(minutesToTime(5000)).toBe("23:59");
  });

  it("finds the quarter hour under the pointer, inside the day", () => {
    expect(quarterOf(7)).toBe(0);
    expect(quarterOf(15)).toBe(15);
    expect(quarterOf(1439)).toBe(1425);
    expect(quarterOf(-30)).toBe(0);
    expect(quarterOf(5000)).toBe(1425);
  });

  it("orders a dragged span whichever way it goes and covers the last quarter", () => {
    expect(snapSpan(540, 600)).toEqual({ start: 540, end: 615 });
    expect(snapSpan(600, 540)).toEqual({ start: 540, end: 615 });
    expect(snapSpan(545, 545)).toEqual({ start: 540, end: 555 });
    expect(snapSpan(1439, 1439)).toEqual({ start: 1425, end: 1440 });
  });
});

describe("layoutDay", () => {
  const at = (id: string, dueTime: string | null) =>
    makeTask({ id, dueDate: "2026-03-10", dueTime });

  it("ignores tasks without a time", () => {
    expect(layoutDay([at("a", null)])).toEqual([]);
  });

  it("puts overlapping tasks side by side and gives separate ones the full width", () => {
    const placed = layoutDay([
      at("a", "09:00"),
      at("b", "09:15"),
      at("c", "09:20"),
      at("d", "11:00"),
    ]);
    const byId = Object.fromEntries(placed.map((p) => [p.task.id, p]));
    expect([byId.a.lane, byId.b.lane, byId.c.lane]).toEqual([0, 1, 2]);
    expect(byId.a.lanes).toBe(3);
    expect(byId.d).toMatchObject({ lane: 0, lanes: 1 });
  });

  it("sizes blocks by duration and frees the lane when a short one ends", () => {
    const timed = (id: string, time: string, durationMinutes: number | null) =>
      makeTask({ id, dueDate: "2026-03-10", dueTime: time, durationMinutes });
    const placed = layoutDay([
      timed("long", "09:00", 120),
      timed("inside", "10:30", 15),
      timed("short", "09:00", 15),
      timed("plain", "13:00", null),
    ]);
    const byId = Object.fromEntries(placed.map((p) => [p.task.id, p]));
    expect(byId.long.span).toBe(120);
    expect(byId.plain.span).toBe(30);
    // "inside" starts while "long" still runs, so they share the width.
    expect(byId.inside.lanes).toBe(2);
    expect(byId.plain.lanes).toBe(1);
  });

  it("sizes blocks by duration and shares the width while a long one runs", () => {
    const timed = (id: string, time: string, durationMinutes: number | null) =>
      makeTask({ id, dueDate: "2026-03-10", dueTime: time, durationMinutes });
    const placed = layoutDay([
      timed("long", "09:00", 120),
      timed("inside", "10:30", 15),
      timed("plain", "13:00", null),
    ]);
    const byId = Object.fromEntries(placed.map((p) => [p.task.id, p]));
    expect(byId.long.span).toBe(120);
    expect(byId.plain.span).toBe(30);
    expect(byId.inside.lanes).toBe(2);
    expect(byId.plain.lanes).toBe(1);
  });

  it("reuses a lane once its block has ended", () => {
    const placed = layoutDay([at("a", "09:00"), at("b", "09:30")]);
    expect(placed.map((p) => [p.lane, p.lanes])).toEqual([
      [0, 1],
      [0, 1],
    ]);
  });
});
