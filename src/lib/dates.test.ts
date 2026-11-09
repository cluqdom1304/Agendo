import { describe, expect, it } from "vitest";

import { isSettledDate } from "./dates";

describe("isSettledDate", () => {
  it("accepts a cleared field and complete years", () => {
    expect(isSettledDate("")).toBe(true);
    expect(isSettledDate("2026-11-23")).toBe(true);
    expect(isSettledDate("1000-01-01")).toBe(true);
  });

  it("refuses the partial years the browser reports while typing", () => {
    for (const year of ["0002", "0020", "0202"]) {
      expect(isSettledDate(`${year}-11-23`)).toBe(false);
    }
  });

  it("refuses text that is not a date", () => {
    expect(isSettledDate("23/11/2026")).toBe(false);
  });
});
