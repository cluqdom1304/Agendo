import { describe, expect, it } from "vitest";

import {
  customToInstant,
  describeRepeat,
  draftReminders,
  dueInstant,
  offsetIsAhead,
  offsetLabel,
  pending,
  sent,
  toInput,
  togglePreset,
  withCustom,
  withoutReminder,
} from "./reminders";
import type { Reminder } from "./types";

const reminder = (overrides: Partial<Reminder> & { id: string }): Reminder => ({
  notifyAt: "2026-03-11T08:00:00Z",
  offsetMinutes: null,
  isSent: false,
  repeat: null,
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
    expect(offsetLabel(10080)).toBe("1 semana antes");
    expect(offsetLabel(20160)).toBe("2 semanas antes");
    expect(offsetLabel(0, true)).toBe("El mismo día");
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

describe("repeating reminders", () => {
  const rule = {
    every: 2,
    unit: "week" as const,
    count: 5,
    until: null,
    occurrence: 3,
  };

  it("describes the interval and how it ends", () => {
    expect(
      describeRepeat({ every: 1, unit: "day", count: null, until: null }),
    ).toBe("Cada día");
    expect(describeRepeat(rule)).toBe("Cada 2 semanas, 5 veces");
    expect(
      describeRepeat({
        every: 1,
        unit: "month",
        count: null,
        until: "2026-04-10",
      }),
    ).toBe("Cada mes, hasta el 10/04/2026");
    expect(
      describeRepeat({ every: 3, unit: "hour", count: null, until: null }),
    ).toBe("Cada 3 horas");
  });

  it("keeps the rule, with its occurrence, when a reminder is sent back", () => {
    const r = reminder({ id: "a", repeat: rule });
    expect(toInput(r)).toEqual({ notifyAt: r.notifyAt, repeat: rule });
    expect(toInput(reminder({ id: "b" }))).toEqual({ notifyAt: r.notifyAt });
  });

  it("adds a custom reminder with its rule and leaves history out", () => {
    const current = [
      reminder({ id: "old", isSent: true }),
      reminder({ id: "keep", repeat: rule }),
    ];
    const next = withCustom(current, "2026-03-20T08:00:00Z", {
      every: 1,
      unit: "day",
      count: null,
      until: null,
    });
    expect(next).toHaveLength(2);
    expect(next[1]).toEqual({
      notifyAt: "2026-03-20T08:00:00Z",
      repeat: { every: 1, unit: "day", count: null, until: null },
    });
  });

  it("separates sounded reminders from pending ones", () => {
    const all = [reminder({ id: "s", isSent: true }), reminder({ id: "p" })];
    expect(sent(all).map((r) => r.id)).toEqual(["s"]);
    expect(pending(all).map((r) => r.id)).toEqual(["p"]);
  });
});

describe("draftReminders", () => {
  it("resolves offsets against the draft date and time", () => {
    const [draft] = draftReminders(
      [{ offsetMinutes: 60 }],
      "2026-03-12",
      "10:00",
    );
    expect(new Date(draft.notifyAt).getTime()).toBe(
      new Date("2026-03-12T09:00").getTime(),
    );
    expect(draft).toMatchObject({ offsetMinutes: 60, isSent: false });
  });

  it("keeps custom instants and rules as they are", () => {
    const repeat = { every: 1, unit: "day" as const, count: 3, until: null };
    const [draft] = draftReminders(
      [{ notifyAt: "2026-03-12T08:00:00Z", repeat }],
      null,
      null,
    );
    expect(draft.notifyAt).toBe("2026-03-12T08:00:00Z");
    expect(draft.repeat).toEqual({ ...repeat, occurrence: 1 });
  });
});
