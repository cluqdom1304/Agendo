// Repeat rules of tasks (F13-05, F17-01): the form edits a frequency, an interval, weekdays (weekly)
// and an end (a date or a number of times); the backend stores them as a small subset of RFC 5545
// (`FREQ=WEEKLY;INTERVAL=2;BYDAY=MO,WE;UNTIL=20261231`).
// The backend adds the anchors (BYMONTHDAY, BYMONTH) itself, so they are ignored here.

import { t } from "./i18n";

export type Freq = "DAILY" | "WEEKLY" | "MONTHLY" | "YEARLY";

export const FREQS: readonly Freq[] = ["DAILY", "WEEKLY", "MONTHLY", "YEARLY"];

export const MAX_INTERVAL = 365;
export const MAX_COUNT = 999;

/** Weekday codes in the order the week starts (Monday). */
export const DAY_CODES = ["MO", "TU", "WE", "TH", "FR", "SA", "SU"] as const;
export type DayCode = (typeof DAY_CODES)[number];

export interface RepeatForm {
  freq: Freq;
  interval: number;
  /** `YYYY-MM-DD`, or empty for no end. */
  until: string;
  /** Weekly only; empty means the task's own weekday. */
  days: DayCode[];
  /** Times the series runs including the first; `null` for no limit. */
  count: number | null;
}

/** Integer in `min..=max` written with digits only, as the backend parses it. */
function boundedInt(value: string, min: number, max: number): number | null {
  if (!/^\d+$/.test(value)) return null;
  const number = Number(value);
  return number >= min && number <= max ? number : null;
}

function isCalendarDate(compact: string): boolean {
  const year = Number(compact.slice(0, 4));
  const month = Number(compact.slice(4, 6));
  const day = Number(compact.slice(6, 8));
  const date = new Date(Date.UTC(year, month - 1, day));
  return (
    date.getUTCFullYear() === year &&
    date.getUTCMonth() === month - 1 &&
    date.getUTCDate() === day
  );
}

/**
 * Reads a stored rule; `null` when the backend would not accept it (same checks as
 * `Rule::parse` in domain/recurrence.rs, see src-tauri/fixtures/parity/recurrence.json).
 */
export function parseRule(text: string | null | undefined): RepeatForm | null {
  if (!text) return null;
  let freq: Freq | null = null;
  let interval = 1;
  let until = "";
  let days: DayCode[] = [];
  let count: number | null = null;
  for (const part of text.split(";")) {
    const separator = part.indexOf("=");
    if (separator < 0) return null;
    const key = part.slice(0, separator);
    const value = part.slice(separator + 1);
    if (key === "FREQ") {
      if (!(FREQS as readonly string[]).includes(value)) return null;
      freq = value as Freq;
    } else if (key === "INTERVAL") {
      const parsed = boundedInt(value, 1, MAX_INTERVAL);
      if (parsed === null) return null;
      interval = parsed;
    } else if (key === "UNTIL") {
      if (!/^\d{8}$/.test(value) || !isCalendarDate(value)) return null;
      until = `${value.slice(0, 4)}-${value.slice(4, 6)}-${value.slice(6, 8)}`;
    } else if (key === "COUNT") {
      const parsed = boundedInt(value, 1, MAX_COUNT);
      if (parsed === null) return null;
      count = parsed;
    } else if (key === "BYDAY") {
      const wanted = value.split(",");
      if (
        !wanted.every((code) => (DAY_CODES as readonly string[]).includes(code))
      ) {
        return null;
      }
      days = DAY_CODES.filter((code) => wanted.includes(code));
    } else if (key === "BYMONTHDAY") {
      if (boundedInt(value, 1, 31) === null) return null;
    } else if (key === "BYMONTH") {
      if (boundedInt(value, 1, 12) === null) return null;
    } else {
      return null;
    }
  }
  if (!freq) return null;
  if (days.length > 0 && freq !== "WEEKLY") return null;
  if (until && count !== null) return null;
  return { freq, interval, until, days, count };
}

/** The interval as the field holds it, kept inside what the backend accepts. */
export function clampInterval(value: number): number {
  if (!Number.isFinite(value)) return 1;
  return Math.min(MAX_INTERVAL, Math.max(1, Math.trunc(value)));
}

/** The count as the field holds it, kept inside what the backend accepts. */
export function clampCount(value: number): number {
  if (!Number.isFinite(value)) return 1;
  return Math.min(MAX_COUNT, Math.max(1, Math.trunc(value)));
}

export function buildRule(form: RepeatForm): string {
  const parts = [
    `FREQ=${form.freq}`,
    `INTERVAL=${clampInterval(form.interval)}`,
  ];
  if (form.freq === "WEEKLY" && form.days.length > 0) {
    const chosen = DAY_CODES.filter((code) => form.days.includes(code));
    parts.push(`BYDAY=${chosen.join(",")}`);
  }
  if (form.count !== null) parts.push(`COUNT=${clampCount(form.count)}`);
  else if (form.until) parts.push(`UNTIL=${form.until.replaceAll("-", "")}`);
  return parts.join(";");
}

/** Whether two rules repeat the same way (the stored one also carries backend anchors). */
export function sameRule(
  a: string | null | undefined,
  b: string | null | undefined,
): boolean {
  const left = parseRule(a);
  const right = parseRule(b);
  if (!left || !right) return !left && !right;
  return (
    left.freq === right.freq &&
    left.interval === right.interval &&
    left.until === right.until &&
    left.count === right.count &&
    left.days.join() === right.days.join()
  );
}

/** «Cada semana», «Cada 2 meses hasta el 31/12/2026». */
export function describeRule(text: string | null | undefined): string {
  const form = parseRule(text);
  if (!form) return "";
  const unit = t.recurrence.units[form.freq];
  const base =
    form.interval === 1
      ? unit.one
      : t.recurrence.every(form.interval, unit.many);
  const withDays =
    form.freq === "WEEKLY" && form.days.length > 0
      ? `${base} ${t.recurrence.on(form.days.map((code) => t.recurrence.dayNames[code]).join(", "))}`
      : base;
  if (form.count !== null) {
    return `${withDays}, ${t.recurrence.times(form.count)}`;
  }
  if (!form.until) return withDays;
  const [year, month, day] = form.until.split("-");
  return `${withDays} ${t.recurrence.until(`${day}/${month}/${year}`)}`;
}
