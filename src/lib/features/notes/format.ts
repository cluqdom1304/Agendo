import { t } from "../../i18n";

// How the note list and the editor show dates, in the user's language.

const TIME = new Intl.DateTimeFormat("es", {
  hour: "2-digit",
  minute: "2-digit",
});
const DAY = new Intl.DateTimeFormat("es", { day: "numeric", month: "short" });
const DAY_YEAR = new Intl.DateTimeFormat("es", {
  day: "numeric",
  month: "short",
  year: "numeric",
});

function sameDay(a: Date, b: Date): boolean {
  return (
    a.getFullYear() === b.getFullYear() &&
    a.getMonth() === b.getMonth() &&
    a.getDate() === b.getDate()
  );
}

/** Today shows the time, this year the day and month, older notes also the year. */
export function formatNoteDate(iso: string, now: Date): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return "";
  if (sameDay(date, now)) return TIME.format(date);
  const yesterday = new Date(now);
  yesterday.setDate(now.getDate() - 1);
  if (sameDay(date, yesterday)) return t.notesApp.yesterday;
  return date.getFullYear() === now.getFullYear()
    ? DAY.format(date)
    : DAY_YEAR.format(date);
}
