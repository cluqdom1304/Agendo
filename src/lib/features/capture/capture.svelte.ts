// What a creation field reads from its text as it is typed (F23-02, D-96), and Esc to take
// the text as typed instead.

import { formatDisplayDate } from "../../dates";
import { t } from "../../i18n";
import {
  addDays,
  parseNatural,
  type Capture,
  type CaptureOptions,
} from "../../naturalInput";
import { DAY_CODES, describeRule } from "../../recurrence";
import { timeRange } from "../../schedule";
import { tagsStore } from "../../stores/tags.svelte";

/** Options for the parser with the app's tags; unknown `#names` stay in the title. */
export function captureOptions(
  today: string,
  fallbackDate?: string,
): CaptureOptions {
  return {
    today,
    fallbackDate,
    tagId: (name) => {
      const wanted = name.toLocaleLowerCase();
      return tagsStore.tags.find(
        (tag) => tag.name.toLocaleLowerCase() === wanted,
      )?.id;
    },
  };
}

/** «Hoy», «Mañana», else «lunes 23/11/2026». */
function dayLabel(date: string, today: string): string {
  if (date === today) return t.capture.today;
  if (date === addDays(today, 1)) return t.capture.tomorrow;
  const [year, month, day] = date.split("-").map(Number);
  const code =
    DAY_CODES[(new Date(Date.UTC(year, month - 1, day)).getUTCDay() + 6) % 7];
  return t.capture.day(t.recurrence.dayNames[code], formatDisplayDate(date));
}

/** The pieces the preview shows, in reading order. */
export function captureChips(capture: Capture, today: string): string[] {
  return [
    ...(capture.date ? [dayLabel(capture.date, today)] : []),
    ...(capture.time ? [timeRange(capture.time, capture.duration)] : []),
    ...(capture.rrule ? [describeRule(capture.rrule)] : []),
    ...capture.tags.map((tag) => `#${tag.name}`),
    ...(capture.priority > 0 ? [t.priority.levels[capture.priority]] : []),
  ];
}

/** A text field that reads a capture from what is typed; Esc drops it until it is cleared. */
export class CaptureField {
  text = $state("");
  ignored = $state(false);

  readonly #options: () => CaptureOptions;
  /** Text already turned into fields: not read again until it changes. */
  #settled = $state<string | null>(null);

  readonly capture = $derived.by(() =>
    this.ignored || this.text.trim() === "" || this.text === this.#settled
      ? null
      : parseNatural(this.text, this.#options()),
  );

  constructor(options: () => CaptureOptions) {
    this.#options = options;
  }

  /** Typing into an emptied field reads the text again. */
  set(value: string): void {
    this.text = value;
    if (value.trim() === "") this.ignored = false;
  }

  today(): string {
    return this.#options().today;
  }

  /** Esc: when something was detected, keeps the text as typed and uses the key up. */
  ignore(): boolean {
    if (this.capture === null) return false;
    this.ignored = true;
    return true;
  }

  /** The capture was copied into the form's fields and `title` is what is left. */
  settle(title: string): void {
    this.text = title;
    this.#settled = title;
  }

  reset(): void {
    this.text = "";
    this.ignored = false;
  }
}
