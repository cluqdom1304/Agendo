// What the views do with the occurrences of events (D-94): spread them over the days they
// cover, name their time, and turn a drag on the calendar into the patch of the whole series.
// Pure, so the store and the components only derive from it.
import type { CalendarEvent, EventOccurrence, EventPatch } from "../../types";
import {
  MINUTES_PER_DAY,
  SNAP_MINUTES,
  addDays,
  daysBetween,
  minutesToTime,
  timeToMinutes,
} from "../calendar/grid";

/** Minutes an event takes on the grid when its end is not known. */
const FALLBACK_MINUTES = 60;

/** Orders a day: all-day events first, then by start time, then by title. */
export function compareOccurrences(
  a: EventOccurrence,
  b: EventOccurrence,
): number {
  return (
    Number(b.allDay) - Number(a.allDay) ||
    (a.startTime ?? "").localeCompare(b.startTime ?? "") ||
    a.title.localeCompare(b.title, "es")
  );
}

/** The occurrences by every local day they cover inside `from`..`to`, each day in order. */
export function occurrencesByDate(
  occurrences: readonly EventOccurrence[],
  from: string,
  to: string,
): Map<string, EventOccurrence[]> {
  const days = new Map<string, EventOccurrence[]>();
  for (const occurrence of occurrences) {
    const first = occurrence.startDate < from ? from : occurrence.startDate;
    const last = occurrence.endDate > to ? to : occurrence.endDate;
    for (let day = first; day <= last; day = addDays(day, 1)) {
      const list = days.get(day);
      if (list) list.push(occurrence);
      else days.set(day, [occurrence]);
    }
  }
  for (const list of days.values()) list.sort(compareOccurrences);
  return days;
}

/** Whether the occurrence is a block of the time grid: timed and inside a single day. */
export function isTimedBlock(occurrence: EventOccurrence): boolean {
  return (
    !occurrence.allDay &&
    occurrence.startTime !== null &&
    occurrence.startDate === occurrence.endDate
  );
}

/** Start and length, in minutes, of a timed occurrence within its day. */
export function blockOf(occurrence: EventOccurrence): {
  start: number;
  span: number;
} {
  const start = timeToMinutes(occurrence.startTime ?? "00:00");
  const end = occurrence.endTime
    ? timeToMinutes(occurrence.endTime)
    : start + FALLBACK_MINUTES;
  return { start, span: Math.max(SNAP_MINUTES, end - start) };
}

/** `10:00 – 11:30`, the start alone, or `null` for an event of the whole day. */
export function timeRangeOf(occurrence: EventOccurrence): string | null {
  if (occurrence.allDay || occurrence.startTime === null) return null;
  return occurrence.endTime && occurrence.endDate === occurrence.startDate
    ? `${occurrence.startTime} – ${occurrence.endTime}`
    : occurrence.startTime;
}

/** The patch that moves the whole series so that the occurrence starting on `occurrenceDate` starts at `startMinutes` on `date`. */
export function movePatch(
  master: CalendarEvent,
  occurrenceDate: string,
  date: string,
  startMinutes: number,
): EventPatch {
  const delta = daysBetween(occurrenceDate, date);
  const start = Math.min(Math.max(startMinutes, 0), MINUTES_PER_DAY - 1);
  const sameDay = master.startDate === master.endDate;
  let endTime = master.endTime;
  if (sameDay && master.startTime !== null && master.endTime !== null) {
    const length =
      timeToMinutes(master.endTime) - timeToMinutes(master.startTime);
    endTime = minutesToTime(Math.min(start + length, MINUTES_PER_DAY - 1));
  }
  return {
    startDate: addDays(master.startDate, delta),
    endDate: addDays(master.endDate, delta),
    startTime: minutesToTime(start),
    ...(endTime === null ? {} : { endTime }),
  };
}

/** The patch that ends the event at `endMinutes` (never before it starts). */
export function resizePatch(
  master: CalendarEvent,
  endMinutes: number,
): EventPatch {
  const start = timeToMinutes(master.startTime ?? "00:00");
  const end = Math.min(
    Math.max(endMinutes, start + SNAP_MINUTES),
    MINUTES_PER_DAY - 1,
  );
  return { endTime: minutesToTime(end) };
}
