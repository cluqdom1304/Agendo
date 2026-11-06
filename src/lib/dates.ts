const pad = (value: number, width = 2) => String(value).padStart(width, "0");

/** `YYYY-MM-DD` → `DD/MM/YYYY` (SPEC F-04). Input comes from the backend, already validated. */
export function formatDisplayDate(isoDate: string): string {
  const [year, month, day] = isoDate.split("-");
  return `${day}/${month}/${year}`;
}

/** Calendar date of `date` in the machine's local time zone, as `YYYY-MM-DD`. */
export function toLocalIsoDate(date: Date): string {
  return `${pad(date.getFullYear(), 4)}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

/** UTC instant in the backend's `YYYY-MM-DDTHH:MM:SSZ` format. */
export function toInstant(date: Date): string {
  return date.toISOString().replace(/\.\d{3}Z$/, "Z");
}
