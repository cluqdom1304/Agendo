// Calendar arithmetic on local calendar dates (`YYYY-MM-DD`). Days are counted in UTC so a
// daylight-saving change can never move a date: the zone only matters for "today", which the
// caller reads once with `toLocalIsoDate`.
import type { Task } from "../../types";

/** 0 = Sunday … 6 = Saturday, like `Date.getDay`. */
export type WeekDay = 0 | 1 | 2 | 3 | 4 | 5 | 6;

const DAY_MS = 86_400_000;
const pad = (value: number, width = 2) => String(value).padStart(width, "0");

function toUtc(iso: string): number {
  const [year, month, day] = iso.split("-").map(Number);
  return Date.UTC(year, month - 1, day);
}

function fromUtc(ms: number): string {
  const date = new Date(ms);
  return `${pad(date.getUTCFullYear(), 4)}-${pad(date.getUTCMonth() + 1)}-${pad(date.getUTCDate())}`;
}

export function addDays(iso: string, days: number): string {
  return fromUtc(toUtc(iso) + days * DAY_MS);
}

/** Same day of another month, or the last day when that month is shorter (31 Jan + 1 → 28 Feb). */
export function addMonths(iso: string, months: number): string {
  const [year, month, day] = iso.split("-").map(Number);
  const index = year * 12 + (month - 1) + months;
  const targetYear = Math.floor(index / 12);
  const targetMonth = index % 12;
  const lastDay = new Date(
    Date.UTC(targetYear, targetMonth + 1, 0),
  ).getUTCDate();
  return `${pad(targetYear, 4)}-${pad(targetMonth + 1)}-${pad(Math.min(day, lastDay))}`;
}

export function weekdayOf(iso: string): WeekDay {
  return new Date(toUtc(iso)).getUTCDay() as WeekDay;
}

export function startOfWeek(iso: string, firstDay: WeekDay): string {
  return addDays(iso, -((weekdayOf(iso) - firstDay + 7) % 7));
}

/** The seven dates of the week containing `iso`. */
export function weekDates(iso: string, firstDay: WeekDay): string[] {
  const start = startOfWeek(iso, firstDay);
  return Array.from({ length: 7 }, (_, index) => addDays(start, index));
}

/** Whole weeks covering the month of `iso` (4 to 6 rows), including days of the neighbours. */
export function monthGrid(iso: string, firstDay: WeekDay): string[][] {
  const first = `${iso.slice(0, 7)}-01`;
  const last = addDays(addMonths(first, 1), -1);
  const weeks: string[][] = [];
  for (
    let start = startOfWeek(first, firstDay);
    start <= last;
    start = addDays(start, 7)
  ) {
    weeks.push(weekDates(start, firstDay));
  }
  return weeks;
}

export function isSameMonth(a: string, b: string): boolean {
  return a.slice(0, 7) === b.slice(0, 7);
}

interface WeekInfo {
  firstDay: number;
}

/** The week starts where the system says (Windows setting through `Intl`); Monday if unknown. */
export function systemFirstDay(
  locale: string = typeof navigator === "undefined" ? "es" : navigator.language,
): WeekDay {
  try {
    const intlLocale = new Intl.Locale(locale) as Intl.Locale & {
      getWeekInfo?: () => WeekInfo;
      weekInfo?: WeekInfo;
    };
    const info = intlLocale.getWeekInfo?.() ?? intlLocale.weekInfo;
    if (info && info.firstDay >= 1 && info.firstDay <= 7) {
      return (info.firstDay % 7) as WeekDay;
    }
  } catch {
    // An invalid locale tag falls through to the default.
  }
  return 1;
}

// ---- time of day ----------------------------------------------------------------------

export const MINUTES_PER_DAY = 24 * 60;
/** Dragging snaps to this many minutes. */
export const SNAP_MINUTES = 15;

export function timeToMinutes(time: string): number {
  const [hours, minutes] = time.split(":").map(Number);
  return hours * 60 + minutes;
}

export function minutesToTime(minutes: number): string {
  const clamped = Math.min(
    Math.max(Math.round(minutes), 0),
    MINUTES_PER_DAY - 1,
  );
  return `${pad(Math.floor(clamped / 60))}:${pad(clamped % 60)}`;
}

/** The quarter hour a pointer position falls in, as its start minute. */
export function quarterOf(minutes: number): number {
  const start = Math.floor(minutes / SNAP_MINUTES) * SNAP_MINUTES;
  return Math.min(Math.max(start, 0), MINUTES_PER_DAY - SNAP_MINUTES);
}

/** The span between two pointer positions, ordered (dragging upwards works too). */
export function snapSpan(
  fromMinutes: number,
  toMinutes: number,
): { start: number; end: number } {
  const a = quarterOf(fromMinutes);
  const b = quarterOf(toMinutes);
  return { start: Math.min(a, b), end: Math.max(a, b) + SNAP_MINUTES };
}

// ---- layout of one day's timed tasks ---------------------------------------------------

/** Minutes a timed task occupies on the grid: tasks have a start time but no duration. */
export const BLOCK_MINUTES = 30;

export interface PlacedTask {
  task: Task;
  start: number;
  lane: number;
  lanes: number;
}

/** Tasks with a time, side by side when their blocks would overlap. */
export function layoutDay(tasks: readonly Task[]): PlacedTask[] {
  const timed = tasks
    .filter((task) => task.dueTime !== null)
    .map((task) => ({ task, start: timeToMinutes(task.dueTime ?? "00:00") }))
    .sort((a, b) => a.start - b.start || (a.task.id < b.task.id ? -1 : 1));

  const placed: PlacedTask[] = [];
  let cluster: PlacedTask[] = [];
  let clusterEnd = -1;
  const laneEnds: number[] = [];

  const closeCluster = () => {
    for (const item of cluster) item.lanes = laneEnds.length;
    cluster = [];
    laneEnds.length = 0;
  };

  for (const { task, start } of timed) {
    if (start >= clusterEnd) closeCluster();
    let lane = laneEnds.findIndex((end) => end <= start);
    if (lane < 0) lane = laneEnds.length;
    laneEnds[lane] = start + BLOCK_MINUTES;
    clusterEnd = Math.max(clusterEnd, start + BLOCK_MINUTES);
    const item: PlacedTask = { task, start, lane, lanes: 1 };
    cluster.push(item);
    placed.push(item);
  }
  closeCluster();
  return placed;
}

/** Dated tasks by day, in the order given. */
export function tasksByDate(tasks: readonly Task[]): Map<string, Task[]> {
  const days = new Map<string, Task[]>();
  for (const task of tasks) {
    if (task.dueDate === null) continue;
    const day = days.get(task.dueDate);
    if (day) day.push(task);
    else days.set(task.dueDate, [task]);
  }
  return days;
}
