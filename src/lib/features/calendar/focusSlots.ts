// Where each focus session sits in the calendar (D-102). Hours are local wall-clock hours, like the
// grid's, so a day with a clock change keeps its 24 rows.
import { toLocalIsoDate } from "../../dates";
import { t } from "../../i18n";
import { spentLabel } from "../../schedule";
import type { Task, TimeEntry } from "../../types";
import { addDays, minutesToTime } from "./grid";

const DAY_MINUTES = 24 * 60;

/** The part of a session that falls on one day. */
export interface FocusSlot {
  entry: TimeEntry;
  task: Task;
  /** Minutes since midnight. */
  start: number;
  span: number;
  /** Local hours of the whole session, which may start or end on another day. */
  from: string;
  to: string;
  /** Time really spent in the whole session, pauses excluded. */
  seconds: number;
  running: boolean;
}

export interface FocusDays {
  slots: Map<string, FocusSlot[]>;
  /** Seconds per day; a session counts on the day it started, as in the weekly summary. */
  seconds: Map<string, number>;
}

const minuteOfDay = (date: Date) => date.getHours() * 60 + date.getMinutes();

function runningSeconds(entry: TimeEntry, now: Date): number {
  const until = entry.pausedAt ? Date.parse(entry.pausedAt) : now.getTime();
  const elapsed = Math.floor((until - Date.parse(entry.startedAt)) / 1000);
  return Math.max(0, elapsed - entry.pausedSeconds);
}

/** Sessions of tasks absent from `tasks` are left out. A running one reaches up to `now`. */
export function focusDays(
  entries: TimeEntry[],
  tasks: Task[],
  now: Date,
): FocusDays {
  const byId = new Map(tasks.map((task) => [task.id, task]));
  const slots = new Map<string, FocusSlot[]>();
  const seconds = new Map<string, number>();
  for (const entry of entries) {
    const task = byId.get(entry.taskId);
    const running = entry.endedAt === null;
    const spent = running ? runningSeconds(entry, now) : entry.spentSeconds;
    // A session cut short at once spent nothing: it is not worth a block.
    if (!task || (!running && spent === 0)) continue;

    const started = new Date(entry.startedAt);
    // The clock of the view may lag behind a session that has just started.
    const ended = new Date(
      Math.max(
        started.getTime(),
        entry.endedAt ? Date.parse(entry.endedAt) : now.getTime(),
      ),
    );
    const first = toLocalIsoDate(started);
    const last = toLocalIsoDate(ended);
    seconds.set(first, (seconds.get(first) ?? 0) + spent);

    for (let date = first; date <= last; date = addDays(date, 1)) {
      const start = date === first ? minuteOfDay(started) : 0;
      const end = date === last ? minuteOfDay(ended) : DAY_MINUTES;
      // Ending exactly at midnight leaves nothing on the next day.
      if (date !== first && end === 0) continue;
      const day = slots.get(date) ?? [];
      day.push({
        entry,
        task,
        start,
        // An hour repeated by a clock change can end "before" it starts.
        span: Math.max(1, end - start),
        from: minutesToTime(minuteOfDay(started)),
        to: minutesToTime(minuteOfDay(ended)),
        seconds: spent,
        running,
      });
      slots.set(date, day);
    }
  }
  return { slots, seconds };
}

/** «09:00 – 09:30 · 25 min · 5 min en pausa». */
export function focusDetail(slot: FocusSlot): string {
  const parts = [t.calendar.span(slot.from, slot.to), spentLabel(slot.seconds)];
  if (slot.running) parts.push(t.calendar.focusRunning);
  if (slot.entry.pausedSeconds > 0) {
    parts.push(t.calendar.focusPaused(spentLabel(slot.entry.pausedSeconds)));
  }
  return parts.join(" · ");
}
