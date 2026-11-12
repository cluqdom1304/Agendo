import { describe, expect, it } from "vitest";

import { formatNoteDate, formatNoteStamp } from "./format";

const now = new Date("2026-03-10T15:00:00");

describe("formatNoteDate", () => {
  it("shows the time for today", () => {
    expect(formatNoteDate("2026-03-10T09:05:00", now)).toMatch(/09:05/);
  });

  it("says yesterday", () => {
    expect(formatNoteDate("2026-03-09T12:00:00", now)).toBe("ayer");
  });

  it("shows day and month this year, and the year before that", () => {
    expect(formatNoteDate("2026-01-05T12:00:00", now)).not.toMatch(/2026/);
    expect(formatNoteDate("2025-01-05T12:00:00", now)).toMatch(/2025/);
  });

  it("returns an empty text for an invalid date", () => {
    expect(formatNoteDate("nope", now)).toBe("");
  });
});

describe("formatNoteStamp", () => {
  it("shows the whole date and the time", () => {
    const stamp = formatNoteStamp("2026-03-10T09:05:00");
    expect(stamp).toMatch(/10 de marzo de 2026/);
    expect(stamp).toMatch(/9:05/);
  });

  it("returns an empty text for an invalid date", () => {
    expect(formatNoteStamp("nope")).toBe("");
  });
});
