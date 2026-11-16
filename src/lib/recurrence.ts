// Repeat rules of tasks (F13-05): the form edits a frequency, an interval and an end date; the
// backend stores them as a small subset of RFC 5545 (`FREQ=WEEKLY;INTERVAL=2;UNTIL=20261231`).
// The backend adds the anchors (BYMONTHDAY, BYMONTH) itself, so they are ignored here.

import { t } from "./i18n";

export type Freq = "DAILY" | "WEEKLY" | "MONTHLY" | "YEARLY";

export const FREQS: readonly Freq[] = ["DAILY", "WEEKLY", "MONTHLY", "YEARLY"];

export const MAX_INTERVAL = 365;

export interface RepeatForm {
  freq: Freq;
  interval: number;
  /** `YYYY-MM-DD`, or empty for no end. */
  until: string;
}

/** Reads a stored rule; `null` when it is not one this app writes. */
export function parseRule(text: string | null | undefined): RepeatForm | null {
  if (!text) return null;
  let freq: Freq | null = null;
  let interval = 1;
  let until = "";
  for (const part of text.split(";")) {
    const [key, value = ""] = part.split("=");
    if (key === "FREQ" && (FREQS as readonly string[]).includes(value)) {
      freq = value as Freq;
    } else if (key === "INTERVAL") {
      interval = Number(value);
    } else if (key === "UNTIL" && /^\d{8}$/.test(value)) {
      until = `${value.slice(0, 4)}-${value.slice(4, 6)}-${value.slice(6, 8)}`;
    }
  }
  if (!freq || !Number.isInteger(interval) || interval < 1) return null;
  return { freq, interval, until };
}

/** The interval as the field holds it, kept inside what the backend accepts. */
export function clampInterval(value: number): number {
  if (!Number.isFinite(value)) return 1;
  return Math.min(MAX_INTERVAL, Math.max(1, Math.trunc(value)));
}

export function buildRule(form: RepeatForm): string {
  const parts = [
    `FREQ=${form.freq}`,
    `INTERVAL=${clampInterval(form.interval)}`,
  ];
  if (form.until) parts.push(`UNTIL=${form.until.replaceAll("-", "")}`);
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
    left.until === right.until
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
  if (!form.until) return base;
  const [year, month, day] = form.until.split("-");
  return `${base} ${t.recurrence.until(`${day}/${month}/${year}`)}`;
}
