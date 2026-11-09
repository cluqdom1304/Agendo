// Display text for calendar dates. Dates are formatted as UTC instants of their own calendar
// day, so the machine's zone and daylight saving never change the day shown.
import { formatDisplayDate } from "../../dates";
import type { ViewMode } from "../../types";
import { weekDates, type WeekDay } from "./grid";

const LOCALE = "es-ES";

function utcDate(iso: string): Date {
  const [year, month, day] = iso.split("-").map(Number);
  return new Date(Date.UTC(year, month - 1, day));
}

const weekdayShort = new Intl.DateTimeFormat(LOCALE, {
  weekday: "short",
  timeZone: "UTC",
});
const monthYear = new Intl.DateTimeFormat(LOCALE, {
  month: "long",
  year: "numeric",
  timeZone: "UTC",
});
const weekdayLong = new Intl.DateTimeFormat(LOCALE, {
  weekday: "long",
  timeZone: "UTC",
});

const capitalize = (text: string) =>
  text.charAt(0).toLocaleUpperCase(LOCALE) + text.slice(1);

/** `lun`, `mar`… */
export function weekdayName(iso: string): string {
  return weekdayShort.format(utcDate(iso)).replace(/\.$/, "");
}

export function dayNumber(iso: string): number {
  return Number(iso.slice(8, 10));
}

/** Heading of the calendar: the month, the week's first and last day, or the day itself. */
export function periodLabel(
  mode: ViewMode,
  anchor: string,
  firstDay: WeekDay,
): string {
  if (mode === "month") return capitalize(monthYear.format(utcDate(anchor)));
  if (mode === "week") {
    const days = weekDates(anchor, firstDay);
    return `${formatDisplayDate(days[0])} – ${formatDisplayDate(days[6])}`;
  }
  return `${capitalize(weekdayLong.format(utcDate(anchor)))}, ${formatDisplayDate(anchor)}`;
}
