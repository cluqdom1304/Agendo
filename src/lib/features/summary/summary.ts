// Pure helpers of the weekly summary (F26-02).

import { spentLabel } from "../../schedule";
import { addDays } from "../calendar/grid";
import type { HabitWeek, TagTally, WeekSummary } from "../../types";

const monthShort = new Intl.DateTimeFormat("es-ES", {
  day: "numeric",
  month: "short",
});
const monthLong = new Intl.DateTimeFormat("es-ES", {
  day: "numeric",
  month: "short",
  year: "numeric",
});
const asDate = (iso: string) => new Date(`${iso}T12:00:00`);

/** «9 mar – 15 mar 2026». */
export function weekLabel(start: string, end: string): string {
  return `${monthShort.format(asDate(start))} – ${monthLong.format(asDate(end))}`;
}

/** The day a week navigation step lands on: seven days away from the one shown. */
export function shiftWeek(anchor: string, weeks: number): string {
  return addDays(anchor, weeks * 7);
}

/** Whole percentage of `done` over `total`; 0 when there is nothing to do. */
export function percent(done: number, total: number): number {
  return total <= 0 ? 0 : Math.round((done * 100) / total);
}

/** «23 s», «5 min 20 s» or «1 h 25 min»: the unit that fits the time, as in the task rows. */
export function focusLabel(seconds: number): string {
  return spentLabel(seconds);
}

/** Whether nothing at all happened in the week. */
export function isQuiet(summary: WeekSummary): boolean {
  return (
    summary.completed + summary.pending + summary.overdue === 0 &&
    summary.events === 0 &&
    summary.habits.length === 0 &&
    summary.focusSeconds === 0
  );
}

export interface TagRow extends TagTally {
  name: string;
  color: string | null;
}

/** Tag rows by name; `name` of a tag that no longer exists is the fallback. */
export function tagRows(
  tallies: TagTally[],
  lookup: (id: string) => { name: string; color: string | null } | undefined,
  labels: { untagged: string; gone: string },
): TagRow[] {
  return tallies
    .map((tally) => {
      if (tally.tagId === null) {
        return { ...tally, name: labels.untagged, color: null };
      }
      const tag = lookup(tally.tagId);
      return {
        ...tally,
        name: tag?.name ?? labels.gone,
        color: tag?.color ?? null,
      };
    })
    .sort((a, b) => {
      if ((a.tagId === null) !== (b.tagId === null)) {
        return a.tagId === null ? 1 : -1;
      }
      return a.name.localeCompare(b.name, "es");
    });
}

/** Habits, the least kept first, so what slipped is what shows first. */
export function habitRows(habits: HabitWeek[]): (HabitWeek & {
  percent: number;
})[] {
  return habits
    .map((habit) => ({
      ...habit,
      percent: percent(habit.done, habit.scheduled),
    }))
    .sort(
      (a, b) => a.percent - b.percent || a.title.localeCompare(b.title, "es"),
    );
}
