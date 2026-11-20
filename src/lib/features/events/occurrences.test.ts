import { describe, expect, it } from "vitest";

import type { CalendarEvent, EventOccurrence } from "../../types";
import {
  blockOf,
  isTimedBlock,
  movePatch,
  occurrencesByDate,
  resizePatch,
  timeRangeOf,
} from "./occurrences";

function occurrence(overrides: Partial<EventOccurrence> = {}): EventOccurrence {
  return {
    eventId: "e1",
    title: "Reunión",
    startDate: "2026-03-10",
    startTime: "10:00",
    endDate: "2026-03-10",
    endTime: "11:00",
    allDay: false,
    recurring: false,
    tagIds: [],
    ...overrides,
  };
}

function master(overrides: Partial<CalendarEvent> = {}): CalendarEvent {
  return {
    id: "e1",
    title: "Reunión",
    notes: "",
    startDate: "2026-03-10",
    startTime: "10:00",
    endDate: "2026-03-10",
    endTime: "11:00",
    allDay: false,
    rrule: null,
    tagIds: [],
    reminderOffsets: [],
    createdAt: "",
    updatedAt: "",
    ...overrides,
  };
}

describe("occurrencesByDate", () => {
  it("spreads a multi-day event over its days, clipped to the window", () => {
    const days = occurrencesByDate(
      [
        occurrence({
          allDay: true,
          startTime: null,
          endTime: null,
          startDate: "2026-03-09",
          endDate: "2026-03-12",
        }),
      ],
      "2026-03-10",
      "2026-03-11",
    );
    expect([...days.keys()]).toEqual(["2026-03-10", "2026-03-11"]);
  });

  it("puts all-day events first, then by time and title", () => {
    const days = occurrencesByDate(
      [
        occurrence({ eventId: "b", title: "B", startTime: "12:00" }),
        occurrence({ eventId: "a", title: "A", startTime: "09:00" }),
        occurrence({
          eventId: "c",
          title: "C",
          allDay: true,
          startTime: null,
          endTime: null,
        }),
      ],
      "2026-03-10",
      "2026-03-10",
    );
    expect(days.get("2026-03-10")?.map((item) => item.eventId)).toEqual([
      "c",
      "a",
      "b",
    ]);
  });
});

describe("blocks", () => {
  it("only single-day timed occurrences are blocks of the grid", () => {
    expect(isTimedBlock(occurrence())).toBe(true);
    expect(isTimedBlock(occurrence({ allDay: true, startTime: null }))).toBe(
      false,
    );
    expect(isTimedBlock(occurrence({ endDate: "2026-03-11" }))).toBe(false);
  });

  it("measures the block, with an hour when the end is missing", () => {
    expect(blockOf(occurrence())).toEqual({ start: 600, span: 60 });
    expect(blockOf(occurrence({ endTime: null }))).toEqual({
      start: 600,
      span: 60,
    });
  });

  it("names the time", () => {
    expect(timeRangeOf(occurrence())).toBe("10:00 – 11:00");
    expect(timeRangeOf(occurrence({ allDay: true, startTime: null }))).toBe(
      null,
    );
    expect(timeRangeOf(occurrence({ endDate: "2026-03-11" }))).toBe("10:00");
  });
});

describe("movePatch", () => {
  it("keeps the length and shifts the series by the days moved", () => {
    expect(movePatch(master(), "2026-03-10", "2026-03-12", 14 * 60)).toEqual({
      startDate: "2026-03-12",
      endDate: "2026-03-12",
      startTime: "14:00",
      endTime: "15:00",
    });
  });

  it("measures the days from the occurrence, not from the first date of the series", () => {
    const series = master({ startDate: "2026-03-03", endDate: "2026-03-03" });
    expect(movePatch(series, "2026-03-10", "2026-03-11", 600)).toMatchObject({
      startDate: "2026-03-04",
      endDate: "2026-03-04",
    });
  });

  it("does not run past the end of the day", () => {
    expect(
      movePatch(master(), "2026-03-10", "2026-03-10", 23 * 60 + 30),
    ).toMatchObject({ startTime: "23:30", endTime: "23:59" });
  });
});

describe("resizePatch", () => {
  it("sets the end, at least a step after the start", () => {
    expect(resizePatch(master(), 12 * 60)).toEqual({ endTime: "12:00" });
    expect(resizePatch(master(), 9 * 60)).toEqual({ endTime: "10:15" });
  });
});
