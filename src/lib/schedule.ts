// Start time + duration of a task (D-33). The end is never stored: it is the start plus the
// duration, always on the same day.

/** Quick choices of the task dialog; any other length is typed as an end time. */
export const DURATION_PRESETS = [15, 30, 60] as const;

export const MINUTES_PER_DAY = 24 * 60;

const pad = (value: number) => String(value).padStart(2, "0");

function toMinutes(time: string): number {
  const [hours, minutes] = time.split(":").map(Number);
  return hours * 60 + minutes;
}

function toClock(minutes: number): string {
  return `${pad(Math.floor(minutes / 60))}:${pad(minutes % 60)}`;
}

/** End of a task as `HH:MM` ("24:00" when it lasts until midnight). */
export function endTimeOf(start: string, duration: number): string {
  return toClock(Math.min(toMinutes(start) + duration, MINUTES_PER_DAY));
}

/** Longest duration that still ends the same day. */
export function maxDuration(start: string): number {
  return MINUTES_PER_DAY - toMinutes(start);
}

/** Minutes from `start` to `end` (`HH:MM`), or `null` if `end` is not after `start`. */
export function durationBetween(start: string, end: string): number | null {
  if (!/^\d{2}:\d{2}$/.test(start) || !/^\d{2}:\d{2}$/.test(end)) return null;
  const minutes = toMinutes(end) - toMinutes(start);
  return minutes > 0 ? minutes : null;
}

/** "15 min", "1 h", "1 h 30 min". */
export function durationLabel(minutes: number): string {
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  if (hours === 0) return `${rest} min`;
  return rest === 0 ? `${hours} h` : `${hours} h ${rest} min`;
}

/** "09:00" alone, or "09:00–10:30" when the task has a duration. */
export function timeRange(start: string, duration: number | null): string {
  return duration === null ? start : `${start}–${endTimeOf(start, duration)}`;
}
