import { describe, expect, it } from "vitest";

import {
  customToInstant,
  dueInstant,
  offsetIsAhead,
  offsetLabel,
  pending,
  togglePreset,
  withCustom,
  withoutReminder,
} from "./reminders";
import type { Reminder } from "./types";

const reminder = (overrides: Partial<Reminder> & { id: string }): Reminder => ({
  notifyAt: "2026-03-11T08:00:00Z",
  offsetMinutes: null,
  isSent: false,
  ...overrides,
});

describe("offsetLabel", () => {
  it("names the options of F-12", () => {
    expect(offsetLabel(0)).toBe("En el momento");
    expect(offsetLabel(15)).toBe("15 min antes");
    expect(offsetLabel(60)).toBe("1 hora antes");
    expect(offsetLabel(120)).toBe("2 horas antes");
    expect(offsetLabel(1440)).toBe("1 día antes");
    expect(offsetLabel(2880)).toBe("2 días antes");
  });
});

describe("sets", () => {
  const current = [
    reminder({ id: "a", offsetMinutes: 15 }),
    reminder({ id: "b", notifyAt: "2026-03-12T07:30:00Z" }),
    reminder({ id: "sent", offsetMinutes: 0, isSent: true }),
  ];

  it("only counts reminders that have not fired", () => {
    expect(pending(current).map((r) => r.id)).toEqual(["a", "b"]);
  });

  it("adds a preset that is off and removes one that is on", () => {
    expect(togglePreset(current, 60)).toEqual([
      { offsetMinutes: 15 },
      { notifyAt: "2026-03-12T07:30:00Z" },
      { offsetMinutes: 60 },
    ]);
    expect(togglePreset(current, 15)).toEqual([
      { notifyAt: "2026-03-12T07:30:00Z" },
    ]);
  });

  it("a preset that already fired counts as off", () => {
    expect(togglePreset(current, 0)).toContainEqual({ offsetMinutes: 0 });
  });

  it("removes one reminder or adds a custom one", () => {
    expect(withoutReminder(current, "b")).toEqual([{ offsetMinutes: 15 }]);
    expect(withCustom(current, "2026-04-01T10:00:00Z")).toEqual([
      { offsetMinutes: 15 },
      { notifyAt: "2026-03-12T07:30:00Z" },
      { notifyAt: "2026-04-01T10:00:00Z" },
    ]);
  });
});

describe("time", () => {
  it("needs both date and time to have a due moment", () => {
    expect(dueInstant(null, "09:00")).toBeNull();
    expect(dueInstant("2026-03-11", null)).toBeNull();
    expect(dueInstant("2026-03-11", "09:00")).toEqual(
      new Date(2026, 2, 11, 9, 0),
    );
  });

  it("offers an option only while its moment is ahead", () => {
    const due = new Date(2026, 2, 11, 9, 0);
    const before = (minutes: number) =>
      new Date(due.getTime() - minutes * 60_000);
    expect(offsetIsAhead(due, 15, before(16))).toBe(true);
    expect(offsetIsAhead(due, 15, before(15))).toBe(false);
    expect(offsetIsAhead(null, 15, before(60))).toBe(false);
  });

  it("converts a local datetime-local value to a backend instant", () => {
    const local = new Date(2026, 2, 11, 9, 30);
    expect(customToInstant("2026-03-11T09:30")).toBe(
      local.toISOString().replace(/\.\d{3}Z$/, "Z"),
    );
    expect(customToInstant("")).toBeNull();
    expect(customToInstant("nope")).toBeNull();
  });
});
