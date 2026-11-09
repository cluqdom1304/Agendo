import { t } from "./i18n";
import { formatDisplayDate } from "./dates";
import type { Reminder, ReminderInput, Repeat } from "./types";

/** The "before" options of F-12, in minutes. 0 is "En el momento". */
export const PRESET_OFFSETS = [0, 15, 60, 1440, 10080] as const;

/** A task without a time can only be warned about whole days ahead. */
export function presetOffsets(hasTime: boolean): readonly number[] {
  return hasTime
    ? PRESET_OFFSETS
    : PRESET_OFFSETS.filter((minutes) => minutes % 1440 === 0);
}

export function offsetLabel(minutes: number, untimed = false): string {
  if (minutes === 0) return untimed ? t.reminders.sameDay : t.reminders.atTime;
  if (minutes % 10080 === 0) return t.reminders.weeksBefore(minutes / 10080);
  if (minutes % 1440 === 0) return t.reminders.daysBefore(minutes / 1440);
  if (minutes % 60 === 0) return t.reminders.hoursBefore(minutes / 60);
  return t.reminders.minutesBefore(minutes);
}

/** Reminders that can still fire; sent ones are history (the backend keeps them). */
export function pending(reminders: Reminder[]): Reminder[] {
  return reminders.filter((reminder) => !reminder.isSent);
}

/** Reminders that already sounded (or were skipped): kept as history, shown greyed out. */
export function sent(reminders: Reminder[]): Reminder[] {
  return reminders.filter((reminder) => reminder.isSent);
}

/** What `reminders_set` needs to keep `reminder` as it is. */
export function toInput(reminder: Reminder): ReminderInput {
  if (reminder.offsetMinutes !== null) {
    return { offsetMinutes: reminder.offsetMinutes };
  }
  return reminder.repeat
    ? { notifyAt: reminder.notifyAt, repeat: reminder.repeat }
    : { notifyAt: reminder.notifyAt };
}

/** "Cada día", "Cada 2 semanas, 5 veces", "Cada mes, hasta el 10/04/2026". */
export function describeRepeat(repeat: Omit<Repeat, "occurrence">): string {
  const unit = t.reminders.units[repeat.unit];
  const every =
    repeat.every === 1
      ? unit.one
      : t.reminders.everyMany(repeat.every, unit.many);
  const end =
    repeat.count !== null
      ? `, ${t.reminders.times(repeat.count)}`
      : repeat.until !== null
        ? `, ${t.reminders.until(formatDisplayDate(repeat.until))}`
        : "";
  return `${every}${end}`;
}

/**
 * The task's local date and time as an instant. Without a time the default notification time
 * stands in (only to place reminders, never the task), or `null` if there is none.
 */
export function dueInstant(
  dueDate: string | null,
  dueTime: string | null,
  defaultTime: string | null = null,
): Date | null {
  const time = dueTime ?? defaultTime;
  if (!dueDate || !time) return null;
  const instant = new Date(`${dueDate}T${time}`);
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
  repeat?: Omit<Repeat, "occurrence">,
): ReminderInput[] {
  const added: ReminderInput = repeat ? { notifyAt, repeat } : { notifyAt };
  return [...pending(current).map(toInput), added];
}

/**
 * The reminders of a task that does not exist yet, shaped like stored ones so the editor that
 * works on saved tasks can show them. Offsets are resolved against the draft's date and time.
 */
export function draftReminders(
  inputs: ReminderInput[],
  dueDate: string | null,
  dueTime: string | null,
  defaultTime: string | null = null,
): Reminder[] {
  const due = dueInstant(dueDate, dueTime, defaultTime);
  return inputs.map((input, index) => {
    const offset = input.offsetMinutes ?? null;
    const at =
      offset !== null && due
        ? new Date(due.getTime() - offset * 60_000).toISOString()
        : (input.notifyAt ?? new Date(0).toISOString());
    return {
      id: `draft-${index}`,
      notifyAt: at.replace(/\.\d{3}Z$/, "Z"),
      offsetMinutes: offset,
      isSent: false,
      repeat: input.repeat ? { occurrence: 1, ...input.repeat } : null,
    };
  });
}
