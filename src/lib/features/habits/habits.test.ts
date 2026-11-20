import { describe, expect, it } from "vitest";

import {
  ALL_DAYS,
  daysLabel,
  doneKeys,
  indexesOf,
  isScheduled,
  lastDays,
  maskOf,
} from "./habits";

describe("habits (D-95)", () => {
  it("lists the last days ending today, oldest first", () => {
    expect(lastDays("2026-03-10", 3)).toEqual([
      "2026-03-08",
      "2026-03-09",
      "2026-03-10",
    ]);
    expect(lastDays("2026-03-02")).toHaveLength(7);
    expect(lastDays("2026-03-02")[0]).toBe("2026-02-24");
  });

  it("reads the weekday mask with Monday as bit 0", () => {
    const mondays = maskOf([0]);
    expect(isScheduled(mondays, "2026-03-09")).toBe(true); // lunes
    expect(isScheduled(mondays, "2026-03-10")).toBe(false);
    const sunday = maskOf([6]);
    expect(isScheduled(sunday, "2026-03-15")).toBe(true); // domingo
    expect(isScheduled(ALL_DAYS, "2026-03-11")).toBe(true);
  });

  it("converts between the mask and the chosen days", () => {
    expect(maskOf([0, 2, 4])).toBe(0b10101);
    expect(indexesOf(0b10101)).toEqual([0, 2, 4]);
    expect(indexesOf(ALL_DAYS)).toEqual([0, 1, 2, 3, 4, 5, 6]);
  });

  it("names the days", () => {
    expect(daysLabel(ALL_DAYS)).toBe("Todos los días");
    expect(daysLabel(maskOf([0, 2, 4]))).toBe("lun, mié, vie");
  });

  it("turns the marks into keys", () => {
    const keys = doneKeys([{ habitId: "h1", date: "2026-03-10" }]);
    expect(keys.has("h1|2026-03-10")).toBe(true);
    expect(keys.has("h1|2026-03-09")).toBe(false);
  });
});
