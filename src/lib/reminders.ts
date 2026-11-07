import { t } from "./i18n";
import type { Reminder, ReminderInput } from "./types";

/** The "before" options of F-12, in minutes. 0 is "En el momento". */
export const PRESET_OFFSETS = [0, 15, 60, 1440] as const;

export function offsetLabel(minutes: number): string {
  if (minutes === 0) return t.reminders.atTime;
  if (minutes % 1440 === 0) return t.reminders.daysBefore(minutes / 1440);
  if (minutes % 60 === 0) return t.reminders.hoursBefore(minutes / 60);
  return t.reminders.minutesBefore(minutes);
}

/** Reminders that can still fire; sent ones are history (the backend keeps them). */
export function pending(reminders: Reminder[]): Reminder[] {
  return reminders.filter((reminder) => !reminder.isSent);
}

/** What `reminders_set` needs to keep `reminder` as it is. */
export function toInput(reminder: Reminder): ReminderInput {
  return reminder.offsetMinutes === null
    ? { notifyAt: reminder.notifyAt }
    : { offsetMinutes: reminder.offsetMinutes };
}

/** The task's local date and time as an instant, or `null` if it has no time. */
export function dueInstant(
  dueDate: string | null,
  dueTime: string | null,
): Date | null {
  if (!dueDate || !dueTime) return null;
  const instant = new Date(`${dueDate}T${dueTime}`);
  return Number.isNaN(instant.getTime()) ? null : instant;
}

/** An option is offered only if its moment is still ahead (the backend refuses the rest). */
export function offsetIsAhead(
  due: Date | null,
  minutes: number,
  now: Date,
): boolean {
  return due !== null && due.getTime() - minutes * 60_000 > now.getTime();
}

/** A `datetime-local` value (`YYYY-MM-DDTHH:MM`, local time) as a backend UTC instant. */
export function customToInstant(value: string): string | null {
  const date = new Date(value);
  if (!value || Number.isNaN(date.getTime())) return null;
  return date.toISOString().replace(/\.\d{3}Z$/, "Z");
}

export function formatMoment(notifyAt: string): string {
  return new Date(notifyAt).toLocaleString("es-ES", {
    dateStyle: "short",
    timeStyle: "short",
  });
}

/** The set after toggling a preset: added if absent, removed if present. */
export function togglePreset(
  current: Reminder[],
  minutes: number,
): ReminderInput[] {
  const active = pending(current);
  const kept = active.filter((reminder) => reminder.offsetMinutes !== minutes);
  const inputs = kept.map(toInput);
  return kept.length === active.length
    ? [...inputs, { offsetMinutes: minutes }]
    : inputs;
}

export function withoutReminder(
  current: Reminder[],
  id: string,
): ReminderInput[] {
  return pending(current)
    .filter((reminder) => reminder.id !== id)
    .map(toInput);
}

export function withCustom(
  current: Reminder[],
  notifyAt: string,
): ReminderInput[] {
  return [...pending(current).map(toInput), { notifyAt }];
}
