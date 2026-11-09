import { describe, expect, it } from "vitest";

import {
  durationBetween,
  durationLabel,
  endTimeOf,
  maxDuration,
  timeRange,
} from "./schedule";

describe("schedule", () => {
  it("computes the end of a task from its start and duration", () => {
    expect(endTimeOf("09:00", 15)).toBe("09:15");
    expect(endTimeOf("09:45", 90)).toBe("11:15");
    expect(endTimeOf("23:00", 60)).toBe("24:00");
  });

  it("limits the duration to the end of the day", () => {
    expect(maxDuration("00:00")).toBe(1440);
    expect(maxDuration("23:30")).toBe(30);
  });

  it("measures the time between a start and an end on the same day", () => {
    expect(durationBetween("09:00", "10:30")).toBe(90);
    expect(durationBetween("09:00", "09:00")).toBeNull();
    expect(durationBetween("10:00", "09:00")).toBeNull();
    expect(durationBetween("", "09:00")).toBeNull();
  });

  it("names durations the way a person says them", () => {
    expect(durationLabel(15)).toBe("15 min");
    expect(durationLabel(60)).toBe("1 h");
    expect(durationLabel(90)).toBe("1 h 30 min");
  });

  it("shows a range only when there is a duration", () => {
    expect(timeRange("09:00", null)).toBe("09:00");
    expect(timeRange("09:00", 45)).toBe("09:00–09:45");
  });
});
