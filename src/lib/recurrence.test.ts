import { describe, expect, it } from "vitest";

import {
  buildRule,
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
    });
    expect(text).toBe("FREQ=WEEKLY;INTERVAL=2;UNTIL=20261231");
    expect(parseRule(text)).toEqual({
      freq: "WEEKLY",
      interval: 2,
      until: "2026-12-31",
    });
    expect(buildRule({ freq: "DAILY", interval: 1, until: "" })).toBe(
      "FREQ=DAILY;INTERVAL=1",
    );
  });

  it("ignores the anchors the backend adds and rejects what it does not write", () => {
    expect(parseRule("FREQ=MONTHLY;INTERVAL=1;BYMONTHDAY=31")).toEqual({
      freq: "MONTHLY",
      interval: 1,
      until: "",
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

  it("describes a rule in Spanish", () => {
    expect(describeRule("FREQ=DAILY;INTERVAL=1")).toBe("Cada día");
    expect(describeRule("FREQ=WEEKLY;INTERVAL=1")).toBe("Cada semana");
    expect(describeRule("FREQ=MONTHLY;INTERVAL=2;UNTIL=20261231")).toBe(
      "Cada 2 meses hasta el 31/12/2026",
    );
    expect(describeRule("FREQ=YEARLY;INTERVAL=1")).toBe("Cada año");
    expect(describeRule(null)).toBe("");
  });
});
