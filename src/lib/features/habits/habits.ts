// What the habit views derive from the habits and their marks (D-95). Pure, so the store and
// the components only call it.
import { t } from "../../i18n";
import { DAY_CODES } from "../../recurrence";
import type { HabitLog } from "../../types";
import { addDays, weekdayOf } from "../calendar/grid";

/** The mask of a habit that is scheduled every day. */
export const ALL_DAYS = 127;
/** Days the habit view shows, today included. */
export const WEEK_DAYS = 7;

/** The `count` days ending on `today`, oldest first. */
export function lastDays(today: string, count = WEEK_DAYS): string[] {
  return Array.from({ length: count }, (_, index) =>
    addDays(today, index - count + 1),
  );
}

/** Bit 0 is Monday ... bit 6 is Sunday, as in the backend. */
function bitOf(date: string): number {
  return 1 << ((weekdayOf(date) + 6) % 7);
}

export function isScheduled(days: number, date: string): boolean {
  return (days & bitOf(date)) !== 0;
}

export function maskOf(selected: readonly number[]): number {
  return selected.reduce((mask, index) => mask | (1 << index), 0);
}

export function indexesOf(mask: number): number[] {
  return DAY_CODES.flatMap((_, index) => ((mask >> index) & 1 ? [index] : []));
}

export const logKey = (habitId: string, date: string) => `${habitId}|${date}`;

/** The marked days as a set of `logKey`s. */
export function doneKeys(logs: readonly HabitLog[]): Set<string> {
  return new Set(logs.map((log) => logKey(log.habitId, log.date)));
}

/** «Todos los días», or the weekdays in short: «lun, mié, vie». */
export function daysLabel(mask: number): string {
  if (mask === ALL_DAYS) return t.habits.everyDay;
  return indexesOf(mask)
    .map((index) => t.recurrence.dayNames[DAY_CODES[index]].slice(0, 3))
    .join(", ");
}
