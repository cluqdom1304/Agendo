import { describe, expect, it } from "vitest";

import {
  buildRule,
  clampCount,
  clampInterval,
  describeRule,
  parseRule,
  sameRule,
} from "./recurrence";

describe("recurrence rules", () => {
  it("builds and reads the stored form", () => {
    const text = buildRule({
      freq: "WEEKLY",
      interval: 2,
      until: "2026-12-31",
      days: [],
      count: null,
    });
    expect(text).toBe("FREQ=WEEKLY;INTERVAL=2;UNTIL=20261231");
    expect(parseRule(text)).toEqual({
      freq: "WEEKLY",
      interval: 2,
      until: "2026-12-31",
      days: [],
      count: null,
    });
    expect(
      buildRule({
        freq: "DAILY",
        interval: 1,
        until: "",
        days: [],
        count: null,
      }),
    ).toBe("FREQ=DAILY;INTERVAL=1");
  });

  it("ignores the anchors the backend adds and rejects what it does not write", () => {
    expect(parseRule("FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=31")).toEqual({
      freq: "MONTHLY",
      interval: 1,
      until: "",
      days: [],
      count: null,
    });
    expect(parseRule(null)).toBeNull();
    expect(parseRule("")).toBeNull();
    expect(parseRule("FREQ=HOURLY")).toBeNull();
    expect(parseRule("INTERVAL=2")).toBeNull();
  });

  it("keeps the interval inside what the backend accepts", () => {
    expect(clampInterval(0)).toBe(1);
    expect(clampInterval(999)).toBe(365);
    expect(clampInterval(2.7)).toBe(2);
    expect(clampInterval(Number.NaN)).toBe(1);
  });

  it("compares rules by how they repeat, not by their anchors", () => {
    expect(
      sameRule(
        "FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=31",
        "FREQ=MONTHLY;INTERVAL=1",
      ),
    ).toBe(true);
    expect(sameRule("FREQ=DAILY;INTERVAL=1", "FREQ=DAILY;INTERVAL=2")).toBe(
      false,
    );
    expect(sameRule(null, undefined)).toBe(true);
    expect(sameRule(null, "FREQ=DAILY;INTERVAL=1")).toBe(false);
  });

  it("writes weekdays in week order and a count instead of an end date (F17-01)", () => {
    const text = buildRule({
      freq: "WEEKLY",
      interval: 1,
      until: "2026-12-31",
      days: ["FR", "MO"],
      count: 5,
    });
    expect(text).toBe("FREQ=WEEKLY;INTERVAL=1;BYDAY=MO,FR;COUNT=5");
    expect(parseRule(text)).toEqual({
      freq: "WEEKLY",
      interval: 1,
      until: "",
      days: ["MO", "FR"],
      count: 5,
    });
    // Weekdays only make sense for a weekly rule.
    expect(
      buildRule({
        freq: "DAILY",
        interval: 1,
        until: "",
        days: ["MO"],
        count: null,
      }),
    ).toBe("FREQ=DAILY;INTERVAL=1");
    expect(clampCount(0)).toBe(1);
    expect(clampCount(5000)).toBe(999);
    expect(sameRule("FREQ=WEEKLY;BYDAY=MO", "FREQ=WEEKLY;BYDAY=TU")).toBe(
      false,
    );
    expect(sameRule("FREQ=DAILY;COUNT=3", "FREQ=DAILY;COUNT=2")).toBe(false);
  });

  it("describes a rule in Spanish", () => {
    expect(describeRule("FREQ=DAILY;INTERVAL=1")).toBe("Cada día");
    expect(describeRule("FREQ=WEEKLY;INTERVAL=1")).toBe("Cada semana");
    expect(describeRule("FREQ=MONTHLY;INTERVAL=2;UNTIL=20261231")).toBe(
      "Cada 2 meses hasta el 31/12/2026",
    );
    expect(describeRule("FREQ=YEARLY;INTERVAL=1")).toBe("Cada año");
    expect(describeRule("FREQ=WEEKLY;INTERVAL=1;BYDAY=MO,WE")).toBe(
      "Cada semana los lunes, miércoles",
    );
    expect(describeRule("FREQ=DAILY;INTERVAL=1;COUNT=1")).toBe(
      "Cada día, 1 vez",
    );
    expect(describeRule("FREQ=DAILY;INTERVAL=2;COUNT=10")).toBe(
      "Cada 2 días, 10 veces",
    );
    expect(describeRule(null)).toBe("");
  });
});
